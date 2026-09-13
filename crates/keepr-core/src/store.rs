use crate::{
    dates, Error, Event, Item, ItemInput, Mutation, Preset, Reminder, Result, Room, Settings,
    Snapshot,
};
use chrono::{DateTime, Days, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use uuid::Uuid;

pub struct Store {
    conn: Connection,
}

fn id() -> String {
    Uuid::new_v4().to_string()
}

fn item_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        id: r.get(0)?,
        input: ItemInput {
            name: r.get(1)?,
            icon: r.get(2)?,
            color: r.get(3)?,
            room_id: r.get(4)?,
            started_on: r.get(5)?,
            due_on: r.get(6)?,
            repeat: r.get(7)?,
            every: r.get(8)?,
            notifications: r.get(9)?,
            advance_days: r.get(10)?,
            notes: r.get(11)?,
        },
        cycle_id: r.get(12)?,
        revision: r.get(13)?,
        completed: r.get(14)?,
        snoozed_until: r.get(15)?,
        deleted: r.get(16)?,
    })
}

fn get_item(conn: &Connection, item_id: &str) -> Result<Item> {
    conn.query_row("SELECT * FROM objects WHERE id=?1", [item_id], item_row)
        .optional()?
        .ok_or(Error::NotFound)
}

fn put_item(conn: &Connection, item: &Item) -> Result<()> {
    let i = &item.input;
    conn.execute("INSERT INTO objects VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
        ON CONFLICT(id) DO UPDATE SET name=excluded.name,icon=excluded.icon,color=excluded.color,room_id=excluded.room_id,
        started_on=excluded.started_on,due_on=excluded.due_on,repeat=excluded.repeat,every=excluded.every,
        notifications=excluded.notifications,advance_days=excluded.advance_days,notes=excluded.notes,cycle_id=excluded.cycle_id,
        revision=excluded.revision,completed=excluded.completed,snoozed_until=excluded.snoozed_until,deleted=excluded.deleted",
        params![item.id,i.name,i.icon,i.color,i.room_id,i.started_on,i.due_on,i.repeat,i.every,i.notifications,i.advance_days,i.notes,item.cycle_id,item.revision,item.completed,item.snoozed_until,item.deleted])?;
    Ok(())
}

