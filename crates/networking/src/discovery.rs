use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use rand::seq::SliceRandom;
use rand::Rng;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::NetworkError;
use crate::message::PeerEntry;
use crate::peer_store::{PeerRecord, PeerReputation, PeerStore};

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub address: SocketAddr,
    pub capabilities: Vec<String>,
    pub last_seen: i64,
}

#[async_trait]
pub trait PeerDiscovery: Send + Sync {
    async fn discover_peers(&self) -> Result<Vec<PeerInfo>, NetworkError>;
    async fn refresh(&self) -> Result<(), NetworkError>;
    fn name(&self) -> &'static str;
}

// --- Bootstrap Discovery ---

pub struct BootstrapDiscovery {
    bootstrap_addrs: Vec<String>,
}

impl BootstrapDiscovery {
    pub fn new(bootstrap_addrs: Vec<String>) -> Self {
        Self { bootstrap_addrs }
    }
}

#[async_trait]
impl PeerDiscovery for BootstrapDiscovery {
    async fn discover_peers(&self) -> Result<Vec<PeerInfo>, NetworkError> {
        let mut peers = Vec::new();
        for addr_str in &self.bootstrap_addrs {
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                peers.push(PeerInfo {
                    id: format!("bootstrap:{}", addr),
                    address: addr,
                    capabilities: vec!["bootstrap".into()],
                    last_seen: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                });
            } else {
                // Try DNS resolution
                if let Ok(addrs) = tokio::net::lookup_host(addr_str).await {
                    for addr in addrs {
                        peers.push(PeerInfo {
                            id: format!("bootstrap:{}", addr),
                            address: addr,
                            capabilities: vec!["bootstrap".into()],
                            last_seen: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64,
                        });
                    }
                }
            }
        }
        debug!("Bootstrap discovery found {} peers", peers.len());
        Ok(peers)
    }

    async fn refresh(&self) -> Result<(), NetworkError> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "bootstrap"
    }
}

// --- mDNS Discovery (simulated) ---

pub struct MdnsDiscovery {
    service_name: String,
    known_peers: Arc<Mutex<Vec<PeerInfo>>>,
}

impl MdnsDiscovery {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            known_peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_local_peer(&self, info: PeerInfo) {
        let mut peers = self.known_peers.lock().await;
        peers.retain(|p| p.id != info.id);
        peers.push(info);
    }
}

#[async_trait]
impl PeerDiscovery for MdnsDiscovery {
    async fn discover_peers(&self) -> Result<Vec<PeerInfo>, NetworkError> {
        let peers = self.known_peers.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let active: Vec<PeerInfo> = peers
            .iter()
            .filter(|p| now - p.last_seen < 300)
            .cloned()
            .collect();
        debug!(
            "mDNS discovery for '{}' found {} peers",
            self.service_name,
            active.len()
        );
        Ok(active)
    }

    async fn refresh(&self) -> Result<(), NetworkError> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "mdns"
    }
}

// --- Kademlia-style DHT Discovery (simulated) ---

pub struct DhtDiscovery {
    node_id: String,
    routing_table: Arc<Mutex<Vec<PeerInfo>>>,
    k_bucket_size: usize,
}

impl DhtDiscovery {
    pub fn new(node_id: String, k_bucket_size: usize) -> Self {
        Self {
            node_id,
            routing_table: Arc::new(Mutex::new(Vec::new())),
            k_bucket_size,
        }
    }

    pub async fn add_peer(&self, peer: PeerInfo) {
        let mut table = self.routing_table.lock().await;
        if table.len() >= self.k_bucket_size * 10 {
            table.remove(0);
        }
        table.retain(|p| p.id != peer.id);
        table.push(peer);
    }
}

#[async_trait]
impl PeerDiscovery for DhtDiscovery {
    async fn discover_peers(&self) -> Result<Vec<PeerInfo>, NetworkError> {
        let table = self.routing_table.lock().await;
        let mut rng = rand::thread_rng();
        let sample_size = self.k_bucket_size.min(table.len());
        let peers: Vec<PeerInfo> = table
            .as_slice()
            .choose_multiple(&mut rng, sample_size)
            .cloned()
            .collect();
        debug!("DHT discovery returned {} peers from routing table", peers.len());
        Ok(peers)
    }

