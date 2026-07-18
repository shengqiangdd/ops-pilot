-- Evolve audit_log to support user/resource/outcome fields.
-- Recreate the table: the old schema (connection_id FK) is not needed for
-- the general-purpose audit trail.

DROP TABLE IF EXISTS audit_log;

CREATE TABLE audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    "user" TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    outcome TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
