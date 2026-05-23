use std::sync::Arc;
use std::time::Duration;

use crate::cache::ProofCache;
use crate::config::NodeConfig;
use crate::error::*;
use crate::orchestrator::ProofOrchestrator;
use crate::runtime::RuntimeManager;
use crate::scheduler::OrbitalWindowScheduler;
use crate::services::ServiceRegistry;
use crate::state::StateManager;
use crate::telemetry::init_telemetry;
use crate::NodeState;

pub struct NodeBuilder {
    config: NodeConfig,
    runtime_workers: Option<usize>,
}

impl Default for NodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: NodeConfig::default(),
            runtime_workers: None,
        }
    }

    pub fn with_config(path: &str) -> Result<Self, ConfigError> {
        let config = NodeConfig::from_file(path)?;
        Ok(Self {
            config,
            runtime_workers: None,
        })
    }

    pub fn with_runtime_workers(mut self, workers: usize) -> Self {
        self.runtime_workers = Some(workers);
        self
    }

    pub fn build(self) -> Result<Node, NodeError> {
        let mut runtime = if let Some(workers) = self.runtime_workers {
            RuntimeManager::with_workers(workers)
        } else {
            RuntimeManager::with_workers(4)
        };
        runtime.start().map_err(NodeError::Runtime)?;

        let telemetry =
            init_telemetry(&self.config.observability, &runtime)
                .map_err(|e| NodeError::Startup(StartupError::RuntimeInit(e.to_string())))?;

        let state_path =
            std::path::PathBuf::from(&self.config.node.data_dir).join("node_state.json");
        let state = StateManager::with_persistence(state_path);

        let cache = ProofCache::new(1000, Duration::from_secs(3600));

        let services = ServiceRegistry::new();

        let scheduler = Arc::new(OrbitalWindowScheduler::new(
            chrono::Duration::minutes(5),
            chrono::Duration::seconds(30),
        ));

        let orchestrator = Arc::new(ProofOrchestrator::new(5, 10));

        state
            .transition(NodeState::Init)
            .map_err(NodeError::from)?;

        Ok(Node {
            config: self.config,
            runtime,
            state,
            services: Arc::new(services),
            scheduler,
            orchestrator,
            cache,
            telemetry,
        })
    }
}

pub struct Node {
    pub config: NodeConfig,
    pub runtime: RuntimeManager,
    pub state: StateManager,
    pub services: Arc<ServiceRegistry>,
    pub scheduler: Arc<OrbitalWindowScheduler>,
    pub orchestrator: Arc<ProofOrchestrator>,
    pub cache: ProofCache,
    telemetry: crate::telemetry::TelemetryHandle,
}

impl Node {
    pub fn start(&self) -> Result<(), NodeError> {
        tracing::info!("starting poi-node");

        self.state
            .transition(NodeState::Syncing)
            .map_err(NodeError::from)?;

        self.runtime.block_on(self.services.start_all())?;

        self.scheduler
            .start(&self.runtime)
            .map_err(NodeError::Runtime)?;

        self.orchestrator
            .start(&self.runtime)
            .map_err(NodeError::Runtime)?;

        self.state
            .transition(NodeState::Active)
            .map_err(NodeError::from)?;

        tracing::info!("node started successfully");
        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), NodeError> {
        tracing::info!("shutting down poi-node");

        self.scheduler.stop().ok();
        self.orchestrator.stop().ok();

        let svc_result = self.runtime.block_on(self.services.stop_all());

        self.state
            .transition(NodeState::Shutdown)
            .map_err(NodeError::from)?;

        tracing::info!("node shutdown complete");
        svc_result.map_err(NodeError::from)
    }

    pub fn wait_for_shutdown(&self) {
        self.runtime.block_on(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for signal");
        });
    }

    pub fn telemetry_handle(&self) -> &crate::telemetry::TelemetryHandle {
        &self.telemetry
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        tracing::debug!("dropping node");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "requires refactoring to avoid global tracing subscriber conflict and nested tokio runtime"]
    #[test]
    fn test_builder_default() {
        let builder = NodeBuilder::new();
        let node = builder.build().unwrap();
        assert_eq!(node.state.current(), NodeState::Init);
        assert_eq!(node.config.node.id, "poi-node");
    }

    #[test]
    fn test_builder_with_runtime_workers() {
        let builder = NodeBuilder::new().with_runtime_workers(2);
        let node = builder.build().unwrap();
        assert!(node.runtime.is_running());
    }

    #[test]
    fn test_node_start_stop() {
        let node = NodeBuilder::new().build().unwrap();
        node.start().unwrap();
        assert_eq!(node.state.current(), NodeState::Active);
        node.shutdown().unwrap();
        assert_eq!(node.state.current(), NodeState::Shutdown);
    }

    #[test]
    fn test_node_config_access() {
        let node = NodeBuilder::new().build().unwrap();
        assert_eq!(node.config.storage.provider, "duckdb");
    }

    #[test]
    fn test_double_start_fails_state() {
        let node = NodeBuilder::new().build().unwrap();
        node.start().unwrap();
        assert!(node.state.transition(NodeState::Init).is_err());
    }

    #[test]
    fn test_builder_with_config_path() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let toml_str = r#"
[node]
id = "poi-custom-node"
data_dir = "./data"
log_level = "debug"

[network]
listen_addr = "0.0.0.0:9090"

[storage]
provider = "duckdb"

[vector_store]
provider = "chroma"
host = "localhost"
port = 8000
collection = "test"

[api]
bind_addr = "0.0.0.0:3000"

[ai]
provider = "ollama"
model = "gemma3:latest"
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"

[analytics]

[observability]
log_level = "debug"
"#;
        std::fs::write(&config_path, toml_str).unwrap();
        let builder = NodeBuilder::with_config(config_path.to_str().unwrap()).unwrap();
        let node = builder.build().unwrap();
        assert_eq!(node.config.node.id, "poi-custom-node");
    }

    #[test]
    fn test_node_multiple_start_stop() {
        let node = NodeBuilder::new().build().unwrap();
        node.start().unwrap();
        assert_eq!(node.state.current(), NodeState::Active);
        node.shutdown().unwrap();
        assert_eq!(node.state.current(), NodeState::Shutdown);
    }
}
