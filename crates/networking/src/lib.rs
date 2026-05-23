use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default)]
    pub external_addr: Option<String>,
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            external_addr: None,
            bootstrap_peers: Vec::new(),
            max_peers: default_max_peers(),
        }
    }
}

fn default_listen_addr() -> String {
    "0.0.0.0:9090".to_string()
}

fn default_max_peers() -> usize {
    50
}

#[derive(Debug)]
pub struct NetworkManager;

impl NetworkManager {
    pub async fn start(_config: &NetworkConfig) -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
