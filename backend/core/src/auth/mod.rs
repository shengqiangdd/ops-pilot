//! Authentication module — user registration, login, JWT token management.
//!
//! Uses argon2 for password hashing and jsonwebtoken for JWT issuance/verification.

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during authentication operations.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("user already exists")]
    UserExists,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("invalid or expired token")]
    InvalidToken,

    #[error("password too short (minimum 8 characters)")]
    PasswordTooShort,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHash(String),

    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("vault not set up for this user")]
    VaultNotSetup,

    #[error("vault passphrase is incorrect")]
    VaultPassphraseMismatch,

    #[error("passphrase too short (minimum 8 characters)")]
    PassphraseTooShort,
}

impl From<AuthError> for ops_pilot_sdk::OpsError {
    fn from(e: AuthError) -> Self {
        match &e {
            AuthError::UserExists => ops_pilot_sdk::OpsError::InvalidInput(e.to_string()),
            AuthError::InvalidCredentials => ops_pilot_sdk::OpsError::AuthFailed(e.to_string()),
            AuthError::InvalidToken => ops_pilot_sdk::OpsError::AuthFailed(e.to_string()),
            AuthError::PasswordTooShort => ops_pilot_sdk::OpsError::InvalidInput(e.to_string()),
            AuthError::Database(_) => ops_pilot_sdk::OpsError::Internal(e.to_string()),
            AuthError::PasswordHash(_) => ops_pilot_sdk::OpsError::Internal(e.to_string()),
            AuthError::Jwt(_) => ops_pilot_sdk::OpsError::AuthFailed(e.to_string()),
            AuthError::VaultNotSetup => ops_pilot_sdk::OpsError::NotFound(e.to_string()),
            AuthError::VaultPassphraseMismatch => {
                ops_pilot_sdk::OpsError::AuthFailed(e.to_string())
            }
            AuthError::PassphraseTooShort => ops_pilot_sdk::OpsError::InvalidInput(e.to_string()),
        }
    }
}

/// A registered user.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    #[serde(skip_serializing)]
    pub vault_key_encrypted: Option<String>,
    #[serde(skip_serializing)]
    pub vault_password_hash: Option<String>,
    pub role: String,
    pub created_at: String,
}

/// User role for RBAC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Operator,
    Viewer,
}

impl Role {
    /// Parse a role from string.
    pub fn parse_role(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "viewer" => Role::Viewer,
            _ => Role::Operator,
        }
    }

    /// Convert role to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Viewer => "viewer",
        }
    }

    /// Check if this role has the given permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        match self {
            Role::Admin => true, // Admin has all permissions
            Role::Operator => matches!(
                permission,
                "hosts:read"
                    | "hosts:write"
                    | "vault:read"
                    | "vault:write"
                    | "modules:read"
                    | "audit:read"
            ),
            Role::Viewer => matches!(permission, "hosts:read" | "modules:read" | "audit:read"),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// JWT claims extracted from a verified token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdClaims {
    pub sub: String,
    pub role: String,
    pub exp: u64,
    pub iat: u64,
}

/// Service for user registration, login, and token verification.
pub struct AuthService {
    pool: SqlitePool,
    jwt_secret: Vec<u8>,
}

impl AuthService {
    /// Create a new AuthService with the given database pool and JWT secret.
    pub fn new(pool: SqlitePool, jwt_secret: String) -> Self {
        Self {
            pool,
            jwt_secret: jwt_secret.into_bytes(),
        }
    }

