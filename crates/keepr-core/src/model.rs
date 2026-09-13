use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Room {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct ItemInput {
    pub name: String,
    pub icon: String,
    pub color: String,
    pub room_id: Option<String>,
    pub started_on: String,
    pub due_on: String,
    pub repeat: String,
    pub every: u32,
    pub notifications: bool,
    pub advance_days: u32,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Item {
    pub id: String,
    pub input: ItemInput,
    pub cycle_id: String,
    pub revision: u32,
    pub completed: bool,
    pub snoozed_until: Option<String>,
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct ItemView {
    pub item: Item,
    pub status: String,
    pub days_left: i32,
    pub remaining: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Event {
    pub id: String,
    pub item_id: String,
    pub name: String,
    pub icon: String,
    pub kind: String,
    pub at: String,
    pub effective_on: Option<String>,
    pub detail: Option<String>,
    pub late_days: i32,
    pub undone: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub pet: String,
    pub home_name: String,
    pub timezone: String,
    pub reminder_time: String,
    pub quiet_start: String,
    pub quiet_end: String,
    pub notifications: bool,
    pub onboarding_done: bool,
    /// UI-only flags with serde defaults so pre-0.1.5 databases keep working.
    #[serde(default)]
    pub tour_seen: bool,
    #[serde(default)]
    pub demo_home: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "ru".into(),
            theme: "system".into(),
            pet: "plant".into(),
            home_name: String::new(),
            timezone: iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".into()),
            reminder_time: "09:00".into(),
            quiet_start: "22:00".into(),
            quiet_end: "08:00".into(),
            notifications: true,
            onboarding_done: false,
            tour_seen: false,
            demo_home: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Health {
    pub score: u32,
    pub mood: String,
    pub completions: u32,
    pub level: u32,
    pub level_progress: u32,
    pub overdue: u32,
    pub today: u32,
    pub soon: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Snapshot {
    pub items: Vec<ItemView>,
    pub rooms: Vec<Room>,
    pub events: Vec<Event>,
    pub settings: Settings,
    pub health: Health,
    pub today: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Mutation {
    pub snapshot: Snapshot,
    pub undo_id: Option<String>,
    pub item_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/generated/")]
pub struct Preset {
    pub id: String,
    pub ru: String,
    pub en: String,
    pub room: String,
    pub icon: String,
    pub color: String,
    pub repeat: String,
    pub every: u32,
    pub shelf_life: bool,
}

#[derive(Clone, Debug)]
pub struct Reminder {
    pub key: String,
    pub item_id: String,
    pub name: String,
    pub due_on: String,
}
