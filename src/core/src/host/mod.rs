//! Host management: model, status enum, and CRUD service backed by SQLite.
//!
//! Credentials (password / private key) are stored encrypted at rest using
//! AES-256-GCM via the `crypto` module.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::crypto::{self, CryptoError, MasterKey};

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

/// Row type for basic host queries (no credentials).
#[derive(Debug, sqlx::FromRow)]
struct HostRow {
    id: String,
    name: String,
    address: String,
    port: i32,
    username: String,
    auth_method: String,
    status: String,
    created_at: String,
    updated_at: String,
}

/// Row type for host queries that include encrypted credentials.
#[derive(Debug, sqlx::FromRow)]
struct HostRowWithCreds {
    id: String,
    name: String,
    address: String,
    port: i32,
    username: String,
    auth_method: String,
    status: String,
    credentials_encrypted: Option<Vec<u8>>,
    credentials_iv: Option<Vec<u8>>,
    created_at: String,
    updated_at: String,
}

impl HostRow {
    fn into_host(self) -> Host {
        Host {
            id: self.id,
            name: self.name,
            address: self.address,
            port: self.port,
            username: self.username,
            auth_method: self.auth_method,
            status: HostStatus::from_str(&self.status),
            password: None,
            private_key: None,
            created_at: NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default(),
            updated_at: NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default(),
        }
    }
}

impl HostRowWithCreds {
    fn into_host_with_creds(self, key: &MasterKey) -> Result<Host, HostError> {
        let (password, private_key) = match (self.credentials_encrypted, self.credentials_iv) {
            (Some(ciphertext), Some(iv)) => {
                let plaintext = crypto::decrypt(&ciphertext, key, &iv)?;
                deserialize_credentials(&plaintext)
            }
            _ => (None, None),
        };

        Ok(Host {
            id: self.id,
            name: self.name,
            address: self.address,
            port: self.port,
            username: self.username,
            auth_method: self.auth_method,
            status: HostStatus::from_str(&self.status),
            password,
            private_key,
            created_at: NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default(),
            updated_at: NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default(),
        })
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
    /// Decrypted password — only populated by `get_decrypted()`, `None` in list views.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Decrypted private key — only populated by `get_decrypted()`, `None` in list views.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
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
    /// Plaintext password — will be encrypted before storage.
    pub password: Option<String>,
    /// Plaintext private key — will be encrypted before storage.
    pub private_key: Option<String>,
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
    pub password: Option<String>,
    pub private_key: Option<String>,
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

    #[error("crypto error: {0}")]
    Crypto(#[from] CryptoError),
}

/// Serialize credentials to a JSON blob for encryption.
fn serialize_credentials(password: &Option<String>, private_key: &Option<String>) -> Option<Vec<u8>> {
    if password.is_none() && private_key.is_none() {
        return None;
    }
    let map = serde_json::json!({
        "password": password,
        "private_key": private_key,
    });
    serde_json::to_vec(&map).ok()
}

/// Deserialize credentials from a decrypted JSON blob.
fn deserialize_credentials(data: &[u8]) -> (Option<String>, Option<String>) {
    let map: serde_json::Value = serde_json::from_slice(data).unwrap_or(serde_json::Value::Null);
    let password = map.get("password").and_then(|v| v.as_str()).map(String::from);
    let private_key = map.get("private_key").and_then(|v| v.as_str()).map(String::from);
    (password, private_key)
}

/// CRUD service for hosts, backed by SQLite.
pub struct HostService {
    pool: SqlitePool,
    key: MasterKey,
}

impl HostService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            key: MasterKey::load(),
        }
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

        // Encrypt credentials
        let (cred_blob, iv_blob) = match serialize_credentials(&input.password, &input.private_key) {
            Some(plaintext) => {
                let (ciphertext, iv) = crypto::encrypt(&plaintext, &self.key)?;
                (Some(ciphertext), Some(iv))
            }
            None => (None, None),
        };

        sqlx::query(
            "INSERT INTO hosts (id, name, address, port, username, auth_method, status, credentials_encrypted, credentials_iv) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.address)
        .bind(port)
        .bind(&input.username)
        .bind(&input.auth_method)
        .bind(status_str)
        .bind(&cred_blob)
        .bind(&iv_blob)
        .execute(&self.pool)
        .await?;

        self.get(&id).await
    }

