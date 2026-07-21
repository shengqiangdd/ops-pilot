//! Baseline check results and report generation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::checks::{CheckResult, CheckStatus};

/// A complete baseline report for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineReport {
    pub host_id: String,
    pub timestamp: DateTime<Utc>,
    pub results: Vec<CheckResult>,
    pub score: u32,
}

/// Compute a compliance score (0–100) from check results.
pub fn compute_score(results: &[CheckResult]) -> u32 {
    if results.is_empty() {
        return 0;
    }
    let passed = results
        .iter()
        .filter(|r| r.status == CheckStatus::Pass)
        .count();
    ((passed as f64 / results.len() as f64) * 100.0).round() as u32
}

pub struct ReportStore {
    pool: SqlitePool,
}

impl ReportStore {
    pub async fn new(pool: SqlitePool) -> Self {
        let store = Self { pool };
        store.ensure_table().await;
        store
    }

    async fn ensure_table(&self) {
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS baseline_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                host_id TEXT NOT NULL,
                results_json TEXT NOT NULL,
                score INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await;
    }

    /// Save a baseline report.
    pub async fn save_report(
        &self,
        host_id: &str,
        results: &[CheckResult],
        score: u32,
    ) -> anyhow::Result<()> {
        let results_json = serde_json::to_string(results)?;
        sqlx::query(
            "INSERT INTO baseline_reports (host_id, results_json, score, created_at)
             VALUES (?, ?, ?, datetime('now'))",
        )
        .bind(host_id)
        .bind(&results_json)
        .bind(score as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get the latest baseline report for a host.
    pub async fn get_latest_report(&self, host_id: &str) -> anyhow::Result<Option<BaselineReport>> {
        let row: Option<(String, i64, String)> = sqlx::query_as(
            "SELECT host_id, score, results_json
             FROM baseline_reports
             WHERE host_id = ?
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(host_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((hid, score, results_json)) => {
                let results: Vec<CheckResult> = serde_json::from_str(&results_json)?;
                Ok(Some(BaselineReport {
                    host_id: hid,
                    timestamp: Utc::now(),
                    results,
                    score: score as u32,
                }))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_score_empty() {
        assert_eq!(compute_score(&[]), 0);
    }

    #[test]
    fn test_compute_score_all_pass() {
        let results = vec![
            CheckResult { name: "a".into(), category: "test".into(), status: CheckStatus::Pass, message: "ok".into(), remediation: None },
            CheckResult { name: "b".into(), category: "test".into(), status: CheckStatus::Pass, message: "ok".into(), remediation: None },
        ];
        assert_eq!(compute_score(&results), 100);
    }

    #[test]
    fn test_compute_score_half() {
        let results = vec![
            CheckResult { name: "a".into(), category: "test".into(), status: CheckStatus::Pass, message: "ok".into(), remediation: None },
            CheckResult { name: "b".into(), category: "test".into(), status: CheckStatus::Fail, message: "fail".into(), remediation: None },
        ];
        assert_eq!(compute_score(&results), 50);
    }

    #[tokio::test]
    async fn test_store_operations() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = ReportStore::new(pool).await;

        let results = vec![
            CheckResult { name: "check1".into(), category: "ssh".into(), status: CheckStatus::Pass, message: "ok".into(), remediation: None },
        ];
        store.save_report("host1", &results, 100).await.unwrap();

        let report = store.get_latest_report("host1").await.unwrap();
        assert!(report.is_some());
        assert_eq!(report.unwrap().score, 100);
    }
}
