use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default = "default_node_id")]
    pub node_id: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            node_id: default_node_id(),
            data_dir: default_data_dir(),
            log_level: default_log_level(),
        }
    }
}

fn default_node_id() -> String {
    "poi-node".to_string()
}

fn default_data_dir() -> String {
    "./data".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProofId(pub uuid::Uuid);

impl Default for ProofId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactProof {
    pub id: ProofId,
    pub peer_id: NodeId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub signature: Vec<u8>,
    pub payload: Vec<u8>,
}

impl ContactProof {
    pub fn new(peer_id: NodeId, payload: Vec<u8>) -> Self {
        Self {
            id: ProofId::default(),
            peer_id,
            timestamp: chrono::Utc::now(),
            signature: Vec::new(),
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofStatus {
    Pending,
    Signed,
    Stored,
    Gossiped,
    Verified,
    Archived,
    Failed,
}
