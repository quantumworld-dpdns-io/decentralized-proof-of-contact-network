use crate::error::{Error, Result};
use crate::hashing::{hash_blake3, hash_concatenation, hash_proof};
use crate::types::{ContactProof, ProofId};

#[derive(Debug, Clone)]
pub struct ProofChain {
    proofs: Vec<ContactProof>,
}

impl ProofChain {
    pub fn new() -> Self {
        ProofChain { proofs: Vec::new() }
    }

    pub fn append(&mut self, mut proof: ContactProof) -> Result<()> {
        let pos = self.proofs.len() as u64;
        proof.metadata.chain_position = Some(pos);
        self.proofs.push(proof);
        Ok(())
    }

    pub fn verify_integrity(&self) -> Result<bool> {
        for (i, proof) in self.proofs.iter().enumerate() {
            let expected_pos = Some(i as u64);
            if proof.metadata.chain_position != expected_pos {
                return Err(Error::ChainBroken(i as u64));
            }
        }
        Ok(true)
    }

    pub fn get_proof(&self, id: &ProofId) -> Option<&ContactProof> {
        self.proofs.iter().find(|p| p.id == *id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ContactProof> {
        self.proofs.iter()
    }

    pub fn len(&self) -> usize {
        self.proofs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.proofs.is_empty()
    }

    pub fn merkle_root(&self) -> String {
        if self.proofs.is_empty() {
            return hash_blake3(b"empty");
        }
        let hashes: Vec<String> = self.proofs.iter().map(|p| hash_proof(p)).collect();
        compute_merkle_root(&hashes)
    }

    pub fn get_proofs(&self) -> &[ContactProof] {
        &self.proofs
    }
}

impl Default for ProofChain {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for ProofChain {
    type Item = ContactProof;
    type IntoIter = std::vec::IntoIter<ContactProof>;

    fn into_iter(self) -> Self::IntoIter {
        self.proofs.into_iter()
    }
}

fn compute_merkle_root(hashes: &[String]) -> String {
    if hashes.is_empty() {
        return hash_blake3(b"empty");
    }
    if hashes.len() == 1 {
        return hashes[0].clone();
    }

    let mut current: Vec<String> = hashes.to_vec();
    while current.len() > 1 {
        let mut next = Vec::new();
        for chunk in current.chunks(2) {
            if chunk.len() == 2 {
                next.push(hash_concatenation(&[chunk[0].clone(), chunk[1].clone()]));
            } else {
                next.push(chunk[0].clone());
            }
        }
        current = next;
    }
    current.into_iter().next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact_proof::ContactProofExt;
    use crate::keypair::KeyPairExt;
    use crate::types::*;
    use chrono::{Duration, Utc};

    fn make_test_proof(keypair: &KeyPair, position: u64) -> ContactProof {
        let window = OrbitalWindow::new(
            Utc::now() - Duration::minutes(5),
            Utc::now() + Duration::hours(1),
            WindowType::Standard,
        );
        let mut proof = ContactProof::new(
            NodeId::new(),
            NodeId::new(),
            window,
            ProofMetadata {
                protocol_version: "1.0".into(),
                chain_position: Some(position),
                confidence_score: 1.0,
                proof_purpose: "test".into(),
            },
        );
        proof.sign(keypair);
        proof
    }

    #[test]
    fn test_proof_chain_new() {
        let chain = ProofChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_proof_chain_append() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        let proof = make_test_proof(&kp, 0);
        chain.append(proof).unwrap();
        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_proof_chain_verify_integrity() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        for _ in 0..3 {
            let proof = make_test_proof(&kp, 0);
            chain.append(proof).unwrap();
        }
        assert!(chain.verify_integrity().unwrap());
    }

    #[test]
    fn test_proof_chain_get_proof() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        let proof = make_test_proof(&kp, 0);
        let id = proof.id;
        chain.append(proof).unwrap();
        let retrieved = chain.get_proof(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }

    #[test]
    fn test_proof_chain_get_proof_not_found() {
        let chain = ProofChain::new();
        let id = ProofId::new();
        assert!(chain.get_proof(&id).is_none());
    }

    #[test]
    fn test_proof_chain_iter() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        for _ in 0..3 {
            chain.append(make_test_proof(&kp, 0)).unwrap();
        }
        let count = chain.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_merkle_root_empty() {
        let chain = ProofChain::new();
        let root = chain.merkle_root();
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn test_merkle_root_single() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        chain.append(make_test_proof(&kp, 0)).unwrap();
        let root = chain.merkle_root();
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn test_merkle_root_multiple() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        for _ in 0..4 {
            chain.append(make_test_proof(&kp, 0)).unwrap();
        }
        let root = chain.merkle_root();
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let kp = KeyPair::generate();
        let mut chain1 = ProofChain::new();
        let mut chain2 = ProofChain::new();
        for _ in 0..3 {
            let proof = make_test_proof(&kp, 0);
            chain1.append(proof.clone()).unwrap();
            chain2.append(proof).unwrap();
        }
        assert_eq!(chain1.merkle_root(), chain2.merkle_root());
    }

    #[test]
    fn test_into_iterator() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        for _ in 0..3 {
            chain.append(make_test_proof(&kp, 0)).unwrap();
        }
        let count = chain.into_iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_append_sets_chain_position() {
        let kp = KeyPair::generate();
        let mut chain = ProofChain::new();
        let proof = make_test_proof(&kp, 0);
        chain.append(proof).unwrap();
        assert_eq!(chain.get_proofs()[0].metadata.chain_position, Some(0));
    }
}
