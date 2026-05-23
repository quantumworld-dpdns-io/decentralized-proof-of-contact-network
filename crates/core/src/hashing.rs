use sha2::Digest;
use uuid::Uuid;

use crate::contact_proof::ContactProofExt;
use crate::types::{ContactProof, ProofId};

pub fn hash_blake3(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub trait ProofIdExt {
    fn from_hash(data: &[u8]) -> ProofId;
}

impl ProofIdExt for ProofId {
    fn from_hash(data: &[u8]) -> ProofId {
        let hash = blake3::hash(data);
        let bytes = hash.as_bytes();
        let uuid_bytes: [u8; 16] = bytes[..16].try_into().expect("blake3 output >= 16 bytes");
        let uuid = Uuid::from_bytes(uuid_bytes);
        ProofId(uuid)
    }
}

impl ProofId {
    pub fn new() -> Self {
        ProofId(Uuid::new_v4())
    }
}

impl Default for ProofId {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hash_proof(proof: &ContactProof) -> String {
    let canonical = proof.canonical_bytes();
    hash_blake3(&canonical)
}

pub fn hash_concatenation(hashes: &[String]) -> String {
    let mut combined = Vec::new();
    for h in hashes {
        combined.extend_from_slice(h.as_bytes());
    }
    hash_blake3(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;

    fn test_proof() -> ContactProof {
        ContactProof {
            id: ProofId::new(),
            proving_node: NodeId("poi-550e8400-e29b-41d4-a716-446655440000".into()),
            target_node: NodeId("poi-550e8400-e29b-41d4-a716-446655440001".into()),
            orbital_window: OrbitalWindow::new(
                Utc::now(),
                Utc::now() + chrono::Duration::hours(1),
                WindowType::Standard,
            ),
            timestamp: Utc::now(),
            signature: Signature("test_sig".into()),
            pqc_signature: None,
            metadata: ProofMetadata {
                protocol_version: "1.0".into(),
                chain_position: None,
                confidence_score: 1.0,
                proof_purpose: "test".into(),
            },
        }
    }

    #[test]
    fn test_hash_blake3() {
        let hash = hash_blake3(b"hello world");
        assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_sha256() {
        let hash = hash_sha256(b"hello world");
        assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_blake3_deterministic() {
        let h1 = hash_blake3(b"test");
        let h2 = hash_blake3(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha256_deterministic() {
        let h1 = hash_sha256(b"test");
        let h2 = hash_sha256(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_proof_id_from_hash() {
        let pid = ProofId::from_hash(b"some data");
        assert_ne!(pid.0.to_string(), Uuid::nil().to_string());
    }

    #[test]
    fn test_proof_id_deterministic() {
        let pid1 = ProofId::from_hash(b"same data");
        let pid2 = ProofId::from_hash(b"same data");
        assert_eq!(pid1, pid2);
    }

    #[test]
    fn test_proof_id_new() {
        let pid1 = ProofId::new();
        let pid2 = ProofId::new();
        assert_ne!(pid1, pid2);
    }

    #[test]
    fn test_hash_proof() {
        let proof = test_proof();
        let h = hash_proof(&proof);
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_hash_proof_deterministic() {
        let proof = test_proof();
        let h1 = hash_proof(&proof);
        let h2 = hash_proof(&proof);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_concatenation() {
        let h1 = hash_blake3(b"a");
        let h2 = hash_blake3(b"b");
        let combined = hash_concatenation(&[h1, h2]);
        assert_eq!(combined.len(), 64);
    }

    #[test]
    fn test_empty_merkle_root() {
        let root = compute_merkle_root(&[]);
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn test_single_merkle_root() {
        let h = hash_blake3(b"single");
        let root = compute_merkle_root(&[h.clone()]);
        assert_eq!(root, h);
    }

    #[test]
    fn test_merkle_root_two_leaves() {
        let h1 = hash_blake3(b"leaf1");
        let h2 = hash_blake3(b"leaf2");
        let root = compute_merkle_root(&[h1.clone(), h2.clone()]);
        let expected = hash_concatenation(&[h1, h2]);
        assert_eq!(root, expected);
    }
}
