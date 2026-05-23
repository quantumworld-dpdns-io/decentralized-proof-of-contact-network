pub mod error;
pub mod types;
pub mod node_id;
pub mod keypair;
pub mod orbital_window;
pub mod contact_proof;
pub mod hashing;
pub mod verification;
pub mod proof_chain;
pub mod config;
pub mod metrics;

pub use error::{Error, Result};
pub use types::{
    ContactProof, KeyPair, NodeId, OrbitalWindow, PqcSignature, ProofId, ProofMetadata, PublicKey,
    SecretKey, Signature, WindowType,
};
pub use node_id::{generate_node_id, validate_node_id};
pub use keypair::KeyPairExt;
pub use orbital_window::{
    daily_window, emergency_window, extended_window, hourly_window, OrbitalWindowBuilder,
};
pub use contact_proof::ContactProofExt;
pub use hashing::{hash_blake3, hash_concatenation, hash_proof, hash_sha256, ProofIdExt};
pub use proof_chain::ProofChain;
pub use verification::{
    verify_chain_integrity, verify_orbital_window, verify_signature, verify_timestamp,
    FullVerificationReport, ProofVerifier,
};
pub use config::{CoreConfig, NetworkConfig, StorageConfig};
pub use metrics::{ProofMetrics, ProofMetricsSnapshot};
