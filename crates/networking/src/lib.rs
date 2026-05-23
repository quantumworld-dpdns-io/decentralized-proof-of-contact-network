pub mod config;
pub mod connection;
pub mod discovery;
pub mod error;
pub mod gossip;
pub mod message;
pub mod network;
pub mod peer_store;
pub mod protocol;
pub mod rate_limit;
pub mod sync;
pub mod transport;

pub use config::NetworkConfig;
pub use connection::{ConnectionManager, ConnectionState, PooledConnection};
pub use discovery::{
    BootstrapDiscovery, CompositeDiscovery, DhtDiscovery, MdnsDiscovery, PeerDiscovery,
    PeerExchange, PeerInfo,
};
pub use error::NetworkError;
pub use gossip::GossipProtocol;
pub use message::{
    Envelope, HandshakePayload, MessageHeader, NetworkMessage, PeerEntry, PeerId, ProofOfContact,
};
pub use network::{NetworkEvent, NetworkManager};
pub use peer_store::{
    InMemoryPeerStore, PeerRecord, PeerReputation, PeerStore, PersistentPeerStore, TrustLevel,
};
pub use protocol::{
    CompositeHandler, HandshakeHandler, Handler, ProofRelayHandler, ProofRequestHandler,
    ProofSubmissionHandler, SyncHandler,
};
pub use rate_limit::RateLimiter;
pub use sync::{StateSync, SyncProgress, SyncState};
pub use transport::{
    Connection as TransportConnection, TcpTransport, Transport, TransportMessage,
};

pub type Result<T> = std::result::Result<T, NetworkError>;
