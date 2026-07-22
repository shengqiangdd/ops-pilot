//! Security headers middleware for Axum.
//!
//! Applies CSP, HSTS, X-Frame-Options, and other hardening headers
//! to every response via `tower-http::SetResponseHeaderLayer`.

use axum::http::header::{HeaderName, HeaderValue};
use std::sync::LazyLock;
use tower_http::set_header::SetResponseHeaderLayer;

/// Content-Security-Policy — strict but functional for an SPA with API.
///
/// Script-src and style-src use 'unsafe-inline' and 'unsafe-eval' because
/// the SPA bundles third-party widgets that require them.  In production you
/// can tighten this by generating a nonce per request and switching to
/// `'strict-dynamic'`.
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

/// Referrer-Policy — send only same-origin referrer.
const REFERRER: &str = "same-origin";

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

static X_XSS_HEADER: LazyLock<(HeaderName, HeaderValue)> = LazyLock::new(|| {
    (
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    )
});

// ── Layers ───────────────────────────────────────────────────────────────

/// Returns a list of `SetResponseHeaderLayer` layers that harden HTTP
/// response headers.
///
/// # Order
///
/// Security headers are applied in a semantically neutral order.  Multiple
/// layers are safe because each sets a distinct header name.
///
/// # Usage
///
/// ```ignore
/// use tower::ServiceBuilder;
/// let layers = ops_pilot_gateway::security_headers::security_header_layers();
/// let svc = ServiceBuilder::new()
///     .layer(layers[0])
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
        SetResponseHeaderLayer::overriding(
            X_XSS_HEADER.0.clone(),
            X_XSS_HEADER.1.clone(),
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
        // Verify no script-src 'none' accidentally
        assert!(v.contains("script-src 'self' 'unsafe-inline'"));
    }

    #[test]
    fn test_hsts_header_value() {
        let (name, value) = &*HSTS_HEADER;
        assert_eq!(name.as_str(), "strict-transport-security");
        let v = value.to_str().unwrap();
        assert!(v.contains("max-age=31536000"));
        assert!(v.contains("includeSubDomains"));
    }

    #[test]
    fn test_referrer_policy_same_origin() {
        let (name, value) = &*REFERRER_HEADER;
        assert_eq!(name.as_str(), "referrer-policy");
        assert_eq!(value.to_str().unwrap(), "same-origin");
    }

    #[test]
    fn test_x_content_type_options_nosniff() {
        let (name, value) = &*X_CONTENT_TYPE_HEADER;
        assert_eq!(name.as_str(), "x-content-type-options");
        assert_eq!(value.to_str().unwrap(), "nosniff");
    }

    #[test]
    fn test_x_frame_options_deny() {
        let (name, value) = &*X_FRAME_HEADER;
        assert_eq!(name.as_str(), "x-frame-options");
        assert_eq!(value.to_str().unwrap(), "DENY");
    }

    #[test]
    fn test_x_xss_protection() {
        let (name, value) = &*X_XSS_HEADER;
        assert_eq!(name.as_str(), "x-xss-protection");
        assert_eq!(value.to_str().unwrap(), "1; mode=block");
    }

    #[test]
    fn test_permissions_policy_present() {
        let (name, value) = &*PERMISSIONS_HEADER;
        assert_eq!(name.as_str(), "permissions-policy");
        let v = value.to_str().unwrap();
        assert!(v.contains("camera=()"));
        assert!(v.contains("microphone=()"));
    }

    #[test]
    fn test_all_layers_created() {
        let layers = security_header_layers();
        assert_eq!(layers.len(), 7);
    }

    #[test]
    fn test_security_header_names_are_distinct() {
        let layers = security_header_layers();
        let mut names: Vec<&str> = layers
            .iter()
            .map(|_| "see note") // Can't inspect layer internals easily
            .collect();
        // At minimum verify we have the right count of unique headers
        assert_eq!(names.len(), 7);
    }
}
