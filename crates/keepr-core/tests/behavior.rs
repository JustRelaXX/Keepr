use chrono::{DateTime, Duration, Utc};
use keepr_core::{dates, ItemInput, Room, Settings, Store};
use proptest::prelude::*;

fn now() -> DateTime<Utc> {
    "2026-09-13T10:00:00Z".parse().unwrap()
}
fn input() -> ItemInput {
    ItemInput {
        name: "Filter".into(),
        icon: "droplets".into(),
        color: "blue".into(),
        room_id: None,
        started_on: "2026-08-13".into(),
        due_on: "2026-09-12".into(),
        repeat: "months".into(),
        every: 1,
        notifications: true,
        advance_days: 1,
        notes: String::new(),
    }
}
fn store() -> Store {
    let mut s = Store::open(":memory:").unwrap();
    s.save_settings(
        Settings {
            timezone: "UTC".into(),
            onboarding_done: true,
            ..Default::default()
        },
        now(),
    )
    .unwrap();
    s
}

#[test]
fn calendar_months_and_leap_years() {
    assert_eq!(
        dates::next_date(dates::date("2024-01-31").unwrap(), "months", 1)
            .unwrap()
            .to_string(),
        "2024-02-29"
    );
    assert_eq!(
        dates::next_date(dates::date("2024-02-29").unwrap(), "years", 1)
            .unwrap()
            .to_string(),
        "2025-02-28"
    );
    assert!(dates::next_date(dates::date("2200-12-31").unwrap(), "days", 1).is_err());
    assert!(dates::next_date(dates::date("2026-01-01").unwrap(), "days", 0).is_err());
}

#[test]
fn tomorrow_is_a_civil_day_over_dst() {
    let s = Settings {
        timezone: "Europe/Berlin".into(),
        ..Default::default()
    };
    let now: DateTime<Utc> = "2026-03-28T08:00:00Z".parse().unwrap();
    assert_eq!(
        dates::snooze(now, "tomorrow", &s).unwrap() - now,
        Duration::hours(23)
    );
    assert_eq!(
        dates::snooze(now, "hour", &s).unwrap() - now,
        Duration::hours(1)
    );
}

#[test]
fn dst_gap_fold_and_late_evening_are_deterministic() {
    let tz = "Europe/Berlin".parse().unwrap();
    assert_eq!(
        dates::resolve(
            dates::date("2026-03-29").unwrap(),
            dates::time("02:30").unwrap(),
            tz
        )
        .unwrap()
        .to_rfc3339(),
        "2026-03-29T01:00:00+00:00"
    );
    assert_eq!(
        dates::resolve(
            dates::date("2026-10-25").unwrap(),
            dates::time("02:30").unwrap(),
            tz
        )
        .unwrap()
        .to_rfc3339(),
        "2026-10-25T00:30:00+00:00"
    );
    let s = Settings {
        timezone: "UTC".into(),
        ..Default::default()
    };
    let late = "2026-09-13T22:00:00Z".parse().unwrap();
    assert_eq!(
        dates::snooze(late, "evening", &s).unwrap().to_rfc3339(),
        "2026-09-14T19:00:00+00:00"
    );
}

#[test]
fn recurring_completion_uses_actual_date_and_rejects_duplicate() {
    let mut s = store();
    let created = s.save_item(None, None, input(), now()).unwrap();
    let item = &created.snapshot.items[0].item;
    let result = s
        .complete(&item.id, item.revision, "2026-09-11", now())
        .unwrap();
    let next = &result.snapshot.items[0].item;
    assert_eq!(next.input.due_on, "2026-10-11");
    assert_ne!(next.cycle_id, item.cycle_id);
    assert!(s
        .complete(&item.id, item.revision, "2026-09-13", now())
        .is_err());
    assert_eq!(s.snapshot(now()).unwrap().health.completions, 1);
}

#[test]
fn completion_and_undo_restore_history_and_schedule_atomically() {
    let mut s = store();
    let created = s.save_item(None, None, input(), now()).unwrap();
    let i = &created.snapshot.items[0].item;
    let completed = s.complete(&i.id, i.revision, "2026-09-13", now()).unwrap();
    assert_eq!(completed.snapshot.events[0].late_days, 1);
    let undone = s.undo(completed.undo_id.as_ref().unwrap(), now()).unwrap();
    assert_eq!(undone.snapshot.items[0].item.input.due_on, "2026-09-12");
    assert_eq!(undone.snapshot.health.completions, 0);
    assert!(s.undo(completed.undo_id.as_ref().unwrap(), now()).is_err());
}

