use axum::extract::State;
use axum::response::Json;
use chrono::Utc;
use serde_json::json;

use crate::error::ApiResult;
use crate::models::{
    AnalyticsQueryRequest, AnalyticsQueryResponse, AnalyticsSummaryResponse,
    AppState, NodeActivityResponse, TopologyEdge, TopologyNode, TopologyResponse,
};

pub async fn get_proofs_summary(
    State(state): State<AppState>,
) -> ApiResult<Json<AnalyticsSummaryResponse>> {
    let proofs = state.proofs.read().await;

    let mut unique_provers: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut unique_targets: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut total_confidence = 0.0;

    for p in proofs.iter() {
        unique_provers.insert(&p.proving_node.0);
        unique_targets.insert(&p.target_node.0);
        total_confidence += p.metadata.confidence_score;
    }

    let avg_confidence = if proofs.is_empty() {
        0.0
    } else {
        total_confidence / proofs.len() as f64
    };

    Ok(Json(AnalyticsSummaryResponse {
        total_proofs: proofs.len() as u64,
        unique_provers: unique_provers.len() as u64,
        unique_targets: unique_targets.len() as u64,
        avg_confidence,
        time_range_days: 1,
    }))
}

pub async fn get_node_activity(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<NodeActivityResponse>>> {
    let proofs = state.proofs.read().await;

    let mut activity: std::collections::HashMap<String, (u64, u64, chrono::DateTime<Utc>)> =
        std::collections::HashMap::new();

    for p in proofs.iter() {
        let entry = activity
            .entry(p.proving_node.0.clone())
            .or_insert((0, 0, Utc::now()));
        entry.0 += 1;
        if p.timestamp > entry.2 {
            entry.2 = p.timestamp;
        }
    }

    let responses: Vec<NodeActivityResponse> = activity
        .into_iter()
        .map(|(node_id, (created, verified, last_active))| {
            let node_id_obj = poi_core::types::NodeId(node_id);
            NodeActivityResponse {
                node_id: node_id_obj,
                proofs_created: created,
                proofs_verified: verified,
                last_active,
                activity_score: 0.5,
            }
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_network_topology(
    State(state): State<AppState>,
) -> ApiResult<Json<TopologyResponse>> {
    let proofs = state.proofs.read().await;

    let mut nodes_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut edge_counts: std::collections::HashMap<(String, String), u32> =
        std::collections::HashMap::new();

    for p in proofs.iter() {
        nodes_set.insert(p.proving_node.0.clone());
        nodes_set.insert(p.target_node.0.clone());
        let key = (p.proving_node.0.clone(), p.target_node.0.clone());
        *edge_counts.entry(key).or_insert(0) += 1;
    }

    let nodes: Vec<TopologyNode> = nodes_set
        .into_iter()
        .map(|id| TopologyNode {
            id: poi_core::types::NodeId(id),
            degree: 0,
            centrality: 0.0,
        })
        .collect();

    let edges: Vec<TopologyEdge> = edge_counts
        .into_iter()
        .map(|((source, target), weight)| TopologyEdge {
            source: poi_core::types::NodeId(source),
            target: poi_core::types::NodeId(target),
            weight: weight as f64,
        })
        .collect();

    Ok(Json(TopologyResponse { nodes, edges }))
}

pub async fn run_analytics_query(
    State(_state): State<AppState>,
    Json(req): Json<AnalyticsQueryRequest>,
) -> ApiResult<Json<AnalyticsQueryResponse>> {
    Ok(Json(AnalyticsQueryResponse {
        columns: vec!["result".into()],
        rows: vec![vec![serde_json::Value::String(format!(
            "Query '{}' received (execution not implemented)",
            req.query
        ))]],
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
    async fn test_get_proofs_summary() {
        let state = test_state();
        let resp = get_proofs_summary(State(state)).await.unwrap();
        assert_eq!(resp.0.total_proofs, 0);
    }

    #[tokio::test]
    async fn test_get_node_activity() {
        let state = test_state();
        let resp = get_node_activity(State(state)).await.unwrap();
        assert!(resp.0.is_empty());
    }

    #[tokio::test]
    async fn test_get_network_topology() {
        let state = test_state();
        let resp = get_network_topology(State(state)).await.unwrap();
        assert!(resp.0.nodes.is_empty());
    }
}
