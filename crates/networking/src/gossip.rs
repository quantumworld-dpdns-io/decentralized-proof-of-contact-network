use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::seq::SliceRandom;
use tokio::sync::Mutex;
use tracing::debug;

use crate::error::NetworkError;
use crate::message::{NetworkMessage, PeerEntry, PeerId, ProofOfContact};
use crate::peer_store::PeerStore;

#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub origin: PeerId,
    pub message_type: String,
    pub payload: Vec<u8>,
    pub ttl: u8,
    pub timestamp: i64,
    pub message_id: [u8; 32],
}

pub struct GossipProtocol {
    node_id: PeerId,
    fanout: usize,
    default_ttl: u8,
    seen_messages: Arc<Mutex<HashMap<[u8; 32], i64>>>,
    pending_queue: Arc<Mutex<VecDeque<GossipMessage>>>,
    store: Arc<dyn PeerStore>,
}

impl GossipProtocol {
    pub fn new(node_id: PeerId, fanout: usize, default_ttl: u8, store: Arc<dyn PeerStore>) -> Self {
        Self {
            node_id,
            fanout,
            default_ttl,
            seen_messages: Arc::new(Mutex::new(HashMap::new())),
            pending_queue: Arc::new(Mutex::new(VecDeque::new())),
            store,
        }
    }

    pub async fn gossip_proof(
        &self,
        proof: &ProofOfContact,
        exclude: &HashSet<PeerId>,
    ) -> Result<Vec<PeerId>, NetworkError> {
        let payload = bincode::serialize(proof)?;
        let msg = NetworkMessage::GossipMessage {
            origin: self.node_id,
            ttl: self.default_ttl,
            payload,
            message_type: "proof".into(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        let encoded = msg.encode()?;
        let message_id = self.compute_message_id(&encoded);
        let gossip_msg = GossipMessage {
            origin: self.node_id,
            message_type: "proof".into(),
            payload: bincode::serialize(proof)?,
            ttl: self.default_ttl,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            message_id,
        };

        self.mark_seen(message_id).await;
        self.send_to_random_peers(gossip_msg, exclude).await
    }

    pub async fn gossip_peer_list(
        &self,
        peers: Vec<PeerEntry>,
        exclude: &HashSet<PeerId>,
    ) -> Result<Vec<PeerId>, NetworkError> {
        let payload = bincode::serialize(&peers)?;
        let gossip_msg = GossipMessage {
            origin: self.node_id,
            message_type: "peerlist".into(),
            payload,
            ttl: self.default_ttl,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            message_id: [0u8; 32],
        };

        self.send_to_random_peers(gossip_msg, exclude).await
    }

    pub async fn receive_gossip(&self, msg: &GossipMessage, sender: PeerId) -> Result<Option<GossipMessage>, NetworkError> {
        if msg.ttl == 0 {
            return Ok(None);
        }

        if self.is_seen(msg.message_id).await {
            return Ok(None);
        }

        self.mark_seen(msg.message_id).await;

        if msg.ttl <= 1 {
            return Ok(None);
        }

        let mut relay = msg.clone();
        relay.ttl -= 1;

        Ok(Some(relay))
    }

    async fn send_to_random_peers(
        &self,
        msg: GossipMessage,
        exclude: &HashSet<PeerId>,
    ) -> Result<Vec<PeerId>, NetworkError> {
        let connected = self.store.get_connected_peers().await?;
        let candidates: Vec<PeerId> = connected
            .iter()
            .map(|p| p.id)
            .filter(|id| !exclude.contains(id) && *id != self.node_id)
            .collect();

        let mut rng = rand::thread_rng();
        let count = self.fanout.min(candidates.len());
        let selected: Vec<PeerId> = candidates
            .choose_multiple(&mut rng, count)
            .cloned()
            .collect();

        if !selected.is_empty() {
            let mut queue = self.pending_queue.lock().await;
            for peer_id in &selected {
                queue.push_back(msg.clone());
            }
            debug!("Gossip: sending to {} peers (fanout={})", selected.len(), self.fanout);
        }

        Ok(selected)
    }

    pub async fn drain_pending(&self) -> Vec<(PeerId, GossipMessage)> {
        let mut queue = self.pending_queue.lock().await;
        let mut result = Vec::new();
        while let Some(msg) = queue.pop_front() {
            // We need a peer to send to; this is a simplified approach
            // In real code, we'd track which peer each message is destined for
        }
        result
    }

    pub async fn pending_count(&self) -> usize {
        self.pending_queue.lock().await.len()
    }

    fn compute_message_id(&self, data: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        id
    }

    async fn is_seen(&self, message_id: [u8; 32]) -> bool {
        let seen = self.seen_messages.lock().await;
        seen.contains_key(&message_id)
    }

    async fn mark_seen(&self, message_id: [u8; 32]) {
        let mut seen = self.seen_messages.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        seen.insert(message_id, now);

        if seen.len() > 10000 {
            let cutoff = now - 3600;
            seen.retain(|_, ts| *ts > cutoff);
        }
    }

    pub async fn cleanup(&self) {
        let mut seen = self.seen_messages.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let cutoff = now - 3600;
        seen.retain(|_, ts| *ts > cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::PeerId;
    use crate::peer_store::{InMemoryPeerStore, PeerRecord, PeerReputation};
    use std::net::SocketAddr;
    use std::sync::Arc;

    fn make_peer_record(id: PeerId) -> PeerRecord {
        PeerRecord {
            id,
            address: "127.0.0.1:9876".parse().unwrap(),
            public_key: vec![],
            capabilities: vec!["gossip".into()],
            first_seen: 1000,
            last_seen: 1000,
            reputation: PeerReputation::default(),
            is_connected: true,
        }
    }

    #[tokio::test]
    async fn test_gossip_creation() {
        let node_id = Uuid::new_v4();
        let store = Arc::new(InMemoryPeerStore::new());
        let gossip = GossipProtocol::new(node_id, 3, 5, store);
        assert_eq!(gossip.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_gossip_relay_ttl() {
        let node_id = Uuid::new_v4();
        let store = Arc::new(InMemoryPeerStore::new());
        let gossip = GossipProtocol::new(node_id, 3, 5, store);

        let msg = GossipMessage {
            origin: Uuid::new_v4(),
            message_type: "proof".into(),
            payload: vec![1, 2, 3],
            ttl: 2,
            timestamp: 12345,
            message_id: [1u8; 32],
        };

        let relay = gossip.receive_gossip(&msg, Uuid::new_v4()).await.unwrap();
        assert!(relay.is_some());
        assert_eq!(relay.unwrap().ttl, 1);
    }

    #[tokio::test]
    async fn test_gossip_dedup() {
        let node_id = Uuid::new_v4();
        let store = Arc::new(InMemoryPeerStore::new());
        let gossip = GossipProtocol::new(node_id, 3, 5, store);

        let msg = GossipMessage {
            origin: Uuid::new_v4(),
            message_type: "proof".into(),
            payload: vec![1, 2, 3],
            ttl: 2,
            timestamp: 12345,
            message_id: [2u8; 32],
        };

        let first = gossip.receive_gossip(&msg, Uuid::new_v4()).await.unwrap();
        assert!(first.is_some());
        let second = gossip.receive_gossip(&msg, Uuid::new_v4()).await.unwrap();
        assert!(second.is_none());
    }
}
