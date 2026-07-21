//! OpenTelemetry 可观测性集成 —— Trace 采集、查询、树形展示。

use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TraceSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub service: String,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_ms: i64,
    pub status: String,
    pub tags_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IngestSpanRequest {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub service: String,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_ms: i64,
    pub status: Option<String>,
    pub tags_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TraceQuery {
    pub service: Option<String>,
    pub operation: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TraceSummary {
    pub trace_id: String,
    pub span_count: i64,
    pub root_operation: String,
    pub root_service: String,
    pub start_time: i64,
    pub duration_ms: i64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TraceTreeNode {
    pub span: TraceSpan,
    pub children: Vec<TraceTreeNode>,
}

pub struct Oteler {
    pool: SqlitePool,
}

impl Oteler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 采集一个 span（INSERT OR REPLACE）。
    pub async fn ingest(&self, req: &IngestSpanRequest) -> Result<TraceSpan, String> {
        sqlx::query(
            "INSERT OR REPLACE INTO trace_spans \
             (trace_id, span_id, parent_span_id, operation_name, service, start_time, end_time, duration_ms, status, tags_json) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&req.trace_id)
        .bind(&req.span_id)
        .bind(&req.parent_span_id)
        .bind(&req.operation_name)
        .bind(&req.service)
        .bind(req.start_time)
        .bind(req.end_time)
        .bind(req.duration_ms)
        .bind(req.status.as_deref().unwrap_or("ok"))
        .bind(&req.tags_json)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(TraceSpan {
            trace_id: req.trace_id.clone(),
            span_id: req.span_id.clone(),
            parent_span_id: req.parent_span_id.clone(),
            operation_name: req.operation_name.clone(),
            service: req.service.clone(),
            start_time: req.start_time,
            end_time: req.end_time,
            duration_ms: req.duration_ms,
            status: req.status.clone().unwrap_or_else(|| "ok".into()),
            tags_json: req.tags_json.clone(),
        })
    }

    /// 查询 trace 列表（聚合为 trace summary）。
    pub async fn query_traces(&self, q: &TraceQuery) -> Result<Vec<TraceSummary>, String> {
        let limit = q.limit.unwrap_or(50).min(200);
        let base = "SELECT trace_id, operation_name as root_operation, service as root_service, \
             start_time, duration_ms, status FROM trace_spans \
             WHERE parent_span_id IS NULL";

        let result = match (&q.service, &q.operation, q.start_time, q.end_time) {
            (Some(s), Some(op), Some(st), Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? AND operation_name LIKE ? \
                     AND start_time >= ? AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(format!("%{op}%")).bind(st).bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (Some(s), Some(op), Some(st), None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? AND operation_name LIKE ? \
                     AND start_time >= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(format!("%{op}%")).bind(st).bind(limit)
                .fetch_all(&self.pool).await
            }
            (Some(s), Some(op), None, Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? AND operation_name LIKE ? \
                     AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(format!("%{op}%")).bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (Some(s), Some(op), None, None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? AND operation_name LIKE ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(format!("%{op}%")).bind(limit)
                .fetch_all(&self.pool).await
            }
            (Some(s), None, Some(st), Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? \
                     AND start_time >= ? AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(st).bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (Some(s), None, Some(st), None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? \
                     AND start_time >= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(st).bind(limit)
                .fetch_all(&self.pool).await
            }
            (Some(s), None, None, Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? \
                     AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (Some(s), None, None, None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND service = ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(s).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, Some(op), Some(st), Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND operation_name LIKE ? \
                     AND start_time >= ? AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(format!("%{op}%")).bind(st).bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, Some(op), Some(st), None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND operation_name LIKE ? \
                     AND start_time >= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(format!("%{op}%")).bind(st).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, Some(op), None, Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND operation_name LIKE ? \
                     AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(format!("%{op}%")).bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, Some(op), None, None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL AND operation_name LIKE ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(format!("%{op}%")).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, None, Some(st), Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL \
                     AND start_time >= ? AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(st).bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, None, Some(st), None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL \
                     AND start_time >= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(st).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, None, None, Some(et)) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL \
                     AND start_time <= ? \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(et).bind(limit)
                .fetch_all(&self.pool).await
            }
            (None, None, None, None) => {
                sqlx::query_as::<_, TraceSummary>(
                    "SELECT trace_id, operation_name as root_operation, service as root_service, \
                     start_time, duration_ms, status FROM trace_spans \
                     WHERE parent_span_id IS NULL \
                     ORDER BY start_time DESC LIMIT ?",
                )
                .bind(limit)
                .fetch_all(&self.pool).await
            }
        };

        let mut summaries = result.map_err(|e| e.to_string())?;

        // 补充 span_count
        for s in &mut summaries {
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM trace_spans WHERE trace_id = ?",
            )
            .bind(&s.trace_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
            s.span_count = count.0;
        }

        Ok(summaries)
    }

    /// 获取 trace 树。
    pub async fn query_trace_tree(&self, trace_id: &str) -> Result<Option<TraceTreeNode>, String> {
        let spans = sqlx::query_as::<_, TraceSpan>(
            "SELECT trace_id, span_id, parent_span_id, operation_name, service, \
             start_time, end_time, duration_ms, status, tags_json \
             FROM trace_spans WHERE trace_id = ? ORDER BY start_time ASC",
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if spans.is_empty() {
            return Ok(None);
        }

        // 构建树
        let mut map: std::collections::HashMap<String, TraceTreeNode> = std::collections::HashMap::new();
        let mut root_ids: Vec<String> = Vec::new();
        let mut children_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for span in &spans {
            let node = TraceTreeNode {
                span: span.clone(),
                children: Vec::new(),
            };
            map.insert(span.span_id.clone(), node);

            if span.parent_span_id.is_none() {
                root_ids.push(span.span_id.clone());
            } else if let Some(parent_id) = &span.parent_span_id {
                children_map.entry(parent_id.clone()).or_default().push(span.span_id.clone());
            }
        }

        // 构建父子关系
        for (parent_id, child_ids) in &children_map {
            let mut children: Vec<TraceTreeNode> = Vec::new();
            for child_id in child_ids {
                if let Some(child) = map.remove(child_id) {
                    children.push(child);
                }
            }
            if let Some(parent) = map.get_mut(parent_id) {
                parent.children = children;
            }
        }

        // 返回第一个 root
        if let Some(root_id) = root_ids.first() {
            Ok(map.remove(root_id))
        } else {
            let span_ids: Vec<String> = spans.iter().map(|s| s.span_id.clone()).collect();
            Ok(span_ids.first().and_then(|id| map.remove(id)))
        }
    }

    /// 列出已知 service。
    pub async fn list_services(&self) -> Result<Vec<String>, String> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT service FROM trace_spans ORDER BY service",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|(s,)| s).collect())
    }
}
