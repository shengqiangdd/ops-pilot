//! Enhanced LLM RCA engine with caching and structured output.
//!
//! Provides `LlmRcaEngine` which wraps the base `LlmAnalyzer` and adds:
//! - SQLite-backed result caching (table `rca_llm_cache`)
//! - Structured JSON output parsing for root cause, confidence, and fix steps
//! - Configurable cache TTL

use std::collections::HashMap;
use std::sync::Arc;

use ops_pilot_sdk::llm::{LlmClient, LlmError, Message};
use sqlx::SqlitePool;

use crate::analyzer::RcaResult;
use crate::llm_analyzer::{AnalyzerError, LlmAnalysisResult};

/// Structured result from LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRcaResult {
    /// The raw LLM analysis text.
    pub raw_analysis: String,
    /// Structured root cause extracted from LLM output.
    pub root_cause: String,
    /// Confidence score (0.0 - 1.0) parsed from LLM output.
    pub confidence: f64,
    /// Ordered list of suggested fix steps.
    pub fix_steps: Vec<String>,
    /// Whether this result came from cache.
    pub from_cache: bool,
    /// Rule-based results that preceded the LLM call.
    pub rule_results: Vec<RcaResult>,
}

/// Cached entry stored in SQLite.
#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    id: String,
    symptoms_hash: String,
    result_json: String,
    created_at: String,
}

/// Enhanced LLM RCA engine with caching.
pub struct LlmRcaEngine {
    client: Arc<dyn LlmClient>,
    db: SqlitePool,
}

impl LlmRcaEngine {
    pub fn new(client: Arc<dyn LlmClient>, db: SqlitePool) -> Self {
        Self { client, db }
    }

