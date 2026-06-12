CREATE TABLE IF NOT EXISTS duplicate_groups (
    id           TEXT PRIMARY KEY,
    hash         TEXT NOT NULL,
    size         INTEGER NOT NULL,
    file_ids     TEXT NOT NULL,   -- JSON array
    keep_id      TEXT,
    wasted_bytes INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS organize_actions (
    id          TEXT PRIMARY KEY,
    file_id     TEXT NOT NULL,
    file_name   TEXT NOT NULL,
    kind        TEXT NOT NULL,
    source_path TEXT NOT NULL,
    target_path TEXT,
    reason      TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'pending',
    undoable    INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX idx_actions_status  ON organize_actions(status);
CREATE INDEX idx_actions_file_id ON organize_actions(file_id);
