use axum::extract::State;
use axum::response::Json;
use prometheus::TextEncoder;
use serde_json::json;
use utoipa::ToSchema;
use crate::models::AppState;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub version: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct VersionResponse {
    pub version: &'static str,
    pub build: &'static str,
    pub rustc: &'static str,
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Health check OK", body = HealthResponse)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "version": "0.1.0",
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/version",
    responses(
        (status = 200, description = "Version information", body = VersionResponse)
    )
)]
pub async fn version_info() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: "0.1.0",
        build: option_env!("PROFILE").unwrap_or("unknown"),
        rustc: option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("1.75"),
    })
}

pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = String::new();
    if let Err(e) = encoder.encode_utf8(&metric_families, &mut buffer) {
        return format!("# Error encoding metrics: {}", e);
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::auth::AuthConfig;
    use crate::config::ApiConfig;
    use crate::models::AppStateInner;

    #[tokio::test]
    async fn test_health_check() {
        let state = Arc::new(AppStateInner::new(ApiConfig::default(), AuthConfig::default()));
        let response = health_check(State(state)).await;
        assert_eq!(response.0["status"], "ok");
    }

    #[tokio::test]
    async fn test_version_info() {
        let response = version_info().await;
        assert_eq!(response.version, "0.1.0");
    }
}
