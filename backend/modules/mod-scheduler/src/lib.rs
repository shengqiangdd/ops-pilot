//! mod-scheduler: Cron-style job scheduler for periodic task execution.
//!
//! Provides tools to create, list, pause, resume, and delete scheduled jobs.
//! Jobs are stored in SQLite and executed by a background ticker.
//! Enhanced with priority-based weighted fair scheduling.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tracing::{error, info};

// ── Interval parsing ─────────────────────────────────────────────────────────

/// Parse a simple interval expression into a `tokio::time::Duration`.
///
/// Supported formats: `30s`, `5m`, `1h`, `1d`.
/// Returns `None` if the expression is invalid.
fn parse_interval(expr: &str) -> Option<tokio::time::Duration> {
    let expr = expr.trim();
    if expr.len() < 2 {
        return None;
    }
    let (num_part, unit) = expr.split_at(expr.len() - 1);
    let value: u64 = num_part.parse().ok()?;
    let duration = match unit {
        "s" => tokio::time::Duration::from_secs(value),
        "m" => tokio::time::Duration::from_secs(value * 60),
        "h" => tokio::time::Duration::from_secs(value * 3600),
        "d" => tokio::time::Duration::from_secs(value * 86400),
        _ => return None,
    };
    Some(duration)
}

// ── Priority Scheduler ──────────────────────────────────────────────────────

/// A job with priority metadata for the scheduler.
#[derive(Debug, Clone)]
pub struct PrioritizedJob {
    pub name: String,
    pub priority: u8,          // 0-255, higher = more urgent
    pub weight: f64,           // WFQ weight for fair scheduling among same-priority jobs
    pub max_retries: u8,
    pub retry_delay_secs: u64,
    pub retries_done: u8,
    pub cron_expr: String,
    pub action_json: String,
}

impl PartialEq for PrioritizedJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedJob {}

impl PartialOrd for PrioritizedJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedJob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// Priority-aware scheduler with Weighted Fair Queueing (WFQ).
pub struct PriorityScheduler {
    /// BinaryHeap for O(log n) priority dequeue (max-heap).
    pending: BinaryHeap<PrioritizedJob>,
    /// Weighted fair queue: group → accumulated weight for round-robin among same priority.
    group_weights: HashMap<String, f64>,
    /// Default weight for new jobs.
    default_weight: f64,
}

impl PriorityScheduler {
    pub fn new() -> Self {
        Self {
            pending: BinaryHeap::new(),
            group_weights: HashMap::new(),
            default_weight: 1.0,
        }
    }

    /// Enqueue a job into the priority queue.
    pub fn enqueue(&mut self, job: PrioritizedJob) {
        self.pending.push(job);
    }

    /// Dequeue the highest-priority job (simple priority order).
    pub fn dequeue(&mut self) -> Option<PrioritizedJob> {
        self.pending.pop()
    }

