-- CMDB: Configuration Management Database

-- Services table
CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    owner TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_services_name ON services(name);
CREATE INDEX IF NOT EXISTS idx_services_status ON services(status);

-- Service-Host association table
CREATE TABLE IF NOT EXISTS service_hosts (
    id TEXT PRIMARY KEY NOT NULL,
    service_id TEXT NOT NULL,
    host_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'app',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE,
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE,
    UNIQUE(service_id, host_id)
);

CREATE INDEX IF NOT EXISTS idx_service_hosts_service ON service_hosts(service_id);
CREATE INDEX IF NOT EXISTS idx_service_hosts_host ON service_hosts(host_id);

-- Service dependencies table
CREATE TABLE IF NOT EXISTS service_dependencies (
    id TEXT PRIMARY KEY NOT NULL,
    source_service_id TEXT NOT NULL,
    target_service_id TEXT NOT NULL,
    dependency_type TEXT NOT NULL DEFAULT 'hard',
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (source_service_id) REFERENCES services(id) ON DELETE CASCADE,
    FOREIGN KEY (target_service_id) REFERENCES services(id) ON DELETE CASCADE,
    UNIQUE(source_service_id, target_service_id)
);

CREATE INDEX IF NOT EXISTS idx_service_deps_source ON service_dependencies(source_service_id);
CREATE INDEX IF NOT EXISTS idx_service_deps_target ON service_dependencies(target_service_id);

-- Config versions table
CREATE TABLE IF NOT EXISTS config_versions (
    id TEXT PRIMARY KEY NOT NULL,
    service_id TEXT NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    version INTEGER NOT NULL DEFAULT 1,
    changed_by TEXT NOT NULL DEFAULT '',
    change_note TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_config_versions_service ON config_versions(service_id);
CREATE INDEX IF NOT EXISTS idx_config_versions_version ON config_versions(service_id, version);
