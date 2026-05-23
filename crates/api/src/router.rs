use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{delete, get, post};
use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::ai;
use crate::analytics;
use crate::auth;
use crate::error::ApiError;
use crate::events;
use crate::health;
use crate::models::AppState;
use crate::nodes;
use crate::proofs;
use crate::stats;
use crate::windows;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::health::health_check,
        crate::health::version_info,
    ),
    components(
        schemas(
            crate::models::ProofCreateRequest,
            crate::models::ProofCreateResponse,
            crate::models::ProofListResponse,
            crate::models::VerifyRequest,
            crate::models::VerifyResponse,
            crate::models::VerificationDetails,
            crate::models::SearchRequest,
            crate::models::SearchResponse,
            crate::models::PeerInfo,
            crate::models::PeerListResponse,
            crate::models::StatsResponse,
            crate::models::NodeInfo,
            crate::models::ConnectRequest,
            crate::models::WindowCreateRequest,
            crate::models::NetworkStatsResponse,
            crate::models::ProofStatsResponse,
            crate::models::AnalyticsSummaryResponse,
            crate::models::NodeActivityResponse,
            crate::models::TopologyResponse,
            crate::models::TopologyNode,
            crate::models::TopologyEdge,
            crate::models::AnalyticsQueryRequest,
            crate::models::AnalyticsQueryResponse,
            crate::models::AiQueryRequest,
            crate::models::AiQueryResponse,
            crate::models::AnalyzeResponse,
            crate::models::AnomalyResponse,
            crate::models::AnomalyItem,
            crate::models::SummarizeRequest,
            crate::models::SummarizeResponse,
            crate::health::HealthResponse,
            crate::health::VersionResponse,
        )
    ),
    tags(
        (name = "poi-api", description = "Proof of Contact API")
    )
)]
pub struct ApiDoc;

pub struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut map = self.requests.lock().unwrap();
        let now = Instant::now();
        let timestamps = map.entry(key.to_string()).or_default();
        timestamps.retain(|t| now.duration_since(*t) < self.window);
        if timestamps.len() >= self.max_requests as usize {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

async fn rate_limit_middleware(
    State(limiter): State<std::sync::Arc<RateLimiter>>,
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let key = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get("X-Real-IP")
                .and_then(|v| v.to_str().ok())
        })
        .unwrap_or("unknown")
        .to_string();

    if limiter.check(&key) {
        Ok(next.run(req).await)
    } else {
        Err(ApiError::RateLimited)
    }
}

async fn request_logging_middleware(
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();
    info!("{} {} {} {:?}", method, uri, status.as_u16(), duration);

    response
}

pub fn create_router(state: AppState) -> Router {
    let api_config = &state.api_config;
    let rate_limit_rpm = api_config.rate_limit_rpm;
    let max_body_size = api_config.max_body_size;

    let rate_limiter = std::sync::Arc::new(RateLimiter::new(rate_limit_rpm, 60));
    let auth_config = state.auth_config.clone();

    let cors = if api_config.allowed_origins.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = api_config
            .allowed_origins
            .iter()
            .map(|o| o.parse().expect("Invalid origin"))
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_config,
            auth::auth_middleware,
        ));

    let mut router = Router::new()
        .route("/api/v1/health", get(health::health_check))
        .route("/api/v1/version", get(health::version_info))
        .route("/api/v1/metrics", get(health::metrics_handler))
        .route("/api/v1/proofs", get(proofs::list_proofs).post(proofs::create_proof))
        .route("/api/v1/proofs/search", post(proofs::search_proofs))
        .route("/api/v1/proofs/verify", post(proofs::verify_proof))
        .route("/api/v1/proofs/:id", get(proofs::get_proof).delete(proofs::delete_proof))
        .route("/api/v1/proofs/:id/chain", get(proofs::get_proof_chain))
        .route("/api/v1/nodes/me", get(nodes::get_node_info))
        .route("/api/v1/nodes/peers", get(nodes::list_peers))
        .route("/api/v1/nodes/connect", post(nodes::connect_peer))
        .route("/api/v1/nodes/peers/:id", delete(nodes::disconnect_peer))
        .route("/api/v1/windows", get(windows::list_windows).post(windows::create_window))
        .route("/api/v1/windows/active", get(windows::get_active_windows))
        .route("/api/v1/windows/:id", get(windows::get_window))
        .route("/api/v1/stats", get(stats::get_stats))
        .route("/api/v1/stats/network", get(stats::get_network_stats))
        .route("/api/v1/stats/proofs", get(stats::get_proof_stats))
        .route("/api/v1/events", get(events::handle_events))
        .route(
            "/api/v1/analytics/proofs/summary",
            get(analytics::get_proofs_summary),
        )
        .route(
            "/api/v1/analytics/nodes/activity",
            get(analytics::get_node_activity),
        )
        .route(
            "/api/v1/analytics/network/topology",
            get(analytics::get_network_topology),
        )
        .route("/api/v1/analytics/query", post(analytics::run_analytics_query))
        .route("/api/v1/ai/query", post(ai::ai_query))
        .route("/api/v1/ai/analyze/:proof_id", post(ai::analyze_proof))
        .route("/api/v1/ai/anomalies", get(ai::detect_anomalies))
        .route("/api/v1/ai/summarize", post(ai::summarize_network))
        .layer(middleware_stack)
        .layer(RequestBodyLimitLayer::new(max_body_size))
        .with_state(state);

    if api_config.enable_swagger {
        router = router.merge(
            SwaggerUi::new("/api/v1/docs")
                .url("/api/v1/openapi.json", ApiDoc::openapi()),
        );
    }

    router
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    use crate::auth::AuthConfig;
    use crate::config::ApiConfig;

    #[tokio::test]
    async fn test_router_health() {
        let state = std::sync::Arc::new(AppStateInner::new(
            ApiConfig::default(),
            AuthConfig::default(),
        ));
        let app = create_router(state);
        let req = Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_router_version() {
        let state = std::sync::Arc::new(AppStateInner::new(
            ApiConfig::default(),
            AuthConfig::default(),
        ));
        let app = create_router(state);
        let req = Request::builder()
            .uri("/api/v1/version")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_router_404() {
        let state = std::sync::Arc::new(AppStateInner::new(
            ApiConfig::default(),
            AuthConfig::default(),
        ));
        let app = create_router(state);
        let req = Request::builder()
            .uri("/api/v1/nonexistent")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 404);
    }
}
