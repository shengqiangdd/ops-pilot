//! 智能根因定位引擎 —— 关联分析、因果链构建、根因评分。

use sqlx::SqlitePool;
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CorrelatedEvent {
    pub id: String,
    pub event_type: String,
    pub message: String,
    pub severity: String,
    pub resource: String,
    pub created_at: String,
    pub relevance_score: f64,
}

#[derive(Debug, Serialize)]
pub struct CorrelationResult {
    pub alert_id: String,
    pub correlated_events: Vec<CorrelatedEvent>,
    pub root_causes: Vec<RootCause>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CausalChainEvent {
    pub id: String,
    pub event_type: String,
    pub message: String,
    pub resource: String,
    pub created_at: String,
    pub sequence: i32,
}

#[derive(Debug, Serialize)]
pub struct CausalChainResult {
    pub incident_id: String,
    pub chain: Vec<CausalChainEvent>,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub struct RootCause {
    pub cause: String,
    pub score: f64,
    pub evidence: Vec<String>,
}

pub struct RcaEngine {
    pool: SqlitePool,
}

impl RcaEngine {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 关联分析：查找同一主机/时间窗口内的相关告警、变更、日志。
    pub async fn correlate(&self, alert_id: &str) -> Result<CorrelationResult, String> {
        // 获取原始告警信息
        let alert = sqlx::query_as::<_, AlertRow>(
            "SELECT id, message, severity, resource, created_at FROM alert_history WHERE id = ?",
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("alert not found: {alert_id}"))?;

        let mut events = Vec::new();

        // 查找同一资源的其他告警（±24h 窗口）
        let related_alerts = sqlx::query_as::<_, AlertRow>(
            "SELECT id, message, severity, resource, created_at FROM alert_history \
             WHERE resource = ? AND id != ? \
             AND created_at >= datetime(?, '-1 day') AND created_at <= datetime(?, '+1 day') \
             ORDER BY created_at DESC LIMIT 10",
        )
        .bind(&alert.resource)
        .bind(alert_id)
        .bind(&alert.created_at)
        .bind(&alert.created_at)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for a in related_alerts {
            events.push(CorrelatedEvent {
                id: a.id.clone(),
                event_type: "alert".into(),
                message: a.message,
                severity: a.severity,
                resource: a.resource,
                created_at: a.created_at,
                relevance_score: 0.8,
            });
        }

        // 查找审计日志中的相关操作
        let audit_logs = sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, action, resource, detail, created_at FROM audit_logs \
             WHERE resource LIKE ? \
             ORDER BY created_at DESC LIMIT 5",
        )
        .bind(format!("%{}%", &alert.resource))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for log in audit_logs {
            events.push(CorrelatedEvent {
                id: log.id,
                event_type: "audit".into(),
                message: format!("{}: {}", log.action, log.detail),
                severity: "info".into(),
                resource: log.resource,
                created_at: log.created_at,
                relevance_score: 0.5,
            });
        }

        // 规则引擎根因评分
        let root_causes = self.score_root_causes(&alert, &events).await;

        Ok(CorrelationResult {
            alert_id: alert_id.into(),
            correlated_events: events,
            root_causes,
        })
    }

    /// 因果链：按时间线构建事件因果链。
    pub async fn causal_chain(&self, incident_id: &str) -> Result<CausalChainResult, String> {
        // 通过 incident_id 关联的告警资源来查找相关事件
        let alert = sqlx::query_as::<_, AlertRow>(
            "SELECT id, message, severity, resource, created_at FROM alert_history WHERE id = ?",
        )
        .bind(incident_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("incident not found: {incident_id}"))?;

        let mut chain = Vec::new();
        let mut seq = 0;

        // 添加触发事件
        chain.push(CausalChainEvent {
            id: alert.id.clone(),
            event_type: "trigger".into(),
            message: alert.message.clone(),
            resource: alert.resource.clone(),
            created_at: alert.created_at.clone(),
            sequence: seq,
        });
        seq += 1;

        // 查找时间线上的相关事件
        let related = sqlx::query_as::<_, AlertRow>(
            "SELECT id, message, severity, resource, created_at FROM alert_history \
             WHERE resource = ? AND id != ? AND created_at <= ? \
             ORDER BY created_at DESC LIMIT 10",
        )
        .bind(&alert.resource)
        .bind(&alert.id)
        .bind(&alert.created_at)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for r in related.into_iter().rev() {
            chain.push(CausalChainEvent {
                id: r.id,
                event_type: "related".into(),
                message: r.message,
                resource: r.resource,
                created_at: r.created_at,
                sequence: seq,
            });
            seq += 1;
        }

        let summary = format!(
            "事故 {} 涉及资源 {}, 共 {} 个相关事件形成因果链",
            incident_id, &alert.resource, chain.len()
        );

        Ok(CausalChainResult {
            incident_id: incident_id.into(),
            chain,
            summary,
        })
    }

    /// 规则引擎根因评分。
    async fn score_root_causes(
        &self,
        alert: &AlertRow,
        events: &[CorrelatedEvent],
    ) -> Vec<RootCause> {
        let mut causes = Vec::new();
        let msg_lower = alert.message.to_lowercase();

        // 规则 1: 高频告警 → 资源过载
        let alert_count = events.iter().filter(|e| e.event_type == "alert").count();
        if alert_count >= 3 {
            causes.push(RootCause {
                cause: "资源过载：同一资源短时间内触发多次告警".into(),
                score: 0.9,
                evidence: events.iter()
                    .filter(|e| e.event_type == "alert")
                    .map(|e| e.message.clone())
                    .collect(),
            });
        }

        // 规则 2: 告警前有配置变更 → 变更引发
        let config_changes: Vec<String> = events.iter()
            .filter(|e| e.event_type == "audit" && e.message.contains("config"))
            .map(|e| e.message.clone())
            .collect();
        if !config_changes.is_empty() {
            causes.push(RootCause {
                cause: "配置变更引发：告警前检测到配置修改操作".into(),
                score: 0.7,
                evidence: config_changes,
            });
        }

        // 规则 3: 根据关键词匹配
        if msg_lower.contains("cpu") || msg_lower.contains("memory") {
            causes.push(RootCause {
                cause: "资源瓶颈：CPU 或内存使用率过高".into(),
                score: 0.6,
                evidence: vec![alert.message.clone()],
            });
        }
        if msg_lower.contains("disk") {
            causes.push(RootCause {
                cause: "磁盘空间不足".into(),
                score: 0.65,
                evidence: vec![alert.message.clone()],
            });
        }
        if msg_lower.contains("connection") || msg_lower.contains("ssh") {
            causes.push(RootCause {
                cause: "网络连接问题".into(),
                score: 0.55,
                evidence: vec![alert.message.clone()],
            });
        }

        // 按分数排序
        causes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        causes
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AlertRow {
    id: String,
    message: String,
    severity: String,
    resource: String,
    created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
struct AuditLogRow {
    id: String,
    action: String,
    resource: String,
    detail: String,
    created_at: String,
}