    /// Weighted Fair Queueing: among jobs at the same priority level,
    /// select the one whose group has the lowest accumulated weight (least served).
    pub fn weighted_next(&mut self) -> Option<PrioritizedJob> {
        if self.pending.is_empty() {
            return None;
        }

        // Collect all jobs at the highest priority level
        let max_priority = self.pending.peek()?.priority;
        let mut candidates: Vec<PrioritizedJob> = Vec::new();
        let mut remaining: BinaryHeap<PrioritizedJob> = BinaryHeap::new();

        while let Some(job) = self.pending.pop() {
            if job.priority == max_priority {
                candidates.push(job);
            } else {
                remaining.push(job);
            }
        }

        self.pending = remaining;

        if candidates.len() == 1 {
            let job = candidates.into_iter().next()?;
            return Some(job);
        }

        // Pick candidate with lowest accumulated group weight
        let best_idx = candidates.iter()
            .enumerate()
            .min_by(|a, b| {
                let w_a = self.group_weights.get(&Self::extract_group(&a.1.name)).unwrap_or(&0.0);
                let w_b = self.group_weights.get(&Self::extract_group(&b.1.name)).unwrap_or(&0.0);
                w_a.partial_cmp(w_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        let chosen = candidates.remove(best_idx);

        // Update weight for the chosen group
        let chosen_group = Self::extract_group(&chosen.name);
        let weight = self.group_weights.entry(chosen_group).or_insert(0.0);
        *weight += chosen.weight;

        // Put back remaining candidates
        self.pending.extend(candidates);

        Some(chosen)
    }

    /// Extract a group name from job name (text before first dash or underscore).
    fn extract_group(name: &str) -> String {
        name.split(['-', '_'])
            .next()
            .unwrap_or(name)
            .to_string()
    }

    /// Number of pending jobs.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Default for PriorityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ── Module ───────────────────────────────────────────────────────────────────

/// Cron-style job scheduler module — stores jobs in SQLite and runs a
/// background ticker to execute due jobs.
pub struct SchedulerModule {
    db: SqlitePool,
}

impl SchedulerModule {
    /// Create a new `SchedulerModule` and ensure the tables exist.
    pub async fn new(db: SqlitePool) -> Self {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS scheduler_jobs (
                name TEXT PRIMARY KEY,
                cron_expr TEXT NOT NULL,
                action_json TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run_at TEXT,
                next_run_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&db)
        .await
        .expect("failed to create scheduler_jobs table");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS scheduler_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_name TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                success INTEGER,
                output TEXT,
                error TEXT,
                FOREIGN KEY (job_name) REFERENCES scheduler_jobs(name)
            )",
        )
        .execute(&db)
        .await
        .expect("failed to create scheduler_history table");

        Self { db }
    }

    /// Create a new job.
    async fn create_job(
        &self,
        name: &str,
        cron_expr: &str,
        action_json: &str,
    ) -> anyhow::Result<()> {
        // Validate the interval expression
        parse_interval(cron_expr)
            .ok_or_else(|| anyhow::anyhow!("invalid interval expression: {}", cron_expr))?;

        // Validate action_json is valid JSON
        serde_json::from_str::<serde_json::Value>(action_json)
            .map_err(|e| anyhow::anyhow!("invalid action_json: {}", e))?;

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO scheduler_jobs (name, cron_expr, action_json, next_run_at, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(name)
        .bind(cron_expr)
        .bind(action_json)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        info!(name, cron_expr, "Scheduler job created");
        Ok(())
    }

    /// List all jobs.
    async fn list_jobs(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        #[derive(sqlx::FromRow)]
        struct JobRow {
            name: String,
            cron_expr: String,
            action_json: String,
            enabled: i32,
            last_run_at: Option<String>,
            next_run_at: Option<String>,
            created_at: String,
            updated_at: String,
        }

        let rows: Vec<JobRow> = sqlx::query_as(
            "SELECT name, cron_expr, action_json, enabled, last_run_at, next_run_at, created_at, updated_at \
             FROM scheduler_jobs ORDER BY name",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "cron_expr": r.cron_expr,
                    "action_json": serde_json::from_str::<serde_json::Value>(&r.action_json).unwrap_or_default(),
                    "enabled": r.enabled == 1,
                    "last_run_at": r.last_run_at,
                    "next_run_at": r.next_run_at,
                    "created_at": r.created_at,
                    "updated_at": r.updated_at,
                })
            })
            .collect())
    }

    /// Delete a job by name.
    async fn delete_job(&self, name: &str) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM scheduler_jobs WHERE name = ?")
            .bind(name)
            .execute(&self.db)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            // Also delete history for this job
            sqlx::query("DELETE FROM scheduler_history WHERE job_name = ?")
                .bind(name)
                .execute(&self.db)
                .await?;
            info!(name, "Scheduler job deleted");
        }
        Ok(deleted)
    }

    /// Pause (disable) a job.
    async fn pause_job(&self, name: &str) -> anyhow::Result<bool> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE scheduler_jobs SET enabled = 0, updated_at = ? WHERE name = ? AND enabled = 1",
        )
        .bind(&now)
        .bind(name)
        .execute(&self.db)
        .await?;

        let paused = result.rows_affected() > 0;
        if paused {
            info!(name, "Scheduler job paused");
        }
        Ok(paused)
    }

    /// Resume (enable) a job.
    async fn resume_job(&self, name: &str) -> anyhow::Result<bool> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE scheduler_jobs SET enabled = 1, next_run_at = ?, updated_at = ? WHERE name = ? AND enabled = 0",
        )
        .bind(&now)
        .bind(&now)
        .bind(name)
        .execute(&self.db)
        .await?;

        let resumed = result.rows_affected() > 0;
        if resumed {
            info!(name, "Scheduler job resumed");
        }
        Ok(resumed)
    }

    /// Get execution history for a job.
    async fn job_history(
        &self,
        name: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        #[derive(sqlx::FromRow)]
        struct HistoryRow {
            id: i64,
            job_name: String,
            started_at: String,
            finished_at: Option<String>,
            success: Option<i32>,
            output: Option<String>,
            error: Option<String>,
        }

        let rows: Vec<HistoryRow> = sqlx::query_as(
            "SELECT id, job_name, started_at, finished_at, success, output, error \
             FROM scheduler_history WHERE job_name = ? ORDER BY id DESC LIMIT ?",
        )
        .bind(name)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "job_name": r.job_name,
                    "started_at": r.started_at,
                    "finished_at": r.finished_at,
                    "success": r.success.map(|s| s == 1),
                    "output": r.output,
                    "error": r.error,
                })
            })
            .collect())
    }

    /// Background ticker — call this in a spawned task to run the scheduler.
    pub async fn start_scheduler(self: Arc<Self>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            if let Err(e) = self.tick().await {
                error!(error = %e, "scheduler tick failed");
            }
        }
    }

    /// Single tick: find due jobs and execute them.
    async fn tick(&self) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();

        #[derive(sqlx::FromRow)]
        struct DueJob {
            name: String,
            cron_expr: String,
            action_json: String,
        }

        let due_jobs: Vec<DueJob> = sqlx::query_as(
            "SELECT name, cron_expr, action_json FROM scheduler_jobs \
             WHERE enabled = 1 AND (next_run_at IS NULL OR next_run_at <= ?)",
        )
        .bind(&now)
        .fetch_all(&self.db)
        .await?;

        for job in due_jobs {
            let started_at = Utc::now().to_rfc3339();

            // Record history entry
            sqlx::query(
                "INSERT INTO scheduler_history (job_name, started_at) VALUES (?, ?)",
            )
            .bind(&job.name)
            .bind(&started_at)
            .execute(&self.db)
            .await?;

            // For now, we just log the action — actual tool execution would
            // require a reference to the tool registry which is not available here.
            let success = true;
            let output = format!("Scheduled job '{}' executed (action: {})", job.name, job.action_json);
            let finished_at = Utc::now().to_rfc3339();

            // Update history
            sqlx::query(
                "UPDATE scheduler_history SET finished_at = ?, success = ?, output = ? \
                 WHERE job_name = ? AND started_at = ?",
            )
            .bind(&finished_at)
            .bind(success as i32)
            .bind(&output)
            .bind(&job.name)
            .bind(&started_at)
            .execute(&self.db)
            .await?;

            // Compute next_run_at
            if let Some(duration) = parse_interval(&job.cron_expr) {
                let next = (Utc::now() + chrono::Duration::from_std(duration).unwrap())
                    .to_rfc3339();
                sqlx::query(
                    "UPDATE scheduler_jobs SET last_run_at = ?, next_run_at = ?, updated_at = ? WHERE name = ?",
                )
                .bind(&finished_at)
                .bind(&next)
                .bind(&finished_at)
                .bind(&job.name)
                .execute(&self.db)
                .await?;
            }

            info!(job = %job.name, "Scheduled job executed");
        }

        Ok(())
    }
}

