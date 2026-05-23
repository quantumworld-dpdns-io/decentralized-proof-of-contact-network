use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use poi_core::types::{ContactProof, NodeId, OrbitalWindow, ProofId, VerificationStatus};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use utoipa::ToSchema;

use crate::auth::AuthConfig;
use crate::config::ApiConfig;
use crate::error::ApiError;
use crate::events::AppEvent;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub api_config: ApiConfig,
    pub auth_config: AuthConfig,
    pub proofs: Arc<RwLock<Vec<ContactProof>>>,
    pub peers: Arc<RwLock<Vec<PeerInfo>>>,
    pub windows: Arc<RwLock<Vec<OrbitalWindow>>>,
    pub event_tx: broadcast::Sender<AppEvent>,
    pub start_time: Instant,
}

impl AppStateInner {
    pub fn new(
        api_config: ApiConfig,
        auth_config: AuthConfig,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            api_config,
            auth_config,
            proofs: Arc::new(RwLock::new(Vec::new())),
            peers: Arc::new(RwLock::new(Vec::new())),
            windows: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            start_time: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofCreateRequest {
    pub target_node: NodeId,
    pub orbital_window: OrbitalWindow,
    pub proof_purpose: String,
    pub proving_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofCreateResponse {
    pub id: ProofId,
    pub proof: ContactProof,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofListResponse {
    pub proofs: Vec<ContactProof>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerifyRequest {
    pub proof_id: ProofId,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerifyResponse {
    pub verified: bool,
    pub details: VerificationDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerificationDetails {
    pub signature_valid: bool,
    pub timestamp_valid: bool,
    pub window_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchRequest {
    pub query: String,
    pub node_id: Option<NodeId>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<ContactProof>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PeerInfo {
    pub id: NodeId,
    pub address: String,
    pub connected_at: DateTime<Utc>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PeerListResponse {
    pub peers: Vec<PeerInfo>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatsResponse {
    pub total_proofs: u64,
    pub verified_proofs: u64,
    pub active_peers: u32,
    pub active_windows: u32,
    pub chain_length: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeInfo {
    pub id: NodeId,
    pub public_key: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub peer_count: u32,
    pub proof_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectRequest {
    pub address: String,
    pub node_id: Option<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WindowCreateRequest {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub window_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkStatsResponse {
    pub total_nodes: u32,
    pub active_nodes: u32,
    pub total_proofs: u64,
    pub proofs_per_second: f64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofStatsResponse {
    pub total: u64,
    pub verified: u64,
    pub failed: u64,
    pub pending: u64,
    pub expired: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalyticsSummaryResponse {
    pub total_proofs: u64,
    pub unique_provers: u64,
    pub unique_targets: u64,
    pub avg_confidence: f64,
    pub time_range_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeActivityResponse {
    pub node_id: NodeId,
    pub proofs_created: u64,
    pub proofs_verified: u64,
    pub last_active: DateTime<Utc>,
    pub activity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TopologyResponse {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TopologyNode {
    pub id: NodeId,
    pub degree: u32,
    pub centrality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TopologyEdge {
    pub source: NodeId,
    pub target: NodeId,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalyticsQueryRequest {
    pub query: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalyticsQueryResponse {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiQueryRequest {
    pub query: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiQueryResponse {
    pub answer: String,
    pub confidence: f64,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalyzeResponse {
    pub proof_id: ProofId,
    pub analysis: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnomalyResponse {
    pub anomalies: Vec<AnomalyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnomalyItem {
    pub proof_id: Option<ProofId>,
    pub node_id: Option<NodeId>,
    pub anomaly_type: String,
    pub severity: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SummarizeRequest {
    pub time_range_days: Option<i64>,
    pub include_graphs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SummarizeResponse {
    pub summary: String,
    pub key_metrics: serde_json::Value,
}

pub type ApiError = ApiError;
