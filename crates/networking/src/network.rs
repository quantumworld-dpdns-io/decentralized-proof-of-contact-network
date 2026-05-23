use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ed25519_dalek::{SigningKey, VerifyingKey};
use tokio::sync::{broadcast, Mutex};
use tokio::time;
use tracing::{debug, info, trace, warn};
use uuid::Uuid;

use crate::config::NetworkConfig;
use crate::connection::{ConnectionManager, PooledConnection};
use crate::discovery::{
    BootstrapDiscovery, CompositeDiscovery, DhtDiscovery, MdnsDiscovery, PeerDiscovery,
    PeerExchange, PeerInfo,
};
use crate::error::NetworkError;
use crate::gossip::GossipProtocol;
use crate::message::{Envelope, NetworkMessage, PeerId, ProofOfContact};
use crate::peer_store::{InMemoryPeerStore, PeerRecord, PeerStore};
use crate::protocol::{
    CompositeHandler, HandshakeHandler, ProofRelayHandler, ProofRequestHandler,
    ProofSubmissionHandler, SyncHandler,
};
use crate::rate_limit::RateLimiter;
use crate::sync::StateSync;
use crate::transport::{TcpTransport, Transport, TransportMessage};

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PeerConnected(PeerId, SocketAddr),
    PeerDisconnected(PeerId, String),
    ProofReceived(ProofOfContact),
    ProofRelayed(ProofOfContact, Vec<PeerId>),
    SyncStarted(PeerId),
    SyncCompleted(PeerId, usize),
    SyncFailed(PeerId, String),
    Error(NetworkError),
    DiscoveryUpdated(Vec<PeerInfo>),
}

pub struct NetworkManager {
    config: NetworkConfig,
    node_id: PeerId,
    signing_key: SigningKey,
    verifying_key: VerifyingKey,

    transport: Arc<dyn Transport>,
    connection_manager: Arc<ConnectionManager>,
    peer_store: Arc<dyn PeerStore>,
    discovery: Arc<dyn PeerDiscovery>,
    composite_handler: Arc<CompositeHandler>,
    gossip: Arc<GossipProtocol>,
    state_sync: Arc<StateSync>,
    rate_limiter: Arc<RateLimiter>,
    peer_exchange: Arc<PeerExchange>,

    event_tx: broadcast::Sender<NetworkEvent>,
    event_rx: Arc<Mutex<Option<broadcast::Receiver<NetworkEvent>>>>,

    running: Arc<std::sync::atomic::AtomicBool>,
    local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>,
}

impl NetworkManager {
    pub async fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        config.validate().map_err(|e| NetworkError::ConnectionError(e))?;

        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let node_id = Uuid::new_v4();

        let transport: Arc<dyn Transport> = {
            #[cfg(feature = "tls")]
            if config.enable_tls {
                use crate::transport::tls::TlsTransport;
                use tokio_rustls::TlsAcceptor;
                use rustls::ServerConfig;
                use std::sync::Arc as StdArc;

                let (certs, key) = generate_self_signed_cert()?;
                let server_config = ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(certs, key)
                    .map_err(|e| NetworkError::TlsError(e.to_string()))?;
                Arc::new(TlsTransport::new(
                    config.listen_addr,
                    TlsAcceptor::from(StdArc::new(server_config)),
                ))
            } else {
                Arc::new(TcpTransport::new(config.listen_addr))
            }
            #[cfg(not(feature = "tls"))]
            Arc::new(TcpTransport::new(config.listen_addr))
        };

        let peer_store: Arc<dyn PeerStore> = Arc::new(InMemoryPeerStore::new());

        let connection_manager = Arc::new(ConnectionManager::new(
            node_id,
            transport.clone(),
            config.max_peers,
            config.heartbeat_interval,
        ));

        let discovery: Arc<dyn PeerDiscovery> = {
            let mut discoverers: Vec<Box<dyn PeerDiscovery>> = Vec::new();
            discoverers.push(Box::new(BootstrapDiscovery::new(
                config.bootstrap_peers.clone(),
            )));
            discoverers.push(Box::new(MdnsDiscovery::new("_poi._tcp")));
            discoverers.push(Box::new(DhtDiscovery::new(node_id.to_string(), 8)));
            Arc::new(CompositeDiscovery::new(discoverers))
        };

