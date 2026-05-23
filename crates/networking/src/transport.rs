use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bytes::BytesMut;
use futures::SinkExt;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};
use tracing::{debug, error, info, warn};

use crate::error::NetworkError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    pub sender: SocketAddr,
    pub data: Vec<u8>,
    pub timestamp: i64,
}

impl TransportMessage {
    pub fn new(sender: SocketAddr, data: Vec<u8>) -> Self {
        Self {
            sender,
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn listen(&self) -> Result<broadcast::Receiver<TransportMessage>, NetworkError>;
    async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn Connection>, NetworkError>;
    async fn send(&self, conn: &mut Box<dyn Connection>, msg: &TransportMessage) -> Result<(), NetworkError>;
    async fn receive(&self, conn: &mut Box<dyn Connection>) -> Result<TransportMessage, NetworkError>;
    fn local_addr(&self) -> Result<SocketAddr, NetworkError>;
}

#[async_trait]
pub trait Connection: Send + Sync {
    async fn send(&mut self, data: &[u8]) -> Result<(), NetworkError>;
    async fn receive(&mut self) -> Result<Vec<u8>, NetworkError>;
    fn remote_addr(&self) -> Result<SocketAddr, NetworkError>;
    fn local_addr(&self) -> Result<SocketAddr, NetworkError>;
    async fn close(&mut self) -> Result<(), NetworkError>;
}

// --- TCP Connection ---

pub struct TcpConnection {
    stream: Arc<Mutex<TcpStream>>,
    remote: SocketAddr,
    local: SocketAddr,
    buf: BytesMut,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        let remote = stream.peer_addr().unwrap();
        let local = stream.local_addr().unwrap();
        Self {
            stream: Arc::new(Mutex::new(stream)),
            remote,
            local,
            buf: BytesMut::with_capacity(4096),
        }
    }
}

#[async_trait]
impl Connection for TcpConnection {
    async fn send(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        let mut stream = self.stream.lock().await;
        let len = data.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(data).await?;
        stream.flush().await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, NetworkError> {
        let mut stream = self.stream.lock().await;
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;
        Ok(buf)
    }

    fn remote_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.remote)
    }

    fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.local)
    }

    async fn close(&mut self) -> Result<(), NetworkError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().map_err(|e| NetworkError::TransportError(e.to_string()))
    }
}

// --- TCP Transport ---

pub struct TcpTransport {
    listen_addr: SocketAddr,
    sender: Arc<Mutex<Option<broadcast::Sender<TransportMessage>>>>,
}

impl TcpTransport {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            sender: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn listen(&self) -> Result<broadcast::Receiver<TransportMessage>, NetworkError> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!("TCP transport listening on {}", self.listen_addr);
        let (tx, rx) = broadcast::channel(1024);
        *self.sender.lock().await = Some(tx.clone());
        let local_addr = listener.local_addr()?;
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("TCP connection accepted from {}", addr);
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let mut conn = TcpConnection::new(stream);
                            loop {
                                match conn.receive().await {
                                    Ok(data) => {
                                        let msg = TransportMessage::new(local_addr, data);
                                        if tx.send(msg).is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        debug!("Connection {} closed: {}", addr, e);
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept TCP connection: {}", e);
                    }
                }
            }
        });
        Ok(rx)
    }

    async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn Connection>, NetworkError> {
        debug!("TCP connecting to {}", addr);
        let stream = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            TcpStream::connect(addr),
        )
        .await
        .map_err(|_| NetworkError::Timeout(format!("TCP connect to {}", addr)))??;
        info!("TCP connected to {}", addr);
        Ok(Box::new(TcpConnection::new(stream)))
    }

    async fn send(&self, conn: &mut Box<dyn Connection>, msg: &TransportMessage) -> Result<(), NetworkError> {
        conn.send(&msg.data).await
    }

    async fn receive(&self, conn: &mut Box<dyn Connection>) -> Result<TransportMessage, NetworkError> {
        let data = conn.receive().await?;
        let addr = conn.remote_addr()?;
        Ok(TransportMessage::new(addr, data))
    }

    fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.listen_addr)
    }
}

