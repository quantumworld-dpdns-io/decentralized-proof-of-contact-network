use axum::extract::{Path, State};
use axum::response::Json;
use chrono::Utc;
use poi_core::types::NodeId;
use serde_json::json;

use crate::error::{ApiError, ApiResult};
use crate::models::{AppState, ConnectRequest, NodeInfo, PeerInfo, PeerListResponse};

pub async fn get_node_info(State(state): State<AppState>) -> ApiResult<Json<NodeInfo>> {
    let proofs = state.proofs.read().await;
    let peers = state.peers.read().await;

    Ok(Json(NodeInfo {
        id: NodeId::new(),
        public_key: "generated-key".into(),
        version: "0.1.0".into(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        peer_count: peers.len() as u32,
        proof_count: proofs.len() as u64,
    }))
}

pub async fn list_peers(State(state): State<AppState>) -> ApiResult<Json<PeerListResponse>> {
    let peers = state.peers.read().await;
    Ok(Json(PeerListResponse {
        peers: peers.clone(),
        total: peers.len() as u32,
    }))
}

pub async fn connect_peer(
    State(state): State<AppState>,
    Json(req): Json<ConnectRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let peer = PeerInfo {
        id: req.node_id.unwrap_or_else(NodeId::new),
        address: req.address,
        connected_at: Utc::now(),
        latency_ms: 0,
    };

    let mut peers = state.peers.write().await;
    peers.push(peer);
    drop(peers);

    Ok(Json(json!({"status": "connected"})))
}

pub async fn disconnect_peer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let node_id = NodeId(id.clone());
    let mut peers = state.peers.write().await;
    let initial_len = peers.len();
    peers.retain(|p| p.id != node_id);
    if peers.len() == initial_len {
        return Err(ApiError::NotFound(format!("Peer {} not found", id)));
    }
    drop(peers);

    Ok(Json(json!({"status": "disconnected", "id": id})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::auth::AuthConfig;
    use crate::config::ApiConfig;
    use crate::models::AppStateInner;

    fn test_state() -> AppState {
        Arc::new(AppStateInner::new(ApiConfig::default(), AuthConfig::default()))
    }

    #[tokio::test]
    async fn test_get_node_info() {
        let state = test_state();
        let resp = get_node_info(State(state)).await.unwrap();
        assert_eq!(resp.0.version, "0.1.0");
    }

    #[tokio::test]
    async fn test_list_peers_empty() {
        let state = test_state();
        let resp = list_peers(State(state)).await.unwrap();
        assert_eq!(resp.0.total, 0);
    }

    #[tokio::test]
    async fn test_connect_peer() {
        let state = test_state();
        let req = ConnectRequest {
            address: "127.0.0.1:9000".into(),
            node_id: None,
        };
        let resp = connect_peer(State(state.clone()), Json(req)).await.unwrap();
        assert_eq!(resp.0["status"], "connected");
        let peers_resp = list_peers(State(state)).await.unwrap();
        assert_eq!(peers_resp.0.total, 1);
    }
}