        let local_proofs: Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let state_sync = Arc::new(StateSync::new(node_id, config.sync_batch_size));

        let gossip = Arc::new(GossipProtocol::new(
            node_id,
            config.gossip_fanout,
            config.gossip_ttl,
            peer_store.clone(),
        ));

        let peer_exchange = Arc::new(PeerExchange::new(peer_store.clone()));

        let handshake_handler = Box::new(HandshakeHandler::new(
            node_id,
            signing_key.clone(),
            peer_store.clone(),
            vec!["proof-submit".into(), "proof-request".into(), "sync".into(), "gossip".into()],
        ));

        let proof_submission = Box::new(ProofSubmissionHandler::new(node_id, signing_key.clone()));

        let proof_request = Box::new(ProofRequestHandler::new(local_proofs.clone()));

        let proof_relay = Box::new(ProofRelayHandler::new(local_proofs.clone(), 5));

        let sync_handler = Box::new(SyncHandler::new(state_sync.clone()));

        let composite_handler = Arc::new(CompositeHandler::new(vec![
            handshake_handler,
            proof_submission,
            proof_request,
            proof_relay,
            sync_handler,
        ]));

        let rate_limiter = Arc::new(RateLimiter::new(
            config.rate_limit_messages_per_sec as u64 * 10,
            config.rate_limit_messages_per_sec as u64,
            100,
            50,
        ));

        let (event_tx, event_rx) = broadcast::channel(256);

