use axum::extract::{Path, Query, State};
use axum::response::Json;
use poi_core::contact_proof::ContactProofExt;
use poi_core::keypair::KeyPairExt;
use poi_core::types::{ContactProof, KeyPair, NodeId, ProofId, ProofMetadata};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::events::AppEvent;
use crate::models::{
    AppState, ProofCreateRequest, ProofCreateResponse, ProofListResponse, SearchRequest,
    SearchResponse, VerifyRequest, VerifyResponse, VerificationDetails,
};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ProofQueryParams {
    pub node_id: Option<String>,
}

pub async fn create_proof(
    State(state): State<AppState>,
    Json(req): Json<ProofCreateRequest>,
) -> ApiResult<Json<ProofCreateResponse>> {
    let kp = KeyPair::generate();
    let node_id = NodeId::new();

    let metadata = ProofMetadata {
        protocol_version: "1.0".into(),
        chain_position: None,
        confidence_score: 1.0,
        proof_purpose: req.proof_purpose,
    };

    let mut proof = ContactProof::new(node_id, req.target_node, req.orbital_window, metadata);
    proof.sign(&kp);

    let mut proofs = state.proofs.write().await;
    proofs.push(proof.clone());
    drop(proofs);

    let _ = state.event_tx.send(AppEvent::ProofCreated(proof.clone()));

    Ok(Json(ProofCreateResponse {
        id: proof.id.clone(),
        proof,
        status: "created".into(),
    }))
}

pub async fn get_proof(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ContactProof>> {
    let proof_id = ProofId(id);
    let proofs = state.proofs.read().await;
    let proof = proofs
        .iter()
        .find(|p| p.id == proof_id)
        .ok_or_else(|| ApiError::NotFound(format!("Proof {} not found", id)))?
        .clone();
    Ok(Json(proof))
}

pub async fn list_proofs(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ProofListResponse>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);

    let proofs = state.proofs.read().await;
    let total = proofs.len() as u64;
    let start = ((page - 1) as usize).min(proofs.len());
    let end = (start + page_size as usize).min(proofs.len());
    let page_proofs = proofs[start..end].to_vec();
    drop(proofs);

    Ok(Json(ProofListResponse {
        proofs: page_proofs,
        total,
        page,
        page_size,
    }))
}

pub async fn delete_proof(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let proof_id = ProofId(id);
    let mut proofs = state.proofs.write().await;
    let initial_len = proofs.len();
    proofs.retain(|p| p.id != proof_id);
    if proofs.len() == initial_len {
        return Err(ApiError::NotFound(format!("Proof {} not found", id)));
    }
    drop(proofs);

    Ok(Json(serde_json::json!({"status": "deleted", "id": id.to_string()})))
}

pub async fn verify_proof(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> ApiResult<Json<VerifyResponse>> {
    let proofs = state.proofs.read().await;
    let proof = proofs
        .iter()
        .find(|p| p.id == req.proof_id)
        .ok_or_else(|| ApiError::NotFound(format!("Proof {} not found", req.proof_id.0)))?
        .clone();
    drop(proofs);

    let public_key = poi_core::types::PublicKey(req.public_key);

    let signature_valid = poi_core::verification::verify_signature(&proof, &public_key).unwrap_or(false);
    let timestamp_valid = poi_core::verification::verify_timestamp(
        &proof,
        std::time::Duration::from_secs(300),
    )
    .unwrap_or(false);
    let window_valid = poi_core::verification::verify_orbital_window(&proof).unwrap_or(false);

    let verified = signature_valid && timestamp_valid && window_valid;

    Ok(Json(VerifyResponse {
        verified,
        details: VerificationDetails {
            signature_valid,
            timestamp_valid,
            window_valid,
        },
    }))
}

pub async fn get_proof_chain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<ContactProof>>> {
    let proof_id = ProofId(id);
    let proofs = state.proofs.read().await;

    let _proof = proofs
        .iter()
        .find(|p| p.id == proof_id)
        .ok_or_else(|| ApiError::NotFound(format!("Proof {} not found", id)))?;

    Ok(Json(proofs.clone()))
}

pub async fn search_proofs(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> ApiResult<Json<SearchResponse>> {
    let proofs = state.proofs.read().await;
    let query = req.query.to_lowercase();

    let results: Vec<ContactProof> = proofs
        .iter()
        .filter(|p| {
            let matches_query = query.is_empty()
                || p.proving_node.0.to_lowercase().contains(&query)
                || p.target_node.0.to_lowercase().contains(&query)
                || p.metadata.proof_purpose.to_lowercase().contains(&query);
            let matches_node = req
                .node_id
                .as_ref()
                .map(|n| p.proving_node == *n || p.target_node == *n)
                .unwrap_or(true);
            let matches_from = req
                .from
                .map(|f| p.timestamp >= f)
                .unwrap_or(true);
            let matches_to = req
                .to
                .map(|t| p.timestamp <= t)
                .unwrap_or(true);
            matches_query && matches_node && matches_from && matches_to
        })
        .cloned()
        .collect();

    let total = results.len() as u64;
    let limit = req.limit.unwrap_or(50) as usize;
    let limited: Vec<ContactProof> = results.into_iter().take(limit).collect();

    Ok(Json(SearchResponse {
        results: limited,
        total,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use chrono::{Duration, Utc};
    use poi_core::types::{NodeId, OrbitalWindow, ProofMetadata, WindowType};

    use crate::auth::AuthConfig;
    use crate::config::ApiConfig;
    use crate::models::AppStateInner;

    fn test_state() -> AppState {
        Arc::new(AppStateInner::new(ApiConfig::default(), AuthConfig::default()))
    }

    fn test_proof() -> ContactProof {
        let kp = KeyPair::generate();
        let window = OrbitalWindow::new(
            Utc::now() - Duration::minutes(5),
            Utc::now() + Duration::hours(1),
            WindowType::Standard,
        );
        let mut proof = ContactProof::new(
            NodeId::new(),
            NodeId::new(),
            window,
            ProofMetadata {
                protocol_version: "1.0".into(),
                chain_position: None,
                confidence_score: 1.0,
                proof_purpose: "test".into(),
            },
        );
        proof.sign(&kp);
        proof
    }

    #[tokio::test]
    async fn test_create_proof() {
        let state = test_state();
        let req = ProofCreateRequest {
            target_node: NodeId::new(),
            orbital_window: OrbitalWindow::new(
                Utc::now(),
                Utc::now() + Duration::hours(1),
                WindowType::Standard,
            ),
            proof_purpose: "test".into(),
            proving_key: None,
        };
        let resp = create_proof(State(state), Json(req)).await.unwrap();
        assert_eq!(resp.0.status, "created");
    }

    #[tokio::test]
    async fn test_get_proof_not_found() {
        let state = test_state();
        let id = Uuid::new_v4();
        let result = get_proof(State(state), Path(id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_proofs_empty() {
        let state = test_state();
        let params = PaginationParams {
            page: Some(1),
            page_size: Some(10),
        };
        let resp = list_proofs(State(state), Query(params)).await.unwrap();
        assert_eq!(resp.0.total, 0);
    }

    #[tokio::test]
    async fn test_delete_proof_not_found() {
        let state = test_state();
        let id = Uuid::new_v4();
        let result = delete_proof(State(state), Path(id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_proofs() {
        let state = test_state();
        let req = SearchRequest {
            query: String::new(),
            node_id: None,
            from: None,
            to: None,
            limit: None,
        };
        let resp = search_proofs(State(state), Json(req)).await.unwrap();
        assert_eq!(resp.0.total, 0);
    }
}