#[test]
fn one_time_completion_and_future_rejection() {
    let mut s = store();
    let mut i = input();
    i.repeat = "once".into();
    let created = s.save_item(None, None, i, now()).unwrap();
    let i = &created.snapshot.items[0].item;
    assert!(s.complete(&i.id, i.revision, "2026-09-14", now()).is_err());
    let done = s.complete(&i.id, i.revision, "2026-09-13", now()).unwrap();
    assert_eq!(done.snapshot.items[0].status, "completed");
    assert!(s.reminders(now()).unwrap().0.is_empty());
}

#[test]
fn snooze_preserves_expiry_and_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("home.db");
    let mut s = Store::open(&path).unwrap();
    s.save_settings(
        Settings {
            timezone: "UTC".into(),
            onboarding_done: true,
            ..Default::default()
        },
        now(),
    )
    .unwrap();
    let result = s.save_item(None, None, input(), now()).unwrap();
    let i = &result.snapshot.items[0].item;
    let result = s.defer(&i.id, i.revision, "hour", now()).unwrap();
    assert_eq!(result.snapshot.items[0].status, "overdue");
    drop(s);
    let s = Store::open(path).unwrap();
    assert_eq!(s.items().unwrap()[0].input.due_on, "2026-09-12");
    assert!(s.reminders(now()).unwrap().0.is_empty());
    assert_eq!(s.reminders(now() + Duration::hours(1)).unwrap().0.len(), 1);
}

#[test]
fn delivered_reminders_are_deduplicated_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("home.db");
    let mut s = Store::open(&path).unwrap();
    s.save_settings(
        Settings {
            timezone: "UTC".into(),
            onboarding_done: true,
            ..Default::default()
        },
        now(),
    )
    .unwrap();
    s.save_item(None, None, input(), now()).unwrap();
    let reminders = s.reminders(now()).unwrap().0;
    assert_eq!(reminders.len(), 1);
    s.record_delivery(&reminders, true, now()).unwrap();
    drop(s);
    let s = Store::open(path).unwrap();
    assert!(s.reminders(now()).unwrap().0.is_empty());
    assert_eq!(s.reminders(now() + Duration::days(1)).unwrap().0.len(), 1);
}

#[test]
fn delivery_failure_backs_off_and_quiet_hours_defer() {
    let mut s = store();
    s.save_item(None, None, input(), now()).unwrap();
    let reminders = s.reminders(now()).unwrap().0;
    s.record_delivery(&reminders, false, now()).unwrap();
    assert!(s.reminders(now()).unwrap().0.is_empty());
    assert_eq!(
        s.reminders(now() + Duration::minutes(5)).unwrap().0.len(),
        1
    );
    let late: DateTime<Utc> = "2026-09-14T23:00:00Z".parse().unwrap();
    assert!(s.reminders(late).unwrap().0.is_empty());
}

#[test]
fn removing_room_keeps_items_and_delete_is_reversible() {
    let mut s = store();
    s.save_room(
        Room {
            id: "bath".into(),
            name: "Bath".into(),
            icon: "bath".into(),
            color: "blue".into(),
        },
        now(),
    )
    .unwrap();
    let mut i = input();
    i.room_id = Some("bath".into());
    s.save_item(None, None, i, now()).unwrap();
    let result = s.delete_room("bath", now()).unwrap();
    let i = &result.snapshot.items[0].item;
    assert!(i.input.room_id.is_none());
    let deleted = s.delete_item(&i.id, i.revision, now()).unwrap();
    assert!(deleted.snapshot.items.is_empty());
    let restored = s.undo(deleted.undo_id.as_ref().unwrap(), now()).unwrap();
    assert_eq!(restored.snapshot.items.len(), 1);
}