    async fn refresh(&self) -> Result<(), NetworkError> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "dht"
    }
}

// --- Gossip-based Peer Exchange ---

pub struct PeerExchange {
    store: Arc<dyn PeerStore>,
    known_peers: Arc<Mutex<HashSet<String>>>,
}

impl PeerExchange {
    pub fn new(store: Arc<dyn PeerStore>) -> Self {
        Self {
            store,
            known_peers: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn exchange(&self, received_peers: &[PeerEntry]) -> Result<Vec<PeerInfo>, NetworkError> {
        let mut new_peers = Vec::new();
        let mut known = self.known_peers.lock().await;

        for entry in received_peers {
            let key = entry.id.to_string();
            if !known.contains(&key) {
                known.insert(key);
                if let Ok(addr) = entry.address.parse::<SocketAddr>() {
                    new_peers.push(PeerInfo {
                        id: entry.id.to_string(),
                        address: addr,
                        capabilities: entry.capabilities.clone(),
                        last_seen: entry.last_seen,
                    });
                }
            }
        }

        debug!("Peer exchange found {} new peers", new_peers.len());
        Ok(new_peers)
    }

    pub async fn get_peers_to_share(&self, max_count: usize) -> Result<Vec<PeerEntry>, NetworkError> {
        let all = self.store.get_all_peers().await?;
        let mut rng = rand::thread_rng();
        let count = max_count.min(all.len());
        let selected: Vec<PeerEntry> = all
            .choose_multiple(&mut rng, count)
            .map(|p| PeerEntry {
                id: p.id,
                address: p.address.to_string(),
                public_key: p.public_key.clone(),
                capabilities: p.capabilities.clone(),
                last_seen: p.last_seen,
            })
            .collect();
        Ok(selected)
    }
}

// --- Composite Discovery ---

pub struct CompositeDiscovery {
    discoverers: Vec<Box<dyn PeerDiscovery>>,
}

impl CompositeDiscovery {
    pub fn new(discoverers: Vec<Box<dyn PeerDiscovery>>) -> Self {
        Self { discoverers }
    }
}

#[async_trait]
impl PeerDiscovery for CompositeDiscovery {
    async fn discover_peers(&self) -> Result<Vec<PeerInfo>, NetworkError> {
        let mut all = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for discoverer in &self.discoverers {
            match discoverer.discover_peers().await {
                Ok(peers) => {
                    for peer in peers {
                        if seen.insert(peer.id.clone()) {
                            all.push(peer);
                        }
                    }
                }
                Err(e) => {
                    warn!("Discovery method '{}' failed: {}", discoverer.name(), e);
                }
            }
        }
        info!("Composite discovery found {} unique peers", all.len());
        Ok(all)
    }

    async fn refresh(&self) -> Result<(), NetworkError> {
        for discoverer in &self.discoverers {
            discoverer.refresh().await.ok();
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "composite"
    }
}

pub async fn discover_peers() -> Vec<PeerInfo> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bootstrap_discovery() {
        let discovery = BootstrapDiscovery::new(vec!["127.0.0.1:9876".into()]);
        let peers = discovery.discover_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].address.to_string(), "127.0.0.1:9876");
    }

    #[tokio::test]
    async fn test_mdns_discovery() {
        let discovery = MdnsDiscovery::new("_poi._tcp");
        let peers = discovery.discover_peers().await.unwrap();
        assert!(peers.is_empty());

        discovery
            .add_local_peer(PeerInfo {
                id: "test-peer".into(),
                address: "127.0.0.1:9876".parse().unwrap(),
                capabilities: vec![],
                last_seen: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            })
            .await;
        let peers = discovery.discover_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
    }

    #[tokio::test]
    async fn test_dht_discovery() {
        let discovery = DhtDiscovery::new("node1".into(), 8);
        discovery
            .add_peer(PeerInfo {
                id: "remote".into(),
                address: "10.0.0.1:9876".parse().unwrap(),
                capabilities: vec!["proof".into()],
                last_seen: 12345,
            })
            .await;
        let peers = discovery.discover_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
    }
}
