//! 操作审计 + 慢查询检测 —— 记录所有操作审计日志并检测慢查询。

use sqlx::SqlitePool;
use serde::Serialize;
use std::time::Instant;

/// 审计日志条目。
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub detail: String,
    pub created_at: String,
}

/// 慢查询条目。
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SlowQueryEntry {
    pub id: String,
    pub query_text: String,
    pub duration_ms: i64,
    pub created_at: String,
}

/// 审计日志记录器。
pub struct AuditLogger {
    pool: SqlitePool,
}

impl AuditLogger {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 记录一条操作审计日志。
    pub async fn log_action(
        &self,
        actor: &str,
        action: &str,
        resource: &str,
        detail: &str,
    ) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO audit_logs (id, actor, action, resource, detail, created_at) \
             VALUES (?, ?, ?, ?, ?, datetime('now'))",
        )
        .bind(&id)
        .bind(actor)
        .bind(action)
        .bind(resource)
        .bind(detail)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 查询审计日志。
    pub async fn list_logs(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogEntry>, String> {
        sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, actor, action, resource, detail, created_at \
             FROM audit_logs ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }
}

/// 慢查询检测器，包装一个异步操作并记录执行时间。
pub struct SlowQueryTracker {
    pool: SqlitePool,
    threshold_ms: i64,
}

impl SlowQueryTracker {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            threshold_ms: 500,
        }
    }

    /// 记录一条慢查询。
    pub async fn record(&self, query_text: &str, duration_ms: i64) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO slow_queries (id, query_text, duration_ms, created_at) \
             VALUES (?, ?, ?, datetime('now'))",
        )
        .bind(&id)
        .bind(query_text)
        .bind(duration_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 查询慢查询列表。
    pub async fn list_slow_queries(
        &self,
        limit: i64,
    ) -> Result<Vec<SlowQueryEntry>, String> {
        sqlx::query_as::<_, SlowQueryEntry>(
            "SELECT id, query_text, duration_ms, created_at \
             FROM slow_queries ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }

    /// 获取内部阈值。
    pub fn threshold_ms(&self) -> i64 {
        self.threshold_ms
    }

    /// 创建一个计时器，drop 时如果超过阈值则记录慢查询。
    pub fn timer(&self, query_text: String) -> SlowQueryTimer {
        SlowQueryTimer {
            pool: self.pool.clone(),
            threshold_ms: self.threshold_ms,
            query_text,
            start: Instant::now(),
        }
    }
}

/// RAII 计时器：drop 时检查是否超过阈值并记录。
pub struct SlowQueryTimer {
    pool: SqlitePool,
    threshold_ms: i64,
    query_text: String,
    start: Instant,
}

impl Drop for SlowQueryTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_millis() as i64;
        if elapsed >= self.threshold_ms {
            let pool = self.pool.clone();
            let text = self.query_text.clone();
            tokio::spawn(async move {
                let _ = sqlx::query(
                    "INSERT INTO slow_queries (id, query_text, duration_ms, created_at) \
                     VALUES (?, ?, ?, datetime('now'))",
                )
                .bind(uuid::Uuid::new_v4().to_string())
                .bind(&text)
                .bind(elapsed)
                .execute(&pool)
                .await;
            });
        }
    }
}
