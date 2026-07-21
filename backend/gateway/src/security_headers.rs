//! Security headers middleware for Axum.
//!
//! Applies CSP, HSTS, X-Frame-Options, and other hardening headers
//! to every response via `tower-http::SetResponseHeaderLayer`.

use axum::http::{
    header::{HeaderName, HeaderValue},
    HeaderMap,
};
use std::sync::LazyLock;
use tower_http::set_header::SetResponseHeaderLayer;

/// Content-Security-Policy — strict but functional for an SPA with API.
const CSP: &str = "default-src 'self'; \
                   script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                   style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                   font-src 'self' https://fonts.gstatic.com; \
                   img-src 'self' data: blob:; \
                   connect-src 'self' ws: wss:; \
                   frame-ancestors 'none'; \
                   base-uri 'self'; \
                   form-action 'self'";

/// HTTP Strict-Transport-Security — 1 year, include subdomains, preload.
const HSTS: &str = "max-age=31536000; includeSubDomains; preload";

/// Referrer-Policy — no referrer for cross-origin requests.
const REFERRER: &str = "strict-origin-when-cross-origin";

/// Permissions-Policy — disable most browser features we don't use.
const PERMISSIONS: &str = "camera=(), microphone=(), geolocation=(), \
                           interest-cohort=(), payment=(), usb=(), \
                           magnetometer=(), gyroscope=()";

// ── Lazy headers ─────────────────────────────────────────────────────────

static CSP_HEADER: LazyLock<(HeaderName, HeaderValue)> = LazyLock::new(|| {
    (
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(CSP),
    )
});

static HSTS_HEADER: LazyLock<(HeaderName, HeaderValue)> = LazyLock::new(|| {
    (
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static(HSTS),
    )
});

static REFERRER_HEADER: LazyLock<(HeaderName, HeaderValue)> = LazyLock::new(|| {
    (
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static(REFERRER),
    )
});

static PERMISSIONS_HEADER: LazyLock<(HeaderName, HeaderValue)> = LazyLock::new(|| {
    (
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(PERMISSIONS),
    )
});

static X_CONTENT_TYPE_HEADER: LazyLock<(HeaderName, HeaderValue)> = LazyLock::new(|| {
    (
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    )
});

static X_FRAME_HEADER: LazyLock<(HeaderName, HeaderValue)> = LazyLock::new(|| {
    (
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    )
});

// ── Layers ───────────────────────────────────────────────────────────────

/// Returns a list of `SetResponseHeaderLayer` layers that harden HTTP
/// response headers.
///
/// Usage:
/// ```ignore
/// use tower::ServiceBuilder;
/// let layers = ops_pilot_gateway::security_headers::security_header_layers();
/// let svc = ServiceBuilder::new()
///     .layer(layers.0)
///     .layer(layers.1)
///     // …
///     .service(router);
/// ```
pub fn security_header_layers() -> Vec<SetResponseHeaderLayer<HeaderValue>> {
    vec![
        SetResponseHeaderLayer::overriding(
            CSP_HEADER.0.clone(),
            CSP_HEADER.1.clone(),
        ),
        SetResponseHeaderLayer::overriding(
            HSTS_HEADER.0.clone(),
            HSTS_HEADER.1.clone(),
        ),
        SetResponseHeaderLayer::overriding(
            REFERRER_HEADER.0.clone(),
            REFERRER_HEADER.1.clone(),
        ),
        SetResponseHeaderLayer::overriding(
            PERMISSIONS_HEADER.0.clone(),
            PERMISSIONS_HEADER.1.clone(),
        ),
        SetResponseHeaderLayer::overriding(
            X_CONTENT_TYPE_HEADER.0.clone(),
            X_CONTENT_TYPE_HEADER.1.clone(),
        ),
        SetResponseHeaderLayer::overriding(
            X_FRAME_HEADER.0.clone(),
            X_FRAME_HEADER.1.clone(),
        ),
    ]
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_header_value() {
        let (name, value) = &*CSP_HEADER;
        assert_eq!(name.as_str(), "content-security-policy");
        let v = value.to_str().unwrap();
        assert!(v.contains("default-src 'self'"));
        assert!(v.contains("frame-ancestors 'none'"));
        assert!(v.contains("script-src 'self'"));
    }

    #[test]
    fn test_hsts_header_value() {
        let (name, value) = &*HSTS_HEADER;
        assert_eq!(name.as_str(), "strict-transport-security");
        assert!(value.to_str().unwrap().contains("max-age=31536000"));
    }

    #[test]
    fn test_all_layers_created() {
        let layers = security_header_layers();
        assert_eq!(layers.len(), 6);
    }
}
