pub mod client;
pub mod error;
pub mod models;

pub use client::ApiClient;
pub use error::{ClientError, Result};
pub use models::{
    AiQueryResponse, AnomalyReport, CreateProofRequest, CreateProofResponse, CreateWindowRequest,
    HealthStatus, NetworkStats, NodeInfo, NodeStats, PeerInfo, ProofAnalysis, ProofStats,
    QueryResult, VerifyResponse,
};
