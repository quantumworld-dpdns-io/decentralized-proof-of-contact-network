use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use ed25519_dalek::SigningKey;
use tokio::sync::Mutex;
use tracing::{debug, info, trace, warn};

use uuid::Uuid;

use crate::error::NetworkError;
use crate::message::{
    Envelope, HandshakePayload, NetworkMessage, PeerEntry, PeerId, ProofOfContact,
};
use crate::peer_store::{PeerRecord, PeerReputation, PeerStore};
use crate::sync::StateSync;
use crate::transport::TransportMessage;

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle_message(
        &self,
        msg: &NetworkMessage,
        sender: PeerId,
    ) -> Result<Option<NetworkMessage>, NetworkError>;
    fn name(&self) -> &'static str;
}

// --- Handshake Handler ---

pub struct HandshakeHandler {
    node_id: PeerId,
    signing_key: SigningKey,
    store: Arc<dyn PeerStore>,
    supported_capabilities: Vec<String>,
}

impl HandshakeHandler {
    pub fn new(
        node_id: PeerId,
        signing_key: SigningKey,
        store: Arc<dyn PeerStore>,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            node_id,
            signing_key,
            store,
            supported_capabilities: capabilities,
        }
    }

    pub fn create_handshake(&self, listen_addr: &str) -> NetworkMessage {
        let addr: std::net::SocketAddr = listen_addr.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
        let payload = HandshakePayload::new(
            &self.signing_key,
            self.node_id,
            addr,
            self.supported_capabilities.clone(),
        );
        NetworkMessage::Handshake(payload)
    }

    fn verify_and_store_peer(&self, payload: &HandshakePayload) -> Result<PeerId, NetworkError> {
        if !payload.verify() {
            return Err(NetworkError::InvalidSignature);
        }
        Ok(payload.node_id)
    }
}

#[async_trait]
impl Handler for HandshakeHandler {
    async fn handle_message(
        &self,
        msg: &NetworkMessage,
        sender: PeerId,
    ) -> Result<Option<NetworkMessage>, NetworkError> {
        match msg {
            NetworkMessage::Handshake(payload) => {
                debug!("Handshake received from {}", payload.node_id);
                let peer_id = self.verify_and_store_peer(payload)?;

                let addr: std::net::SocketAddr = payload
                    .listen_addr
                    .parse()
                    .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

                let record = PeerRecord {
                    id: peer_id,
                    address: addr,
                    public_key: payload.public_key.clone(),
                    capabilities: payload.capabilities.clone(),
                    first_seen: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    last_seen: payload.timestamp,
                    reputation: PeerReputation::default(),
                    is_connected: true,
                };

                self.store.add_peer(record).await?;

                let ack_payload = HandshakePayload::new(
                    &self.signing_key,
                    self.node_id,
                    addr,
                    self.supported_capabilities.clone(),
                );
                Ok(Some(NetworkMessage::HandshakeAck(ack_payload)))
            }
            NetworkMessage::HandshakeAck(payload) => {
                debug!("Handshake ACK received from {}", payload.node_id);
                let peer_id = self.verify_and_store_peer(payload)?;

                let addr: std::net::SocketAddr = payload
                    .listen_addr
                    .parse()
                    .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

                let record = PeerRecord {
                    id: peer_id,
                    address: addr,
                    public_key: payload.public_key.clone(),
                    capabilities: payload.capabilities.clone(),
                    first_seen: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    last_seen: payload.timestamp,
                    reputation: PeerReputation::default(),
                    is_connected: true,
                };

                self.store.add_peer(record).await?;
                Ok(None)
            }
            _ => Err(NetworkError::ProtocolViolation(format!(
                "HandshakeHandler received unexpected message type {}",
                msg.message_type()
            ))),
        }
    }

    fn name(&self) -> &'static str {
        "handshake"
    }
}

// --- Proof Submission Handler ---

pub struct ProofSubmissionHandler {
    node_id: PeerId,
    local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>,
    signing_key: SigningKey,
}

impl ProofSubmissionHandler {
    pub fn new(node_id: PeerId, signing_key: SigningKey) -> Self {
        Self {
            node_id,
            local_proofs: Arc::new(Mutex::new(HashMap::new())),
            signing_key,
        }
    }

    pub async fn get_proof(&self, id: uuid::Uuid) -> Option<ProofOfContact> {
        let proofs = self.local_proofs.lock().await;
        proofs.get(&id).cloned()
    }

