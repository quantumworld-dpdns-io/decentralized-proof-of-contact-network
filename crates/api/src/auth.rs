use std::collections::HashSet;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub api_key: Option<String>,
    pub jwt_secret: Option<String>,
    pub enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            jwt_secret: None,
            enabled: false,
        }
    }
}

impl AuthConfig {
    pub fn with_api_key(key: impl Into<String>) -> Self {
        Self {
            api_key: Some(key.into()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub roles: HashSet<String>,
}

pub async fn auth_middleware(
    State(auth_config): State<AuthConfig>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if !auth_config.enabled {
        return Ok(next.run(req).await);
    }

    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match (&auth_config.api_key, api_key) {
        (Some(expected), Some(provided)) if expected == &provided => {
            let user = AuthenticatedUser {
                id: "api-key-user".to_string(),
                roles: HashSet::from(["admin".to_string()]),
            };
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        (Some(_), _) => Err(ApiError::Unauthorized("Invalid API key".into())),
        (None, _) => Ok(next.run(req).await),
    }
}

pub fn require_role(role: &str) -> impl Fn(Request, Next) -> Result<Response, ApiError> + Clone {
    let role = role.to_string();
    move |req: Request, next: Next| {
        let role = role.clone();
        async move {
            if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
                if user.roles.contains(&role) {
                    return Ok(next.run(req).await);
                }
            }
            Err(ApiError::Unauthorized(format!(
                "Role '{}' required",
                role
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::middleware::from_fn_with_state;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_auth_middleware_no_auth() {
        let auth_config = AuthConfig::default();
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(from_fn_with_state(auth_config.clone(), auth_middleware));

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_auth_middleware_valid_key() {
        let auth_config = AuthConfig::with_api_key("test-key-123");
        let mut config = auth_config.clone();
        config.enabled = true;
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(from_fn_with_state(config.clone(), auth_middleware));

        let req = Request::builder()
            .uri("/")
            .header("X-API-Key", "test-key-123")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_key() {
        let auth_config = AuthConfig::with_api_key("test-key-123");
        let mut config = auth_config.clone();
        config.enabled = true;
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(from_fn_with_state(config.clone(), auth_middleware));

        let req = Request::builder()
            .uri("/")
            .header("X-API-Key", "wrong-key")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 401);
    }
}
