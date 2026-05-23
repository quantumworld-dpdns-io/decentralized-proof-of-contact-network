use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::Utc;
use ed25519_dalek::{Signature as DalekSignature, Verifier, VerifyingKey};

use crate::error::{Error, Result};
use crate::keypair::KeyPairExt;
use crate::types::{
    ContactProof, KeyPair, NodeId, OrbitalWindow, ProofId, ProofMetadata, PublicKey, Signature,
};

pub trait ContactProofExt: Sized {
    fn new(
        proving_node: NodeId,
        target_node: NodeId,
        orbital_window: OrbitalWindow,
        metadata: ProofMetadata,
    ) -> Self;
    fn sign(&mut self, keypair: &KeyPair);
    fn verify(&self, public_key: &PublicKey) -> Result<bool>;
    fn pqc_sign(&mut self, keypair: &KeyPair) -> Result<()>;
    fn pqc_verify(&self, public_key: &PublicKey) -> Result<bool>;
    fn to_json(&self) -> String;
    fn from_json(s: &str) -> Result<Self> where Self: Sized;
    fn to_cbor(&self) -> Vec<u8>;
    fn from_cbor(data: &[u8]) -> Result<Self> where Self: Sized;
    fn is_valid(&self) -> Result<bool>;
    fn canonical_bytes(&self) -> Vec<u8>;
}

impl ContactProofExt for ContactProof {
    fn new(
        proving_node: NodeId,
        target_node: NodeId,
        orbital_window: OrbitalWindow,
        metadata: ProofMetadata,
    ) -> Self {
        let id = ProofId::new();
        ContactProof {
            id,
            proving_node,
            target_node,
            orbital_window,
            timestamp: Utc::now(),
            signature: Signature(String::new()),
            pqc_signature: None,
            metadata,
        }
    }

    fn sign(&mut self, keypair: &KeyPair) {
        let data = self.canonical_bytes();
        self.signature = keypair.sign(&data);
    }

    fn verify(&self, public_key: &PublicKey) -> Result<bool> {
        if self.signature.0.is_empty() {
            return Ok(false);
        }
        let data = self.canonical_bytes();
        let pk_bytes: [u8; 32] = BASE64
            .decode(&public_key.0)
            .map_err(|e| Error::InvalidKey(format!("invalid public key base64: {}", e)))?
            .try_into()
            .map_err(|_| Error::InvalidKey("public key must be 32 bytes".into()))?;
        let verifying_key = VerifyingKey::from_bytes(&pk_bytes)
            .map_err(|e| Error::CryptoError(e.to_string()))?;
        let sig_bytes: [u8; 64] = BASE64
            .decode(&self.signature.0)
            .map_err(|_| Error::InvalidSignature)?
            .try_into()
            .map_err(|_| Error::InvalidSignature)?;
        let dalek_sig = DalekSignature::from_bytes(&sig_bytes);
        Ok(verifying_key.verify(&data, &dalek_sig).is_ok())
    }

    fn pqc_sign(&mut self, keypair: &KeyPair) -> Result<()> {
        let data = self.canonical_bytes();
        let pqc_sig = keypair.pqc_sign(&data)?;
        self.pqc_signature = Some(pqc_sig);
        Ok(())
    }

    fn pqc_verify(&self, public_key: &PublicKey) -> Result<bool> {
        let pqc_sig = match &self.pqc_signature {
            Some(s) => s,
            None => return Ok(false),
        };
        let kp = KeyPair {
            public: public_key.clone(),
            secret: crate::types::SecretKey(vec![0u8; 32]),
        };
        let data = self.canonical_bytes();
        kp.pqc_verify(&data, pqc_sig)
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialization to json must not fail")
    }

    fn from_json(s: &str) -> Result<Self> {
        serde_json::from_str(s).map_err(|e| Error::SerializationError(e.to_string()))
    }

    fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).expect("serialization to cbor must not fail");
        buf
    }

    fn from_cbor(data: &[u8]) -> Result<Self> {
        ciborium::from_reader(data).map_err(|e| Error::SerializationError(e.to_string()))
    }

    fn is_valid(&self) -> Result<bool> {
        if self.orbital_window.end_time <= self.orbital_window.start_time {
            return Err(Error::InvalidProof(
                "orbital window end must be after start".into(),
            ));
        }
        if self.orbital_window.end_time < Utc::now() {
            return Err(Error::WindowExpired);
        }
        if self.signature.0.is_empty() {
            return Err(Error::InvalidSignature);
        }
        if self.metadata.confidence_score < 0.0 || self.metadata.confidence_score > 1.0 {
            return Err(Error::InvalidProof(
                "confidence score must be between 0 and 1".into(),
            ));
        }
        if self.proving_node.0.is_empty() || self.target_node.0.is_empty() {
            return Err(Error::InvalidProof("node ids must not be empty".into()));
        }
        Ok(true)
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.id.0.as_bytes());
        data.extend_from_slice(self.proving_node.0.as_bytes());
        data.extend_from_slice(self.target_node.0.as_bytes());
        data.extend_from_slice(self.orbital_window.id.as_bytes());
        data.extend_from_slice(
            self.orbital_window.start_time.timestamp().to_be_bytes().as_ref(),
        );
        data.extend_from_slice(
            self.orbital_window.end_time.timestamp().to_be_bytes().as_ref(),
        );
        data.extend_from_slice(self.timestamp.timestamp().to_be_bytes().as_ref());
        data.extend_from_slice(self.metadata.protocol_version.as_bytes());
        if let Some(pos) = self.metadata.chain_position {
            data.extend_from_slice(pos.to_be_bytes().as_ref());
        }
        data.extend_from_slice(self.metadata.proof_purpose.as_bytes());
        data
    }
}

impl ContactProof {
    pub fn id(&self) -> &ProofId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair::KeyPairExt;
    use crate::types::*;
    use chrono::Duration;

    fn test_metadata() -> ProofMetadata {
        ProofMetadata {
            protocol_version: "1.0".into(),
            chain_position: None,
            confidence_score: 1.0,
            proof_purpose: "test".into(),
        }
    }

    fn test_window() -> OrbitalWindow {
        OrbitalWindow::new(
            Utc::now(),
            Utc::now() + Duration::hours(1),
            WindowType::Standard,
        )
    }

    #[test]
    fn test_contact_proof_new() {
        let kp = KeyPair::generate();
        let proof = ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        assert!(proof.signature.0.is_empty());
        assert!(proof.pqc_signature.is_none());
        assert_eq!(proof.metadata.protocol_version, "1.0");
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = KeyPair::generate();
        let mut proof =
            ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        proof.sign(&kp);
        assert!(!proof.signature.0.is_empty());
        assert!(proof.verify(&kp.public).unwrap());
    }

    #[test]
    fn test_verify_wrong_key() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let mut proof =
            ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        proof.sign(&kp1);
        assert!(!proof.verify(&kp2.public).unwrap());
    }

    #[test]
    fn test_json_roundtrip() {
        let kp = KeyPair::generate();
        let mut proof =
            ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        proof.sign(&kp);
        let json = proof.to_json();
        let deserialized = ContactProof::from_json(&json).unwrap();
        assert_eq!(proof.id, deserialized.id);
        assert_eq!(proof.proving_node, deserialized.proving_node);
        assert_eq!(proof.signature.0, deserialized.signature.0);
    }

    #[test]
    fn test_cbor_roundtrip() {
        let kp = KeyPair::generate();
        let mut proof =
            ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        proof.sign(&kp);
        let cbor = proof.to_cbor();
        let deserialized = ContactProof::from_cbor(&cbor).unwrap();
        assert_eq!(proof.id, deserialized.id);
        assert_eq!(proof.proving_node, deserialized.proving_node);
    }

    #[test]
    fn test_is_valid_valid_proof() {
        let kp = KeyPair::generate();
        let mut proof =
            ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        proof.sign(&kp);
        assert!(proof.is_valid().unwrap());
    }

    #[test]
    fn test_is_valid_expired_window() {
        let kp = KeyPair::generate();
        let past_window = OrbitalWindow::new(
            Utc::now() - Duration::hours(2),
            Utc::now() - Duration::hours(1),
            WindowType::Standard,
        );
        let mut proof =
            ContactProof::new(NodeId::new(), NodeId::new(), past_window, test_metadata());
        proof.sign(&kp);
        let result = proof.is_valid();
        assert!(matches!(result, Err(Error::WindowExpired)));
    }

    #[test]
    fn test_is_valid_empty_signature() {
        let proof = ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        let result = proof.is_valid();
        assert!(matches!(result, Err(Error::InvalidSignature)));
    }

    #[test]
    fn test_is_valid_invalid_confidence() {
        let kp = KeyPair::generate();
        let mut meta = test_metadata();
        meta.confidence_score = 1.5;
        let mut proof = ContactProof::new(NodeId::new(), NodeId::new(), test_window(), meta);
        proof.sign(&kp);
        assert!(proof.is_valid().is_err());
    }

    #[test]
    fn test_canonical_bytes_deterministic() {
        let proof = ContactProof::new(NodeId::new(), NodeId::new(), test_window(), test_metadata());
        let b1 = proof.canonical_bytes();
        let b2 = proof.canonical_bytes();
        assert_eq!(b1, b2);
    }
}
