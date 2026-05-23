use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ProofId(pub Uuid);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContactProof {
    pub id: ProofId,
    pub proving_node: NodeId,
    pub target_node: NodeId,
    pub orbital_window: OrbitalWindow,
    pub timestamp: DateTime<Utc>,
    pub signature: Signature,
    pub pqc_signature: Option<PqcSignature>,
    pub metadata: ProofMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrbitalWindow {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub window_type: WindowType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum WindowType {
    Standard,
    Extended,
    Emergency,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofMetadata {
    pub protocol_version: String,
    pub chain_position: Option<u64>,
    pub confidence_score: f64,
    pub proof_purpose: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct Signature(pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct PqcSignature(pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct PublicKey(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, Zeroize)]
#[zeroize(drop)]
pub struct SecretKey(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct KeyPair {
    pub public: PublicKey,
    pub secret: SecretKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerId(pub String);

impl std::fmt::Display for PeerId {
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
pub struct ProofChain {
    pub id: Uuid,
    pub proofs: Vec<ContactProof>,
    pub depth: usize,
}
