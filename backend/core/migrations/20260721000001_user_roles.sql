-- Add user roles and role-based access control

-- Add role column to users table (default: operator)
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'operator';

-- Create roles table for custom role definitions (optional, for future use)
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    permissions TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create user_sessions table for login tracking
CREATE TABLE IF NOT EXISTS user_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_created_at ON user_sessions(created_at);

-- Insert default roles
INSERT OR IGNORE INTO roles (id, name, permissions) VALUES
    ('role-admin', 'admin', '["users:read","users:write","users:delete","hosts:read","hosts:write","hosts:delete","vault:read","vault:write","audit:read","modules:read","modules:write","settings:read","settings:write"]'),
    ('role-operator', 'operator', '["hosts:read","hosts:write","vault:read","vault:write","modules:read"]'),
    ('role-viewer', 'viewer', '["hosts:read","modules:read","audit:read"]');