    /// Get a host by ID (without decrypting credentials).
    pub async fn get(&self, id: &str) -> Result<Host, HostError> {
        let row: HostRow = sqlx::query_as(
            "SELECT id, name, address, port, username, auth_method, status, created_at, updated_at FROM hosts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| HostError::NotFound(id.to_string()))?;

        Ok(row.into_host())
    }

    /// Get a host by ID with decrypted credentials.
    pub async fn get_decrypted(&self, id: &str) -> Result<Host, HostError> {
        let row: HostRowWithCreds = sqlx::query_as(
            "SELECT id, name, address, port, username, auth_method, status, credentials_encrypted, credentials_iv, created_at, updated_at FROM hosts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| HostError::NotFound(id.to_string()))?;

        row.into_host_with_creds(&self.key)
    }

    /// List all hosts (without decrypting credentials).
    pub async fn list(&self) -> Result<Vec<Host>, HostError> {
        let rows: Vec<HostRow> = sqlx::query_as(
            "SELECT id, name, address, port, username, auth_method, status, created_at, updated_at FROM hosts ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(HostRow::into_host).collect())
    }

    /// Update a host by ID (partial update).
    pub async fn update(&self, id: &str, input: UpdateHost) -> Result<Host, HostError> {
        // Fetch existing encrypted credentials so we can preserve them if not updated
        let existing: HostRowWithCreds = sqlx::query_as(
            "SELECT id, name, address, port, username, auth_method, status, credentials_encrypted, credentials_iv, created_at, updated_at FROM hosts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| HostError::NotFound(id.to_string()))?;

        let name = input.name.unwrap_or(existing.name);
        let address = input.address.unwrap_or(existing.address);
        let port = input.port.unwrap_or(existing.port);
        let username = input.username.unwrap_or(existing.username);
        let auth_method = input.auth_method.unwrap_or(existing.auth_method);
        let status = input.status.unwrap_or(HostStatus::from_str(&existing.status));

        // Update credentials only if provided
        let (cred_blob, iv_blob) = if input.password.is_some() || input.private_key.is_some() {
            let plaintext = serialize_credentials(&input.password, &input.private_key);
            match plaintext {
                Some(pt) => {
                    let (ct, iv) = crypto::encrypt(&pt, &self.key)?;
                    (Some(ct), Some(iv))
                }
                None => (None, None),
            }
        } else {
            // Preserve existing credentials
            (existing.credentials_encrypted, existing.credentials_iv)
        };

        sqlx::query(
            "UPDATE hosts SET name = ?, address = ?, port = ?, username = ?, auth_method = ?, status = ?, credentials_encrypted = ?, credentials_iv = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&name)
        .bind(&address)
        .bind(port)
        .bind(&username)
        .bind(&auth_method)
        .bind(status.as_str())
        .bind(&cred_blob)
        .bind(&iv_blob)
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

    /// Build an `SshConfig` from a host record (with decrypted credentials).
    pub async fn ssh_config_for(&self, id: &str) -> Result<crate::ssh::SshConfig, HostError> {
        let host = self.get_decrypted(id).await?;
        let mut config = crate::ssh::SshConfig::new(&host.address, &host.username)
            .port(host.port as u16);

        if let Some(ref pw) = host.password {
            config = config.password(pw);
        }
        if let Some(ref key) = host.private_key {
            // Write the private key to a temp file and point key_path at it
            let path = std::env::temp_dir().join(format!("ops_pilot_key_{}", host.id));
            std::fs::write(&path, key)
                .map_err(|e| HostError::InvalidInput(format!("failed to write temp key: {e}")))?;
            config = config.key_path(path.to_string_lossy());
        }

        Ok(config)
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
            password: None,
            private_key: None,
        }
    }

    fn sample_create_with_creds() -> CreateHost {
        CreateHost {
            password: Some("s3cret!".into()),
            private_key: Some("-----BEGIN RSA PRIVATE KEY-----\nfake\n-----END RSA PRIVATE KEY-----".into()),
            ..sample_create()
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
                        password: None,
                        private_key: None,
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
                    password: None,
                    private_key: None,
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

    // ── Credential encryption tests ─────────────────────────────────────

    #[tokio::test]
    async fn test_create_host_with_credentials() {
        let svc = setup().await;
        let host = svc.create(sample_create_with_creds()).await.unwrap();

        // get() should NOT return credentials
        let fetched = svc.get(&host.id).await.unwrap();
        assert!(fetched.password.is_none());
        assert!(fetched.private_key.is_none());

        // get_decrypted() SHOULD return credentials
        let decrypted = svc.get_decrypted(&host.id).await.unwrap();
        assert_eq!(decrypted.password.as_deref(), Some("s3cret!"));
        assert!(decrypted.private_key.as_ref().unwrap().contains("RSA PRIVATE KEY"));
    }

    #[tokio::test]
    async fn test_credentials_not_in_list() {
        let svc = setup().await;
        svc.create(sample_create_with_creds()).await.unwrap();

        let hosts = svc.list().await.unwrap();
        assert_eq!(hosts.len(), 1);
        // List should not include credentials
        assert!(hosts[0].password.is_none());
        assert!(hosts[0].private_key.is_none());
    }

    #[tokio::test]
    async fn test_update_preserves_existing_credentials() {
        let svc = setup().await;
        let host = svc.create(sample_create_with_creds()).await.unwrap();

        // Update only the name, credentials should be preserved
        let updated = svc
            .update(
                &host.id,
                UpdateHost {
                    name: Some("new-name".into()),
                    ..UpdateHost {
                        name: None,
                        address: None,
                        port: None,
                        username: None,
                        auth_method: None,
                        status: None,
                        password: None,
                        private_key: None,
                    }
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "new-name");

        // Credentials should still be there
        let decrypted = svc.get_decrypted(&host.id).await.unwrap();
        assert_eq!(decrypted.password.as_deref(), Some("s3cret!"));
    }

    #[tokio::test]
    async fn test_update_credentials() {
        let svc = setup().await;
        let host = svc.create(sample_create_with_creds()).await.unwrap();

        // Update credentials
        svc.update(
            &host.id,
            UpdateHost {
                password: Some("new_password".into()),
                private_key: Some("new-key-data".into()),
                ..UpdateHost {
                    name: None,
                    address: None,
                    port: None,
                    username: None,
                    auth_method: None,
                    status: None,
                    password: None,
                    private_key: None,
                }
            },
        )
        .await
        .unwrap();

        let decrypted = svc.get_decrypted(&host.id).await.unwrap();
        assert_eq!(decrypted.password.as_deref(), Some("new_password"));
        assert_eq!(decrypted.private_key.as_deref(), Some("new-key-data"));
    }

    #[tokio::test]
    async fn test_host_without_credentials() {
        let svc = setup().await;
        let host = svc.create(sample_create()).await.unwrap();

        let decrypted = svc.get_decrypted(&host.id).await.unwrap();
        assert!(decrypted.password.is_none());
        assert!(decrypted.private_key.is_none());
    }
}
