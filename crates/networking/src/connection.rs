use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{debug, info, trace, warn};

use crate::error::NetworkError;
use crate::message::PeerId;
use crate::transport::Transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Handshaking,
    Connected,
    Reconnecting,
}

#[derive(Debug, Clone)]
pub struct PooledConnection {
    pub peer_id: PeerId,
    pub address: SocketAddr,
    pub state: ConnectionState,
    pub connected_at: Option<i64>,
    pub last_activity: i64,
    pub latency_ms: f64,
    pub reconnect_count: u32,
}

pub struct ConnectionManager {
    node_id: PeerId,
    transport: Arc<dyn Transport>,
    connections: Arc<RwLock<HashMap<PeerId, PooledConnection>>>,
    pending_connections: Arc<Mutex<HashMap<PeerId, Instant>>>,
    max_peers: usize,
    heartbeat_interval: Duration,
    reconnect_base_delay: Duration,
    reconnect_max_delay: Duration,
    active: Arc<std::sync::atomic::AtomicBool>,
}

impl ConnectionManager {
    pub fn new(
        node_id: PeerId,
        transport: Arc<dyn Transport>,
        max_peers: usize,
        heartbeat_interval: Duration,
    ) -> Self {
        Self {
            node_id,
            transport,
            connections: Arc::new(RwLock::new(HashMap::new())),
            pending_connections: Arc::new(Mutex::new(HashMap::new())),
            max_peers,
            heartbeat_interval,
            reconnect_base_delay: Duration::from_secs(1),
            reconnect_max_delay: Duration::from_secs(120),
            active: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    pub async fn connect(&self, addr: SocketAddr) -> Result<PooledConnection, NetworkError> {
        {
            let conns = self.connections.read().await;
            if conns.len() >= self.max_peers {
                return Err(NetworkError::ConnectionError(format!(
                    "Max peers ({}) reached",
                    self.max_peers
                )));
            }
        }

        let conn = self.transport.connect(addr).await?;
        let remote_addr = conn.remote_addr()?;

        // For now, generate a temporary peer ID
        let peer_id = Uuid::new_v4();

        let pooled = PooledConnection {
            peer_id,
            address: remote_addr,
            state: ConnectionState::Connected,
            connected_at: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            latency_ms: 0.0,
            reconnect_count: 0,
        };

        let mut conns = self.connections.write().await;
        conns.insert(peer_id, pooled.clone());
        info!("Connected to {} (peer={})", remote_addr, peer_id);

        Ok(pooled)
    }

    pub async fn disconnect(&self, peer_id: PeerId) -> Result<(), NetworkError> {
        let mut conns = self.connections.write().await;
        if let Some(conn) = conns.remove(&peer_id) {
            info!("Disconnected from {} ({})", conn.address, peer_id);
        }
        Ok(())
    }

    pub async fn disconnect_all(&self) -> Result<(), NetworkError> {
        let mut conns = self.connections.write().await;
        conns.clear();
        info!("Disconnected all peers");
        Ok(())
    }

    pub async fn get_connection(&self, peer_id: PeerId) -> Option<PooledConnection> {
        let conns = self.connections.read().await;
        conns.get(&peer_id).cloned()
    }

    pub async fn get_connected_peers(&self) -> Vec<PooledConnection> {
        let conns = self.connections.read().await;
        conns
            .values()
            .filter(|c| c.state == ConnectionState::Connected)
            .cloned()
            .collect()
    }

    pub async fn is_connected(&self, peer_id: PeerId) -> bool {
        let conns = self.connections.read().await;
        conns
            .get(&peer_id)
            .map(|c| c.state == ConnectionState::Connected)
            .unwrap_or(false)
    }

    pub async fn connected_count(&self) -> usize {
        let conns = self.connections.read().await;
        conns
            .values()
            .filter(|c| c.state == ConnectionState::Connected)
            .count()
    }

    pub async fn update_activity(&self, peer_id: PeerId) {
        let mut conns = self.connections.write().await;
        if let Some(conn) = conns.get_mut(&peer_id) {
            conn.last_activity = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
        }
    }

    pub async fn start_heartbeats(&self) {
        let active = self.active.clone();
        let connections = self.connections.clone();
        let interval = self.heartbeat_interval;
        let node_id = self.node_id;

        tokio::spawn(async move {
            while active.load(std::sync::atomic::Ordering::Relaxed) {
                time::sleep(interval).await;
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let conns = connections.read().await;
                for (_, conn) in conns.iter() {
                    if conn.state == ConnectionState::Connected {
                        trace!("Heartbeat to peer {}", conn.peer_id);
                    }
                    // Check for stale connections
                    if now - conn.last_activity > 60 {
                        debug!(
                            "Peer {} inactive for {}s",
                            conn.peer_id,
                            now - conn.last_activity
                        );
                    }
                }
            }
        });
    }

    pub async fn start_reconnect_loop(&self) {
        let active = self.active.clone();
        let connections = self.connections.clone();
        let base_delay = self.reconnect_base_delay;
        let max_delay = self.reconnect_max_delay;
        let transport = self.transport.clone();

        tokio::spawn(async move {
            while active.load(std::sync::atomic::Ordering::Relaxed) {
                time::sleep(Duration::from_secs(30)).await;
                let conns = connections.read().await;
                for (peer_id, conn) in conns.iter() {
                    if conn.state == ConnectionState::Reconnecting {
                        let delay = base_delay
                            * (2u32.pow(conn.reconnect_count.min(6)))
                            .min(max_delay.as_secs() as u32);
                        let delay = Duration::from_secs(delay as u64);
                        debug!(
                            "Reconnecting to {} in {:?} (attempt {})",
                            peer_id, delay, conn.reconnect_count
                        );
                    }
                }
            }
        });
    }

    pub async fn get_latencies(&self) -> HashMap<PeerId, f64> {
        let conns = self.connections.read().await;
        conns
            .values()
            .map(|c| (c.peer_id, c.latency_ms))
            .collect()
    }

    pub fn shutdown(&self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager_basic() {
        let node_id = Uuid::new_v4();
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let transport = Arc::new(crate::transport::TcpTransport::new(addr));
        let manager = ConnectionManager::new(node_id, transport, 50, Duration::from_secs(15));

        assert_eq!(manager.connected_count().await, 0);
        assert_eq!(manager.get_connected_peers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_connection_pooled_struct() {
        let conn = PooledConnection {
            peer_id: Uuid::new_v4(),
            address: "127.0.0.1:9876".parse().unwrap(),
            state: ConnectionState::Connected,
            connected_at: Some(12345),
            last_activity: 12345,
            latency_ms: 10.0,
            reconnect_count: 0,
        };
        assert_eq!(conn.state, ConnectionState::Connected);
        assert!(conn.connected_at.is_some());
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Connecting);
    }
}
