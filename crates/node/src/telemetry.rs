use crate::config::ObservabilityConfig;
use crate::RuntimeManager;
use axum::routing::get;
use axum::Router;
use prometheus::{Encoder, Registry, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct TelemetryHandle {
    pub metrics_registry: Option<Registry>,
    metrics_server: Option<JoinHandle<()>>,
}

pub fn init_telemetry(
    config: &ObservabilityConfig,
    runtime: &RuntimeManager,
) -> Result<TelemetryHandle, Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            let level = if config.log_level.is_empty() {
                "info"
            } else {
                &config.log_level
            };
            tracing_subscriber::EnvFilter::new(level)
        });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    let (metrics_registry, metrics_server) =
        if let Some(prometheus_port) = config.prometheus_port {
            let registry = Registry::new();
            let reg = registry.clone();

            let app: Router = Router::new().route(
                "/metrics",
                get(move || {
                    let reg = reg.clone();
                    async move {
                        let encoder = TextEncoder::new();
                        let metric_families = reg.gather();
                        let mut buffer = vec![];
                        encoder.encode(&metric_families, &mut buffer).unwrap();
                        (
                            axum::http::StatusCode::OK,
                            [(
                                axum::http::header::CONTENT_TYPE,
                                "text/plain; charset=utf-8",
                            )],
                            axum::body::Body::from(buffer),
                        )
                    }
                }),
            );

            let addr = SocketAddr::from(([0, 0, 0, 0], prometheus_port));
            let handle = runtime.spawn(async move {
                tracing::info!("starting metrics server on {}", addr);
                let listener = TcpListener::bind(addr)
                    .await
                    .expect("failed to bind metrics server");
                axum::serve(listener, app)
                    .await
                    .expect("metrics server error");
            });

            (Some(registry), Some(handle))
        } else {
            (None, None)
        };

    Ok(TelemetryHandle {
        metrics_registry,
        metrics_server,
    })
}

impl TelemetryHandle {
    pub fn registry(&self) -> Option<&Registry> {
        self.metrics_registry.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuntimeManager;

    #[test]
    fn test_init_telemetry_no_metrics() {
        let config = ObservabilityConfig {
            otlp_endpoint: None,
            prometheus_port: None,
            log_level: "error".to_string(),
        };
        let mut rt = RuntimeManager::with_workers(1);
        rt.start().unwrap();
        let handle = init_telemetry(&config, &rt).unwrap();
        assert!(handle.metrics_registry.is_none());
        assert!(handle.registry().is_none());
    }
}
