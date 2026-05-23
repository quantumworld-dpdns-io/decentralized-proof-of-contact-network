use axum::extract::{Path, State};
use axum::response::Json;
use poi_core::types::ProofId;
use serde_json::json;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::models::{
    AiQueryRequest, AiQueryResponse, AnalyzeResponse, AnomalyItem, AnomalyResponse, AppState,
    SummarizeRequest, SummarizeResponse,
};

pub async fn ai_query(
    State(_state): State<AppState>,
    Json(req): Json<AiQueryRequest>,
) -> ApiResult<Json<AiQueryResponse>> {
    Ok(Json(AiQueryResponse {
        answer: format!(
            "AI query '{}' received. Analysis is not yet implemented.",
            req.query
        ),
        confidence: 0.0,
        sources: Vec::new(),
    }))
}

pub async fn analyze_proof(
    State(state): State<AppState>,
    Path(proof_id): Path<Uuid>,
) -> ApiResult<Json<AnalyzeResponse>> {
    let pid = ProofId(proof_id);
    let proofs = state.proofs.read().await;
    let _proof = proofs
        .iter()
        .find(|p| p.id == pid)
        .ok_or_else(|| ApiError::NotFound(format!("Proof {} not found", proof_id)))?;
    drop(proofs);

    Ok(Json(AnalyzeResponse {
        proof_id: pid,
        analysis: "Proof analysis not yet implemented.".into(),
        recommendations: vec!["Verify the proof signature".into()],
    }))
}

pub async fn detect_anomalies(
    State(state): State<AppState>,
) -> ApiResult<Json<AnomalyResponse>> {
    let proofs = state.proofs.read().await;
    let mut anomalies = Vec::new();

    for proof in proofs.iter() {
        if proof.metadata.confidence_score < 0.3 {
            anomalies.push(AnomalyItem {
                proof_id: Some(proof.id.clone()),
                node_id: Some(proof.proving_node.clone()),
                anomaly_type: "low_confidence".into(),
                severity: "medium".into(),
                description: format!(
                    "Low confidence score: {}",
                    proof.metadata.confidence_score
                ),
                timestamp: proof.timestamp,
            });
        }
    }

    Ok(Json(AnomalyResponse { anomalies }))
}

pub async fn summarize_network(
    State(state): State<AppState>,
    Json(_req): Json<SummarizeRequest>,
) -> ApiResult<Json<SummarizeResponse>> {
    let proofs = state.proofs.read().await;
    let peers = state.peers.read().await;

    Ok(Json(SummarizeResponse {
        summary: format!(
            "Network summary: {} proofs, {} peers connected.",
            proofs.len(),
            peers.len()
        ),
        key_metrics: json!({
            "total_proofs": proofs.len(),
            "total_peers": peers.len(),
        }),
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
    async fn test_ai_query() {
        let state = test_state();
        let req = AiQueryRequest {
            query: "test".into(),
            context: None,
        };
        let resp = ai_query(State(state), Json(req)).await.unwrap();
        assert!(resp.0.answer.contains("test"));
    }

    #[tokio::test]
    async fn test_analyze_proof_not_found() {
        let state = test_state();
        let id = Uuid::new_v4();
        let result = analyze_proof(State(state), Path(id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_detect_anomalies() {
        let state = test_state();
        let resp = detect_anomalies(State(state)).await.unwrap();
        assert!(resp.0.anomalies.is_empty());
    }

    #[tokio::test]
    async fn test_summarize_network() {
        let state = test_state();
        let req = SummarizeRequest {
            time_range_days: None,
            include_graphs: None,
        };
        let resp = summarize_network(State(state), Json(req)).await.unwrap();
        assert!(resp.0.summary.contains("0 proofs"));
    }
}
