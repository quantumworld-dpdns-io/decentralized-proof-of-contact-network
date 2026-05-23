use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::Mutex;
use tracing::{debug, info, trace, warn};

use crate::error::NetworkError;
use crate::message::{NetworkMessage, PeerId, ProofOfContact};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    Idle,
    Syncing,
    Complete,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SyncProgress {
    pub state: SyncState,
    pub peer_id: PeerId,
    pub proofs_received: usize,
    pub proofs_sent: usize,
    pub total_expected: usize,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

impl Default for SyncProgress {
    fn default() -> Self {
        Self {
            state: SyncState::Idle,
            peer_id: PeerId::nil(),
            proofs_received: 0,
            proofs_sent: 0,
            total_expected: 0,
            started_at: 0,
            completed_at: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub proof_id: Option<uuid::Uuid>,
}

pub struct StateSync {
    node_id: PeerId,
    local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>,
    sync_sessions: Arc<Mutex<HashMap<PeerId, SyncProgress>>>,
    batch_size: usize,
    active: Arc<std::sync::atomic::AtomicBool>,
}

impl StateSync {
    pub fn new(node_id: PeerId, batch_size: usize) -> Self {
        Self {
            node_id,
            local_proofs: Arc::new(Mutex::new(HashMap::new())),
            sync_sessions: Arc::new(Mutex::new(HashMap::new())),
            batch_size,
            active: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    pub async fn add_local_proof(&self, proof: ProofOfContact) {
        let mut proofs = self.local_proofs.lock().await;
        proofs.insert(proof.id, proof);
    }

    pub async fn get_local_proof(&self, proof_id: uuid::Uuid) -> Option<ProofOfContact> {
        let proofs = self.local_proofs.lock().await;
        proofs.get(&proof_id).cloned()
    }

    pub async fn get_all_local_proofs(&self) -> Vec<ProofOfContact> {
        let proofs = self.local_proofs.lock().await;
        proofs.values().cloned().collect()
    }

    pub async fn get_local_proof_ids(&self) -> HashSet<uuid::Uuid> {
        let proofs = self.local_proofs.lock().await;
        proofs.keys().cloned().collect()
    }

    pub async fn get_local_proof_count(&self) -> usize {
        self.local_proofs.lock().await.len()
    }

    pub async fn handle_sync_request(
        &self,
        req: &NetworkMessage,
    ) -> Result<Option<NetworkMessage>, NetworkError> {
        match req {
            NetworkMessage::SyncRequest {
                node_id: peer_id,
                last_sync,
                known_proofs,
                timestamp: _,
            } => {
                let local_ids = self.get_local_proof_ids().await;
                let needed: Vec<uuid::Uuid> = local_ids
                    .difference(&known_proofs.iter().cloned().collect())
                    .cloned()
                    .collect();

                if needed.is_empty() {
                    return Ok(Some(NetworkMessage::SyncResponse {
                        node_id: self.node_id,
                        proofs: Vec::new(),
                        more_available: false,
                        batch_seq: 0,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                    }));
                }

                let batch: Vec<ProofOfContact> = {
                    let proofs = self.local_proofs.lock().await;
                    needed
                        .iter()
                        .take(self.batch_size)
                        .filter_map(|id| proofs.get(id).cloned())
                        .collect()
                };

                let more_available = needed.len() > self.batch_size;

                let sync_progress = SyncProgress {
                    state: SyncState::Syncing,
                    peer_id: *peer_id,
                    proofs_sent: batch.len(),
                    proofs_received: 0,
                    total_expected: needed.len(),
                    started_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    completed_at: None,
                };

                let mut sessions = self.sync_sessions.lock().await;
                sessions.insert(*peer_id, sync_progress);

                Ok(Some(NetworkMessage::SyncResponse {
                    node_id: self.node_id,
                    proofs: batch,
                    more_available,
                    batch_seq: 0,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                }))
            }
            _ => Err(NetworkError::ProtocolViolation(
                "Expected SyncRequest".into(),
            )),
        }
    }

    pub async fn handle_sync_response(
        &self,
        resp: &NetworkMessage,
    ) -> Result<Vec<ProofOfContact>, NetworkError> {
        match resp {
            NetworkMessage::SyncResponse {
                node_id,
                proofs,
                more_available,
                batch_seq: _,
                timestamp: _,
            } => {
                let mut new_proofs = Vec::new();
                {
                    let mut local = self.local_proofs.lock().await;
                    for proof in proofs {
                        if !local.contains_key(&proof.id) {
                            local.insert(proof.id, proof.clone());
                            new_proofs.push(proof.clone());
                        }
                    }
                }

                let mut sessions = self.sync_sessions.lock().await;
                if let Some(progress) = sessions.get_mut(node_id) {
                    progress.proofs_received += new_proofs.len();
                    if !more_available {
                        progress.state = SyncState::Complete;
                        progress.completed_at = Some(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64,
                        );
                    }
                }

                debug!(
                    "Sync from {}: received {} new proofs (total local: {})",
                    node_id,
                    new_proofs.len(),
                    self.local_proofs.lock().await.len()
                );

                Ok(new_proofs)
            }
            _ => Err(NetworkError::ProtocolViolation(
                "Expected SyncResponse".into(),
            )),
        }
    }

    pub fn build_merkle_tree(proofs: &[ProofOfContact]) -> Option<MerkleNode> {
        if proofs.is_empty() {
            return None;
        }

        let leaves: Vec<MerkleNode> = proofs
            .iter()
            .map(|p| {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(p.id.as_bytes());
                hasher.update(p.prover_id.as_bytes());
                hasher.update(p.verifier_id.as_bytes());
                let hash = hasher.finalize();
                let mut h = [0u8; 32];
                h.copy_from_slice(&hash);
                MerkleNode {
                    hash: h,
                    left: None,
                    right: None,
                    proof_id: Some(p.id),
                }
            })
            .collect();

        Self::build_merkle_inner(leaves)
    }

    fn build_merkle_inner(nodes: Vec<MerkleNode>) -> Option<MerkleNode> {
        if nodes.is_empty() {
            return None;
        }
        if nodes.len() == 1 {
            return Some(nodes.into_iter().next().unwrap());
        }

        let mut parents = Vec::new();
        for chunk in nodes.chunks(2) {
            if chunk.len() == 2 {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0].hash);
                hasher.update(&chunk[1].hash);
                let hash = hasher.finalize();
                let mut h = [0u8; 32];
                h.copy_from_slice(&hash);
                parents.push(MerkleNode {
                    hash: h,
                    left: Some(Box::new(chunk[0].clone())),
                    right: Some(Box::new(chunk[1].clone())),
                    proof_id: None,
                });
            } else {
                parents.push(chunk[0].clone());
            }
        }

        Self::build_merkle_inner(parents)
    }

    pub fn verify_merkle_proof(root_hash: &[u8; 32], proof_id: uuid::Uuid, siblings: &[[u8; 32]]) -> bool {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(proof_id.as_bytes());
        let mut current = [0u8; 32];
        current.copy_from_slice(&hasher.finalize());

        for sibling in siblings {
            let mut hasher = Sha256::new();
            hasher.update(&current);
            hasher.update(sibling);
            current.copy_from_slice(&hasher.finalize());
        }

        &current == root_hash
    }

    pub async fn get_sync_progress(&self, peer_id: PeerId) -> Option<SyncProgress> {
        let sessions = self.sync_sessions.lock().await;
        sessions.get(&peer_id).cloned()
    }

    pub async fn clear_sync_session(&self, peer_id: PeerId) {
        let mut sessions = self.sync_sessions.lock().await;
        sessions.remove(&peer_id);
    }

    pub fn shutdown(&self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_proof(id: uuid::Uuid) -> ProofOfContact {
        ProofOfContact {
            id,
            prover_id: "alice".into(),
            verifier_id: "bob".into(),
            timestamp: 12345,
            location_data: vec![1, 2, 3],
            signature: vec![],
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_state_sync_basic() {
        let node_id = Uuid::new_v4();
        let sync = StateSync::new(node_id, 10);

        let proof = make_proof(uuid::Uuid::new_v4());
        sync.add_local_proof(proof.clone()).await;
        assert_eq!(sync.get_local_proof_count().await, 1);

        let retrieved = sync.get_local_proof(proof.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, proof.id);
    }

    #[tokio::test]
    async fn test_sync_request_response() {
        let node_id = Uuid::new_v4();
        let sync = StateSync::new(node_id, 10);

        let proof = make_proof(uuid::Uuid::new_v4());
        sync.add_local_proof(proof.clone()).await;

        let req = NetworkMessage::SyncRequest {
            node_id: Uuid::new_v4(),
            last_sync: 0,
            known_proofs: Vec::new(),
            timestamp: 12345,
        };

        let resp = sync.handle_sync_request(&req).await.unwrap();
        assert!(resp.is_some());
        if let Some(NetworkMessage::SyncResponse { proofs, .. }) = resp {
            assert_eq!(proofs.len(), 1);
            assert_eq!(proofs[0].id, proof.id);
        } else {
            panic!("Expected SyncResponse");
        }
    }

    #[tokio::test]
    async fn test_sync_response_handling() {
        let node_id = Uuid::new_v4();
        let sync = StateSync::new(node_id, 10);
        assert_eq!(sync.get_local_proof_count().await, 0);

        let proof = make_proof(uuid::Uuid::new_v4());
        let resp = NetworkMessage::SyncResponse {
            node_id: Uuid::new_v4(),
            proofs: vec![proof.clone()],
            more_available: false,
            batch_seq: 0,
            timestamp: 12345,
        };

        let new_proofs = sync.handle_sync_response(&resp).await.unwrap();
        assert_eq!(new_proofs.len(), 1);
        assert_eq!(sync.get_local_proof_count().await, 1);
    }

    #[tokio::test]
    async fn test_sync_dedup() {
        let node_id = Uuid::new_v4();
        let sync = StateSync::new(node_id, 10);

        let proof = make_proof(uuid::Uuid::new_v4());
        sync.add_local_proof(proof.clone()).await;

        let resp = NetworkMessage::SyncResponse {
            node_id: Uuid::new_v4(),
            proofs: vec![proof.clone()],
            more_available: false,
            batch_seq: 0,
            timestamp: 12345,
        };

        let new_proofs = sync.handle_sync_response(&resp).await.unwrap();
        assert!(new_proofs.is_empty());
    }

    #[test]
    fn test_merkle_tree() {
        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();
        let proofs = vec![make_proof(id1), make_proof(id2)];

        let root = StateSync::build_merkle_tree(&proofs);
        assert!(root.is_some());
        assert!(root.unwrap().proof_id.is_none());
    }

    #[test]
    fn test_merkle_verify() {
        use sha2::{Digest, Sha256};

        let id = uuid::Uuid::new_v4();
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        let leaf_hash = hasher.finalize();
        let mut leaf = [0u8; 32];
        leaf.copy_from_slice(&leaf_hash);

        let mut hasher = Sha256::new();
        hasher.update(&leaf);
        hasher.update(&leaf);
        let mut root = [0u8; 32];
        root.copy_from_slice(&hasher.finalize());

        assert!(StateSync::verify_merkle_proof(&root, id, &[leaf]));
    }
}
