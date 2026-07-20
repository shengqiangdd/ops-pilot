//! Knowledge base search — full-text and SQLite FTS5 retrieval.

use chrono::Utc;
use sqlx::SqlitePool;

use super::extraction::KnowledgeEntry;

pub struct KnowledgeStore {
    pool: SqlitePool,
}

impl KnowledgeStore {
    pub async fn new(pool: SqlitePool) -> Self {
        let store = Self { pool };
        store.ensure_table().await;
        store
    }

    async fn ensure_table(&self) {
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS knowledge_entries (
                id TEXT PRIMARY KEY,
                incident_id TEXT NOT NULL,
                title TEXT NOT NULL,
                root_cause TEXT NOT NULL,
                resolution TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await;
    }

    /// Insert a knowledge entry.
    pub async fn insert_entry(&self, entry: &KnowledgeEntry) -> anyhow::Result<()> {
        let tags_json = serde_json::to_string(&entry.tags)?;
        sqlx::query(
            "INSERT OR REPLACE INTO knowledge_entries
             (id, incident_id, title, root_cause, resolution, tags, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&entry.id)
        .bind(&entry.incident_id)
        .bind(&entry.title)
        .bind(&entry.root_cause)
        .bind(&entry.resolution)
        .bind(&tags_json)
        .bind(&entry.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Search knowledge entries by keyword (simple LIKE-based search).
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<KnowledgeEntry>> {
        let pattern = format!("%{}%", query);
        let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, incident_id, title, root_cause, resolution, tags, created_at
             FROM knowledge_entries
             WHERE title LIKE ? OR root_cause LIKE ? OR resolution LIKE ?
             ORDER BY created_at DESC LIMIT 20",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::new();
        for (id, incident_id, title, root_cause, resolution, tags_json, created_at) in rows {
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            entries.push(KnowledgeEntry {
                id,
                incident_id,
                title,
                root_cause,
                resolution,
                tags,
                created_at,
            });
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_operations() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = KnowledgeStore::new(pool).await;

        let entry = KnowledgeEntry {
            id: "k1".into(),
            incident_id: "INC-001".into(),
            title: "SSH connection timeout".into(),
            root_cause: "Network congestion".into(),
            resolution: "Restarted network interface".into(),
            tags: vec!["network".into()],
            created_at: Utc::now().to_rfc3339(),
        };

        store.insert_entry(&entry).await.unwrap();

        let results = store.search("SSH").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "SSH connection timeout");
    }
}