// --- TLS Transport (feature-gated) ---

#[cfg(feature = "tls")]
pub mod tls {
    use std::net::SocketAddr;
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::net::TcpListener;
    use tokio::sync::broadcast;
    use tokio_rustls::TlsAcceptor;
    use tracing::{debug, info, warn};

    use super::{Connection, Transport, TransportMessage};
    use crate::error::NetworkError;

    pub struct TlsTransport {
        inner: super::TcpTransport,
        acceptor: Option<TlsAcceptor>,
    }

    impl TlsTransport {
        pub fn new(listen_addr: SocketAddr, acceptor: TlsAcceptor) -> Self {
            Self {
                inner: super::TcpTransport::new(listen_addr),
                acceptor: Some(acceptor),
            }
        }
    }

    #[async_trait]
    impl Transport for TlsTransport {
        async fn listen(&self) -> Result<broadcast::Receiver<TransportMessage>, NetworkError> {
            let listener = TcpListener::bind(self.inner.listen_addr).await?;
            info!("TLS transport listening on {}", self.inner.listen_addr);
            let (tx, rx) = broadcast::channel(1024);
            let local_addr = listener.local_addr()?;
            let acceptor = self.acceptor.clone().unwrap();

            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, addr)) => {
                            debug!("TLS connection accepted from {}", addr);
                            let tx = tx.clone();
                            let acceptor = acceptor.clone();
                            tokio::spawn(async move {
                                match acceptor.accept(stream).await {
                                    Ok(tls_stream) => {
                                        let (mut reader, mut writer) =
                                            tokio::io::split(tls_stream);
                                        let mut buf = vec![0u8; 4096];
                                        loop {
                                            match reader.read(&mut buf).await {
                                                Ok(0) => break,
                                                Ok(n) => {
                                                    let msg = TransportMessage::new(
                                                        local_addr,
                                                        buf[..n].to_vec(),
                                                    );
                                                    if tx.send(msg).is_err() {
                                                        break;
                                                    }
                                                }
                                                Err(e) => {
                                                    debug!(
                                                        "TLS connection {} error: {}",
                                                        addr, e
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("TLS handshake failed from {}: {}", addr, e);
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            warn!("Failed to accept TLS connection: {}", e);
                        }
                    }
                }
            });
            Ok(rx)
        }

        async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn Connection>, NetworkError> {
            let stream = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                tokio::net::TcpStream::connect(addr),
            )
            .await
            .map_err(|_| NetworkError::Timeout(format!("TLS connect to {}", addr)))??;

            use tokio_rustls::TlsConnector as RustlsConnector;
            use std::sync::Arc as StdArc;
            use rustls::ClientConfig;

            let config = ClientConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_root_certificates(rustls::RootCertStore::empty());

            let connector = RustlsConnector::from(StdArc::new(config));
            let tls_stream = connector
                .connect(
                    tokio_rustls::TlsConnector::from(StdArc::new(config))
                        .connect(
                            dns_name: rustls::pki_types::ServerName::try_from("localhost")
                                .map_err(|_| NetworkError::TlsError("invalid DNS name".into()))?,
                            stream,
                        )
                        .await
                        .map_err(|e| NetworkError::TlsError(e.to_string()))?,
                )
                .await
                .map_err(|e| NetworkError::TlsError(e.to_string()))?;

            debug!("TLS connected to {}", addr);
            use super::TcpConnection;
            // We need a wrapper for TLS connections. For now, wrap in a simple struct.
            Ok(Box::new(TlsConnection::new(tls_stream, addr)))
        }

        async fn send(
            &self,
            conn: &mut Box<dyn Connection>,
            msg: &TransportMessage,
        ) -> Result<(), NetworkError> {
            conn.send(&msg.data).await
        }

        async fn receive(
            &self,
            conn: &mut Box<dyn Connection>,
        ) -> Result<TransportMessage, NetworkError> {
            let data = conn.receive().await?;
            let addr = conn.remote_addr()?;
            Ok(TransportMessage::new(addr, data))
        }

        fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
            self.inner.local_addr()
        }
    }

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_rustls::TlsStream;
    use std::sync::Arc as StdArc;
    use tokio::sync::Mutex;

