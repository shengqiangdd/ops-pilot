-- Add owner_id column to hosts for per-user isolation

ALTER TABLE hosts ADD COLUMN owner_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_hosts_owner_id ON hosts(owner_id);