    /// Register a new user with username, email, and plaintext password.
    ///
    /// Passwords must be at least 8 characters. Returns the created user
    /// or `AuthError::UserExists` if the username or email is taken.
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, AuthError> {
        if password.len() < 8 {
            return Err(AuthError::PasswordTooShort);
        }

        let salt = SaltString::generate(&mut rand::rng());
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?
            .to_string();

        let id = Uuid::new_v4().to_string();

        let result = sqlx::query(
            "INSERT INTO users (id, username, email, password_hash) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(username)
        .bind(email)
        .bind(&hash)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(User {
                id,
                username: username.to_string(),
                email: email.to_string(),
                password_hash: hash,
                vault_key_encrypted: None,
                vault_password_hash: None,
                role: "operator".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            }),
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.message().contains("UNIQUE") {
                    Err(AuthError::UserExists)
                } else {
                    Err(AuthError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(AuthError::Database(e)),
        }
    }

    /// Authenticate a user by username and password.
    ///
    /// Returns a JWT token on success, or `InvalidCredentials` on failure.
    pub async fn login(&self, username: &str, password: &str) -> Result<String, AuthError> {
        let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let now = chrono::Utc::now().timestamp() as u64;
        let role = Role::parse_role(&user.role);
        let claims = UserIdClaims {
            sub: user.id,
            role: role.as_str().to_string(),
            iat: now,
            exp: now + 86400, // 24 hours
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )?;

        Ok(token)
    }

    /// Verify a JWT token and return the embedded claims.
    pub fn verify_token(&self, token: &str) -> Result<UserIdClaims, AuthError> {
        let token_data = decode::<UserIdClaims>(
            token,
            &DecodingKey::from_secret(&self.jwt_secret),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    // ── Vault operations ──────────────────────────────────────────────────

    /// Derive a 32-byte key from a passphrase using argon2id.
    fn derive_key_from_passphrase(passphrase: &str, salt: &str) -> Result<[u8; 32], AuthError> {
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(passphrase.as_bytes(), salt.as_bytes(), &mut key)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?;
        Ok(key)
    }

    /// Set or update the vault passphrase for a user.
    ///
    /// 1. Derive vault_key = argon2id(passphrase, salt1)
    /// 2. Derive vault_password_hash = argon2id(passphrase, salt2)
    /// 3. intermediate_key = argon2id(login_password, salt3)
    /// 4. vault_key_encrypted = aes_gcm_encrypt(vault_key, intermediate_key)
    /// 5. Store vault_key_encrypted + vault_password_hash in users table
    pub async fn set_vault_passphrase(
        &self,
        user_id: &str,
        login_password: &str,
        passphrase: &str,
    ) -> Result<(), AuthError> {
        if passphrase.len() < 8 {
            return Err(AuthError::PassphraseTooShort);
        }

        // Verify login password is correct first
        let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?;
        Argon2::default()
            .verify_password(login_password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        // Generate three random salts
        let salt1 = SaltString::generate(&mut rand::rng());
        let salt2 = SaltString::generate(&mut rand::rng());
        let salt3 = SaltString::generate(&mut rand::rng());

        // Derive vault_key from passphrase
        let vault_key = Self::derive_key_from_passphrase(passphrase, salt1.as_str())?;

        // Derive vault_password_hash from passphrase (different salt)
        let vault_password_hash = Argon2::default()
            .hash_password(passphrase.as_bytes(), &salt2)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?
            .to_string();

        // Derive intermediate_key from login_password (different salt)
        let intermediate_bytes = Self::derive_key_from_passphrase(login_password, salt3.as_str())?;
        let intermediate_master = crate::crypto::MasterKey::from_bytes(intermediate_bytes);

        // Encrypt vault_key with intermediate_key
        let (ciphertext, iv) = crate::crypto::encrypt(&vault_key, &intermediate_master)
            .map_err(|e| AuthError::PasswordHash(format!("encryption error: {e}")))?;

        // Encode as base64 for storage
        let ct_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &ciphertext);
        let iv_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &iv);
        let vault_key_encrypted = format!("{salt1}:{salt3}:{ct_b64}:{iv_b64}");

        sqlx::query(
            "UPDATE users SET vault_key_encrypted = ?, vault_password_hash = ? WHERE id = ?",
        )
        .bind(&vault_key_encrypted)
        .bind(&vault_password_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Verify vault passphrase and return the decrypted vault key.
    ///
    /// 1. Verify login password is correct
    /// 2. Verify argon2id(passphrase, salt2) == vault_password_hash
    /// 3. Parse vault_key_encrypted to extract salt1, salt3, ciphertext, iv
    /// 4. intermediate_key = argon2id(login_password, salt3)
    /// 5. vault_key = aes_gcm_decrypt(vault_key_encrypted, intermediate_key)
    /// 6. Return vault_key
    pub async fn unlock_vault(
        &self,
        user_id: &str,
        login_password: &str,
        passphrase: &str,
    ) -> Result<[u8; 32], AuthError> {
        // Fetch user
        let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify login password first
        let parsed_login_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?;
        Argon2::default()
            .verify_password(login_password.as_bytes(), &parsed_login_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        // Verify vault is set up
        let vault_password_hash = user
            .vault_password_hash
            .as_ref()
            .ok_or(AuthError::VaultNotSetup)?;

        // Verify passphrase
        let parsed_hash = PasswordHash::new(vault_password_hash)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?;
        Argon2::default()
            .verify_password(passphrase.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::VaultPassphraseMismatch)?;

        // Parse vault_key_encrypted
        let vault_key_encrypted = user
            .vault_key_encrypted
            .as_ref()
            .ok_or(AuthError::VaultNotSetup)?;

        let parts: Vec<&str> = vault_key_encrypted.splitn(4, ':').collect();
        if parts.len() != 4 {
            return Err(AuthError::VaultNotSetup);
        }
        let (salt1, salt3, ct_b64, iv_b64) = (parts[0], parts[1], parts[2], parts[3]);

        let ciphertext = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, ct_b64)
            .map_err(|e| AuthError::PasswordHash(format!("base64 decode error: {e}")))?;
        let iv = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, iv_b64)
            .map_err(|e| AuthError::PasswordHash(format!("base64 decode error: {e}")))?;

        // Derive intermediate_key from login_password
        let intermediate_bytes = Self::derive_key_from_passphrase(login_password, salt3)?;
        let intermediate_master = crate::crypto::MasterKey::from_bytes(intermediate_bytes);

        // Decrypt vault_key
        let vault_key_bytes = crate::crypto::decrypt(&ciphertext, &intermediate_master, &iv)
            .map_err(|_| AuthError::VaultPassphraseMismatch)?;

        if vault_key_bytes.len() != 32 {
            return Err(AuthError::VaultNotSetup);
        }

        let mut vault_key = [0u8; 32];
        vault_key.copy_from_slice(&vault_key_bytes);

        // Verify the vault_key by re-deriving from passphrase and comparing
        let verify_key = Self::derive_key_from_passphrase(passphrase, salt1)?;
        if vault_key != verify_key {
            return Err(AuthError::VaultNotSetup);
        }

        Ok(vault_key)
    }

    /// Check if a user has vault set up (has vault_password_hash).
    pub async fn has_vault(&self, user_id: &str) -> Result<bool, AuthError> {
        #[derive(sqlx::FromRow)]
        struct VaultHashRow {
            vault_password_hash: Option<String>,
        }

        let row: Option<VaultHashRow> =
            sqlx::query_as("SELECT vault_password_hash FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        match row {
            Some(r) => Ok(r.vault_password_hash.is_some()),
            None => Ok(false),
        }
    }

    /// Get a user by ID (without sensitive fields).
    pub async fn get_user(&self, user_id: &str) -> Result<Option<UserInfo>, AuthError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: String,
            username: String,
            email: String,
            role: String,
            created_at: String,
        }

        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, username, email, role, created_at FROM users WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserInfo {
            id: r.id,
            username: r.username,
            email: r.email,
            role: r.role,
            created_at: r.created_at,
        }))
    }

