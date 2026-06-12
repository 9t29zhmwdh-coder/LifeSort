CREATE TABLE IF NOT EXISTS scan_sessions (
    id          TEXT PRIMARY KEY,
    path        TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    file_count  INTEGER NOT NULL DEFAULT 0,
    status      TEXT NOT NULL DEFAULT 'running'
);

CREATE TABLE IF NOT EXISTS file_entries (
    id                  TEXT PRIMARY KEY,
    scan_session_id     TEXT NOT NULL REFERENCES scan_sessions(id),
    path                TEXT NOT NULL,
    name                TEXT NOT NULL,
    extension           TEXT,
    size                INTEGER NOT NULL DEFAULT 0,
    mime_type           TEXT NOT NULL DEFAULT '',
    kind                TEXT NOT NULL DEFAULT 'unknown',
    hash                TEXT,
    created_at          TEXT,
    modified_at         TEXT NOT NULL,
    exif_date           TEXT,
    width               INTEGER,
    height              INTEGER,
    category            TEXT,
    subcategory         TEXT,
    confidence          REAL,
    tags                TEXT,     -- JSON array
    extracted_date      TEXT,
    extracted_amount    REAL,
    extracted_sender    TEXT,
    ai_summary          TEXT,
    classified_by       TEXT,
    duplicate_group_id  TEXT
);

CREATE INDEX idx_files_session  ON file_entries(scan_session_id);
CREATE INDEX idx_files_kind     ON file_entries(kind);
CREATE INDEX idx_files_category ON file_entries(category);
CREATE INDEX idx_files_hash     ON file_entries(hash);
CREATE INDEX idx_files_size     ON file_entries(size);
