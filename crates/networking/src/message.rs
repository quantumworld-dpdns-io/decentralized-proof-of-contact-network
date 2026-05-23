use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PeerId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofOfContact {
    pub id: Uuid,
    pub prover_id: String,
    pub verifier_id: String,
    pub timestamp: i64,
    pub location_data: Vec<u8>,
    pub signature: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

impl ProofOfContact {
    pub fn new(
        prover_id: String,
        verifier_id: String,
        location_data: Vec<u8>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            prover_id,
            verifier_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            location_data,
            signature: Vec::new(),
            metadata,
        }
    }

    pub fn sign(&mut self, signing_key: &SigningKey) {
        let data = self.sig_data();
        self.signature = signing_key.sign(&data).to_bytes().to_vec();
    }

    pub fn verify(&self, verifying_key: &VerifyingKey) -> bool {
        if self.signature.len() != 64 {
            return false;
        }
        let data = self.sig_data();
        let sig = match Signature::from_slice(&self.signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        verifying_key.verify(data.as_ref(), &sig).is_ok()
    }

    fn sig_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(self.prover_id.as_bytes());
        data.extend_from_slice(self.verifier_id.as_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.location_data);
        data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub sender: PeerId,
    pub timestamp: i64,
    pub nonce: [u8; 32],
    pub signature: Vec<u8>,
    pub message_type: u8,
}

impl MessageHeader {
    pub fn new(sender: PeerId, message_type: u8) -> Self {
        let nonce = {
            let mut n = [0u8; 32];
            use rand::Rng;
            rand::thread_rng().fill(&mut n);
            n
        };
        Self {
            sender,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            nonce,
            signature: Vec::new(),
            message_type,
        }
    }

    pub fn sign(&mut self, signing_key: &SigningKey) {
        let data = self.sig_data();
        self.signature = signing_key.sign(&data).to_bytes().to_vec();
    }

    pub fn verify(&self, verifying_key: &VerifyingKey) -> bool {
        if self.signature.len() != 64 {
            return false;
        }
        let data = self.sig_data();
        let sig = match Signature::from_slice(&self.signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        verifying_key.verify(data.as_ref(), &sig).is_ok()
    }

    fn sig_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.sender.as_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.nonce);
        data.push(self.message_type);
        data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    pub node_id: PeerId,
    pub public_key: Vec<u8>,
    pub supported_protocols: Vec<String>,
    pub listen_addr: String,
    pub capabilities: Vec<String>,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

impl HandshakePayload {
    pub fn new(
        signing_key: &SigningKey,
        node_id: PeerId,
        listen_addr: SocketAddr,
        capabilities: Vec<String>,
    ) -> Self {
        let verifying_key = signing_key.verifying_key();
        let mut payload = Self {
            node_id,
            public_key: verifying_key.to_bytes().to_vec(),
            supported_protocols: vec![
                "proof-submit/1.0".into(),
                "proof-request/1.0".into(),
                "sync/1.0".into(),
                "gossip/1.0".into(),
            ],
            listen_addr: listen_addr.to_string(),
            capabilities,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            signature: Vec::new(),
        };
        let data = payload.sig_data();
        payload.signature = signing_key.sign(&data).to_bytes().to_vec();
        payload
    }

    pub fn verify(&self) -> bool {
        if self.signature.len() != 64 {
            return false;
        }
        let key_bytes: [u8; 32] = match self.public_key.as_slice().try_into() {
            Ok(k) => k,
            Err(_) => return false,
        };
        let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let data = self.sig_data();
        let sig = match Signature::from_slice(&self.signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        verifying_key.verify(data.as_ref(), &sig).is_ok()
    }

    fn sig_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.node_id.as_bytes());
        data.extend_from_slice(&self.public_key);
        for p in &self.supported_protocols {
            data.extend_from_slice(p.as_bytes());
        }
        data.extend_from_slice(self.listen_addr.as_bytes());
        for c in &self.capabilities {
            data.extend_from_slice(c.as_bytes());
        }
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Handshake(HandshakePayload),
    HandshakeAck(HandshakePayload),
    ProofSubmission {
        proof: ProofOfContact,
    },
    ProofRequest {
        proof_id: Uuid,
        requestor: PeerId,
        timestamp: i64,
    },
    ProofResponse {
        proof_id: Uuid,
        proof: Option<Box<ProofOfContact>>,
        responder: PeerId,
        timestamp: i64,
    },
    PeerList {
        peers: Vec<PeerEntry>,
        timestamp: i64,
    },
    Heartbeat {
        node_id: PeerId,
        timestamp: i64,
        load: u8,
    },
    SyncRequest {
        node_id: PeerId,
        last_sync: i64,
        known_proofs: Vec<Uuid>,
        timestamp: i64,
    },
    SyncResponse {
        node_id: PeerId,
        proofs: Vec<ProofOfContact>,
        more_available: bool,
        batch_seq: u32,
        timestamp: i64,
    },
    WindowAnnouncement {
        node_id: PeerId,
        window_start: i64,
        window_end: i64,
        location_hint: Option<String>,
        timestamp: i64,
    },
    GossipMessage {
        origin: PeerId,
        ttl: u8,
        payload: Vec<u8>,
        message_type: String,
        timestamp: i64,
    },
    Disconnect {
        node_id: PeerId,
        reason: String,
        timestamp: i64,
    },
}

impl NetworkMessage {
    pub fn message_type(&self) -> u8 {
        match self {
            NetworkMessage::Handshake(_) => 0,
            NetworkMessage::HandshakeAck(_) => 1,
            NetworkMessage::ProofSubmission { .. } => 2,
            NetworkMessage::ProofRequest { .. } => 3,
            NetworkMessage::ProofResponse { .. } => 4,
            NetworkMessage::PeerList { .. } => 5,
            NetworkMessage::Heartbeat { .. } => 6,
            NetworkMessage::SyncRequest { .. } => 7,
            NetworkMessage::SyncResponse { .. } => 8,
            NetworkMessage::WindowAnnouncement { .. } => 9,
            NetworkMessage::GossipMessage { .. } => 10,
            NetworkMessage::Disconnect { .. } => 11,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }

    pub fn timestamp(&self) -> i64 {
        match self {
            NetworkMessage::Handshake(p) => p.timestamp,
            NetworkMessage::HandshakeAck(p) => p.timestamp,
            NetworkMessage::ProofSubmission { proof } => proof.timestamp,
            NetworkMessage::ProofRequest { timestamp, .. } => *timestamp,
            NetworkMessage::ProofResponse { timestamp, .. } => *timestamp,
            NetworkMessage::PeerList { timestamp, .. } => *timestamp,
            NetworkMessage::Heartbeat { timestamp, .. } => *timestamp,
            NetworkMessage::SyncRequest { timestamp, .. } => *timestamp,
            NetworkMessage::SyncResponse { timestamp, .. } => *timestamp,
            NetworkMessage::WindowAnnouncement { timestamp, .. } => *timestamp,
            NetworkMessage::GossipMessage { timestamp, .. } => *timestamp,
            NetworkMessage::Disconnect { timestamp, .. } => *timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEntry {
    pub id: PeerId,
    pub address: String,
    pub public_key: Vec<u8>,
    pub capabilities: Vec<String>,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub header: MessageHeader,
    pub body: Vec<u8>,
}

impl Envelope {
    pub fn new(sender: PeerId, msg: &NetworkMessage, signing_key: &SigningKey) -> Result<Self, bincode::Error> {
        let body = msg.encode()?;
        let mut header = MessageHeader::new(sender, msg.message_type());
        header.sign(signing_key);
        Ok(Self { header, body })
    }

    pub fn decode_message(&self, verifying_key: &VerifyingKey) -> Result<NetworkMessage, crate::error::NetworkError> {
        if !self.header.verify(verifying_key) {
            return Err(crate::error::NetworkError::InvalidSignature);
        }
        let msg = NetworkMessage::decode(&self.body)?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn test_message_roundtrip() {
        let msg = NetworkMessage::Heartbeat {
            node_id: PeerId::new(),
            timestamp: 12345,
            load: 42,
        };
        let encoded = msg.encode().unwrap();
        let decoded = NetworkMessage::decode(&encoded).unwrap();
        assert_eq!(msg.message_type(), decoded.message_type());
        assert_eq!(msg.timestamp(), decoded.timestamp());
    }

    #[test]
    fn test_envelope_signing() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let sender = PeerId::new();
        let msg = NetworkMessage::Heartbeat {
            node_id: sender,
            timestamp: 12345,
            load: 42,
        };
        let envelope = Envelope::new(sender, &msg, &signing_key).unwrap();
        let decoded = envelope.decode_message(&verifying_key).unwrap();
        assert_eq!(decoded.timestamp(), 12345);
    }

    #[test]
    fn test_envelope_tampered() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let sender = PeerId::new();
        let msg = NetworkMessage::Heartbeat {
            node_id: sender,
            timestamp: 12345,
            load: 42,
        };
        let mut envelope = Envelope::new(sender, &msg, &signing_key).unwrap();
        envelope.body[0] ^= 0xFF;
        assert!(envelope.decode_message(&verifying_key).is_err());
    }

    #[test]
    fn test_proof_signing_verification() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let mut proof = ProofOfContact::new(
            "alice".into(),
            "bob".into(),
            vec![1, 2, 3],
            HashMap::new(),
        );
        proof.sign(&signing_key);
        assert!(proof.verify(&verifying_key));
    }

    #[test]
    fn test_handshake_verify() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let node_id = PeerId::new();
        let addr: SocketAddr = "127.0.0.1:9876".parse().unwrap();
        let payload = HandshakePayload::new(
            &signing_key,
            node_id,
            addr,
            vec!["tcp".into(), "gossip".into()],
        );
        assert!(payload.verify());
    }
}