    /// List all users (admin only).
    pub async fn list_users(&self) -> Result<Vec<UserInfo>, AuthError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: String,
            username: String,
            email: String,
            role: String,
            created_at: String,
        }

        let rows: Vec<UserRow> = sqlx::query_as(
            "SELECT id, username, email, role, created_at FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| UserInfo {
                id: r.id,
                username: r.username,
                email: r.email,
                role: r.role,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Create a new user with a specific role (admin only).
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        role: &str,
    ) -> Result<UserInfo, AuthError> {
        if password.len() < 8 {
            return Err(AuthError::PasswordTooShort);
        }

        let salt = SaltString::generate(&mut rand::rng());
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?
            .to_string();

        let id = Uuid::new_v4().to_string();
        let role_str = Role::parse_role(role).as_str();

        let result = sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(username)
        .bind(email)
        .bind(&hash)
        .bind(role_str)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(UserInfo {
                id,
                username: username.to_string(),
                email: email.to_string(),
                role: role_str.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            }),
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.message().contains("UNIQUE") {
                    Err(AuthError::UserExists)
                } else {
                    Err(AuthError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(AuthError::Database(e)),
        }
    }

    /// Update a user's role (admin only).
    pub async fn update_user_role(
        &self,
        user_id: &str,
        role: &str,
    ) -> Result<(), AuthError> {
        let role_str = Role::parse_role(role).as_str();
        sqlx::query("UPDATE users SET role = ? WHERE id = ?")
            .bind(role_str)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete a user (admin only).
    pub async fn delete_user(&self, user_id: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Log a user action (login, logout, etc).
    pub async fn log_user_action(
        &self,
        user_id: &str,
        action: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), AuthError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, action, ip_address, user_agent) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(action)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Find existing user by username, or create a new one for OAuth login.
    pub async fn find_or_create_oauth_user(
        &self,
        username: &str,
        email: &str,
        default_role: &str,
    ) -> Result<UserInfo, AuthError> {
        // Try to find existing user via raw query
        let existing: Option<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, username, email, role, created_at FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, uname, em, role, created_at)) = existing {
            return Ok(UserInfo { id, username: uname, email: em, role, created_at });
        }

        // Create new user with a random password (OAuth users don't need local password)
        let random_pw = format!("oauth-{}", uuid::Uuid::new_v4());
        self.create_user(username, email, &random_pw, default_role).await
    }

    /// Generate a JWT token for the given username and role.
    pub fn generate_token(&self, username: &str, role: &str) -> Result<String, AuthError> {
        let now = chrono::Utc::now().timestamp() as u64;
        let claims = UserIdClaims {
            sub: username.to_string(),
            role: role.to_string(),
            iat: now,
            exp: now + 86400,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )?;
        Ok(token)
    }
}

