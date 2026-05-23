pub mod ai;
pub mod analytics;
pub mod auth;
pub mod config;
pub mod error;
pub mod events;
pub mod health;
pub mod models;
pub mod nodes;
pub mod proofs;
pub mod router;
pub mod stats;
pub mod windows;

pub use config::ApiConfig;
pub use error::ApiError;
pub use models::{AppState, AppStateInner, PeerInfo};
pub use router::create_router;

use std::sync::Arc;

use axum::serve;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::auth::AuthConfig;

pub async fn start_server(api_config: ApiConfig, auth_config: AuthConfig) -> anyhow::Result<()> {
    let state = Arc::new(AppStateInner::new(api_config.clone(), auth_config));
    let router = create_router(state);

    let listener = TcpListener::bind(&api_config.bind_addr).await.map_err(|e| {
        error!("Failed to bind to {}: {}", api_config.bind_addr, e);
        e
    })?;

    info!("API server listening on {}", api_config.bind_addr);

    serve(listener, router)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.bind_addr, "0.0.0.0:3000");
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
        assert_eq!(config.rate_limit_rpm, 60);
        assert!(config.enable_swagger);
    }

    #[test]
    fn test_api_config_serde() {
        let config = ApiConfig {
            bind_addr: "127.0.0.1:8080".into(),
            allowed_origins: vec!["http://localhost:3000".into()],
            tls_cert: None,
            tls_key: None,
            max_body_size: 5 * 1024 * 1024,
            rate_limit_rpm: 30,
            enable_swagger: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ApiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bind_addr, "127.0.0.1:8080");
        assert!(!deserialized.enable_swagger);
    }
}