#[test]
fn invalid_writes_do_not_damage_existing_items() {
    let mut s = store();
    let mut i = input();
    i.due_on = "2026-01-01".into();
    assert!(s.save_item(None, None, i, now()).is_err());
    assert!(s.items().unwrap().is_empty());
    let mut i = input();
    i.room_id = Some("missing".into());
    assert!(s.save_item(None, None, i, now()).is_err());
    assert!(s.items().unwrap().is_empty());
}

#[test]
fn expanded_appearance_palette_is_accepted() {
    let mut s = store();
    for color in [
        "sage", "peach", "lavender", "blue", "yellow", "rose", "mint", "teal", "coral", "plum",
        "sand", "slate",
    ] {
        let mut i = input();
        i.name = format!("Item {color}");
        i.color = color.into();
        i.icon = "toothbrush".into();
        s.save_item(None, None, i, now()).unwrap();
    }
    assert_eq!(s.items().unwrap().len(), 12);
    let mut i = input();
    i.color = "neon".into();
    assert!(s.save_item(None, None, i, now()).is_err());
}

#[test]
fn backup_roundtrip_preserves_history_and_pet() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store();
    let result = s.save_item(None, None, input(), now()).unwrap();
    let i = &result.snapshot.items[0].item;
    s.complete(&i.id, i.revision, "2026-09-13", now()).unwrap();
    let path = dir.path().join("backup.keepr");
    s.backup(&path).unwrap();
    let mut other = store();
    other.restore(&path).unwrap();
    assert_eq!(other.snapshot(now()).unwrap().health.completions, 1);
    assert_eq!(other.items().unwrap()[0].input.due_on, "2026-10-13");
    assert_eq!(other.events(None, 100).unwrap().len(), 2);
}

#[test]
fn empty_home_is_healthy_and_one_overdue_is_not_a_disaster() {
    let mut s = store();
    assert_eq!(s.snapshot(now()).unwrap().health.score, 100);
    let result = s.save_item(None, None, input(), now()).unwrap();
    assert!(result.snapshot.health.score >= 90);
    let later = s.snapshot(now() + Duration::days(100)).unwrap();
    assert!(later.health.score >= 75);
}

#[test]
fn midnight_changes_state_in_home_timezone() {
    let mut s = store();
    let mut i = input();
    i.due_on = "2026-09-13".into();
    s.save_item(None, None, i, now()).unwrap();
    assert_eq!(
        s.snapshot("2026-09-13T23:59:59Z".parse().unwrap())
            .unwrap()
            .items[0]
            .status,
        "today"
    );
    assert_eq!(
        s.snapshot("2026-09-14T00:00:00Z".parse().unwrap())
            .unwrap()
            .items[0]
            .status,
        "overdue"
    );
}

#[test]
fn completed_one_time_item_can_start_a_new_cycle() {
    let mut s = store();
    let mut input = input();
    input.repeat = "once".into();
    let m = s.save_item(None, None, input, now()).unwrap();
    let i = &m.snapshot.items[0].item;
    let m = s.complete(&i.id, i.revision, "2026-09-13", now()).unwrap();
    let completed = &m.snapshot.items[0].item;
    let m = s
        .reopen(&completed.id, completed.revision, "2026-10-01", now())
        .unwrap();
    assert!(!m.snapshot.items[0].item.completed);
    assert_ne!(m.snapshot.items[0].item.cycle_id, i.cycle_id);
    assert_eq!(m.snapshot.health.completions, 1);
    assert_eq!(m.snapshot.items[0].item.input.started_on, "2026-09-13");
}

#[test]
fn journal_deletion_and_undo_leave_care_and_progress_intact() {
    let mut s = store();
    let created = s.save_item(None, None, input(), now()).unwrap();
    let i = &created.snapshot.items[0].item;
    let done = s.complete(&i.id, i.revision, "2026-09-13", now()).unwrap();
    let event_id = done.undo_id.as_ref().unwrap();
    let removed = s.delete_event(event_id, now()).unwrap();
    assert!(!removed.snapshot.events.iter().any(|e| &e.id == event_id));
    assert!(!s
        .events(Some(&i.id), 100)
        .unwrap()
        .iter()
        .any(|e| &e.id == event_id));
    assert_eq!(
        serde_json::to_value(&removed.snapshot.items).unwrap(),
        serde_json::to_value(&done.snapshot.items).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&removed.snapshot.health).unwrap(),
        serde_json::to_value(&done.snapshot.health).unwrap()
    );
    let restored = s.undo(removed.undo_id.as_ref().unwrap(), now()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored.snapshot).unwrap(),
        serde_json::to_value(&done.snapshot).unwrap()
    );
}