    pub async fn get_all_proofs(&self) -> Vec<ProofOfContact> {
        let proofs = self.local_proofs.lock().await;
        proofs.values().cloned().collect()
    }

    pub async fn proof_count(&self) -> usize {
        self.local_proofs.lock().await.len()
    }
}

#[async_trait]
impl Handler for ProofSubmissionHandler {
    async fn handle_message(
        &self,
        msg: &NetworkMessage,
        sender: PeerId,
    ) -> Result<Option<NetworkMessage>, NetworkError> {
        match msg {
            NetworkMessage::ProofSubmission { proof } => {
                trace!("Proof submission from {} (proof={})", sender, proof.id);
                let mut proofs = self.local_proofs.lock().await;
                if !proofs.contains_key(&proof.id) {
                    proofs.insert(proof.id, proof.clone());
                    info!(
                        "Received new proof {} from {}",
                        proof.id, proof.prover_id
                    );
                }
                Ok(None)
            }
            _ => Err(NetworkError::ProtocolViolation(
                "Expected ProofSubmission".into(),
            )),
        }
    }

    fn name(&self) -> &'static str {
        "proof_submission"
    }
}

// --- Proof Request Handler ---

pub struct ProofRequestHandler {
    local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>,
}

impl ProofRequestHandler {
    pub fn new(local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>) -> Self {
        Self { local_proofs }
    }
}

#[async_trait]
impl Handler for ProofRequestHandler {
    async fn handle_message(
        &self,
        msg: &NetworkMessage,
        sender: PeerId,
    ) -> Result<Option<NetworkMessage>, NetworkError> {
        match msg {
            NetworkMessage::ProofRequest {
                proof_id,
                requestor,
                timestamp,
            } => {
                trace!(
                    "Proof request from {} for proof {}",
                    sender,
                    proof_id
                );
                let proofs = self.local_proofs.lock().await;
                let proof = proofs.get(proof_id).cloned();
                Ok(Some(NetworkMessage::ProofResponse {
                    proof_id: *proof_id,
                    proof: proof.map(Box::new),
                    responder: sender,
                    timestamp: *timestamp,
                }))
            }
            _ => Err(NetworkError::ProtocolViolation(
                "Expected ProofRequest".into(),
            )),
        }
    }

    fn name(&self) -> &'static str {
        "proof_request"
    }
}

// --- Proof Relay Handler ---

pub struct ProofRelayHandler {
    local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>,
    relayed: Arc<Mutex<HashSet<uuid::Uuid>>>,
    max_relay_depth: u8,
}

impl ProofRelayHandler {
    pub fn new(
        local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>,
        max_relay_depth: u8,
    ) -> Self {
        Self {
            local_proofs,
            relayed: Arc::new(Mutex::new(HashSet::new())),
            max_relay_depth,
        }
    }

    pub async fn should_relay(&self, proof_id: uuid::Uuid) -> bool {
        let relayed = self.relayed.lock().await;
        !relayed.contains(&proof_id)
    }

    pub async fn mark_relayed(&self, proof_id: uuid::Uuid) {
        let mut relayed = self.relayed.lock().await;
        relayed.insert(proof_id);
        if relayed.len() > 10000 {
            relayed.clear();
        }
    }
}

#[async_trait]
impl Handler for ProofRelayHandler {
    async fn handle_message(
        &self,
        msg: &NetworkMessage,
        sender: PeerId,
    ) -> Result<Option<NetworkMessage>, NetworkError> {
        match msg {
            NetworkMessage::ProofSubmission { proof } => {
                if self.should_relay(proof.id).await {
                    let mut proofs = self.local_proofs.lock().await;
                    if !proofs.contains_key(&proof.id) {
                        proofs.insert(proof.id, proof.clone());
                        self.mark_relayed(proof.id).await;
                        debug!("Relaying proof {} from {}", proof.id, sender);
                    }
                }
                Ok(None)
            }
            _ => Err(NetworkError::ProtocolViolation(
                "Expected ProofSubmission for relay".into(),
            )),
        }
    }

    fn name(&self) -> &'static str {
        "proof_relay"
    }
}

// --- Sync Handler ---

pub struct SyncHandler {
    state_sync: Arc<StateSync>,
}

impl SyncHandler {
    pub fn new(state_sync: Arc<StateSync>) -> Self {
        Self { state_sync }
    }
}

