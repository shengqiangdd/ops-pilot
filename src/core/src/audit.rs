//! AuditTrail: writes audit entries to the database and publishes events.

use crate::db::Database;
use crate::event::EventBus;
use ops_pilot_sdk::events::OpsEvent;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use tracing::debug;

/// Row type for audit log queries.
#[derive(Debug, sqlx::FromRow)]
struct AuditRow {
    id: String,
    #[sqlx(rename = "user")]
    user: String,
    action: String,
    resource: String,
    outcome: String,
    created_at: String,
}

impl AuditRow {
    fn into_entry(self) -> AuditEntry {
        AuditEntry {
            id: self.id,
            user: self.user,
            action: self.action,
            resource: self.resource,
            outcome: self.outcome,
            created_at: self.created_at,
        }
    }
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    pub id: String,
    pub user: String,
    pub action: String,
    pub resource: String,
    pub outcome: String,
    pub created_at: String,
}

/// Errors specific to audit operations.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// AuditTrail wraps Database for audit log writes and publishes AuditLog events.
pub struct AuditTrail {
    pool: SqlitePool,
    event_bus: EventBus,
}

impl AuditTrail {
    /// Create a new AuditTrail from a Database and EventBus.
    pub fn new(db: &Database, event_bus: EventBus) -> Self {
        Self {
            pool: db.pool.clone(),
            event_bus,
        }
    }

    /// Log an audit entry and publish an AuditLog event.
    pub async fn log(
        &self,
        user: &str,
        action: &str,
        resource: &str,
        outcome: &str,
    ) -> Result<AuditEntry, AuditError> {
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"INSERT INTO audit_log (id, "user", action, resource, outcome) VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(user)
        .bind(action)
        .bind(resource)
        .bind(outcome)
        .execute(&self.pool)
        .await?;

        debug!(id = %id, user, action, resource, outcome, "audit entry written");

        // Publish event — ignore errors (no subscribers is valid).
        let _ = self.event_bus.publish(OpsEvent::AuditLog {
            user: user.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            outcome: outcome.to_string(),
        });

        // Fetch the row back to get created_at.
        let row: AuditRow = sqlx::query_as(
            r#"SELECT id, "user", action, resource, outcome, created_at FROM audit_log WHERE id = ?"#,
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into_entry())
    }

    /// List the most recent audit entries, ordered by created_at descending.
    pub async fn list_recent(&self, limit: usize) -> Result<Vec<AuditEntry>, AuditError> {
        let rows: Vec<AuditRow> = sqlx::query_as(
            r#"SELECT id, "user", action, resource, outcome, created_at FROM audit_log ORDER BY created_at DESC, rowid DESC LIMIT ?"#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(AuditRow::into_entry).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> (AuditTrail, tokio::sync::broadcast::Receiver<OpsEvent>) {
        let db = Database::open_in_memory().await.unwrap();
        let bus = EventBus::new(64);
        let rx = bus.subscribe();
        let trail = AuditTrail::new(&db, bus);
        (trail, rx)
    }

    #[tokio::test]
    async fn log_writes_to_database() {
        let (trail, _rx) = setup().await;

        let entry = trail
            .log("alice", "connect", "host/prod-web", "success")
            .await
            .unwrap();

        assert_eq!(entry.user, "alice");
        assert_eq!(entry.action, "connect");
        assert_eq!(entry.resource, "host/prod-web");
        assert_eq!(entry.outcome, "success");
        assert!(!entry.id.is_empty());
        assert!(!entry.created_at.is_empty());
    }

    #[tokio::test]
    async fn log_publishes_event() {
        let (trail, mut rx) = setup().await;

        trail
            .log("bob", "delete", "key/secret", "denied")
            .await
            .unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            OpsEvent::AuditLog {
                user,
                action,
                resource,
                outcome,
            } => {
                assert_eq!(user, "bob");
                assert_eq!(action, "delete");
                assert_eq!(resource, "key/secret");
                assert_eq!(outcome, "denied");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_recent_returns_descending_order() {
        let (trail, _rx) = setup().await;

        trail.log("u1", "a1", "r1", "ok").await.unwrap();
        trail.log("u2", "a2", "r2", "ok").await.unwrap();
        trail.log("u3", "a3", "r3", "ok").await.unwrap();

        let entries = trail.list_recent(2).await.unwrap();
        assert_eq!(entries.len(), 2);
        // Most recent first.
        assert_eq!(entries[0].user, "u3");
        assert_eq!(entries[1].user, "u2");
    }

    #[tokio::test]
    async fn list_recent_empty_table() {
        let (trail, _rx) = setup().await;

        let entries = trail.list_recent(10).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn list_recent_limit_exceeds_count() {
        let (trail, _rx) = setup().await;

        trail.log("u1", "a1", "r1", "ok").await.unwrap();

        let entries = trail.list_recent(100).await.unwrap();
        assert_eq!(entries.len(), 1);
    }
}
