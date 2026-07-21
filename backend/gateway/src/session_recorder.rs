//! 终端操作回放 —— 记录 SSH 会话命令和输出，支持回放和查询。

use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionRecord {
    pub id: String,
    pub session_id: String,
    pub host: String,
    pub user_name: String,
    pub command: String,
    pub output: Option<String>,
    pub exit_code: Option<i32>,
    pub recorded_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SessionSummary {
    pub session_id: String,
    pub host: String,
    pub user_name: String,
    pub command_count: i64,
    pub started_at: String,
    pub last_activity: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordRequest {
    pub session_id: String,
    pub host: String,
    pub user_name: String,
    pub command: String,
    pub output: Option<String>,
    pub exit_code: Option<i32>,
}

pub struct SessionRecorder {
    pool: SqlitePool,
}

impl SessionRecorder {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 记录一条终端操作。
    pub async fn record(&self, req: &RecordRequest) -> Result<SessionRecord, String> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO session_records (id, session_id, host, user_name, command, output, exit_code, recorded_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))",
        )
        .bind(&id)
        .bind(&req.session_id)
        .bind(&req.host)
        .bind(&req.user_name)
        .bind(&req.command)
        .bind(&req.output)
        .bind(req.exit_code)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query_as::<_, SessionRecord>(
            "SELECT id, session_id, host, user_name, command, output, exit_code, recorded_at \
             FROM session_records WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }

    /// 按时间顺序回放一个 session 的所有记录。
    pub async fn replay(&self, session_id: &str) -> Result<Vec<SessionRecord>, String> {
        sqlx::query_as::<_, SessionRecord>(
            "SELECT id, session_id, host, user_name, command, output, exit_code, recorded_at \
             FROM session_records WHERE session_id = ? ORDER BY recorded_at ASC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }

    /// 列出 session 列表（按主机/用户筛选，聚合为摘要）。
    pub async fn list_sessions(
        &self,
        host: Option<&str>,
        user: Option<&str>,
    ) -> Result<Vec<SessionSummary>, String> {
        let result = match (host, user) {
            (Some(h), Some(u)) => {
                sqlx::query_as::<_, SessionSummary>(
                    "SELECT session_id, host, user_name, COUNT(*) as command_count, \
                     MIN(recorded_at) as started_at, MAX(recorded_at) as last_activity \
                     FROM session_records WHERE host = ? AND user_name = ? \
                     GROUP BY session_id ORDER BY last_activity DESC",
                )
                .bind(h).bind(u)
                .fetch_all(&self.pool).await
            }
            (Some(h), None) => {
                sqlx::query_as::<_, SessionSummary>(
                    "SELECT session_id, host, user_name, COUNT(*) as command_count, \
                     MIN(recorded_at) as started_at, MAX(recorded_at) as last_activity \
                     FROM session_records WHERE host = ? \
                     GROUP BY session_id ORDER BY last_activity DESC",
                )
                .bind(h)
                .fetch_all(&self.pool).await
            }
            (None, Some(u)) => {
                sqlx::query_as::<_, SessionSummary>(
                    "SELECT session_id, host, user_name, COUNT(*) as command_count, \
                     MIN(recorded_at) as started_at, MAX(recorded_at) as last_activity \
                     FROM session_records WHERE user_name = ? \
                     GROUP BY session_id ORDER BY last_activity DESC",
                )
                .bind(u)
                .fetch_all(&self.pool).await
            }
            (None, None) => {
                sqlx::query_as::<_, SessionSummary>(
                    "SELECT session_id, host, user_name, COUNT(*) as command_count, \
                     MIN(recorded_at) as started_at, MAX(recorded_at) as last_activity \
                     FROM session_records \
                     GROUP BY session_id ORDER BY last_activity DESC",
                )
                .fetch_all(&self.pool).await
            }
        };

        result.map_err(|e| e.to_string())
    }
}