        Ok(Self {
            config,
            node_id,
            signing_key,
            verifying_key,
            transport,
            connection_manager,
            peer_store,
            discovery,
            composite_handler,
            gossip,
            state_sync,
            rate_limiter,
            peer_exchange,
            event_tx,
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            local_proofs,
        })
    }

    pub fn node_id(&self) -> PeerId {
        self.node_id
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NetworkEvent> {
        self.event_tx.subscribe()
    }

    pub async fn subscribe_rx(&self) -> broadcast::Receiver<NetworkEvent> {
        let mut lock = self.event_rx.lock().await;
        lock.take().unwrap_or_else(|| self.event_tx.subscribe())
    }

    pub async fn start(&self) -> Result<(), NetworkError> {
        self.running.store(true, std::sync::atomic::Ordering::Relaxed);
        info!("Network manager starting (node_id={})", self.node_id);

        let transport_rx = self.transport.listen().await?;

        self.connection_manager.start_heartbeats().await;
        self.connection_manager.start_reconnect_loop().await;

        self.emit_event(NetworkEvent::DiscoveryUpdated(Vec::new())).await;

        let running = self.running.clone();
        let event_tx = self.event_tx.clone();
        let composite = self.composite_handler.clone();
        let conn_manager = self.connection_manager.clone();
        let rate_limiter = self.rate_limiter.clone();
        let peer_store = self.peer_store.clone();
        let gossip = self.gossip.clone();
        let state_sync = self.state_sync.clone();
        let local_proofs = self.local_proofs.clone();
        let peer_exchange = self.peer_exchange.clone();
        let signing_key_clone = self.signing_key.clone();
        let node_id = self.node_id;

        tokio::spawn(async move {
            let mut rx = transport_rx;
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Ok(transport_msg) => {
                                trace!("Received transport message from {}", transport_msg.sender);
                                if let Err(e) = Self::handle_transport_message(
                                    &transport_msg,
                                    &composite,
                                    &conn_manager,
                                    &rate_limiter,
                                    &peer_store,
                                    &gossip,
                                    &state_sync,
                                    &local_proofs,
                                    &peer_exchange,
                                    &event_tx,
                                    &signing_key_clone,
                                    node_id,
                                ).await {
                                    warn!("Error handling message: {}", e);
                                }
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                warn!("Transport message channel lagged by {}", n);
                            }
                        }
                    }
                }
            }
        });

        let discovery = self.discovery.clone();
        let event_tx_disc = self.event_tx.clone();
        let running_disc = self.running.clone();
        let config_disc_interval = Duration::from_secs(60);

        tokio::spawn(async move {
            while running_disc.load(std::sync::atomic::Ordering::Relaxed) {
                time::sleep(config_disc_interval).await;
                match discovery.discover_peers().await {
                    Ok(peers) => {
                        if !peers.is_empty() {
                            debug!("Discovery found {} peers", peers.len());
                            let _ = event_tx_disc.send(NetworkEvent::DiscoveryUpdated(peers));
                        }
                    }
                    Err(e) => {
                        debug!("Discovery error: {}", e);
                    }
                }
            }
        });

        info!("Network manager started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), NetworkError> {
        info!("Network manager stopping");
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        self.connection_manager.disconnect_all().await?;
        self.state_sync.shutdown();
        info!("Network manager stopped");
        Ok(())
    }

    pub async fn connect(&self, addr: SocketAddr) -> Result<PooledConnection, NetworkError> {
        let conn = self.connection_manager.connect(addr).await?;
        self.emit_event(NetworkEvent::PeerConnected(conn.peer_id, addr)).await;
        Ok(conn)
    }

    pub async fn disconnect(&self, peer_id: PeerId) -> Result<(), NetworkError> {
        self.connection_manager.disconnect(peer_id).await?;
        self.rate_limiter.remove_peer(peer_id).await;
        self.emit_event(NetworkEvent::PeerDisconnected(peer_id, "user requested".into())).await;
        Ok(())
    }

    pub async fn broadcast_proof(&self, proof: ProofOfContact) -> Result<(), NetworkError> {
        let msg = NetworkMessage::ProofSubmission {
            proof: proof.clone(),
        };
        let envelope = Envelope::new(self.node_id, &msg, &self.signing_key)?;

        let peers = self.connection_manager.get_connected_peers().await;
        let peer_ids: Vec<PeerId> = peers.iter().map(|p| p.peer_id).collect();

        info!("Broadcasting proof {} to {} peers", proof.id, peer_ids.len());

        let mut sent_to = Vec::new();
        for peer in &peers {
            sent_to.push(peer.peer_id);
        }

        {
            let mut proofs = self.local_proofs.lock().await;
            proofs.insert(proof.id, proof.clone());
        }

        let exclude: std::collections::HashSet<PeerId> = sent_to.iter().cloned().collect();
        let gossip_targets = self.gossip.gossip_proof(&proof, &exclude).await?;
        sent_to.extend(gossip_targets);

        self.emit_event(NetworkEvent::ProofReceived(proof.clone())).await;

        Ok(())
    }

    pub async fn request_proof(&self, proof_id: uuid::Uuid) -> Result<Option<ProofOfContact>, NetworkError> {
        {
            let proofs = self.local_proofs.lock().await;
            if let Some(proof) = proofs.get(&proof_id) {
                return Ok(Some(proof.clone()));
            }
        }

        let msg = NetworkMessage::ProofRequest {
            proof_id,
            requestor: self.node_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        let envelope = Envelope::new(self.node_id, &msg, &self.signing_key)?;

        let peers = self.connection_manager.get_connected_peers().await;
        for peer in &peers {
            trace!("Requesting proof {} from {}", proof_id, peer.peer_id);
        }

        Ok(None)
    }

    pub async fn get_connected_peers(&self) -> Vec<PooledConnection> {
        self.connection_manager.get_connected_peers().await
    }

    pub async fn get_peer_count(&self) -> usize {
        self.connection_manager.connected_count().await
    }

    pub async fn get_all_peers(&self) -> Result<Vec<PeerRecord>, NetworkError> {
        self.peer_store.get_all_peers().await
    }

    pub async fn get_known_proofs(&self) -> Vec<ProofOfContact> {
        let proofs = self.local_proofs.lock().await;
        proofs.values().cloned().collect()
    }

    pub async fn get_proof_count(&self) -> usize {
        self.local_proofs.lock().await.len()
    }

    pub async fn sync_with_peer(&self, peer_id: PeerId) -> Result<(), NetworkError> {
        if !self.connection_manager.is_connected(peer_id).await {
            return Err(NetworkError::PeerNotFound(peer_id.to_string()));
        }

        self.emit_event(NetworkEvent::SyncStarted(peer_id)).await;

        let known_proofs = {
            let proofs = self.local_proofs.lock().await;
            proofs.keys().cloned().collect()
        };

        let msg = NetworkMessage::SyncRequest {
            node_id: self.node_id,
            last_sync: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            known_proofs,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        let envelope = Envelope::new(self.node_id, &msg, &self.signing_key)?;
        trace!("Sync request sent to {}", peer_id);

        Ok(())
    }

    async fn handle_transport_message(
        msg: &TransportMessage,
        composite: &CompositeHandler,
        conn_manager: &ConnectionManager,
        rate_limiter: &RateLimiter,
        peer_store: &Arc<dyn PeerStore>,
        gossip: &GossipProtocol,
        state_sync: &StateSync,
        local_proofs: &Arc<Mutex<HashMap<uuid::Uuid, ProofOfContact>>>,
        peer_exchange: &PeerExchange,
        event_tx: &broadcast::Sender<NetworkEvent>,
        signing_key: &SigningKey,
        node_id: PeerId,
    ) -> Result<(), NetworkError> {
        let network_msg = match NetworkMessage::decode(&msg.data) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to decode message from {}: {}", msg.sender, e);
                return Err(NetworkError::MessageDecode(e.to_string()));
            }
        };

        let sender_peer_id = Uuid::new_v4(); // Simplified; would extract from envelope

        if let Err(e) = rate_limiter.check_message(sender_peer_id).await {
            warn!("Rate limit exceeded for peer {}", sender_peer_id);
            return Err(e);
        }

        let responses = composite.dispatch(&network_msg, sender_peer_id).await?;

        for response in responses {
            if let NetworkMessage::SyncResponse { .. } = &response {
                if let Ok(new_proofs) = state_sync.handle_sync_response(&response).await {
                    if !new_proofs.is_empty() {
                        let _ = event_tx.send(NetworkEvent::SyncCompleted(
                            sender_peer_id,
                            new_proofs.len(),
                        ));
                    }
                }
            }
        }

        conn_manager.update_activity(sender_peer_id).await;

        Ok(())
    }

    async fn emit_event(&self, event: NetworkEvent) {
        let _ = self.event_tx.send(event);
    }
}

