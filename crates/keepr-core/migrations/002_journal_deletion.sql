-- Keep domain facts for progress and undo while allowing users to clear their journal.
-- A new token per deletion prevents an old Undo action restoring a later deletion.
ALTER TABLE events ADD COLUMN deleted_token TEXT;
CREATE UNIQUE INDEX events_deleted_token ON events(deleted_token) WHERE deleted_token IS NOT NULL;
INSERT INTO schema_migrations VALUES(2, strftime('%Y-%m-%dT%H:%M:%SZ','now'));
