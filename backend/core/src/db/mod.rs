//! Database layer: migrations, connection pool, and query helpers.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use tracing::info;

/// Database wrapper around SQLite connection pool.
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    /// Create a new Database by connecting to the given SQLite path and running migrations.
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = Self::create_pool(database_url).await?;
        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    /// Create an in-memory database for testing.
    pub async fn open_in_memory() -> anyhow::Result<Self> {
        let pool = Self::create_pool("sqlite::memory:").await?;
        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    /// Create a connection pool to the given SQLite database URL.
    async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(pool)
    }

    /// Run pending migrations from the migrations/ directory.
    async fn run_migrations(&self) -> anyhow::Result<()> {
        let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        if migrations_dir.exists() {
            info!("Running migrations from: {}", migrations_dir.display());
            sqlx::migrate!()
                .run(&self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
            info!("Migrations completed successfully");
        } else {
            info!("No migrations directory found, skipping");
        }
        Ok(())
    }

    /// Close the connection pool.
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::FromRow;

    #[derive(FromRow)]
    struct CountRow {
        count: i64,
    }

    #[derive(FromRow)]
    struct ConnectionRow {
        name: String,
        host: String,
        status: String,
    }

    #[derive(FromRow)]
    struct AuditLogRow {
        user: String,
        action: String,
        resource: String,
        outcome: String,
    }

    #[tokio::test]
    async fn test_open_in_memory() {
        let db = Database::open_in_memory().await.unwrap();

        // Verify the tables exist
        let result: CountRow = sqlx::query_as("SELECT COUNT(*) as count FROM connections")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(result.count, 0);

        let result: CountRow = sqlx::query_as("SELECT COUNT(*) as count FROM audit_log")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(result.count, 0);
    }

    #[tokio::test]
    async fn test_insert_and_query_connection() {
        let db = Database::open_in_memory().await.unwrap();
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO connections (id, name, host, port, username, auth_type, tags, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind("test-server")
        .bind("192.168.1.100")
        .bind(22)
        .bind("admin")
        .bind("password")
        .bind("tag1,tag2")
        .bind("active")
        .execute(&db.pool)
        .await
        .unwrap();

        let row: ConnectionRow =
            sqlx::query_as("SELECT name, host, status FROM connections WHERE id = ?")
                .bind(&id)
                .fetch_one(&db.pool)
                .await
                .unwrap();

        assert_eq!(row.name, "test-server");
        assert_eq!(row.host, "192.168.1.100");
        assert_eq!(row.status, "active");
    }

    #[tokio::test]
    async fn test_insert_audit_log() {
        let db = Database::open_in_memory().await.unwrap();
        let log_id = uuid::Uuid::new_v4().to_string();

        // Insert audit log (new schema: user, action, resource, outcome)
        sqlx::query(
            r#"INSERT INTO audit_log (id, "user", action, resource, outcome)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&log_id)
        .bind("admin")
        .bind("connect")
        .bind("host/prod-web")
        .bind("success")
        .execute(&db.pool)
        .await
        .unwrap();

        let row: AuditLogRow = sqlx::query_as(
            r#"SELECT "user", action, resource, outcome FROM audit_log WHERE id = ?"#,
        )
        .bind(&log_id)
        .fetch_one(&db.pool)
        .await
        .unwrap();

        assert_eq!(row.user, "admin");
        assert_eq!(row.action, "connect");
        assert_eq!(row.resource, "host/prod-web");
        assert_eq!(row.outcome, "success");
    }
}
