ALTER TABLE environments ADD COLUMN source_sync_status TEXT NOT NULL DEFAULT 'pending';
ALTER TABLE environments ADD COLUMN source_sync_error TEXT NULL;
ALTER TABLE environments ADD COLUMN source_synced_at TEXT NULL;
