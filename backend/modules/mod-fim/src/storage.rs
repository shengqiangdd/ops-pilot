//! Baseline and scan result storage in SQLite.

use sqlx::SqlitePool;

pub struct FimStore {
    pool: SqlitePool,
}

impl FimStore {
    pub async fn new(pool: SqlitePool) -> Self {
        let store = Self { pool };
        store.ensure_table().await;
        store
    }

    async fn ensure_table(&self) {
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS fim_baselines (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                host_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                sha256_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(host_id, file_path)
            )",
        )
        .execute(&self.pool)
        .await;
    }

    /// Insert or update a baseline entry.
    pub async fn upsert_baseline(
        &self,
        host_id: &str,
        file_path: &str,
        sha256_hash: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO fim_baselines (host_id, file_path, sha256_hash, created_at)
             VALUES (?, ?, ?, datetime('now'))
             ON CONFLICT(host_id, file_path)
             DO UPDATE SET sha256_hash = excluded.sha256_hash, created_at = datetime('now')",
        )
        .bind(host_id)
        .bind(file_path)
        .bind(sha256_hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get all baseline entries for a host.
    pub async fn get_baseline(&self, host_id: &str) -> anyhow::Result<Vec<(String, String)>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT file_path, sha256_hash FROM fim_baselines WHERE host_id = ?",
        )
        .bind(host_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Delete baseline for a host.
    pub async fn clear_baseline(&self, host_id: &str) -> anyhow::Result<u64> {
        let result = sqlx::query("DELETE FROM fim_baselines WHERE host_id = ?")
            .bind(host_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_operations() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = FimStore::new(pool).await;

        store.upsert_baseline("host1", "/etc/passwd", "abc123").await.unwrap();
        store.upsert_baseline("host1", "/etc/shadow", "def456").await.unwrap();

        let baseline = store.get_baseline("host1").await.unwrap();
        assert_eq!(baseline.len(), 2);

        let cleared = store.clear_baseline("host1").await.unwrap();
        assert_eq!(cleared, 2);

        let baseline = store.get_baseline("host1").await.unwrap();
        assert!(baseline.is_empty());
    }
}