    pub struct TlsConnection {
        stream: Arc<Mutex<TlsStream<tokio::net::TcpStream>>>,
        remote: SocketAddr,
        local: SocketAddr,
    }

    impl TlsConnection {
        pub fn new(stream: TlsStream<tokio::net::TcpStream>, remote: SocketAddr) -> Self {
            let local = stream.get_ref().0.local_addr().unwrap();
            Self {
                stream: Arc::new(Mutex::new(stream)),
                remote,
                local,
            }
        }
    }

    #[async_trait]
    impl super::Connection for TlsConnection {
        async fn send(&mut self, data: &[u8]) -> Result<(), NetworkError> {
            let mut stream = self.stream.lock().await;
            let len = data.len() as u32;
            stream.write_all(&len.to_be_bytes()).await?;
            stream.write_all(data).await?;
            stream.flush().await?;
            Ok(())
        }

        async fn receive(&mut self) -> Result<Vec<u8>, NetworkError> {
            let mut stream = self.stream.lock().await;
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await?;
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];
            stream.read_exact(&mut buf).await?;
            Ok(buf)
        }

        fn remote_addr(&self) -> Result<SocketAddr, NetworkError> {
            Ok(self.remote)
        }

        fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
            Ok(self.local)
        }

        async fn close(&mut self) -> Result<(), NetworkError> {
            let mut stream = self.stream.lock().await;
            stream.shutdown().await.map_err(|e| NetworkError::TransportError(e.to_string()))
        }
    }
}

// --- QUIC Transport (feature-gated) ---

#[cfg(feature = "quic")]
pub mod quic {
    use std::net::SocketAddr;
    use std::sync::Arc;

    use async_trait::async_trait;
    use quinn::{ClientConfig, Endpoint, ServerConfig};
    use tokio::sync::broadcast;
    use tracing::{debug, info, warn};

    use super::{Connection, Transport, TransportMessage};
    use crate::error::NetworkError;

    pub struct QuicTransport {
        listen_addr: SocketAddr,
        endpoint: Option<Endpoint>,
    }

    impl QuicTransport {
        pub fn new(listen_addr: SocketAddr) -> Self {
            Self {
                listen_addr,
                endpoint: None,
            }
        }
    }

