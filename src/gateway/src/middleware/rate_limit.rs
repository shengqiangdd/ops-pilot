//! Rate limiting middleware for login endpoint using governor.

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiter keyed by IP address: 5 requests per minute.
pub type LoginRateLimiter = RateLimiter<
    String,
    governor::state::keyed::DefaultKeyedStateStore<String>,
    governor::clock::DefaultClock,
>;

/// Create a login rate limiter: 5 requests per minute per IP.
pub fn login_limiter() -> Arc<LoginRateLimiter> {
    Arc::new(RateLimiter::keyed(
        Quota::per_minute(NonZeroU32::new(5).unwrap()),
    ))
}

/// Extract client IP from request headers or socket address.
fn extract_client_ip(req: &Request) -> String {
    // Check X-Forwarded-For first (for reverse proxies)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            if let Some(first) = val.split(',').next() {
                return first.trim().to_string();
            }
        }
    }

    // Check X-Real-IP
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(val) = real_ip.to_str() {
            return val.trim().to_string();
        }
    }

    // Fall back to ConnectInfo if available, otherwise "unknown"
    if let Some(conn_info) = req.extensions().get::<ConnectInfo<std::net::SocketAddr>>() {
        return conn_info.0.ip().to_string();
    }

    "unknown".into()
}

/// Axum middleware that rate-limits requests using a keyed rate limiter.
///
/// Extracts the client IP and checks against the limiter. Returns 429 if exceeded.
pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<Arc<LoginRateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_client_ip(&request);

    if limiter.check_key(&ip).is_err() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[test]
    fn test_extract_client_ip_forwarded() {
        let req = Request::builder()
            .header("x-forwarded-for", "1.2.3.4, 5.6.7.8")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_ip(&req), "1.2.3.4");
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let req = Request::builder()
            .header("x-real-ip", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_unknown() {
        let req = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_client_ip(&req), "unknown");
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limiter = login_limiter();
        // 5 requests should all succeed
        for _ in 0..5 {
            assert!(limiter.check_key(&"1.2.3.4".to_string()).is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = login_limiter();
        for _ in 0..5 {
            let _ = limiter.check_key(&"10.0.0.1".to_string());
        }
        // 6th request should be blocked
        assert!(limiter.check_key(&"10.0.0.1".to_string()).is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_keys_by_ip() {
        let limiter = login_limiter();
        // Exhaust one IP
        for _ in 0..5 {
            let _ = limiter.check_key(&"1.1.1.1".to_string());
        }
        // Different IP should still work
        assert!(limiter.check_key(&"2.2.2.2".to_string()).is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_middleware_allows_normal() {
        let limiter = login_limiter();
        let app = axum::Router::new()
            .route("/test", axum::routing::get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                limiter.clone(),
                rate_limit_middleware,
            ))
            .with_state(limiter);

        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", "192.168.1.1")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limit_middleware_blocks_after_limit() {
        let limiter = login_limiter();
        let app = axum::Router::new()
            .route("/test", axum::routing::get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                limiter.clone(),
                rate_limit_middleware,
            ))
            .with_state(limiter);

        let ip = "172.16.0.1";
        // Exhaust the limit
        for _ in 0..5 {
            let req = Request::builder()
                .uri("/test")
                .header("x-forwarded-for", ip)
                .body(Body::empty())
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
        // Next request should be 429
        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