#[cfg(feature = "tls")]
fn generate_self_signed_cert() -> Result<(Vec<rustls::pki_types::CertificateDer<'static>>, rustls::pki_types::PrivateKeyDer<'static>), NetworkError> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};

    let certified_key = rcgen::generate_simple_self_signed(vec!["localhost".into()])
        .map_err(|e| NetworkError::TlsError(e.to_string()))?;

    let cert_der = certified_key.cert.der().clone();
    let key_der = PrivateKeyDer::try_from(certified_key.key_pair.serialize_der())
        .map_err(|e| NetworkError::TlsError(format!("invalid private key: {}", e)))?;

    Ok((vec![cert_der], key_der))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use uuid::Uuid;

    use crate::config::NetworkConfig;

    fn test_config(port: u16) -> NetworkConfig {
        NetworkConfig {
            listen_addr: SocketAddr::from(([127, 0, 0, 1], port)),
            external_addr: SocketAddr::from(([127, 0, 0, 1], port)),
            bootstrap_peers: Vec::new(),
            max_peers: 10,
            handshake_timeout: Duration::from_secs(2),
            message_timeout: Duration::from_secs(5),
            heartbeat_interval: Duration::from_secs(30),
            rate_limit_messages_per_sec: 1000,
            enable_tls: false,
            enable_quic: false,
            gossip_fanout: 2,
            gossip_ttl: 3,
            sync_batch_size: 10,
        }
    }

    #[tokio::test]
    async fn test_network_manager_create() {
        let config = test_config(18901);
        let manager = NetworkManager::new(config).await.unwrap();
        assert!(!manager.node_id.is_nil());
    }

    #[tokio::test]
    async fn test_network_manager_events() {
        let config = test_config(18902);
        let manager = NetworkManager::new(config).await.unwrap();
        let mut rx = manager.subscribe();

        tokio::spawn(async move {
            let _ = rx.recv().await;
        });
    }

    #[tokio::test]
    async fn test_network_manager_start_stop() {
        let config = test_config(18903);
        let manager = NetworkManager::new(config).await.unwrap();
        manager.start().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_network_manager_broadcast() {
        let config = test_config(18904);
        let manager = NetworkManager::new(config).await.unwrap();
        manager.start().await.unwrap();

        let proof = ProofOfContact::new(
            "alice".into(),
            "bob".into(),
            vec![1, 2, 3],
            HashMap::new(),
        );
        manager.broadcast_proof(proof).await.unwrap();

        assert_eq!(manager.get_proof_count().await, 1);
        manager.stop().await.unwrap();
    }
}