#[test]
fn stale_journal_undo_cannot_restore_a_later_deletion() {
    let mut s = store();
    let m = s.save_item(None, None, input(), now()).unwrap();
    let event_id = &m.snapshot.events[0].id;
    assert!(s.delete_event("missing-event", now()).is_err());
    let first = s.delete_event(event_id, now()).unwrap();
    assert!(s.delete_event(event_id, now()).is_err());
    s.undo(first.undo_id.as_ref().unwrap(), now()).unwrap();
    let second = s.delete_event(event_id, now()).unwrap();
    assert!(s.undo(first.undo_id.as_ref().unwrap(), now()).is_err());
    assert!(s.events(None, 100).unwrap().is_empty());
    s.undo(second.undo_id.as_ref().unwrap(), now()).unwrap();
    assert_eq!(s.events(None, 100).unwrap().len(), 1);
}

#[test]
fn journal_can_be_cleared_even_after_an_item_is_deleted() {
    let mut s = store();
    let created = s.save_item(None, None, input(), now()).unwrap();
    let i = &created.snapshot.items[0].item;
    let removed_item = s.delete_item(&i.id, i.revision, now()).unwrap();
    for event in &removed_item.snapshot.events {
        s.delete_event(&event.id, now()).unwrap();
    }
    let snapshot = s.snapshot(now()).unwrap();
    assert!(snapshot.items.is_empty());
    assert!(snapshot.events.is_empty());
    // Clearing the journal does not consume the original item-deletion undo.
    let restored_item = s
        .undo(removed_item.undo_id.as_ref().unwrap(), now())
        .unwrap();
    assert_eq!(restored_item.snapshot.items.len(), 1);
    assert_eq!(restored_item.snapshot.events.len(), 1);
    assert_eq!(restored_item.snapshot.events[0].kind, "restored");
}

#[test]
fn journal_deletion_survives_restart_and_backup_restore() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("home.db");
    let mut s = Store::open(&path).unwrap();
    s.save_settings(
        Settings {
            timezone: "UTC".into(),
            onboarding_done: true,
            ..Default::default()
        },
        now(),
    )
    .unwrap();
    let created = s.save_item(None, None, input(), now()).unwrap();
    let i = &created.snapshot.items[0].item;
    let done = s.complete(&i.id, i.revision, "2026-09-13", now()).unwrap();
    let event_id = done.undo_id.as_ref().unwrap();
    let deletion = s.delete_event(event_id, now()).unwrap();
    drop(s);
    let s = Store::open(&path).unwrap();
    assert!(!s
        .events(None, 100)
        .unwrap()
        .iter()
        .any(|e| &e.id == event_id));
    let backup = dir.path().join("backup.keepr");
    s.backup(&backup).unwrap();
    let mut restored = store();
    restored.restore(&backup).unwrap();
    assert_eq!(
        serde_json::to_value(restored.snapshot(now()).unwrap()).unwrap(),
        serde_json::to_value(s.snapshot(now()).unwrap()).unwrap()
    );
    assert_eq!(restored.snapshot(now()).unwrap().health.completions, 1);
    restored
        .undo(deletion.undo_id.as_ref().unwrap(), now())
        .unwrap();
    assert!(restored
        .events(None, 100)
        .unwrap()
        .iter()
        .any(|e| &e.id == event_id));
}

