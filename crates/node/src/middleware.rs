use axum::{
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::Next,
    response::{Json, Response},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(v) if v.starts_with("Bearer ") => Ok(next.run(req).await),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing or invalid Authorization header"})),
        )),
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    max_requests: u64,
    window_seconds: u64,
    counters: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut counters = self.counters.lock().expect("rate limiter lock poisoned");
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_seconds);

        let entry = counters.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > window {
            *entry = (1, now);
            true
        } else if entry.0 < self.max_requests {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let key = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Try to extract RateLimiter from extensions; if not present, allow through
    let limiter = req.extensions().get::<Arc<RateLimiter>>().cloned();

    if let Some(limiter) = limiter {
        if limiter.check(key) {
            Ok(next.run(req).await)
        } else {
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "Rate limit exceeded"})),
            ))
        }
    } else {
        Ok(next.run(req).await)
    }
}

pub async fn request_logging_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();
    tracing::info!(
        "{} {} {} {:?}",
        method,
        uri,
        status.as_u16(),
        duration
    );

    Ok(response)
}

pub fn cors_config(allowed_origins: &[String]) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    if allowed_origins.is_empty() {
        cors.allow_origin(Any)
    } else {
        let origins: Vec<axum::http::HeaderValue> = allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        cors.allow_origin(AllowOrigin::list(origins))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 60);
        assert!(limiter.check("client-1"));
        assert!(limiter.check("client-1"));
        assert!(limiter.check("client-1"));
        assert!(!limiter.check("client-1"));
        // Different client should be allowed
        assert!(limiter.check("client-2"));
    }

    #[test]
    fn test_cors_config_empty() {
        let cors = cors_config(&[]);
        // Just verify it doesn't panic
        let _ = cors;
    }

    #[test]
    fn test_cors_config_with_origins() {
        let origins = vec!["http://localhost:3000".to_string()];
        let cors = cors_config(&origins);
        let _ = cors;
    }
}