    /// Ensure the cache table exists.
    pub async fn ensure_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS rca_llm_cache (
                id TEXT PRIMARY KEY,
                symptoms_hash TEXT NOT NULL,
                result_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Hash the symptoms map for cache key.
    fn symptoms_hash(symptoms: &HashMap<String, f64>) -> String {
        let mut sorted: Vec<_> = symptoms.iter().collect();
        sorted.sort_by_key(|(k, _)| k.clone());
        let s: String = sorted.iter().map(|(k, v)| format!("{}={:.2}", k, v)).collect();
        use std::collections::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Look up a cached result by symptoms hash.
    pub async fn cache_lookup(&self, symptoms: &HashMap<String, f64>) -> anyhow::Result<Option<LlmRcaResult>> {
        let hash = Self::symptoms_hash(symptoms);
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT result_json FROM rca_llm_cache WHERE symptoms_hash = ?",
        )
        .bind(&hash)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some((json,)) => {
                let mut result: LlmRcaResult = serde_json::from_str(&json)?;
                result.from_cache = true;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Store a result in cache.
    pub async fn cache_store(&self, symptoms: &HashMap<String, f64>, result: &LlmRcaResult) -> anyhow::Result<()> {
        let hash = Self::symptoms_hash(symptoms);
        let json = serde_json::to_string(result)?;
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO rca_llm_cache (id, symptoms_hash, result_json) VALUES (?, ?, ?)",
        )
        .bind(&id)
        .bind(&hash)
        .bind(&json)
        .execute(&self.db)
        .await?;

        // Keep only last 50 entries
        sqlx::query(
            "DELETE FROM rca_llm_cache WHERE id NOT IN (SELECT id FROM rca_llm_cache ORDER BY created_at DESC LIMIT 50)",
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Analyze symptoms with LLM, using cache when available.
    pub async fn analyze(
        &self,
        symptoms: &HashMap<String, f64>,
        rule_results: &[RcaResult],
        description: &str,
        use_cache: bool,
    ) -> Result<LlmRcaResult, AnalyzerError> {
        // Check cache first
        if use_cache {
            if let Ok(Some(cached)) = self.cache_lookup(symptoms).await {
                return Ok(cached);
            }
        }

        // Build LLM prompt
        let messages = self.build_prompt(symptoms, rule_results, description);

        // Call LLM
        match self.client.complete(&messages).await {
            Ok(response) => {
                let parsed = Self::parse_response(&response, rule_results);
                // Store in cache
                let _ = self.cache_store(symptoms, &parsed).await;
                Ok(parsed)
            }
            Err(e) => {
                tracing::warn!("LLM RCA analysis failed: {}", e);
                Ok(LlmRcaResult {
                    raw_analysis: format!("LLM unavailable: {e}"),
                    root_cause: "Unable to determine root cause".to_string(),
                    confidence: 0.0,
                    fix_steps: vec![],
                    from_cache: false,
                    rule_results: rule_results.to_vec(),
                })
            }
        }
    }

    /// Build the LLM prompt messages.
    fn build_prompt(
        &self,
        symptoms: &HashMap<String, f64>,
        rule_results: &[RcaResult],
        description: &str,
    ) -> Vec<Message> {
        let symptoms_str = symptoms
            .iter()
            .map(|(k, v)| format!("  - {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let rules_str = if rule_results.is_empty() {
            "  (no rules matched)".to_string()
        } else {
            rule_results
                .iter()
                .map(|r| format!(
                    "  - [{}] {} (confidence: {:.0}%): {}",
                    r.severity, r.rule_name, r.confidence * 100.0, r.description
                ))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let system_prompt = "You are an expert SRE/infrastructure operations assistant. \
            Analyze system symptoms and rule-based diagnostic results. \
            Return your analysis as a JSON object with these fields: \
            {\"root_cause\": \"...\", \"confidence\": 0.0-1.0, \"fix_steps\": [\"step1\", \"step2\"]}. \
            Be specific and technical. Only output valid JSON, no markdown fences.";

        let user_prompt = format!(
            "Analyze this infrastructure issue:\n\n\
             ## Issue\n{description}\n\n\
             ## Symptoms\n{symptoms_str}\n\n\
             ## Rule-Based Findings\n{rules_str}\n\n\
             Provide root cause analysis as JSON."
        );

        vec![Message::system(system_prompt), Message::user(user_prompt)]
    }

    /// Parse LLM response into structured LlmRcaResult.
    fn parse_response(response: &str, rule_results: &[RcaResult]) -> LlmRcaResult {
        // Try to extract JSON from response (may be wrapped in markdown fences)
        let json_str = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: Option<ParsedLlmOutput> = serde_json::from_str(json_str).ok();

        LlmRcaResult {
            raw_analysis: response.to_string(),
            root_cause: parsed
                .as_ref()
                .map(|p| p.root_cause.clone())
                .unwrap_or_else(|| response.lines().next().unwrap_or("Unknown").to_string()),
            confidence: parsed.as_ref().map(|p| p.confidence).unwrap_or(0.5),
            fix_steps: parsed
                .as_ref()
                .map(|p| p.fix_steps.clone())
                .unwrap_or_default(),
            from_cache: false,
            rule_results: rule_results.to_vec(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ParsedLlmOutput {
    root_cause: String,
    confidence: f64,
    fix_steps: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ops_pilot_sdk::llm::{LlmError, Message};
    use std::pin::Pin;

    struct MockLlm {
        response: String,
    }

    #[async_trait]
    impl LlmClient for MockLlm {
        async fn complete(&self, _msgs: &[Message]) -> Result<String, LlmError> {
            Ok(self.response.clone())
        }
        async fn complete_stream(
            &self,
            _msgs: &[Message],
        ) -> Result<Pin<Box<dyn futures_util::Stream<Item = Result<String, LlmError>> + Send>>, LlmError> {
            Err(LlmError::StreamClosed)
        }
    }

    #[tokio::test]
    async fn test_parse_json_response() {
        let response = r#"{"root_cause": "memory leak in worker", "confidence": 0.85, "fix_steps": ["restart workers", "profile memory"]}"#;
        let result = LlmRcaEngine::parse_response(response, &[]);
        assert_eq!(result.root_cause, "memory leak in worker");
        assert_eq!(result.confidence, 0.85);
        assert_eq!(result.fix_steps.len(), 2);
        assert!(!result.from_cache);
    }

    #[tokio::test]
    async fn test_parse_markdown_wrapped_json() {
        let response = "```json\n{\"root_cause\": \"disk full\", \"confidence\": 0.9, \"fix_steps\": [\"clean logs\"]}\n```";
        let result = LlmRcaEngine::parse_response(response, &[]);
        assert_eq!(result.root_cause, "disk full");
        assert_eq!(result.confidence, 0.9);
    }

    #[tokio::test]
    async fn test_parse_non_json_fallback() {
        let response = "The root cause is likely a memory leak in the application.";
        let result = LlmRcaEngine::parse_response(response, &[]);
        assert!(result.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_symptoms_hash_deterministic() {
        let mut m1 = HashMap::new();
        m1.insert("cpu".into(), 90.0);
        m1.insert("mem".into(), 80.0);
        let mut m2 = HashMap::new();
        m2.insert("mem".into(), 80.0);
        m2.insert("cpu".into(), 90.0);
        assert_eq!(LlmRcaEngine::symptoms_hash(&m1), LlmRcaEngine::symptoms_hash(&m2));
    }

    #[tokio::test]
    async fn test_analyze_with_cache() {
        let pool = setup_db().await;
        let client = Arc::new(MockLlm {
            response: r#"{"root_cause": "test cause", "confidence": 0.7, "fix_steps": ["fix1"]}"#.into(),
        });
        let engine = LlmRcaEngine::new(client, pool);
        engine.ensure_table().await.unwrap();

        let mut symptoms = HashMap::new();
        symptoms.insert("cpu".into(), 95.0);

        // First call - should hit LLM
        let r1 = engine.analyze(&symptoms, &[], "test", true).await.unwrap();
        assert!(!r1.from_cache);

        // Second call - should hit cache
        let r2 = engine.analyze(&symptoms, &[], "test", true).await.unwrap();
        assert!(r2.from_cache);
    }

    #[tokio::test]
    async fn test_analyze_skip_cache() {
        let pool = setup_db().await;
        let client = Arc::new(MockLlm {
            response: r#"{"root_cause": "fresh analysis", "confidence": 0.8, "fix_steps": []}"#.into(),
        });
        let engine = LlmRcaEngine::new(client, pool);
        engine.ensure_table().await.unwrap();

        let mut symptoms = HashMap::new();
        symptoms.insert("cpu".into(), 95.0);

        // First call
        let _ = engine.analyze(&symptoms, &[], "test", true).await.unwrap();

        // Second call with use_cache=false
        let r = engine.analyze(&symptoms, &[], "test", false).await.unwrap();
        assert!(!r.from_cache);
    }

    async fn setup_db() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }
}
