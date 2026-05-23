use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use poi_core::{ContactProof, NodeId, OrbitalWindow, ProofMetadata, WindowType};

// ── Proof operations ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProofRequest {
    pub proving_node: NodeId,
    pub target_node: NodeId,
    pub orbital_window: OrbitalWindow,
    pub metadata: ProofMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProofResponse {
    pub proof: ContactProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub status: String,
    pub details: Option<serde_json::Value>,
}

// ── Node operations ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub version: String,
    pub uptime_seconds: u64,
    pub proof_count: u64,
    pub peer_count: u64,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub connected_since: Option<DateTime<Utc>>,
    pub protocol_version: String,
    pub latency_ms: Option<u64>,
}

// ── Window operations ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWindowRequest {
    #[serde(rename = "type")]
    pub window_type: WindowType,
    pub duration_minutes: Option<i64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

// ── Stats ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub total_proofs: u64,
    pub active_peers: u64,
    pub uptime_seconds: u64,
    pub memory_usage_bytes: Option<u64>,
    pub cpu_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_peers: u64,
    pub active_connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStats {
    pub total_proofs: u64,
    pub verified_proofs: u64,
    pub pending_proofs: u64,
    pub expired_proofs: u64,
    pub failed_proofs: u64,
    pub avg_confidence_score: f64,
}

// ── Analytics ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: Option<u64>,
}

// ── AI ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiQueryResponse {
    pub answer: String,
    pub confidence: f64,
    pub sources: Option<Vec<String>>,
    pub processing_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofAnalysis {
    pub proof_id: String,
    pub risk_score: f64,
    pub anomalies: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub id: String,
    pub severity: String,
    pub description: String,
    pub affected_proofs: Vec<String>,
    pub detected_at: DateTime<Utc>,
    pub resolved: bool,
}

// ── Health ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub node_id: NodeId,
    pub checks: HashMap<String, String>,
}
