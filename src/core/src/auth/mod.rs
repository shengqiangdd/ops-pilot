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
}

/// A registered user.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: String,
}

/// JWT claims extracted from a verified token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdClaims {
    pub sub: String,
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

        let salt = SaltString::generate(&mut rand::thread_rng());
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

        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|e| AuthError::PasswordHash(e.to_string()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let now = chrono::Utc::now().timestamp() as u64;
        let claims = UserIdClaims {
            sub: user.id,
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

        let user = svc.register("alice", "alice@example.com", "password123").await.unwrap();
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

        svc.register("bob", "bob@example.com", "password123").await.unwrap();
        let err = svc.register("bob", "bob2@example.com", "password123").await.unwrap_err();
        assert!(matches!(err, AuthError::UserExists));
    }

    #[tokio::test]
    async fn test_register_short_password() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        let err = svc.register("charlie", "c@example.com", "short").await.unwrap_err();
        assert!(matches!(err, AuthError::PasswordTooShort));
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let pool = setup_db().await;
        let svc = AuthService::new(pool, "secret".into());

        svc.register("dave", "dave@example.com", "password123").await.unwrap();
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

        svc1.register("eve", "eve@example.com", "password123").await.unwrap();
        let token = svc1.login("eve", "password123").await.unwrap();

        let err = svc2.verify_token(&token).unwrap_err();
        assert!(matches!(err, AuthError::Jwt(_)));
    }

    #[tokio::test]
    async fn test_user_id_claims_serialization() {
        let claims = UserIdClaims {
            sub: "user-123".into(),
            iat: 1000000,
            exp: 10086400,
        };
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: UserIdClaims = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sub, "user-123");
        assert_eq!(deserialized.exp, 10086400);
    }
}
