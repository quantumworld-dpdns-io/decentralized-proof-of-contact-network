use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct NodeId(pub String);

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
