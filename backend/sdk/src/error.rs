//! Unified error type for the OpsPilot platform.

use thiserror::Error;

/// Base error type for all OpsPilot modules and services.
///
/// Core modules return domain-specific errors internally, but expose
/// `OpsError` at service boundaries. Use `From` conversions to bridge.
///
/// `OpsError` implements `Into<anyhow::Error>` automatically (via `thiserror::Error`),
/// so it can be used with `?` in functions returning `anyhow::Result`.
#[derive(Debug, Error)]
pub enum OpsError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation failed: {0}")]
    InvalidInput(String),

    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("permission denied: {0}")]
    Forbidden(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("external service error: {0}")]
    External(String),
}