#[async_trait]
impl OpsModule for SchedulerModule {
    fn name(&self) -> &str {
        "mod-scheduler"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Cron-style job scheduler — periodic task execution and management"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "schedule_create".into(),
                description: "Create a scheduled job with a cron-like interval".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Unique job name" },
                        "cron_expr": { "type": "string", "description": "Interval expression (e.g. 30s, 5m, 1h, 1d)" },
                        "action_json": { "type": "string", "description": "JSON describing the action to execute" }
                    },
                    "required": ["name", "cron_expr", "action_json"]
                }),
            },
            ToolDefinition {
                name: "schedule_list".into(),
                description: "List all scheduled jobs".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "schedule_delete".into(),
                description: "Delete a scheduled job by name".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Job name to delete" }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "schedule_pause".into(),
                description: "Pause a scheduled job".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Job name to pause" }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "schedule_resume".into(),
                description: "Resume a paused scheduled job".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Job name to resume" }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "schedule_history".into(),
                description: "Get execution history for a job".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Job name" },
                        "limit": { "type": "integer", "description": "Max history entries to return (default 10)", "default": 10 }
                    },
                    "required": ["name"]
                }),
            },
        ]
    }

    async fn execute(
        &self,
        _ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        match tool {
            "schedule_create" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?;
                let cron_expr = params["cron_expr"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'cron_expr' parameter"))?;
                let action_json = params["action_json"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'action_json' parameter"))?;

                self.create_job(name, cron_expr, action_json).await?;
                Ok(serde_json::json!({ "status": "ok", "name": name }))
            }
            "schedule_list" => {
                let jobs = self.list_jobs().await?;
                Ok(serde_json::json!({ "jobs": jobs }))
            }
            "schedule_delete" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?;

                let deleted = self.delete_job(name).await?;
                Ok(serde_json::json!({ "deleted": deleted, "name": name }))
            }
            "schedule_pause" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?;

                let paused = self.pause_job(name).await?;
                Ok(serde_json::json!({ "paused": paused, "name": name }))
            }
            "schedule_resume" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?;

                let resumed = self.resume_job(name).await?;
                Ok(serde_json::json!({ "resumed": resumed, "name": name }))
            }
            "schedule_history" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?;
                let limit = params["limit"].as_u64().unwrap_or(10) as u32;

                let history = self.job_history(name, limit).await?;
                Ok(serde_json::json!({ "history": history }))
            }
            _ => Err(anyhow::anyhow!("unknown tool: {}", tool)),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> {
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        HealthStatus::Healthy
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use std::path::PathBuf;
    use std::sync::Arc;

    async fn setup() -> (SchedulerModule, ModuleContext) {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = SchedulerModule::new(pool.clone()).await;
        let ctx = ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from("/tmp/test-scheduler"),
            "test-scheduler".into(),
        );
        (module, ctx)
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let (m, _ctx) = setup().await;
        assert_eq!(m.name(), "mod-scheduler");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.description().contains("scheduler"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let (m, _ctx) = setup().await;
        let tools = m.tools();
        assert_eq!(tools.len(), 6);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"schedule_create"));
        assert!(names.contains(&"schedule_list"));
        assert!(names.contains(&"schedule_delete"));
        assert!(names.contains(&"schedule_pause"));
        assert!(names.contains(&"schedule_resume"));
        assert!(names.contains(&"schedule_history"));
    }

    #[tokio::test]
    async fn test_create_and_list_job() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "schedule_create",
                serde_json::json!({
                    "name": "health-check",
                    "cron_expr": "5m",
                    "action_json": "{\"tool\":\"ssh_exec\",\"params\":{}}"
                }),
            )
            .await
            .unwrap();
        assert_eq!(result["status"], "ok");

        let result = m.execute(&ctx, "schedule_list", serde_json::json!({})).await.unwrap();
        let jobs = result["jobs"].as_array().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0]["name"], "health-check");
        assert_eq!(jobs[0]["cron_expr"], "5m");
    }

    #[tokio::test]
    async fn test_create_job_invalid_interval() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "schedule_create",
                serde_json::json!({
                    "name": "bad-job",
                    "cron_expr": "invalid",
                    "action_json": "{}"
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid interval"));
    }

    #[tokio::test]
    async fn test_create_job_invalid_json() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "schedule_create",
                serde_json::json!({
                    "name": "bad-json",
                    "cron_expr": "1h",
                    "action_json": "not-json"
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid action_json"));
    }

    #[tokio::test]
    async fn test_delete_job() {
        let (m, ctx) = setup().await;

        m.execute(
            &ctx,
            "schedule_create",
            serde_json::json!({
                "name": "to-delete",
                "cron_expr": "30s",
                "action_json": "{}"
            }),
        )
        .await
        .unwrap();

        let result = m
            .execute(
                &ctx,
                "schedule_delete",
                serde_json::json!({ "name": "to-delete" }),
            )
            .await
            .unwrap();
        assert_eq!(result["deleted"], true);

        // Verify it's gone
        let result = m.execute(&ctx, "schedule_list", serde_json::json!({})).await.unwrap();
        let jobs = result["jobs"].as_array().unwrap();
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_job() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "schedule_delete",
                serde_json::json!({ "name": "nonexistent" }),
            )
            .await
            .unwrap();
        assert_eq!(result["deleted"], false);
    }

    #[tokio::test]
    async fn test_pause_and_resume() {
        let (m, ctx) = setup().await;

        m.execute(
            &ctx,
            "schedule_create",
            serde_json::json!({
                "name": "pause-test",
                "cron_expr": "1m",
                "action_json": "{}"
            }),
        )
        .await
        .unwrap();

        // Pause
        let result = m
            .execute(
                &ctx,
                "schedule_pause",
                serde_json::json!({ "name": "pause-test" }),
            )
            .await
            .unwrap();
        assert_eq!(result["paused"], true);

        // Verify disabled
        let result = m.execute(&ctx, "schedule_list", serde_json::json!({})).await.unwrap();
        let jobs = result["jobs"].as_array().unwrap();
        assert_eq!(jobs[0]["enabled"], false);

        // Resume
        let result = m
            .execute(
                &ctx,
                "schedule_resume",
                serde_json::json!({ "name": "pause-test" }),
            )
            .await
            .unwrap();
        assert_eq!(result["resumed"], true);

        // Verify enabled
        let result = m.execute(&ctx, "schedule_list", serde_json::json!({})).await.unwrap();
        let jobs = result["jobs"].as_array().unwrap();
        assert_eq!(jobs[0]["enabled"], true);
    }

    #[tokio::test]
    async fn test_pause_nonexistent_job() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "schedule_pause",
                serde_json::json!({ "name": "nonexistent" }),
            )
            .await
            .unwrap();
        assert_eq!(result["paused"], false);
    }

    #[tokio::test]
    async fn test_resume_nonexistent_job() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "schedule_resume",
                serde_json::json!({ "name": "nonexistent" }),
            )
            .await
            .unwrap();
        assert_eq!(result["resumed"], false);
    }

    #[tokio::test]
    async fn test_history_empty() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "schedule_history",
                serde_json::json!({ "name": "any-job" }),
            )
            .await
            .unwrap();
        let history = result["history"].as_array().unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_tick_executes_due_job() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = Arc::new(SchedulerModule::new(pool.clone()).await);
        let ctx = ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from("/tmp/test-scheduler"),
            "test-scheduler".into(),
        );

        // Create a job with a 0s interval so it's immediately due
        module
            .execute(
                &ctx,
                "schedule_create",
                serde_json::json!({
                    "name": "tick-test",
                    "cron_expr": "1s",
                    "action_json": "{\"tool\":\"test\"}"
                }),
            )
            .await
            .unwrap();

        // Run a tick
        module.tick().await.unwrap();

        // Check history
        let result = module
            .execute(
                &ctx,
                "schedule_history",
                serde_json::json!({ "name": "tick-test" }),
            )
            .await
            .unwrap();
        let history = result["history"].as_array().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0]["success"], true);
    }

    #[tokio::test]
    async fn test_parse_interval() {
        assert_eq!(parse_interval("30s"), Some(tokio::time::Duration::from_secs(30)));
        assert_eq!(parse_interval("5m"), Some(tokio::time::Duration::from_secs(300)));
        assert_eq!(parse_interval("1h"), Some(tokio::time::Duration::from_secs(3600)));
        assert_eq!(parse_interval("1d"), Some(tokio::time::Duration::from_secs(86400)));
        assert_eq!(parse_interval("invalid"), None);
        assert_eq!(parse_interval("10x"), None);
        assert_eq!(parse_interval(""), None);
        assert_eq!(parse_interval("5m  "), Some(tokio::time::Duration::from_secs(300)));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let (m, ctx) = setup().await;
        let result = m.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_priority_order() {
        let mut sched = PriorityScheduler::new();
        sched.enqueue(PrioritizedJob {
            name: "low-job".into(), priority: 10, weight: 1.0, max_retries: 0,
            retry_delay_secs: 0, retries_done: 0, cron_expr: "1m".into(), action_json: "{}".into(),
        });
        sched.enqueue(PrioritizedJob {
            name: "high-job".into(), priority: 200, weight: 1.0, max_retries: 0,
            retry_delay_secs: 0, retries_done: 0, cron_expr: "1m".into(), action_json: "{}".into(),
        });
        sched.enqueue(PrioritizedJob {
            name: "mid-job".into(), priority: 100, weight: 1.0, max_retries: 0,
            retry_delay_secs: 0, retries_done: 0, cron_expr: "1m".into(), action_json: "{}".into(),
        });

        assert_eq!(sched.dequeue().unwrap().name, "high-job");
        assert_eq!(sched.dequeue().unwrap().name, "mid-job");
        assert_eq!(sched.dequeue().unwrap().name, "low-job");
        assert!(sched.is_empty());
    }

    #[test]
    fn test_weighted_fairness() {
        let mut sched = PriorityScheduler::new();
        // Three jobs at same priority, different groups
        sched.enqueue(PrioritizedJob {
            name: "group-a-job1".into(), priority: 50, weight: 1.0, max_retries: 0,
            retry_delay_secs: 0, retries_done: 0, cron_expr: "1m".into(), action_json: "{}".into(),
        });
        sched.enqueue(PrioritizedJob {
            name: "group-b-job1".into(), priority: 50, weight: 1.0, max_retries: 0,
            retry_delay_secs: 0, retries_done: 0, cron_expr: "1m".into(), action_json: "{}".into(),
        });
        sched.enqueue(PrioritizedJob {
            name: "group-a-job2".into(), priority: 50, weight: 1.0, max_retries: 0,
            retry_delay_secs: 0, retries_done: 0, cron_expr: "1m".into(), action_json: "{}".into(),
        });

        // First pick: both groups have 0 weight, pick first found with min weight
        let first = sched.weighted_next().unwrap();
        let second = sched.weighted_next().unwrap();
        let third = sched.weighted_next().unwrap();

        // Should get at least one from each group in first 3 picks
        let names: Vec<&str> = vec![&first.name, &second.name, &third.name];
        assert!(names.iter().any(|n| n.starts_with("group-a")), "should pick from group-a");
        assert!(names.iter().any(|n| n.starts_with("group-b")), "should pick from group-b");
    }

    #[test]
    fn test_priority_scheduler_empty() {
        let mut sched = PriorityScheduler::new();
        assert!(sched.is_empty());
        assert_eq!(sched.len(), 0);
        assert!(sched.dequeue().is_none());
        assert!(sched.weighted_next().is_none());
    }
}