    #[async_trait]
    impl Transport for QuicTransport {
        async fn listen(&self) -> Result<broadcast::Receiver<TransportMessage>, NetworkError> {
            let (tx, rx) = broadcast::channel(1024);
            let listen_addr = self.listen_addr;
            let endpoint = Endpoint::server(
                make_server_config().map_err(|e| NetworkError::QuicError(e.to_string()))?,
                listen_addr,
            )
            .map_err(|e| NetworkError::QuicError(e.to_string()))?;

            info!("QUIC transport listening on {}", listen_addr);
            tokio::spawn(async move {
                loop {
                    match endpoint.accept().await {
                        Some(conn) => {
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                match conn.await {
                                    Ok(conn) => {
                                        debug!("QUIC connection accepted from {}", conn.remote_address());
                                        loop {
                                            match conn.accept_bi().await {
                                                Ok((_send, mut recv)) => {
                                                    let mut buf = vec![0u8; 4096];
                                                    match recv.read_to_end(65536).await {
                                                        Ok(data) => {
                                                            let msg = TransportMessage::new(
                                                                conn.remote_address(),
                                                                data,
                                                            );
                                                            if tx.send(msg).is_err() {
                                                                break;
                                                            }
                                                        }
                                                        Err(e) => {
                                                            debug!("QUIC read error: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }
                                                Err(_) => break,
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("QUIC connection failed: {}", e);
                                    }
                                }
                            });
                        }
                        None => break,
                    }
                }
            });
            Ok(rx)
        }

        async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn Connection>, NetworkError> {
            let endpoint = Endpoint::client(
                "0.0.0.0:0".parse().unwrap(),
            )
            .map_err(|e| NetworkError::QuicError(e.to_string()))?;

            let conn = endpoint
                .connect(addr, "localhost")
                .map_err(|e| NetworkError::QuicError(e.to_string()))?
                .await
                .map_err(|e| NetworkError::QuicError(e.to_string()))?;

            debug!("QUIC connected to {}", addr);
            Ok(Box::new(QuicConnection::new(conn)))
        }

        async fn send(
            &self,
            conn: &mut Box<dyn Connection>,
            msg: &TransportMessage,
        ) -> Result<(), NetworkError> {
            conn.send(&msg.data).await
        }

        async fn receive(
            &self,
            conn: &mut Box<dyn Connection>,
        ) -> Result<TransportMessage, NetworkError> {
            let data = conn.receive().await?;
            let addr = conn.remote_addr()?;
            Ok(TransportMessage::new(addr, data))
        }

        fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
            Ok(self.listen_addr)
        }
    }

    fn make_server_config() -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
        let key = quinn::PrivateKey::from_der(&cert.serialize_private_key_der())?;
        let cert = quinn::Certificate::from_der(&cert.serialize_der()?)?;
        let transport = quinn::TransportConfig::default();
        let mut server_config = ServerConfig::with_single_cert(vec![cert], key)?;
        server_config.transport = transport.into();
        Ok(server_config)
    }

    pub struct QuicConnection {
        conn: quinn::Connection,
        remote: SocketAddr,
        local: SocketAddr,
    }

    impl QuicConnection {
        pub fn new(conn: quinn::Connection) -> Self {
            let remote = conn.remote_address();
            let local = conn.local_address();
            Self { conn, remote, local }
        }
    }

    #[async_trait]
    impl super::Connection for QuicConnection {
        async fn send(&mut self, data: &[u8]) -> Result<(), NetworkError> {
            let mut send = self
                .conn
                .open_uni()
                .await
                .map_err(|e| NetworkError::QuicError(e.to_string()))?;
            send.write_all(data)
                .await
                .map_err(|e| NetworkError::QuicError(e.to_string()))?;
            send.finish()
                .await
                .map_err(|e| NetworkError::QuicError(e.to_string()))?;
            Ok(())
        }

        async fn receive(&mut self) -> Result<Vec<u8>, NetworkError> {
            let (_send, mut recv) = self
                .conn
                .accept_bi()
                .await
                .map_err(|e| NetworkError::QuicError(e.to_string()))?;
            let data = recv
                .read_to_end(65536)
                .await
                .map_err(|e| NetworkError::QuicError(e.to_string()))?;
            Ok(data)
        }

        fn remote_addr(&self) -> Result<SocketAddr, NetworkError> {
            Ok(self.remote)
        }

        fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
            Ok(self.local)
        }

        async fn close(&mut self) -> Result<(), NetworkError> {
            self.conn.close(0u8.into(), b"goodbye");
            Ok(())
        }
    }
}

// Utility for framed reading
pub async fn read_frame(stream: &mut (impl AsyncReadExt + Unpin)) -> Result<Vec<u8>, NetworkError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

pub async fn write_frame(stream: &mut (impl AsyncWriteExt + Unpin), data: &[u8]) -> Result<(), NetworkError> {
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(data).await?;
    stream.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_message() {
        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        let msg = TransportMessage::new(addr, vec![1, 2, 3]);
        assert_eq!(msg.sender, addr);
        assert_eq!(msg.data, vec![1, 2, 3]);
        assert!(msg.timestamp > 0);
    }

    #[tokio::test]
    async fn test_tcp_roundtrip() {
        let listen_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let transport = TcpTransport::new(listen_addr);
        let mut rx = transport.listen().await.unwrap();
        let actual_listen = transport.local_addr().unwrap();

        let connect_addr = actual_listen;
        let msg_data = vec![0xDE, 0xAD, 0xBE, 0xEF];

        tokio::spawn(async move {
            let mut conn = tokio::net::TcpStream::connect(connect_addr).await.unwrap();
            let len = (msg_data.len() as u32).to_be_bytes();
            conn.write_all(&len).await.unwrap();
            conn.write_all(&msg_data).await.unwrap();
            conn.flush().await.unwrap();
        });

        let received = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(received.data, msg_data);
    }
}