/// Public user info (without sensitive fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                vault_key_encrypted TEXT,
                vault_password_hash TEXT,
                role TEXT NOT NULL DEFAULT 'operator',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_register_and_login() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "test-secret-key".into());

        let user = svc
            .register("alice", "alice@example.com", "password123")
            .await
            .unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");

        let token = svc.login("alice", "password123").await.unwrap();
        assert!(!token.is_empty());

        let claims = svc.verify_token(&token).unwrap();
        assert_eq!(claims.sub, user.id);
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        svc.register("bob", "bob@example.com", "password123")
            .await
            .unwrap();
        let err = svc
            .register("bob", "bob2@example.com", "password123")
            .await
            .unwrap_err();
        assert!(matches!(err, AuthError::UserExists));
    }

    #[tokio::test]
    async fn test_register_short_password() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let err = svc
            .register("charlie", "c@example.com", "short")
            .await
            .unwrap_err();
        assert!(matches!(err, AuthError::PasswordTooShort));
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        svc.register("dave", "dave@example.com", "password123")
            .await
            .unwrap();
        let err = svc.login("dave", "wrongpassword").await.unwrap_err();
        assert!(matches!(err, AuthError::InvalidCredentials));
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let err = svc.login("nobody", "password123").await.unwrap_err();
        assert!(matches!(err, AuthError::InvalidCredentials));
    }

    #[tokio::test]
    async fn test_verify_invalid_token() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let err = svc.verify_token("not.a.valid.token").unwrap_err();
        assert!(matches!(err, AuthError::Jwt(_)));
    }

    #[tokio::test]
    async fn test_verify_token_wrong_secret() {
        let pool = setup_db().await;
        let svc1 = AuthService::new(pool.clone(), "secret-one".into());
        let svc2 = AuthService::new(pool, "secret-two".into());

        svc1.register("eve", "eve@example.com", "password123")
            .await
            .unwrap();
        let token = svc1.login("eve", "password123").await.unwrap();

        let err = svc2.verify_token(&token).unwrap_err();
        assert!(matches!(err, AuthError::Jwt(_)));
    }

    #[tokio::test]
    async fn test_user_id_claims_serialization() {
        let claims = UserIdClaims {
            sub: "user-123".into(),
            role: "admin".into(),
            iat: 1000000,
            exp: 10086400,
        };
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: UserIdClaims = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sub, "user-123");
        assert_eq!(deserialized.role, "admin");
        assert_eq!(deserialized.exp, 10086400);
    }

    // ── Vault tests ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_vault_passphrase() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("vault_user", "v@test.com", "password123")
            .await
            .unwrap();
        svc.set_vault_passphrase(&user.id, "password123", "my-vault-pass")
            .await
            .unwrap();

        assert!(svc.has_vault(&user.id).await.unwrap());
    }

    #[tokio::test]
    async fn test_unlock_vault_correct_passphrase() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("unlock_user", "u@test.com", "password123")
            .await
            .unwrap();
        svc.set_vault_passphrase(&user.id, "password123", "my-vault-pass")
            .await
            .unwrap();

        let key = svc
            .unlock_vault(&user.id, "password123", "my-vault-pass")
            .await
            .unwrap();
        assert_eq!(key.len(), 32);
    }

    #[tokio::test]
    async fn test_unlock_vault_wrong_passphrase() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("wrong_pass", "w@test.com", "password123")
            .await
            .unwrap();
        svc.set_vault_passphrase(&user.id, "password123", "my-vault-pass")
            .await
            .unwrap();

        let result = svc
            .unlock_vault(&user.id, "password123", "wrong-passphrase")
            .await;
        assert!(matches!(result, Err(AuthError::VaultPassphraseMismatch)));
    }

    #[tokio::test]
    async fn test_unlock_vault_not_setup() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("no_vault", "n@test.com", "password123")
            .await
            .unwrap();
        let result = svc.unlock_vault(&user.id, "password123", "anything").await;
        assert!(matches!(result, Err(AuthError::VaultNotSetup)));
    }

    #[tokio::test]
    async fn test_unlock_vault_wrong_login_password() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("wrong_login", "wl@test.com", "password123")
            .await
            .unwrap();
        svc.set_vault_passphrase(&user.id, "password123", "my-vault-pass")
            .await
            .unwrap();

        let result = svc
            .unlock_vault(&user.id, "wrongpassword", "my-vault-pass")
            .await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_update_vault_passphrase() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("update_vault", "uv@test.com", "password123")
            .await
            .unwrap();
        svc.set_vault_passphrase(&user.id, "password123", "old-pass")
            .await
            .unwrap();

        let old_key = svc
            .unlock_vault(&user.id, "password123", "old-pass")
            .await
            .unwrap();

        // Update passphrase
        svc.set_vault_passphrase(&user.id, "password123", "new-pass")
            .await
            .unwrap();

        // Old passphrase should fail
        let result = svc.unlock_vault(&user.id, "password123", "old-pass").await;
        assert!(matches!(result, Err(AuthError::VaultPassphraseMismatch)));

        // New passphrase should work
        let new_key = svc
            .unlock_vault(&user.id, "password123", "new-pass")
            .await
            .unwrap();
        assert_eq!(new_key.len(), 32);
        // Keys should be different since salts are random
        assert_ne!(old_key, new_key);
    }

    #[tokio::test]
    async fn test_vault_passphrase_too_short() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("short_pass", "sp@test.com", "password123")
            .await
            .unwrap();
        let result = svc
            .set_vault_passphrase(&user.id, "password123", "short")
            .await;
        assert!(matches!(result, Err(AuthError::PassphraseTooShort)));
    }

    #[tokio::test]
    async fn test_has_vault_false_by_default() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let user = svc
            .register("no_vault2", "nv2@test.com", "password123")
            .await
            .unwrap();
        assert!(!svc.has_vault(&user.id).await.unwrap());
    }
}
