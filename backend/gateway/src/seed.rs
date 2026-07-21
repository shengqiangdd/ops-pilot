//! Seed data for demo/development purposes.
//!
//! Run with: ops-pilot seed

use sqlx::SqlitePool;
use tracing::info;

pub async fn run_seed(db: &SqlitePool) -> anyhow::Result<()> {
    info!("Seeding demo data...");

    // 1. Users (admin/operator/viewer)
    let users = vec![
        ("admin", "admin@opspilot.local", "admin", "admin"),
        ("operator", "ops@opspilot.local", "operator", "operator"),
        ("viewer", "view@opspilot.local", "viewer", "viewer"),
    ];
    for (username, email, role, password) in &users {
        sqlx::query(
            "INSERT OR IGNORE INTO users (id, username, email, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(username)
        .bind(email)
        .bind(format!("$argon2id$v=19$m=4096,t=3,p=1${}", password)) // placeholder hash
        .bind(role)
        .execute(db).await?;
    }
    info!("  Seeded {} users", users.len());

    // 2. Hosts (5 demo hosts)
    let hosts = vec![
        ("web-prod-1", "10.0.1.10", 22, "linux", "online"),
        ("web-prod-2", "10.0.1.11", 22, "linux", "online"),
        ("db-primary", "10.0.2.10", 22, "linux", "online"),
        ("db-replica", "10.0.2.11", 22, "linux", "online"),
        ("monitor-01", "10.0.3.10", 22, "linux", "offline"),
    ];
    for (name, ip, port, os, status) in &hosts {
        sqlx::query(
            "INSERT OR IGNORE INTO hosts (id, name, address, port, os_type, status, created_at) VALUES (?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(name)
        .bind(ip)
        .bind(port)
        .bind(os)
        .bind(status)
        .execute(db).await?;
    }
    info!("  Seeded {} hosts", hosts.len());

    // 3. Alert rules (5 demo rules)
    let rules = vec![
        ("high_cpu", "CPU > 90%", "warning", "cpu_usage > 90"),
        ("high_memory", "Memory > 85%", "warning", "memory_usage > 85"),
        ("disk_full", "Disk > 95%", "critical", "disk_usage > 95"),
        ("service_down", "Service unreachable", "critical", "ping_failed > 3"),
        ("night_batch", "Night batch ops", "info", "hour < 6 AND ops > 10"),
    ];
    for (name, desc, severity, condition) in &rules {
        sqlx::query(
            "INSERT OR IGNORE INTO alert_rules (id, name, description, severity, condition_expr, enabled, created_at) VALUES (?, ?, ?, ?, ?, 1, datetime('now'))"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(name)
        .bind(desc)
        .bind(severity)
        .bind(condition)
        .execute(db).await?;
    }
    info!("  Seeded {} alert rules", rules.len());

    // 4. Alert history (10 demo alerts)
    let alerts = vec![
        ("CPU spike on web-prod-1", "critical", "web-prod-1"),
        ("Memory warning on db-primary", "warning", "db-primary"),
        ("Disk usage high on monitor-01", "critical", "monitor-01"),
        ("SSH connection failed", "warning", "web-prod-2"),
        ("Service health check passed", "info", "web-prod-1"),
        ("Deployment completed successfully", "info", "web-prod-2"),
        ("Network latency spike", "warning", "db-replica"),
        ("SSL certificate expiring soon", "warning", "web-prod-1"),
        ("Container restarted", "info", "db-primary"),
        ("Firewall rule updated", "info", "monitor-01"),
    ];
    for (msg, severity, resource) in &alerts {
        sqlx::query(
            "INSERT INTO alert_history (id, message, severity, resource, created_at) VALUES (?, ?, ?, ?, datetime('now', '-' || abs(random() % 7200) || ' seconds'))"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(msg)
        .bind(severity)
        .bind(resource)
        .execute(db).await?;
    }
    info!("  Seeded {} alert history entries", alerts.len());

    // 5. Audit log (20 demo entries)
    let actions = vec![
        ("admin", "ssh.connect", "host:web-prod-1", "success"),
        ("admin", "config.update", "module:monitor", "success"),
        ("operator", "host.add", "host:db-replica", "success"),
        ("operator", "alert.acknowledge", "alert:cpu-spike", "success"),
        ("admin", "user.create", "user:operator", "success"),
        ("system", "health.check", "module:all", "success"),
        ("admin", "vault.unlock", "vault:secrets", "success"),
        ("operator", "runbook.execute", "runbook:restart-service", "success"),
        ("admin", "webhook.register", "webhook:slack", "success"),
        ("system", "backup.create", "db:ops-pilot", "success"),
        ("admin", "host.delete", "host:test-host", "success"),
        ("operator", "audit.export", "audit:csv", "success"),
        ("admin", "role.update", "role:operator", "success"),
        ("system", "certificate.renew", "cert:api.example.com", "success"),
        ("operator", "job.trigger", "job:health-check", "success"),
        ("admin", "security.scan", "host:web-prod-1", "success"),
        ("system", "metrics.collect", "host:db-primary", "success"),
        ("operator", "knowledge.create", "knowledge:ssh-timeout", "success"),
        ("admin", "channel.create", "channel:dingtalk", "success"),
        ("system", "incident.close", "incident:inc-001", "success"),
    ];
    for (user, action, resource, outcome) in &actions {
        sqlx::query(
            "INSERT INTO audit_log (id, \"user\", action, resource, outcome, created_at) VALUES (?, ?, ?, ?, ?, datetime('now', '-' || abs(random() % 86400) || ' seconds'))"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(user)
        .bind(action)
        .bind(resource)
        .bind(outcome)
        .execute(db).await?;
    }
    info!("  Seeded {} audit log entries", actions.len());

    // 6. Knowledge base entries (5)
    let knowledge = vec![
        ("SSH connection timeout", "Network congestion or firewall blocking port 22", "Check network connectivity, verify firewall rules, restart sshd service"),
        ("High CPU usage", "Runaway process or insufficient resources", "Identify top CPU consumers with top/htop, consider scaling up or optimizing code"),
        ("Disk space critical", "Log files or temp data consuming space", "Clean up old logs, expand disk, or move data to external storage"),
        ("Database slow queries", "Missing indexes or lock contention", "Analyze query plan, add indexes, optimize slow queries, check connection pool"),
        ("SSL certificate expiring", "Certificate approaching renewal date", "Renew certificate via certbot or manual process, update server configuration"),
    ];
    for (title, cause, resolution) in &knowledge {
        sqlx::query(
            "INSERT OR IGNORE INTO knowledge_entries (id, incident_id, title, root_cause, resolution, tags, created_at) VALUES (?, ?, ?, ?, ?, '[]', datetime('now'))"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(title)
        .bind(cause)
        .bind(resolution)
        .execute(db).await?;
    }
    info!("  Seeded {} knowledge entries", knowledge.len());

    // 7b. Ensure reports table exists (for OpsReport persistence)
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS reports (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            report_type TEXT NOT NULL,
            period_start TEXT NOT NULL,
            period_end TEXT NOT NULL,
            generated_at TEXT NOT NULL,
            summary_json TEXT NOT NULL,
            sections_json TEXT NOT NULL
        )"#
    )
    .execute(db)
    .await?;
    info!("  Ensured reports table");

    // 8. Ensure delivery_queue table exists (for retry/dead-letter)
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS delivery_queue (
            id TEXT PRIMARY KEY,
            channel_id TEXT NOT NULL,
            channel_type TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            retries INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            next_retry_at TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL
        )"#
    )
    .execute(db)
    .await?;
    info!("  Ensured delivery_queue table");

    // 9. Ensure dashboard_layouts table exists
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS dashboard_layouts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            layout_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"#
    )
    .execute(db)
    .await?;
    info!("  Ensured dashboard_layouts table");

    // 10. Ensure audit_logs table exists
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS audit_logs (
            id TEXT PRIMARY KEY,
            actor TEXT NOT NULL,
            action TEXT NOT NULL,
            resource TEXT NOT NULL,
            detail TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"#
    )
    .execute(db)
    .await?;
    info!("  Ensured audit_logs table");

    // 11. Ensure slow_queries table exists
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS slow_queries (
            id TEXT PRIMARY KEY,
            query_text TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"#
    )
    .execute(db)
    .await?;
    info!("  Ensured slow_queries table");

    info!("Seed complete!");
    Ok(())
}