fn record(
    conn: &Connection,
    item: &Item,
    kind: &str,
    previous: Option<&Item>,
    now: DateTime<Utc>,
    dates: (Option<&str>, Option<&str>),
    late: i32,
) -> Result<String> {
    let (effective, detail) = dates;
    let event_id = id();
    let previous = previous.map(serde_json::to_string).transpose()?;
    conn.execute("INSERT INTO events(id,item_id,name,icon,kind,at,effective_on,detail,late_days,previous_json,result_revision) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![event_id,item.id,item.input.name,item.input.icon,kind,now.to_rfc3339(),effective,detail,late,previous,item.revision])?;
    Ok(event_id)
}

fn checked_item(conn: &Connection, item_id: &str, revision: u32) -> Result<Item> {
    let item = get_item(conn, item_id)?;
    if item.deleted {
        return Err(Error::NotFound);
    }
    if item.revision != revision {
        return Err(Error::Conflict);
    }
    Ok(item)
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
        )?;
        let version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version > 2 {
            return Err(Error::Validation("newer_database"));
        }
        if version == 0 {
            conn.execute_batch("BEGIN IMMEDIATE;")?;
            if let Err(e) = conn.execute_batch(include_str!("../migrations/001_initial.sql")) {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(e.into());
            }
            conn.execute_batch("PRAGMA user_version=1; COMMIT;")?;
        }
        if version < 2 {
            if version == 1 && path != Path::new(":memory:") {
                let backup = path.with_extension("before-v2.keepr");
                if !backup.exists() {
                    conn.backup(rusqlite::MAIN_DB, &backup, None)?;
                }
            }
            let tx = conn.transaction()?;
            tx.execute_batch(include_str!("../migrations/002_journal_deletion.sql"))?;
            tx.execute_batch("PRAGMA user_version=2;")?;
            tx.commit()?;
        }
        conn.execute(
            "INSERT OR IGNORE INTO settings VALUES(1,?1)",
            [serde_json::to_string(&Settings::default())?],
        )?;
        Ok(Self { conn })
    }

    pub fn settings(&self) -> Result<Settings> {
        let raw: String = self
            .conn
            .query_row("SELECT data FROM settings WHERE id=1", [], |r| r.get(0))?;
        let settings: Settings = serde_json::from_str(&raw)?;
        dates::validate_settings(&settings)?;
        Ok(settings)
    }

    pub fn items(&self) -> Result<Vec<Item>> {
        Ok(self
            .conn
            .prepare("SELECT * FROM objects WHERE deleted=0 ORDER BY completed,due_on,name")?
            .query_map([], item_row)?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn rooms(&self) -> Result<Vec<Room>> {
        Ok(self
            .conn
            .prepare("SELECT id,name,icon,color FROM rooms ORDER BY rowid")?
            .query_map([], |r| {
                Ok(Room {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    icon: r.get(2)?,
                    color: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn events(&self, item_id: Option<&str>, limit: u32) -> Result<Vec<Event>> {
        Ok(self.conn.prepare("SELECT id,item_id,name,icon,kind,at,effective_on,detail,late_days,undone FROM events WHERE deleted_token IS NULL AND (?1 IS NULL OR item_id=?1) ORDER BY at DESC,rowid DESC LIMIT ?2")?.query_map(params![item_id,limit.min(10000)], |r| Ok(Event {
            id:r.get(0)?, item_id:r.get(1)?, name:r.get(2)?, icon:r.get(3)?, kind:r.get(4)?, at:r.get(5)?, effective_on:r.get(6)?, detail:r.get(7)?, late_days:r.get(8)?, undone:r.get(9)?,
        }))?.collect::<rusqlite::Result<_>>()?)
    }

    pub fn snapshot(&self, now: DateTime<Utc>) -> Result<Snapshot> {
        let settings = self.settings()?;
        let today = now.with_timezone(&dates::zone(&settings)?).date_naive();
        let items = self
            .items()?
            .into_iter()
            .map(|item| dates::view(item, today))
            .collect::<Result<Vec<_>>>()?;
        let completions: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM events WHERE kind='done' AND undone=0",
            [],
            |r| r.get(0),
        )?;
        let timely: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM events WHERE kind='done' AND undone=0 AND late_days=0 AND at>=?1",
            [(now - Duration::days(7)).to_rfc3339()],
            |r| r.get(0),
        )?;
        let health = dates::health(&items, completions, timely);
        Ok(Snapshot {
            items,
            rooms: self.rooms()?,
            events: self.events(None, 10000)?,
            settings,
            health,
            today: today.to_string(),
        })
    }

    fn mutation(
        &self,
        now: DateTime<Utc>,
        undo_id: Option<String>,
        item_id: Option<String>,
    ) -> Result<Mutation> {
        Ok(Mutation {
            snapshot: self.snapshot(now)?,
            undo_id,
            item_id,
        })
    }

    pub fn save_item(
        &mut self,
        item_id: Option<String>,
        revision: Option<u32>,
        mut input: ItemInput,
        now: DateTime<Utc>,
    ) -> Result<Mutation> {
        input.name = input.name.trim().into();
        dates::validate_input(&input)?;
        let tx = self.conn.transaction()?;
        if let Some(room) = &input.room_id {
            if !tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM rooms WHERE id=?1)",
                [room],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(Error::NotFound);
            }
        }
        let previous = item_id
            .as_ref()
            .map(|i| checked_item(&tx, i, revision.unwrap_or(0)))
            .transpose()?;
        let mut item = previous.clone().unwrap_or_else(|| Item {
            id: id(),
            input: input.clone(),
            cycle_id: id(),
            revision: 0,
            completed: false,
            snoozed_until: None,
            deleted: false,
        });
        if previous
            .as_ref()
            .is_some_and(|p| p.input.due_on != input.due_on)
        {
            item.snoozed_until = None;
        }
        item.input = input;
        item.revision += 1;
        put_item(&tx, &item)?;
        record(
            &tx,
            &item,
            if previous.is_some() {
                "updated"
            } else {
                "created"
            },
            None,
            now,
            (None, None),
            0,
        )?;
        tx.commit()?;
        self.mutation(now, None, Some(item.id))
    }

    pub fn complete(
        &mut self,
        item_id: &str,
        revision: u32,
        completed_on: &str,
        now: DateTime<Utc>,
    ) -> Result<Mutation> {
        let today = now
            .with_timezone(&dates::zone(&self.settings()?)?)
            .date_naive();
        let completed = dates::date(completed_on)?;
        if completed > today {
            return Err(Error::Validation("future_completion"));
        }
        let tx = self.conn.transaction()?;
        let previous = checked_item(&tx, item_id, revision)?;
        if previous.completed {
            return Err(Error::Conflict);
        }
        if completed < dates::date(&previous.input.started_on)? {
            return Err(Error::Validation("date_order"));
        }
        let mut item = previous.clone();
        tx.execute(
            "INSERT INTO cycles VALUES(?1,?2,?3,?4,?5)",
            params![
                item.cycle_id,
                item.id,
                item.input.started_on,
                item.input.due_on,
                completed_on
            ],
        )?;
        let late = (completed - dates::date(&item.input.due_on)?)
            .num_days()
            .max(0) as i32;
        if item.input.repeat == "once" {
            item.completed = true;
        } else {
            item.input.started_on = completed_on.into();
            item.input.due_on =
                dates::next_date(completed, &item.input.repeat, item.input.every)?.to_string();
            item.cycle_id = id();
        }
        item.snoozed_until = None;
        item.revision += 1;
        put_item(&tx, &item)?;
        let undo_id = record(
            &tx,
            &item,
            "done",
            Some(&previous),
            now,
            (Some(completed_on), Some(&previous.input.due_on)),
            late,
        )?;
        tx.commit()?;
        self.mutation(now, Some(undo_id), Some(item.id))
    }

    pub fn reopen(
        &mut self,
        item_id: &str,
        revision: u32,
        due_on: &str,
        now: DateTime<Utc>,
    ) -> Result<Mutation> {
        let today = now
            .with_timezone(&dates::zone(&self.settings()?)?)
            .date_naive();
        if dates::date(due_on)? < today {
            return Err(Error::Validation("date_order"));
        }
        let tx = self.conn.transaction()?;
        let previous = checked_item(&tx, item_id, revision)?;
        if !previous.completed {
            return Err(Error::Conflict);
        }
        let mut item = previous.clone();
        item.completed = false;
        item.cycle_id = id();
        item.revision += 1;
        item.input.started_on = today.to_string();
        item.input.due_on = due_on.into();
        item.snoozed_until = None;
        put_item(&tx, &item)?;
        record(&tx, &item, "renewed", None, now, (None, Some(due_on)), 0)?;
        tx.commit()?;
        self.mutation(now, None, Some(item.id))
    }

    pub fn defer(
        &mut self,
        item_id: &str,
        revision: u32,
        choice: &str,
        now: DateTime<Utc>,
    ) -> Result<Mutation> {
        let until = dates::snooze(now, choice, &self.settings()?)?.to_rfc3339();
        let tx = self.conn.transaction()?;
        let previous = checked_item(&tx, item_id, revision)?;
        if previous.completed {
            return Err(Error::Conflict);
        }
        let mut item = previous.clone();
        item.snoozed_until = Some(until.clone());
        item.revision += 1;
        put_item(&tx, &item)?;
        let undo = record(
            &tx,
            &item,
            "deferred",
            Some(&previous),
            now,
            (None, Some(&until)),
            0,
        )?;
        tx.commit()?;
        self.mutation(now, Some(undo), Some(item.id))
    }

    pub fn delete_item(
        &mut self,
        item_id: &str,
        revision: u32,
        now: DateTime<Utc>,
    ) -> Result<Mutation> {
        let tx = self.conn.transaction()?;
        let previous = checked_item(&tx, item_id, revision)?;
        let mut item = previous.clone();
        item.deleted = true;
        item.revision += 1;
        put_item(&tx, &item)?;
        let undo = record(&tx, &item, "deleted", Some(&previous), now, (None, None), 0)?;
        tx.commit()?;
        self.mutation(now, Some(undo), None)
    }

    /// Removing a journal entry must not reverse care, reschedule an item or
    /// lower the user's progress. It only changes visibility of the event.
    pub fn delete_event(&mut self, event_id: &str, now: DateTime<Utc>) -> Result<Mutation> {
        let token = id();
        let changed = self.conn.execute(
            "UPDATE events SET deleted_token=?1 WHERE id=?2 AND deleted_token IS NULL",
            params![token, event_id],
        )?;
        if changed == 0 {
            return Err(Error::Validation("event_unavailable"));
        }
        self.mutation(now, Some(format!("journal:{token}")), None)
    }

    pub fn undo(&mut self, event_id: &str, now: DateTime<Utc>) -> Result<Mutation> {
        if let Some(token) = event_id.strip_prefix("journal:") {
            let changed = self.conn.execute(
                "UPDATE events SET deleted_token=NULL WHERE deleted_token=?1",
                [token],
            )?;
            if changed == 0 {
                return Err(Error::Validation("event_unavailable"));
            }
            return self.mutation(now, None, None);
        }
        let tx = self.conn.transaction()?;
        let (previous, revision, kind): (String,u32,String) = tx.query_row("SELECT previous_json,result_revision,kind FROM events WHERE id=?1 AND undone=0 AND previous_json IS NOT NULL", [event_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?.ok_or(Error::Conflict)?;
        let mut item: Item = serde_json::from_str(&previous)?;
        if get_item(&tx, &item.id)?.revision != revision {
            return Err(Error::Conflict);
        }
        item.revision = revision + 1;
        if let Some(room) = &item.input.room_id {
            if !tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM rooms WHERE id=?1)",
                [room],
                |r| r.get::<_, bool>(0),
            )? {
                item.input.room_id = None;
            }
        }
        if kind == "done" {
            tx.execute("DELETE FROM cycles WHERE id=?1", [&item.cycle_id])?;
        }
        put_item(&tx, &item)?;
        tx.execute("UPDATE events SET undone=1 WHERE id=?1", [event_id])?;
        record(&tx, &item, "restored", None, now, (None, None), 0)?;
        tx.commit()?;
        self.mutation(now, None, Some(item.id))
    }

    pub fn save_room(&mut self, mut room: Room, now: DateTime<Utc>) -> Result<Mutation> {
        room.name = room.name.trim().into();
        if room.name.is_empty() || room.name.chars().count() > 60 {
            return Err(Error::Validation("invalid_name"));
        }
        dates::validate_appearance(&room.icon, &room.color)?;
        if room.id.is_empty() {
            room.id = id();
        }
        self.conn.execute("INSERT INTO rooms VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,icon=excluded.icon,color=excluded.color",params![room.id,room.name,room.icon,room.color])?;
        self.mutation(now, None, None)
    }

    pub fn delete_room(&mut self, room_id: &str, now: DateTime<Utc>) -> Result<Mutation> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE objects SET revision=revision+1 WHERE room_id=?1",
            [room_id],
        )?;
        if tx.execute("DELETE FROM rooms WHERE id=?1", [room_id])? == 0 {
            return Err(Error::NotFound);
        }
        tx.commit()?;
        self.mutation(now, None, None)
    }

    pub fn save_settings(&mut self, settings: Settings, now: DateTime<Utc>) -> Result<Mutation> {
        dates::validate_settings(&settings)?;
        self.conn.execute(
            "UPDATE settings SET data=?1 WHERE id=1",
            [serde_json::to_string(&settings)?],
        )?;
        self.mutation(now, None, None)
    }

    pub fn presets() -> Result<Vec<Preset>> {
        Ok(serde_json::from_str(include_str!(
            "../../../resources/presets.json"
        ))?)
    }

    pub fn onboard(&mut self, demo: bool, now: DateTime<Utc>) -> Result<Mutation> {
        let mut settings = self.settings()?;
        if settings.onboarding_done {
            return Err(Error::Conflict);
        }
        let today = now.with_timezone(&dates::zone(&settings)?).date_naive();
        let tx = self.conn.transaction()?;
        if demo {
            let rooms = [
                ("bathroom", "Ванная", "Bathroom", "bath", "blue"),
                ("kitchen", "Кухня", "Kitchen", "cooking-pot", "peach"),
                ("bedroom", "Спальня", "Bedroom", "bed", "lavender"),
            ];
            for (key, ru, en, icon, color) in rooms {
                tx.execute(
                    "INSERT OR IGNORE INTO rooms VALUES(?1,?2,?3,?4)",
                    params![
                        key,
                        if settings.language == "ru" { ru } else { en },
                        icon,
                        color
                    ],
                )?;
            }
            for (preset_id, left) in [
                ("toothbrush", 18),
                ("sponge", -2),
                ("water-filter", 0),
                ("bedding", 4),
                ("pillow", 120),
                ("kitchen-towel", 2),
            ] {
                let p = Self::presets()?
                    .into_iter()
                    .find(|p| p.id == preset_id)
                    .ok_or(Error::NotFound)?;
                let due = today + Duration::days(left);
                let start = if left < 0 {
                    today - Duration::days(12)
                } else {
                    today - Duration::days(20)
                };
                let item = Item {
                    id: id(),
                    cycle_id: id(),
                    revision: 1,
                    completed: false,
                    snoozed_until: None,
                    deleted: false,
                    input: ItemInput {
                        name: if settings.language == "ru" {
                            p.ru
                        } else {
                            p.en
                        },
                        icon: p.icon,
                        color: p.color,
                        room_id: Some(p.room),
                        started_on: start.to_string(),
                        due_on: due.to_string(),
                        repeat: p.repeat,
                        every: p.every,
                        notifications: true,
                        advance_days: 1,
                        notes: String::new(),
                    },
                };
                put_item(&tx, &item)?;
                record(&tx, &item, "created", None, now, (None, None), 0)?;
            }
        }
        settings.onboarding_done = true;
        settings.demo_home = demo;
        tx.execute(
            "UPDATE settings SET data=?1 WHERE id=1",
            [serde_json::to_string(&settings)?],
        )?;
        tx.commit()?;
        self.mutation(now, None, None)
    }

    /// Clears a demo (or any) home back to a clean slate: items, rooms,
    /// history and pending notifications go away atomically. Settings —
    /// language, theme, pet, reminder times — are preserved, as is the
    /// completed onboarding. There is no undo; the UI confirms first.
    pub fn reset_home(&mut self, now: DateTime<Utc>) -> Result<Mutation> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(
            "DELETE FROM notification_jobs; DELETE FROM events; DELETE FROM cycles; DELETE FROM objects; DELETE FROM rooms;",
        )?;
        let raw: String = tx.query_row("SELECT data FROM settings WHERE id=1", [], |r| r.get(0))?;
        let mut settings: Settings = serde_json::from_str(&raw)?;
        settings.demo_home = false;
        dates::validate_settings(&settings)?;
        tx.execute(
            "UPDATE settings SET data=?1 WHERE id=1",
            [serde_json::to_string(&settings)?],
        )?;
        tx.commit()?;
        self.mutation(now, None, None)
    }

    /// Returns due work and the next meaningful wake-up. Receipt identity is
    /// cycle + due date + civil reminder day; cosmetic edits cannot re-send it.
    pub fn reminders(&self, now: DateTime<Utc>) -> Result<(Vec<Reminder>, DateTime<Utc>)> {
        let settings = self.settings()?;
        let tz = dates::zone(&settings)?;
        let today = now.with_timezone(&tz).date_naive();
        let mut next = dates::resolve(today + Days::new(1), dates::time("00:00")?, tz)?;
        if !settings.notifications || !settings.onboarding_done {
            return Ok((vec![], next));
        }
        let mut reminders = Vec::new();
        for item in self
            .items()?
            .into_iter()
            .filter(|i| !i.completed && i.input.notifications)
        {
            let mut explicit = false;
            let (key, mut at) = if let Some(raw) = &item.snoozed_until {
                let until = DateTime::parse_from_rfc3339(raw)
                    .map_err(|_| Error::Validation("invalid_date"))?
                    .with_timezone(&Utc);
                if until.with_timezone(&tz).date_naive() >= today {
                    explicit = true;
                    (format!("{}:snooze:{}", item.cycle_id, raw), until)
                } else {
                    (
                        format!("{}:{}:{}", item.cycle_id, item.input.due_on, today),
                        dates::resolve(today, dates::time(&settings.reminder_time)?, tz)?,
                    )
                }
            } else {
                let first =
                    dates::date(&item.input.due_on)? - Days::new(item.input.advance_days.into());
                let day = first.max(today);
                (
                    format!("{}:{}:{}", item.cycle_id, item.input.due_on, day),
                    dates::resolve(day, dates::time(&settings.reminder_time)?, tz)?,
                )
            };
            let receipt: Option<(String, Option<String>)> = self
                .conn
                .query_row(
                    "SELECT status,retry_at FROM notification_jobs WHERE key=?1",
                    [&key],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            if let Some((status, retry)) = receipt {
                if status == "sent" {
                    continue;
                }
                if let Some(retry) = retry {
                    at = at.max(
                        DateTime::parse_from_rfc3339(&retry)
                            .map_err(|_| Error::Validation("invalid_date"))?
                            .with_timezone(&Utc),
                    );
                }
            }
            if !explicit {
                if let Some(end) = dates::quiet_until(at.max(now), &settings)? {
                    at = at.max(end);
                }
            }
            if at <= now {
                reminders.push(Reminder {
                    key,
                    item_id: item.id,
                    name: item.input.name,
                    due_on: item.input.due_on,
                });
            } else {
                next = next.min(at);
            }
        }
        Ok((reminders, next))
    }

    pub fn record_delivery(
        &mut self,
        reminders: &[Reminder],
        delivered: bool,
        now: DateTime<Utc>,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        for r in reminders {
            tx.execute("INSERT INTO notification_jobs(key,item_id,status,attempts,retry_at,sent_at) VALUES(?1,?2,?3,1,?4,?5)
                ON CONFLICT(key) DO UPDATE SET status=excluded.status,attempts=attempts+1,retry_at=excluded.retry_at,sent_at=excluded.sent_at",
                params![r.key,r.item_id,if delivered {"sent"} else {"retry"},(now+Duration::minutes(5)).to_rfc3339(),if delivered {Some(now.to_rfc3339())} else {None}])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn backup(&self, path: &Path) -> Result<()> {
        self.conn.backup(rusqlite::MAIN_DB, path, None)?;
        Ok(())
    }

    pub fn restore(&mut self, path: &Path) -> Result<()> {
        let source = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let version: u32 = source.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let check: String = source.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        if !(1..=2).contains(&version) || check != "ok" {
            return Err(Error::Validation("invalid_backup"));
        }
        // Re-import data into our schema: never execute schema or triggers from an external database.
        let settings: Settings = serde_json::from_str(&source.query_row::<String, _, _>(
            "SELECT data FROM settings WHERE id=1",
            [],
            |r| r.get(0),
        )?)?;
        dates::validate_settings(&settings)?;
        let items: Vec<Item> = source
            .prepare("SELECT * FROM objects")?
            .query_map([], item_row)?
            .collect::<rusqlite::Result<_>>()?;
        for item in &items {
            dates::validate_input(&item.input)?;
        }
        let rooms: Vec<Room> = source
            .prepare("SELECT id,name,icon,color FROM rooms")?
            .query_map([], |r| {
                Ok(Room {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    icon: r.get(2)?,
                    color: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let tx = self.conn.transaction()?;
        tx.execute_batch("DELETE FROM notification_jobs; DELETE FROM events; DELETE FROM cycles; DELETE FROM objects; DELETE FROM rooms;")?;
        for room in rooms {
            dates::validate_appearance(&room.icon, &room.color)?;
            tx.execute(
                "INSERT INTO rooms VALUES(?1,?2,?3,?4)",
                params![room.id, room.name, room.icon, room.color],
            )?;
        }
        for item in &items {
            put_item(&tx, item)?;
        }
        let mut stmt = source.prepare(if version == 1 {
            "SELECT id,item_id,name,icon,kind,at,effective_on,detail,late_days,undone,NULL AS deleted_token FROM events"
        } else {
            "SELECT id,item_id,name,icon,kind,at,effective_on,detail,late_days,undone,deleted_token FROM events"
        })?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            tx.execute("INSERT INTO events(id,item_id,name,icon,kind,at,effective_on,detail,late_days,undone,deleted_token) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,Option<String>>(7)?,r.get::<_,i32>(8)?,r.get::<_,bool>(9)?,r.get::<_,Option<String>>(10)?])?;
        }
        let mut stmt =
            source.prepare("SELECT id,object_id,started_on,due_on,completed_on FROM cycles")?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            tx.execute(
                "INSERT INTO cycles VALUES(?1,?2,?3,?4,?5)",
                params![
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?
                ],
            )?;
        }
        tx.execute(
            "UPDATE settings SET data=?1 WHERE id=1",
            [serde_json::to_string(&settings)?],
        )?;
        tx.commit()?;
        Ok(())
    }
}
