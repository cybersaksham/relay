CREATE TABLE IF NOT EXISTS environment_sync_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_id TEXT NOT NULL REFERENCES environments(id),
    stream TEXT NOT NULL,
    chunk TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_environment_sync_events_environment_id_id
    ON environment_sync_events(environment_id, id);
