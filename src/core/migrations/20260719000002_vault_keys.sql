-- Add vault key columns to users table for per-user credential encryption

ALTER TABLE users ADD COLUMN vault_key_encrypted TEXT;
ALTER TABLE users ADD COLUMN vault_password_hash TEXT;
