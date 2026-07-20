//! Runbook plan — step definitions, dependency graph, approval gates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// A single step in a runbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStep {
    pub id: String,
    pub name: String,
    pub command: String,
    pub requires_approval: bool,
    pub timeout_seconds: u64,
}

/// A complete runbook with ordered steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub name: String,
    pub description: String,
    pub steps: Vec<RunbookStep>,
}

/// Result of running a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub status: String,
    pub output: String,
    pub duration_ms: u64,
}

/// Result of running an entire runbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub runbook_name: String,
    pub host_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub success: bool,
    pub steps: Vec<StepResult>,
}

/// Parse a natural language description into runbook steps.
pub fn parse_description_to_steps(description: &str) -> Vec<RunbookStep> {
    let mut steps = Vec::new();
    let lines: Vec<&str> = description.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let needs_approval = trimmed.to_lowercase().contains("confirm")
            || trimmed.to_lowercase().contains("approve")
            || trimmed.to_lowercase().contains("verify");

        steps.push(RunbookStep {
            id: format!("step-{}", i + 1),
            name: format!("Step {}: {}", i + 1, &trimmed[..trimmed.len().min(50)]),
            command: trimmed.to_string(),
            requires_approval: needs_approval,
            timeout_seconds: 300,
        });
    }

    if steps.is_empty() {
        steps.push(RunbookStep {
            id: "step-1".into(),
            name: "Execute command".into(),
            command: description.to_string(),
            requires_approval: false,
            timeout_seconds: 300,
        });
    }

    steps
}

pub struct RunbookStore {
    pool: SqlitePool,
}

impl RunbookStore {
    pub async fn new(pool: SqlitePool) -> Self {
        let store = Self { pool };
        store.ensure_table().await;
        store
    }

    async fn ensure_table(&self) {
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS runbooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                description TEXT NOT NULL,
                steps_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS runbook_executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                runbook_name TEXT NOT NULL,
                host_id TEXT NOT NULL,
                result_json TEXT NOT NULL,
                success INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await;
    }

    pub async fn save_runbook(&self, runbook: &Runbook) -> anyhow::Result<()> {
        let steps_json = serde_json::to_string(&runbook.steps)?;
        sqlx::query(
            "INSERT OR REPLACE INTO runbooks (name, description, steps_json) VALUES (?, ?, ?)",
        )
        .bind(&runbook.name)
        .bind(&runbook.description)
        .bind(&steps_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_runbook(&self, name: &str) -> anyhow::Result<Option<Runbook>> {
        let row: Option<(String, String, String)> =
            sqlx::query_as("SELECT name, description, steps_json FROM runbooks WHERE name = ?")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;

        match row {
            Some((name, description, steps_json)) => {
                let steps: Vec<RunbookStep> = serde_json::from_str(&steps_json)?;
                Ok(Some(Runbook {
                    name,
                    description,
                    steps,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn save_execution(
        &self,
        runbook_name: &str,
        result: &ExecutionResult,
    ) -> anyhow::Result<()> {
        let result_json = serde_json::to_string(result)?;
        sqlx::query(
            "INSERT INTO runbook_executions (runbook_name, host_id, result_json, success)
             VALUES (?, ?, ?, ?)",
        )
        .bind(runbook_name)
        .bind(&result.host_id)
        .bind(&result_json)
        .bind(result.success as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_steps() {
        let desc = "Check disk space\nRestart nginx service\nVerify service health";
        let steps = parse_description_to_steps(desc);
        assert_eq!(steps.len(), 3);
        assert!(!steps[0].requires_approval);
    }

    #[test]
    fn test_parse_steps_approval() {
        let desc = "Check disk space\nConfirm before restart";
        let steps = parse_description_to_steps(desc);
        assert!(steps[1].requires_approval);
    }
}
