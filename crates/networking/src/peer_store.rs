use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use tokio::fs;
use tokio::sync::Mutex;

use crate::message::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Unknown,
    Low,
    Medium,
    High,
    Trusted,
}

impl TrustLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 100.0 {
            TrustLevel::Trusted
        } else if score >= 50.0 {
            TrustLevel::High
        } else if score >= 10.0 {
            TrustLevel::Medium
        } else if score >= 0.0 {
            TrustLevel::Low
        } else {
            TrustLevel::Unknown
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerReputation {
    pub score: f64,
    pub trust_level: TrustLevel,
    pub total_interactions: u64,
    pub successful_interactions: u64,
    pub failed_interactions: u64,
    pub last_interaction: i64,
    pub average_latency: f64,
    pub latency_samples: Vec<f64>,
}

impl Default for PeerReputation {
    fn default() -> Self {
        Self {
            score: 0.0,
            trust_level: TrustLevel::Unknown,
            total_interactions: 0,
            successful_interactions: 0,
            failed_interactions: 0,
            last_interaction: 0,
            average_latency: 0.0,
            latency_samples: Vec::with_capacity(100),
        }
    }
}

impl PeerReputation {
    pub fn record_success(&mut self, latency_ms: f64) {
        self.total_interactions += 1;
        self.successful_interactions += 1;
        self.last_interaction = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.latency_samples.push(latency_ms);
        if self.latency_samples.len() > 100 {
            self.latency_samples.remove(0);
        }
        self.average_latency = self.latency_samples.iter().sum::<f64>() / self.latency_samples.len() as f64;
        self.score = (self.successful_interactions as f64
            / (self.total_interactions.max(1) as f64))
            * 100.0
            - (self.average_latency / 1000.0);
        self.score = self.score.clamp(-50.0, 150.0);
        self.trust_level = TrustLevel::from_score(self.score);
    }

    pub fn record_failure(&mut self) {
        self.total_interactions += 1;
        self.failed_interactions += 1;
        self.last_interaction = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.score = (self.successful_interactions as f64
            / (self.total_interactions.max(1) as f64))
            * 100.0
            - (self.failed_interactions as f64 * 10.0);
        self.score = self.score.clamp(-50.0, 150.0);
        self.trust_level = TrustLevel::from_score(self.score);
    }
}

#[derive(Debug, Clone)]
pub struct PeerRecord {
    pub id: PeerId,
    pub address: SocketAddr,
    pub public_key: Vec<u8>,
    pub capabilities: Vec<String>,
    pub first_seen: i64,
    pub last_seen: i64,
    pub reputation: PeerReputation,
    pub is_connected: bool,
}

#[async_trait]
pub trait PeerStore: Send + Sync {
    async fn add_peer(&self, peer: PeerRecord) -> crate::error::Result<()>;
    async fn get_peer(&self, id: PeerId) -> crate::error::Result<PeerRecord>;
    async fn update_peer(&self, peer: PeerRecord) -> crate::error::Result<()>;
    async fn remove_peer(&self, id: PeerId) -> crate::error::Result<()>;
    async fn get_all_peers(&self) -> crate::error::Result<Vec<PeerRecord>>;
    async fn get_connected_peers(&self) -> crate::error::Result<Vec<PeerRecord>>;
    async fn get_peers_by_capability(&self, capability: &str) -> crate::error::Result<Vec<PeerRecord>>;
    async fn peer_count(&self) -> usize;
    async fn contains(&self, id: PeerId) -> bool;
}

#[derive(Clone)]
pub struct InMemoryPeerStore {
    peers: Arc<Mutex<HashMap<PeerId, PeerRecord>>>,
}

impl InMemoryPeerStore {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl PeerStore for InMemoryPeerStore {
    async fn add_peer(&self, peer: PeerRecord) -> crate::error::Result<()> {
        let mut peers = self.peers.lock().await;
        peers.insert(peer.id, peer);
        Ok(())
    }

    async fn get_peer(&self, id: PeerId) -> crate::error::Result<PeerRecord> {
        let peers = self.peers.lock().await;
        peers
            .get(&id)
            .cloned()
            .ok_or_else(|| crate::error::NetworkError::PeerNotFound(id.to_string()))
    }

    async fn update_peer(&self, peer: PeerRecord) -> crate::error::Result<()> {
        let mut peers = self.peers.lock().await;
        peers.insert(peer.id, peer);
        Ok(())
    }

    async fn remove_peer(&self, id: PeerId) -> crate::error::Result<()> {
        let mut peers = self.peers.lock().await;
        peers.remove(&id);
        Ok(())
    }

    async fn get_all_peers(&self) -> crate::error::Result<Vec<PeerRecord>> {
        let peers = self.peers.lock().await;
        Ok(peers.values().cloned().collect())
    }

    async fn get_connected_peers(&self) -> crate::error::Result<Vec<PeerRecord>> {
        let peers = self.peers.lock().await;
        Ok(peers
            .values()
            .filter(|p| p.is_connected)
            .cloned()
            .collect())
    }

    async fn get_peers_by_capability(&self, capability: &str) -> crate::error::Result<Vec<PeerRecord>> {
        let peers = self.peers.lock().await;
        Ok(peers
            .values()
            .filter(|p| p.capabilities.iter().any(|c| c == capability))
            .cloned()
            .collect())
    }

    async fn peer_count(&self) -> usize {
        let peers = self.peers.lock().await;
        peers.len()
    }

    async fn contains(&self, id: PeerId) -> bool {
        let peers = self.peers.lock().await;
        peers.contains_key(&id)
    }
}

#[derive(Clone)]
pub struct PersistentPeerStore {
    inner: InMemoryPeerStore,
    path: PathBuf,
    save_interval: std::time::Duration,
}

impl PersistentPeerStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            inner: InMemoryPeerStore::new(),
            path,
            save_interval: std::time::Duration::from_secs(30),
        }
    }

    async fn load_from_disk(&self) -> crate::error::Result<Vec<PeerRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read(&self.path).await.map_err(|e| crate::error::NetworkError::IoError(e.to_string()))?;
        let peers: Vec<PeerRecord> = bincode::deserialize(&data)?;
        Ok(peers)
    }

    async fn save_to_disk(&self) -> crate::error::Result<()> {
        let peers = self.inner.get_all_peers().await?;
        let data = bincode::serialize(&peers)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await.ok();
        }
        fs::write(&self.path, data).await.map_err(|e| crate::error::NetworkError::IoError(e.to_string()))?;
        Ok(())
    }

    pub async fn initialize(&self) -> crate::error::Result<()> {
        let peers = self.load_from_disk().await?;
        for peer in peers {
            self.inner.add_peer(peer).await?;
        }
        Ok(())
    }

    pub async fn flush(&self) -> crate::error::Result<()> {
        self.save_to_disk().await
    }
}

