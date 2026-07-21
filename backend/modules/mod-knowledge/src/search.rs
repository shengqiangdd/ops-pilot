//! Knowledge base search — hybrid TF-IDF + SQLite LIKE retrieval.

use std::collections::HashMap;

use chrono::Utc;
use sqlx::SqlitePool;

use super::extraction::KnowledgeEntry;

/// TF-IDF search result with relevance score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: KnowledgeEntry,
    pub score: f64,
}

/// In-memory TF-IDF index for knowledge entries.
pub struct TfIdfIndex {
    doc_count: usize,
    /// term → (document_frequency, {doc_id → term_frequency})
    index: HashMap<String, (usize, HashMap<usize, usize>)>,
    /// All indexed documents
    doc_store: Vec<KnowledgeEntry>,
    /// doc_id lookup by entry.id
    id_map: HashMap<String, usize>,
}

impl TfIdfIndex {
    pub fn new() -> Self {
        Self {
            doc_count: 0,
            index: HashMap::new(),
            doc_store: Vec::new(),
            id_map: HashMap::new(),
        }
    }

    /// Tokenize a text string into lowercase terms.
    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() > 1)
            .map(|t| t.to_lowercase())
            .collect()
    }

    /// Add a document to the TF-IDF index.
    pub fn add_document(&mut self, doc: KnowledgeEntry) {
        let doc_id = self.doc_store.len();
        if let Some(old_id) = self.id_map.insert(doc.id.clone(), doc_id) {
            // Replace existing document — remove old tokens
            // (simplified: just increment doc_count; proper impl would decrement old DFs)
            let _ = old_id;
        }
        self.doc_store.push(doc);
        self.doc_count = self.doc_store.len();

        // Tokenize combined text (title + root_cause + resolution)
        let combined = format!("{} {} {}", &self.doc_store[doc_id].title, &self.doc_store[doc_id].root_cause, &self.doc_store[doc_id].resolution);
        let tokens = Self::tokenize(&combined);

        // Count term frequencies in this document
        let mut term_freqs: HashMap<String, usize> = HashMap::new();
        for token in &tokens {
            *term_freqs.entry(token.clone()).or_insert(0) += 1;
        }

        // Build inverted index: term → (df, {doc_id → tf})
        for (term, freq) in &term_freqs {
            let entry = self.index.entry(term.clone()).or_insert((0, HashMap::new()));
            if !entry.1.contains_key(&doc_id) {
                entry.0 += 1; // new document for this term
            }
            entry.1.insert(doc_id, *freq);
        }
    }

    /// TF: augmented term frequency — 0.5 + 0.5 * (freq / max_freq)
    fn tf(term_freq: usize, max_freq: usize) -> f64 {
        if max_freq == 0 {
            return 0.0;
        }
        0.5 + 0.5 * (term_freq as f64 / max_freq as f64)
    }

    /// IDF: Okapi BM25 style — log((N - n + 0.5) / (n + 0.5) + 1)
    fn idf(doc_freq: usize, total_docs: usize) -> f64 {
        let n = doc_freq as f64;
        let nn = total_docs as f64;
        ((nn - n + 0.5) / (n + 0.5) + 1.0).ln()
    }

    /// Compute TF-IDF score for a term in a specific document.
    fn tfidf_score(&self, term: &str, doc_id: usize) -> f64 {
        let (_, ref doc_freqs) = self.index.get(term).unwrap();
        let tf = match doc_freqs.get(&doc_id) {
            Some(&freq) => freq,
            None => return 0.0,
        };
        let max_freq = doc_freqs.values().copied().max().unwrap_or(1);
        let df = self.index.get(term).map(|(df, _)| *df).unwrap_or(0);

        Self::tf(tf, max_freq) * Self::idf(df, self.doc_count)
    }

    /// Search the index and return top-k results ranked by TF-IDF score.
    pub fn search(&self, query: &str, k: usize) -> Vec<SearchResult> {
        let query_terms = Self::tokenize(query);
        if query_terms.is_empty() || self.doc_count == 0 {
            return Vec::new();
        }

        // Score each document
        let mut scores: Vec<(usize, f64)> = (0..self.doc_count)
            .map(|doc_id| {
                let score: f64 = query_terms.iter()
                    .map(|term| self.tfidf_score(term, doc_id))
                    .sum();
                (doc_id, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);

        scores.into_iter()
            .map(|(doc_id, score)| SearchResult {
                entry: self.doc_store[doc_id].clone(),
                score,
            })
            .collect()
    }
}

/// Knowledge store with TF-IDF hybrid search.
pub struct KnowledgeStore {
    pool: SqlitePool,
    tfidf_index: tokio::sync::RwLock<TfIdfIndex>,
}

impl KnowledgeStore {
    pub async fn new(pool: SqlitePool) -> Self {
        let store = Self {
            pool,
            tfidf_index: tokio::sync::RwLock::new(TfIdfIndex::new()),
        };
        store.ensure_table().await;
        store.load_index().await;
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

    /// Load all existing entries into the TF-IDF index at startup.
    async fn load_index(&self) {
        if let Ok(rows) = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
            "SELECT id, incident_id, title, root_cause, resolution, tags, created_at FROM knowledge_entries",
        )
        .fetch_all(&self.pool)
        .await
        {
            let mut index = self.tfidf_index.write().await;
            for (id, incident_id, title, root_cause, resolution, tags_json, created_at) in rows {
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                index.add_document(KnowledgeEntry {
                    id, incident_id, title, root_cause, resolution, tags, created_at,
                });
            }
        }
    }

    /// Insert a knowledge entry and add to TF-IDF index.
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

        // Add to TF-IDF index
        let mut index = self.tfidf_index.write().await;
        index.add_document(entry.clone());

        Ok(())
    }

    /// Hybrid search: TF-IDF ranking with LIKE fallback.
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<KnowledgeEntry>> {
        // 1. Try TF-IDF search first
        let index = self.tfidf_index.read().await;
        let tfidf_results = index.search(query, 20);
        drop(index);

        if !tfidf_results.is_empty() {
            return Ok(tfidf_results.into_iter().map(|r| r.entry).collect());
        }

        // 2. Fallback to LIKE search
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

    #[test]
    fn test_tokenize() {
        let tokens = TfIdfIndex::tokenize("Hello, World! This is a TEST.");
        assert_eq!(tokens, vec!["hello", "world", "this", "is", "test"]);
    }

    #[test]
    fn test_tf_calculation() {
        // tf(5, 10) = 0.5 + 0.5 * (5/10) = 0.75
        assert!((TfIdfIndex::tf(5, 10) - 0.75).abs() < 1e-10);
        assert!((TfIdfIndex::tf(0, 10) - 0.5).abs() < 1e-10);
        assert!((TfIdfIndex::tf(10, 10) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_idf_calculation() {
        // idf(1, 10) = ln((10-1+0.5)/(1+0.5) + 1) = ln(7.333) ≈ 1.992
        let idf = TfIdfIndex::idf(1, 10);
        assert!(idf > 1.9 && idf < 2.1);
        // idf(10, 10) = ln((10-10+0.5)/(10+0.5) + 1) = ln(1.0476) ≈ 0.0465
        let idf_all = TfIdfIndex::idf(10, 10);
        assert!(idf_all >= 0.0 && idf_all < 0.1);
    }

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

    #[tokio::test]
    async fn test_tfidf_add_and_search() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = KnowledgeStore::new(pool).await;

        let e1 = KnowledgeEntry {
            id: "k1".into(), incident_id: "i1".into(),
            title: "Database connection pool exhausted".into(),
            root_cause: "Too many concurrent connections".into(),
            resolution: "Increase pool size and add connection timeout".into(),
            tags: vec!["database".into()],
            created_at: Utc::now().to_rfc3339(),
        };
        let e2 = KnowledgeEntry {
            id: "k2".into(), incident_id: "i2".into(),
            title: "SSH key rotation failed".into(),
            root_cause: "Expired certificate".into(),
            resolution: "Regenerate SSH keys".into(),
            tags: vec!["ssh".into()],
            created_at: Utc::now().to_rfc3339(),
        };

        store.insert_entry(&e1).await.unwrap();
        store.insert_entry(&e2).await.unwrap();

        let results = store.search("database connection").await.unwrap();
        assert!(!results.is_empty());
        // e1 should rank higher because "database" and "connection" appear in it
        assert_eq!(results[0].id, "k1");
    }

    #[tokio::test]
    async fn test_tfidf_ranking() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = KnowledgeStore::new(pool).await;

        // Document with "network" appearing many times vs once
        let e1 = KnowledgeEntry {
            id: "k1".into(), incident_id: "i1".into(),
            title: "Network network network failure".into(),
            root_cause: "network congestion".into(),
            resolution: "restart network interface".into(),
            tags: vec![],
            created_at: Utc::now().to_rfc3339(),
        };
        let e2 = KnowledgeEntry {
            id: "k2".into(), incident_id: "i2".into(),
            title: "Single network issue".into(),
            root_cause: "unknown".into(),
            resolution: "check logs".into(),
            tags: vec![],
            created_at: Utc::now().to_rfc3339(),
        };

        store.insert_entry(&e1).await.unwrap();
        store.insert_entry(&e2).await.unwrap();

        let results = store.search("network").await.unwrap();
        assert_eq!(results.len(), 2);
        // e1 has higher TF for "network"
        assert_eq!(results[0].id, "k1");
    }

    #[tokio::test]
    async fn test_empty_query() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = KnowledgeStore::new(pool).await;
        let results = store.search("").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_tfidf_empty_index() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = KnowledgeStore::new(pool).await;
        let results = store.search("anything").await.unwrap();
        assert!(results.is_empty());
    }
}
