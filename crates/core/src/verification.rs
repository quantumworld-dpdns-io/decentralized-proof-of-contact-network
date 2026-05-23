use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::Utc;
use ed25519_dalek::{Signature as DalekSignature, VerifyingKey};

use crate::contact_proof::ContactProofExt;
use crate::error::{Error, Result};
use crate::types::{ContactProof, PublicKey};

#[derive(Debug, Clone)]
pub struct ProofVerifier {
    pub verify_signature: bool,
    pub verify_timestamp: bool,
    pub verify_orbital_window: bool,
    pub verify_chain_integrity: bool,
    pub max_time_drift: Duration,
}

impl Default for ProofVerifier {
    fn default() -> Self {
        ProofVerifier {
            verify_signature: true,
            verify_timestamp: true,
            verify_orbital_window: true,
            verify_chain_integrity: false,
            max_time_drift: Duration::from_secs(300),
        }
    }
}

impl ProofVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn verify_all(
        &self,
        proof: &ContactProof,
        public_key: &PublicKey,
        chain: Option<&[ContactProof]>,
    ) -> FullVerificationReport {
        let signature_valid = if self.verify_signature {
            verify_signature(proof, public_key).unwrap_or(false)
        } else {
            true
        };

        let timestamp_valid = if self.verify_timestamp {
            verify_timestamp(proof, self.max_time_drift).unwrap_or(false)
        } else {
            true
        };

        let orbital_window_valid = if self.verify_orbital_window {
            verify_orbital_window(proof).unwrap_or(false)
        } else {
            true
        };

        let chain_integrity_valid = if self.verify_chain_integrity {
            match chain {
                Some(c) => verify_chain_integrity(proof, c).unwrap_or(false),
                None => true,
            }
        } else {
            true
        };

        let all_passed =
            signature_valid && timestamp_valid && orbital_window_valid && chain_integrity_valid;

        FullVerificationReport {
            signature_valid,
            timestamp_valid,
            orbital_window_valid,
            chain_integrity_valid,
            all_passed,
        }
    }
}

pub fn verify_signature(proof: &ContactProof, public_key: &PublicKey) -> Result<bool> {
    if proof.signature.0.is_empty() {
        return Err(Error::InvalidSignature);
    }
    let data = proof.canonical_bytes();
    let pk_bytes: [u8; 32] = BASE64
        .decode(&public_key.0)
        .map_err(|e| Error::InvalidKey(format!("invalid public key base64: {}", e)))?
        .try_into()
        .map_err(|_| Error::InvalidKey("public key must be 32 bytes".into()))?;
    let verifying_key = VerifyingKey::from_bytes(&pk_bytes)
        .map_err(|e| Error::CryptoError(e.to_string()))?;
    let sig_bytes: [u8; 64] = BASE64
        .decode(&proof.signature.0)
        .map_err(|_| Error::InvalidSignature)?
        .try_into()
        .map_err(|_| Error::InvalidSignature)?;
    let dalek_sig =
        DalekSignature::from_bytes(&sig_bytes).map_err(|e| Error::CryptoError(e.to_string()))?;
    Ok(verifying_key.verify(&data, &dalek_sig).is_ok())
}

pub fn verify_timestamp(proof: &ContactProof, max_drift: Duration) -> Result<bool> {
    let now = Utc::now();
    let drift = (now - proof.timestamp)
        .to_std()
        .map_err(|_| Error::InvalidTimestamp("timestamp is in the future".into()))?;
    if drift > max_drift {
        return Err(Error::InvalidTimestamp(format!(
            "timestamp drift {}s exceeds max {}s",
            drift.as_secs(),
            max_drift.as_secs()
        )));
    }
    Ok(true)
}

pub fn verify_orbital_window(proof: &ContactProof) -> Result<bool> {
    if proof.orbital_window.end_time <= proof.orbital_window.start_time {
        return Err(Error::InvalidProof(
            "orbital window end must be after start".into(),
        ));
    }
    let now = Utc::now();
    if now < proof.orbital_window.start_time {
        return Err(Error::InvalidProof("orbital window has not started".into()));
    }
    if now > proof.orbital_window.end_time {
        return Err(Error::WindowExpired);
    }
    Ok(true)
}

pub fn verify_chain_integrity(proof: &ContactProof, chain: &[ContactProof]) -> Result<bool> {
    if chain.is_empty() {
        return Err(Error::InvalidProof("chain is empty".into()));
    }

    let pos = match proof.metadata.chain_position {
        Some(p) => p,
        None => return Err(Error::InvalidProof("proof has no chain position".into())),
    };

    if pos as usize >= chain.len() {
        return Err(Error::ChainBroken(pos));
    }

    let chain_proof = &chain[pos as usize];
    if chain_proof.id != proof.id {
        return Err(Error::ChainBroken(pos));
    }

    for i in 1..chain.len() {
        let prev_pos = chain[i - 1]
            .metadata
            .chain_position
            .ok_or(Error::InvalidProof(format!(
                "proof at index {} has no chain position",
                i - 1
            )))?;
        let curr_pos = chain[i]
            .metadata
            .chain_position
            .ok_or(Error::InvalidProof(format!(
                "proof at index {} has no chain position",
                i
            )))?;
        if curr_pos != prev_pos + 1 {
            return Err(Error::ChainBroken(curr_pos));
        }
    }

    Ok(true)
}

