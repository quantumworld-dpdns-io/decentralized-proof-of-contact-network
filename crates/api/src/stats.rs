use axum::extract::State;
use axum::response::Json;
use chrono::Utc;

use crate::error::ApiResult;
use crate::models::{AppState, NetworkStatsResponse, ProofStatsResponse, StatsResponse};

pub async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<StatsResponse>> {
    let proofs = state.proofs.read().await;
    let peers = state.peers.read().await;
    let windows = state.windows.read().await;

    let active_windows = windows
        .iter()
        .filter(|w| w.start_time <= Utc::now() && Utc::now() <= w.end_time)
        .count() as u32;

    Ok(Json(StatsResponse {
        total_proofs: proofs.len() as u64,
        verified_proofs: 0,
        active_peers: peers.len() as u32,
        active_windows,
        chain_length: 0,
        uptime_seconds: state.start_time.elapsed().as_secs(),
    }))
}

pub async fn get_network_stats(
    State(state): State<AppState>,
) -> ApiResult<Json<NetworkStatsResponse>> {
    let peers = state.peers.read().await;
    let proofs = state.proofs.read().await;

    Ok(Json(NetworkStatsResponse {
        total_nodes: peers.len() as u32 + 1,
        active_nodes: peers.len() as u32,
        total_proofs: proofs.len() as u64,
        proofs_per_second: 0.0,
        avg_latency_ms: 0.0,
    }))
}

pub async fn get_proof_stats(
    State(state): State<AppState>,
) -> ApiResult<Json<ProofStatsResponse>> {
    let proofs = state.proofs.read().await;
    let now = Utc::now();

    let expired = proofs
        .iter()
        .filter(|p| p.orbital_window.end_time < now)
        .count() as u64;

    let pending = proofs
        .iter()
        .filter(|p| p.orbital_window.start_time > now)
        .count() as u64;

    let total = proofs.len() as u64;

    Ok(Json(ProofStatsResponse {
        total,
        verified: 0,
        failed: 0,
        pending,
        expired,
    }))
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
    async fn test_get_stats() {
        let state = test_state();
        let resp = get_stats(State(state)).await.unwrap();
        assert_eq!(resp.0.total_proofs, 0);
    }

    #[tokio::test]
    async fn test_get_network_stats() {
        let state = test_state();
        let resp = get_network_stats(State(state)).await.unwrap();
        assert_eq!(resp.0.total_nodes, 1);
    }

    #[tokio::test]
    async fn test_get_proof_stats() {
        let state = test_state();
        let resp = get_proof_stats(State(state)).await.unwrap();
        assert_eq!(resp.0.total, 0);
    }
}