#[async_trait]
impl Handler for SyncHandler {
    async fn handle_message(
        &self,
        msg: &NetworkMessage,
        sender: PeerId,
    ) -> Result<Option<NetworkMessage>, NetworkError> {
        match msg {
            NetworkMessage::SyncRequest { .. } => {
                self.state_sync.handle_sync_request(msg).await
            }
            NetworkMessage::SyncResponse { .. } => {
                self.state_sync.handle_sync_response(msg).await?;
                Ok(None)
            }
            _ => Err(NetworkError::ProtocolViolation(
                "Expected SyncRequest or SyncResponse".into(),
            )),
        }
    }

    fn name(&self) -> &'static str {
        "sync"
    }
}

// --- Composite Handler ---

pub struct CompositeHandler {
    handlers: Vec<Box<dyn Handler>>,
}

impl CompositeHandler {
    pub fn new(handlers: Vec<Box<dyn Handler>>) -> Self {
        Self { handlers }
    }

    pub async fn dispatch(
        &self,
        msg: &NetworkMessage,
        sender: PeerId,
    ) -> Result<Vec<NetworkMessage>, NetworkError> {
        let mut responses = Vec::new();
        for handler in &self.handlers {
            match handler.handle_message(msg, sender).await {
                Ok(Some(resp)) => responses.push(resp),
                Ok(None) => {}
                Err(e) => {
                    debug!("Handler '{}' error: {}", handler.name(), e);
                }
            }
        }
        Ok(responses)
    }
}

use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer_store::InMemoryPeerStore;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_handshake_handler() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let node_id = Uuid::new_v4();
        let store = Arc::new(InMemoryPeerStore::new());
        let handler = HandshakeHandler::new(
            node_id,
            signing_key,
            store.clone(),
            vec!["tcp".into()],
        );

        let mut csprng = OsRng;
        let remote_key = SigningKey::generate(&mut csprng);
        let remote_id = Uuid::new_v4();
        let addr: SocketAddr = "127.0.0.1:9876".parse().unwrap();
        let payload = HandshakePayload::new(
            &remote_key,
            remote_id,
            addr,
            vec!["tcp".into()],
        );
        let msg = NetworkMessage::Handshake(payload);

        let resp = handler
            .handle_message(&msg, remote_id)
            .await
            .unwrap();
        assert!(resp.is_some());
        if let Some(NetworkMessage::HandshakeAck(_)) = resp {
            // ok
        } else {
            panic!("Expected HandshakeAck");
        }
        assert!(store.contains(remote_id).await);
    }

    #[tokio::test]
    async fn test_proof_submission() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let node_id = Uuid::new_v4();
        let handler = ProofSubmissionHandler::new(node_id, signing_key);

        let proof = ProofOfContact::new(
            "alice".into(),
            "bob".into(),
            vec![1, 2, 3],
            HashMap::new(),
        );
        let msg = NetworkMessage::ProofSubmission { proof: proof.clone() };

        handler
            .handle_message(&msg, Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(handler.proof_count().await, 1);

        let retrieved = handler.get_proof(proof.id).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_proof_request() {
        let local_proofs = Arc::new(Mutex::new(HashMap::new()));
        let proof = ProofOfContact::new(
            "alice".into(),
            "bob".into(),
            vec![1, 2, 3],
            HashMap::new(),
        );
        local_proofs.lock().await.insert(proof.id, proof.clone());

        let handler = ProofRequestHandler::new(local_proofs);
        let msg = NetworkMessage::ProofRequest {
            proof_id: proof.id,
            requestor: Uuid::new_v4(),
            timestamp: 12345,
        };

        let resp = handler
            .handle_message(&msg, Uuid::new_v4())
            .await
            .unwrap();
        assert!(resp.is_some());
        if let Some(NetworkMessage::ProofResponse { proof: p, .. }) = resp {
            assert!(p.is_some());
            assert_eq!(p.unwrap().id, proof.id);
        } else {
            panic!("Expected ProofResponse");
        }
    }

    #[tokio::test]
    async fn test_composite_handler() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let node_id = Uuid::new_v4();
        let store = Arc::new(InMemoryPeerStore::new());

        let handshake = Box::new(HandshakeHandler::new(
            node_id,
            signing_key,
            store,
            vec!["tcp".into()],
        ));
        let _composite = CompositeHandler::new(vec![handshake]);
    }
}