#[derive(Debug, Clone)]
pub struct FullVerificationReport {
    pub signature_valid: bool,
    pub timestamp_valid: bool,
    pub orbital_window_valid: bool,
    pub chain_integrity_valid: bool,
    pub all_passed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact_proof::ContactProofExt;
    use crate::keypair::KeyPairExt;
    use crate::types::*;
    use chrono::Duration as ChronoDuration;

    fn test_proof(keypair: &KeyPair) -> ContactProof {
        let window = OrbitalWindow::new(
            Utc::now() - ChronoDuration::minutes(5),
            Utc::now() + ChronoDuration::hours(1),
            WindowType::Standard,
        );
        let mut proof = ContactProof::new(
            NodeId::new(),
            NodeId::new(),
            window,
            ProofMetadata {
                protocol_version: "1.0".into(),
                chain_position: Some(0),
                confidence_score: 1.0,
                proof_purpose: "test".into(),
            },
        );
        proof.sign(keypair);
        proof
    }

    #[test]
    fn test_verify_signature_valid() {
        let kp = KeyPair::generate();
        let proof = test_proof(&kp);
        assert!(verify_signature(&proof, &kp.public).unwrap());
    }

    #[test]
    fn test_verify_signature_invalid_key() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let proof = test_proof(&kp1);
        assert!(!verify_signature(&proof, &kp2.public).unwrap());
    }

    #[test]
    fn test_verify_timestamp_valid() {
        let kp = KeyPair::generate();
        let proof = test_proof(&kp);
        assert!(verify_timestamp(&proof, Duration::from_secs(3600)).unwrap());
    }

    #[test]
    fn test_verify_timestamp_excessive_drift() {
        let kp = KeyPair::generate();
        let proof = test_proof(&kp);
        let result = verify_timestamp(&proof, Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_orbital_window_valid() {
        let kp = KeyPair::generate();
        let proof = test_proof(&kp);
        assert!(verify_orbital_window(&proof).unwrap());
    }

    #[test]
    fn test_verify_orbital_window_expired() {
        let kp = KeyPair::generate();
        let expired_window = OrbitalWindow::new(
            Utc::now() - ChronoDuration::hours(2),
            Utc::now() - ChronoDuration::hours(1),
            WindowType::Standard,
        );
        let mut proof = ContactProof::new(
            NodeId::new(),
            NodeId::new(),
            expired_window,
            ProofMetadata {
                protocol_version: "1.0".into(),
                chain_position: None,
                confidence_score: 1.0,
                proof_purpose: "test".into(),
            },
        );
        proof.sign(&kp);
        let result = verify_orbital_window(&proof);
        assert!(matches!(result, Err(Error::WindowExpired)));
    }

    #[test]
    fn test_verify_chain_integrity() {
        let kp = KeyPair::generate();
        let mut chain = Vec::new();
        for i in 0..3 {
            let window = OrbitalWindow::new(
                Utc::now() - ChronoDuration::minutes(5),
                Utc::now() + ChronoDuration::hours(1),
                WindowType::Standard,
            );
            let mut proof = ContactProof::new(
                NodeId::new(),
                NodeId::new(),
                window,
                ProofMetadata {
                    protocol_version: "1.0".into(),
                    chain_position: Some(i),
                    confidence_score: 1.0,
                    proof_purpose: "test".into(),
                },
            );
            proof.sign(&kp);
            chain.push(proof);
        }
        let last = chain.last().unwrap();
        assert!(verify_chain_integrity(last, &chain).unwrap());
    }

    #[test]
    fn test_verify_chain_integrity_broken() {
        let kp = KeyPair::generate();
        let mut chain = Vec::new();
        for i in 0..3 {
            let window = OrbitalWindow::new(
                Utc::now() - ChronoDuration::minutes(5),
                Utc::now() + ChronoDuration::hours(1),
                WindowType::Standard,
            );
            let mut proof = ContactProof::new(
                NodeId::new(),
                NodeId::new(),
                window,
                ProofMetadata {
                    protocol_version: "1.0".into(),
                    chain_position: Some(i * 2),
                    confidence_score: 1.0,
                    proof_purpose: "test".into(),
                },
            );
            proof.sign(&kp);
            chain.push(proof);
        }
        let last = chain.last().unwrap();
        let result = verify_chain_integrity(last, &chain);
        assert!(result.is_err());
    }

    #[test]
    fn test_proof_verifier_default() {
        let verifier = ProofVerifier::default();
        assert!(verifier.verify_signature);
        assert!(verifier.verify_timestamp);
        assert!(verifier.verify_orbital_window);
        assert!(!verifier.verify_chain_integrity);
    }

    #[test]
    fn test_verify_all() {
        let kp = KeyPair::generate();
        let proof = test_proof(&kp);
        let verifier = ProofVerifier::default();
        let report = verifier.verify_all(&proof, &kp.public, None);
        assert!(report.all_passed);
    }
}
