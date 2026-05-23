use std::net::SocketAddr;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub external_addr: SocketAddr,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
    pub handshake_timeout: Duration,
    pub message_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub rate_limit_messages_per_sec: u32,
    pub enable_tls: bool,
    pub enable_quic: bool,
    pub gossip_fanout: usize,
    pub gossip_ttl: u8,
    pub sync_batch_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:9876".parse().unwrap(),
            external_addr: "127.0.0.1:9876".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            max_peers: 50,
            handshake_timeout: Duration::from_secs(10),
            message_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(15),
            rate_limit_messages_per_sec: 100,
            enable_tls: true,
            enable_quic: false,
            gossip_fanout: 3,
            gossip_ttl: 5,
            sync_batch_size: 100,
        }
    }
}

impl NetworkConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_peers == 0 {
            return Err("max_peers must be greater than 0".to_string());
        }
        if self.handshake_timeout.is_zero() {
            return Err("handshake_timeout must be non-zero".to_string());
        }
        if self.message_timeout.is_zero() {
            return Err("message_timeout must be non-zero".to_string());
        }
        if self.heartbeat_interval.is_zero() {
            return Err("heartbeat_interval must be non-zero".to_string());
        }
        if self.rate_limit_messages_per_sec == 0 {
            return Err("rate_limit_messages_per_sec must be greater than 0".to_string());
        }
        if self.gossip_fanout == 0 {
            return Err("gossip_fanout must be greater than 0".to_string());
        }
        if self.gossip_ttl == 0 {
            return Err("gossip_ttl must be greater than 0".to_string());
        }
        if self.sync_batch_size == 0 {
            return Err("sync_batch_size must be greater than 0".to_string());
        }
        Ok(())
    }
}
