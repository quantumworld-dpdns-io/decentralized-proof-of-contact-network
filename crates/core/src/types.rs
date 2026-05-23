use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerId(pub String);

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OrbitalWindowId(pub String);

impl std::fmt::Display for OrbitalWindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    Verified,
    Failed,
    Pending,
    Expired,
}

impl VerificationStatus {
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Verified)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Failed => "failed",
            Self::Pending => "pending",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub id: Uuid,
    pub prover_id: NodeId,
    pub verifier_id: NodeId,
    pub timestamp: DateTime<Utc>,
    pub verification_status: VerificationStatus,
    pub confidence_score: f64,
    pub orbital_window: OrbitalWindowId,
    pub signature: Vec<u8>,
    pub metadata: serde_json::Value,
}

impl Proof {
    pub fn is_verified(&self) -> bool {
        self.verification_status.is_verified()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofChain {
    pub id: Uuid,
    pub proofs: Vec<Proof>,
    pub depth: usize,
}