#[async_trait]
impl PeerStore for PersistentPeerStore {
    async fn add_peer(&self, peer: PeerRecord) -> crate::error::Result<()> {
        self.inner.add_peer(peer).await?;
        self.save_to_disk().await
    }

    async fn get_peer(&self, id: PeerId) -> crate::error::Result<PeerRecord> {
        self.inner.get_peer(id).await
    }

    async fn update_peer(&self, peer: PeerRecord) -> crate::error::Result<()> {
        self.inner.update_peer(peer).await?;
        self.save_to_disk().await
    }

    async fn remove_peer(&self, id: PeerId) -> crate::error::Result<()> {
        self.inner.remove_peer(id).await?;
        self.save_to_disk().await
    }

    async fn get_all_peers(&self) -> crate::error::Result<Vec<PeerRecord>> {
        self.inner.get_all_peers().await
    }

    async fn get_connected_peers(&self) -> crate::error::Result<Vec<PeerRecord>> {
        self.inner.get_connected_peers().await
    }

    async fn get_peers_by_capability(&self, capability: &str) -> crate::error::Result<Vec<PeerRecord>> {
        self.inner.get_peers_by_capability(capability).await
    }

    async fn peer_count(&self) -> usize {
        self.inner.peer_count().await
    }

    async fn contains(&self, id: PeerId) -> bool {
        self.inner.contains(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn make_peer(id: PeerId) -> PeerRecord {
        PeerRecord {
            id,
            address: "127.0.0.1:9876".parse().unwrap(),
            public_key: vec![1, 2, 3],
            capabilities: vec!["proof".into()],
            first_seen: 1000,
            last_seen: 1000,
            reputation: PeerReputation::default(),
            is_connected: false,
        }
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryPeerStore::new();
        let peer = make_peer(Uuid::new_v4());
        assert!(!store.contains(peer.id).await);
        store.add_peer(peer.clone()).await.unwrap();
        assert!(store.contains(peer.id).await);
        let got = store.get_peer(peer.id).await.unwrap();
        assert_eq!(got.id, peer.id);
        assert_eq!(store.peer_count().await, 1);
    }

    #[tokio::test]
    async fn test_reputation() {
        let mut rep = PeerReputation::default();
        assert_eq!(rep.trust_level, TrustLevel::Unknown);
        rep.record_success(50.0);
        assert_eq!(rep.total_interactions, 1);
        assert_eq!(rep.successful_interactions, 1);
        rep.record_failure();
        assert_eq!(rep.failed_interactions, 1);
    }

    #[tokio::test]
    async fn test_persistent_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("peers.bin");
        let store = PersistentPeerStore::new(path.clone());
        store.initialize().await.unwrap();
        let peer = make_peer(Uuid::new_v4());
        store.add_peer(peer.clone()).await.unwrap();
        assert!(store.contains(peer.id).await);
        store.flush().await.unwrap();
        let store2 = PersistentPeerStore::new(path);
        store2.initialize().await.unwrap();
        assert!(store2.contains(peer.id).await);
    }
}
