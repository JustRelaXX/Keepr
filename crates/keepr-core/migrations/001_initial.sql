CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
CREATE TABLE rooms(id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT NOT NULL, color TEXT NOT NULL);
CREATE TABLE objects(
 id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT NOT NULL, color TEXT NOT NULL,
 room_id TEXT REFERENCES rooms(id) ON DELETE SET NULL,
 started_on TEXT NOT NULL, due_on TEXT NOT NULL, repeat TEXT NOT NULL CHECK(repeat IN ('once','days','weeks','months','years')),
 every INTEGER NOT NULL CHECK(every BETWEEN 1 AND 999), notifications INTEGER NOT NULL,
 advance_days INTEGER NOT NULL, notes TEXT NOT NULL, cycle_id TEXT NOT NULL,
 revision INTEGER NOT NULL, completed INTEGER NOT NULL DEFAULT 0, snoozed_until TEXT,
 deleted INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX objects_schedule ON objects(deleted, completed, due_on);
CREATE INDEX objects_room ON objects(room_id);
CREATE TABLE cycles(
 id TEXT PRIMARY KEY, object_id TEXT NOT NULL REFERENCES objects(id),
 started_on TEXT NOT NULL, due_on TEXT NOT NULL, completed_on TEXT NOT NULL
);
CREATE TABLE events(
 id TEXT PRIMARY KEY, item_id TEXT NOT NULL REFERENCES objects(id), name TEXT NOT NULL,
 icon TEXT NOT NULL, kind TEXT NOT NULL, at TEXT NOT NULL, effective_on TEXT,
 detail TEXT, late_days INTEGER NOT NULL DEFAULT 0, undone INTEGER NOT NULL DEFAULT 0,
 previous_json TEXT, result_revision INTEGER
);
CREATE INDEX events_at ON events(at DESC);
CREATE INDEX events_item ON events(item_id, at DESC);
CREATE TABLE notification_jobs(
 key TEXT PRIMARY KEY, item_id TEXT NOT NULL REFERENCES objects(id), status TEXT NOT NULL,
 attempts INTEGER NOT NULL DEFAULT 0, retry_at TEXT, sent_at TEXT
);
CREATE TABLE settings(id INTEGER PRIMARY KEY CHECK(id=1), data TEXT NOT NULL);
INSERT INTO schema_migrations VALUES(1, strftime('%Y-%m-%dT%H:%M:%SZ','now'));
