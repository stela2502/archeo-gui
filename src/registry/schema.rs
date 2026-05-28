pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS scan_runs (
    id TEXT PRIMARY KEY,
    root_path TEXT NOT NULL,
    profile_name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS file_entries (
    id TEXT PRIMARY KEY,

    scan_id TEXT NOT NULL,

    rel_path TEXT NOT NULL,
    file_name TEXT NOT NULL,

    extension TEXT,
    file_kind TEXT NOT NULL,

    size_bytes INTEGER NOT NULL,

    modified_at TEXT,

    included INTEGER NOT NULL,

    exclusion_reason TEXT,

    FOREIGN KEY(scan_id)
        REFERENCES scan_runs(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_file_entries_scan_id
    ON file_entries(scan_id);

CREATE INDEX IF NOT EXISTS idx_file_entries_rel_path
    ON file_entries(rel_path);

CREATE INDEX IF NOT EXISTS idx_file_entries_kind
    ON file_entries(file_kind);

CREATE TABLE IF NOT EXISTS artifact_groups (
    id TEXT PRIMARY KEY,

    scan_id TEXT NOT NULL,

    name TEXT NOT NULL,

    group_kind TEXT NOT NULL,

    ai_statement TEXT NOT NULL,

    confidence REAL NOT NULL,

    human_status TEXT NOT NULL,

    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(scan_id)
        REFERENCES scan_runs(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_artifact_groups_scan_id
    ON artifact_groups(scan_id);

CREATE INDEX IF NOT EXISTS idx_artifact_groups_kind
    ON artifact_groups(group_kind);

CREATE TABLE IF NOT EXISTS artifact_group_members (
    group_id TEXT NOT NULL,

    file_id TEXT NOT NULL,

    role TEXT,

    PRIMARY KEY(group_id, file_id),

    FOREIGN KEY(group_id)
        REFERENCES artifact_groups(id)
        ON DELETE CASCADE,

    FOREIGN KEY(file_id)
        REFERENCES file_entries(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_group_members_group
    ON artifact_group_members(group_id);

CREATE INDEX IF NOT EXISTS idx_group_members_file
    ON artifact_group_members(file_id);

CREATE TABLE IF NOT EXISTS prompt_snapshots (
    id TEXT PRIMARY KEY,

    scan_id TEXT NOT NULL,

    model TEXT NOT NULL,

    prompt TEXT NOT NULL,

    created_at TEXT NOT NULL,

    FOREIGN KEY(scan_id)
        REFERENCES scan_runs(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_prompt_snapshots_scan_id
    ON prompt_snapshots(scan_id);

CREATE TABLE IF NOT EXISTS ai_assumptions (
    id TEXT PRIMARY KEY,

    scan_id TEXT NOT NULL,

    scope_kind TEXT NOT NULL,

    scope_id TEXT NOT NULL,

    statement TEXT NOT NULL,

    confidence REAL NOT NULL,

    model TEXT NOT NULL,

    prompt_snapshot_id TEXT NOT NULL,

    human_status TEXT NOT NULL,

    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(scan_id)
        REFERENCES scan_runs(id)
        ON DELETE CASCADE,

    FOREIGN KEY(prompt_snapshot_id)
        REFERENCES prompt_snapshots(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_assumptions_scan_id
    ON ai_assumptions(scan_id);

CREATE INDEX IF NOT EXISTS idx_ai_assumptions_scope
    ON ai_assumptions(scope_kind, scope_id);

PRAGMA foreign_keys = ON;
"#;