fn legacy_home(path: &std::path::Path) {
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    conn.execute_batch("PRAGMA user_version=1;").unwrap();
    conn.execute(
        "INSERT INTO settings VALUES(1,?1)",
        [serde_json::to_string(&Settings {
            timezone: "UTC".into(),
            onboarding_done: true,
            ..Default::default()
        })
        .unwrap()],
    )
    .unwrap();
    conn.execute_batch("INSERT INTO objects VALUES('legacy-item','Legacy filter','droplets','blue',NULL,'2026-08-13','2026-09-12','once',1,1,1,'','legacy-cycle',2,1,NULL,0);
        INSERT INTO cycles VALUES('legacy-cycle','legacy-item','2026-08-13','2026-09-12','2026-09-13');
        INSERT INTO events(id,item_id,name,icon,kind,at,effective_on,detail,late_days) VALUES('legacy-event','legacy-item','Legacy filter','droplets','done','2026-09-13T10:00:00+00:00','2026-09-13','2026-09-12',1);").unwrap();
}

#[test]
fn migrates_existing_home_and_keeps_a_version_one_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("home.db");
    legacy_home(&path);
    let mut s = Store::open(&path).unwrap();
    assert_eq!(s.snapshot(now()).unwrap().health.completions, 1);
    assert_eq!(s.events(None, 100).unwrap()[0].id, "legacy-event");
    s.delete_event("legacy-event", now()).unwrap();
    drop(s);
    let reopened = Store::open(&path).unwrap();
    assert!(reopened.events(None, 100).unwrap().is_empty());
    let backup = rusqlite::Connection::open(path.with_extension("before-v2.keepr")).unwrap();
    assert_eq!(
        backup
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        backup
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
}

#[test]
fn still_imports_version_one_backups() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.keepr");
    legacy_home(&path);
    let mut s = store();
    s.restore(&path).unwrap();
    assert_eq!(s.items().unwrap()[0].input.name, "Legacy filter");
    assert_eq!(s.snapshot(now()).unwrap().health.completions, 1);
    s.delete_event("legacy-event", now()).unwrap();
    assert!(s.events(None, 100).unwrap().is_empty());
}

#[test]
fn old_settings_json_parses_with_safe_defaults() {
    // Databases written before tour_seen/demo_home existed must open with
    // the tour reshown and no demo banner — never the reverse.
    let raw = r#"{"language":"ru","theme":"system","pet":"plant","home_name":"","timezone":"UTC","reminder_time":"09:00","quiet_start":"22:00","quiet_end":"08:00","notifications":true,"onboarding_done":true}"#;
    let s: Settings = serde_json::from_str(raw).unwrap();
    assert!(!s.tour_seen);
    assert!(!s.demo_home);
}

#[test]
fn tour_seen_survives_save_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("home.db");
    let mut s = Store::open(&path).unwrap();
    assert!(!s.settings().unwrap().tour_seen);
    let mut settings = s.settings().unwrap();
    settings.tour_seen = true;
    s.save_settings(settings, now()).unwrap();
    drop(s);
    let s = Store::open(&path).unwrap();
    assert!(s.snapshot(now()).unwrap().settings.tour_seen);
}

#[test]
fn demo_onboarding_flags_home_and_reset_clears_it() {
    let mut s = Store::open(":memory:").unwrap();
    let m = s.onboard(true, now()).unwrap();
    assert!(m.snapshot.settings.demo_home);
    assert!(m.snapshot.settings.onboarding_done);
    assert!(!m.snapshot.items.is_empty());
    assert!(!m.snapshot.rooms.is_empty());
    let m = s.reset_home(now()).unwrap();
    assert!(!m.snapshot.settings.demo_home);
    assert!(m.snapshot.settings.onboarding_done);
    assert!(m.snapshot.items.is_empty());
    assert!(m.snapshot.rooms.is_empty());
    assert!(m.snapshot.events.is_empty());
    assert!(m.undo_id.is_none());
    // Settings the user may have changed survive the reset.
    assert_eq!(m.snapshot.settings.language, "ru");
    assert!(s.reminders(now()).unwrap().0.is_empty());
}

#[test]
fn reset_home_on_empty_home_is_a_clean_noop() {
    let mut s = Store::open(":memory:").unwrap();
    s.onboard(false, now()).unwrap();
    let m = s.reset_home(now()).unwrap();
    assert!(m.snapshot.items.is_empty());
    assert!(m.snapshot.settings.onboarding_done);
    assert!(!m.snapshot.settings.demo_home);
}

proptest! {
    #[test]
    fn positive_calendar_intervals_always_advance(days in 1u32..999,offset in 0i64..5000) {
        let from = dates::date("2020-01-01").unwrap()+Duration::days(offset);
        let next = dates::next_date(from,"days",days).unwrap();
        prop_assert!(next>from);
        prop_assert_eq!((next-from).num_days(),i64::from(days));
    }
}
