//! Host management: model, status enum, and CRUD service backed by SQLite.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Host status lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostStatus {
    Online,
    Offline,
    Unknown,
    Maintenance,
}

impl HostStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Unknown => "unknown",
            Self::Maintenance => "maintenance",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "online" => Self::Online,
            "offline" => Self::Offline,
            "maintenance" => Self::Maintenance,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for HostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Host model representing a managed server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: i32,
    pub username: String,
    pub auth_method: String,
    pub status: HostStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Payload for creating a new host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHost {
    pub name: String,
    pub address: String,
    pub port: Option<i32>,
    pub username: String,
    pub auth_method: String,
    pub status: Option<HostStatus>,
}

/// Payload for updating an existing host. All fields optional (partial update).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHost {
    pub name: Option<String>,
    pub address: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub auth_method: Option<String>,
    pub status: Option<HostStatus>,
}

/// Error type for host operations.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("host not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// CRUD service for hosts, backed by SQLite.
pub struct HostService {
    pool: SqlitePool,
}

impl HostService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new host record.
    pub async fn create(&self, input: CreateHost) -> Result<Host, HostError> {
        if input.name.is_empty() {
            return Err(HostError::InvalidInput("name cannot be empty".into()));
        }
        if input.address.is_empty() {
            return Err(HostError::InvalidInput("address cannot be empty".into()));
        }
        if input.username.is_empty() {
            return Err(HostError::InvalidInput("username cannot be empty".into()));
        }
        if input.auth_method.is_empty() {
            return Err(HostError::InvalidInput("auth_method cannot be empty".into()));
        }

        let id = Uuid::new_v4().to_string();
        let port = input.port.unwrap_or(22);
        let status = input.status.unwrap_or(HostStatus::Unknown);
        let status_str = status.as_str();

        sqlx::query(
            "INSERT INTO hosts (id, name, address, port, username, auth_method, status) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.address)
        .bind(port)
        .bind(&input.username)
        .bind(&input.auth_method)
        .bind(status_str)
        .execute(&self.pool)
        .await?;

        self.get(&id).await
    }

    /// Get a host by ID.
    pub async fn get(&self, id: &str) -> Result<Host, HostError> {
        let row: (String, String, String, i32, String, String, String, String, String) =
            sqlx::query_as(
                "SELECT id, name, address, port, username, auth_method, status, created_at, updated_at FROM hosts WHERE id = ?",
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| HostError::NotFound(id.to_string()))?;

        Ok(Host {
            id: row.0,
            name: row.1,
            address: row.2,
            port: row.3,
            username: row.4,
            auth_method: row.5,
            status: HostStatus::from_str(&row.6),
            created_at: NaiveDateTime::parse_from_str(&row.7, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default(),
            updated_at: NaiveDateTime::parse_from_str(&row.8, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default(),
        })
    }

    /// List all hosts.
    pub async fn list(&self) -> Result<Vec<Host>, HostError> {
        let rows: Vec<(String, String, String, i32, String, String, String, String, String)> =
            sqlx::query_as(
                "SELECT id, name, address, port, username, auth_method, status, created_at, updated_at FROM hosts ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| Host {
                id: row.0,
                name: row.1,
                address: row.2,
                port: row.3,
                username: row.4,
                auth_method: row.5,
                status: HostStatus::from_str(&row.6),
                created_at: NaiveDateTime::parse_from_str(&row.7, "%Y-%m-%d %H:%M:%S")
                    .unwrap_or_default(),
                updated_at: NaiveDateTime::parse_from_str(&row.8, "%Y-%m-%d %H:%M:%S")
                    .unwrap_or_default(),
            })
            .collect())
    }

    /// Update a host by ID (partial update).
    pub async fn update(&self, id: &str, input: UpdateHost) -> Result<Host, HostError> {
        // Verify the host exists first.
        let existing = self.get(id).await?;

        let name = input.name.unwrap_or(existing.name);
        let address = input.address.unwrap_or(existing.address);
        let port = input.port.unwrap_or(existing.port);
        let username = input.username.unwrap_or(existing.username);
        let auth_method = input.auth_method.unwrap_or(existing.auth_method);
        let status = input.status.unwrap_or(existing.status);

        sqlx::query(
            "UPDATE hosts SET name = ?, address = ?, port = ?, username = ?, auth_method = ?, status = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&name)
        .bind(&address)
        .bind(port)
        .bind(&username)
        .bind(&auth_method)
        .bind(status.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get(id).await
    }

    /// Delete a host by ID.
    pub async fn delete(&self, id: &str) -> Result<(), HostError> {
        let result = sqlx::query("DELETE FROM hosts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(HostError::NotFound(id.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    async fn setup() -> HostService {
        let db = Database::open_in_memory().await.unwrap();
        HostService::new(db.pool)
    }

    fn sample_create() -> CreateHost {
        CreateHost {
            name: "web-server-1".into(),
            address: "192.168.1.10".into(),
            port: Some(22),
            username: "admin".into(),
            auth_method: "password".into(),
            status: None,
        }
    }

    #[tokio::test]
    async fn test_create_host() {
        let svc = setup().await;
        let host = svc.create(sample_create()).await.unwrap();
        assert_eq!(host.name, "web-server-1");
        assert_eq!(host.address, "192.168.1.10");
        assert_eq!(host.port, 22);
        assert_eq!(host.username, "admin");
        assert_eq!(host.status, HostStatus::Unknown);
    }

    #[tokio::test]
    async fn test_create_host_default_port() {
        let svc = setup().await;
        let input = CreateHost {
            port: None,
            ..sample_create()
        };
        let host = svc.create(input).await.unwrap();
        assert_eq!(host.port, 22);
    }

    #[tokio::test]
    async fn test_create_host_empty_name() {
        let svc = setup().await;
        let input = CreateHost {
            name: String::new(),
            ..sample_create()
        };
        assert!(svc.create(input).await.is_err());
    }

    #[tokio::test]
    async fn test_get_host() {
        let svc = setup().await;
        let created = svc.create(sample_create()).await.unwrap();
        let fetched = svc.get(&created.id).await.unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, "web-server-1");
    }

    #[tokio::test]
    async fn test_get_host_not_found() {
        let svc = setup().await;
        let result = svc.get("nonexistent-id").await;
        assert!(matches!(result, Err(HostError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_hosts_empty() {
        let svc = setup().await;
        let hosts = svc.list().await.unwrap();
        assert!(hosts.is_empty());
    }

    #[tokio::test]
    async fn test_list_hosts() {
        let svc = setup().await;
        svc.create(sample_create()).await.unwrap();
        svc.create(CreateHost {
            name: "db-server".into(),
            address: "192.168.1.20".into(),
            ..sample_create()
        })
        .await
        .unwrap();

        let hosts = svc.list().await.unwrap();
        assert_eq!(hosts.len(), 2);
    }

    #[tokio::test]
    async fn test_update_host() {
        let svc = setup().await;
        let created = svc.create(sample_create()).await.unwrap();

        let updated = svc
            .update(
                &created.id,
                UpdateHost {
                    name: Some("renamed-server".into()),
                    status: Some(HostStatus::Online),
                    ..UpdateHost {
                        name: None,
                        address: None,
                        port: None,
                        username: None,
                        auth_method: None,
                        status: None,
                    }
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "renamed-server");
        assert_eq!(updated.status, HostStatus::Online);
        assert_eq!(updated.address, "192.168.1.10"); // unchanged
    }

    #[tokio::test]
    async fn test_update_host_not_found() {
        let svc = setup().await;
        let result = svc
            .update(
                "nonexistent",
                UpdateHost {
                    name: Some("x".into()),
                    address: None,
                    port: None,
                    username: None,
                    auth_method: None,
                    status: None,
                },
            )
            .await;
        assert!(matches!(result, Err(HostError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_host() {
        let svc = setup().await;
        let created = svc.create(sample_create()).await.unwrap();
        svc.delete(&created.id).await.unwrap();
        assert!(matches!(svc.get(&created.id).await, Err(HostError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_host_not_found() {
        let svc = setup().await;
        let result = svc.delete("nonexistent").await;
        assert!(matches!(result, Err(HostError::NotFound(_))));
    }

    #[test]
    fn test_host_status_serde() {
        assert_eq!(serde_json::to_string(&HostStatus::Online).unwrap(), "\"online\"");
        assert_eq!(serde_json::to_string(&HostStatus::Offline).unwrap(), "\"offline\"");
        assert_eq!(serde_json::to_string(&HostStatus::Unknown).unwrap(), "\"unknown\"");
        assert_eq!(
            serde_json::to_string(&HostStatus::Maintenance).unwrap(),
            "\"maintenance\""
        );

        assert_eq!(
            serde_json::from_str::<HostStatus>("\"online\"").unwrap(),
            HostStatus::Online
        );
    }

    #[test]
    fn test_host_status_display() {
        assert_eq!(HostStatus::Online.to_string(), "online");
        assert_eq!(HostStatus::Offline.to_string(), "offline");
    }

    #[test]
    fn test_host_status_from_str() {
        assert_eq!(HostStatus::from_str("online"), HostStatus::Online);
        assert_eq!(HostStatus::from_str("offline"), HostStatus::Offline);
        assert_eq!(HostStatus::from_str("maintenance"), HostStatus::Maintenance);
        assert_eq!(HostStatus::from_str("bogus"), HostStatus::Unknown);
    }
}
