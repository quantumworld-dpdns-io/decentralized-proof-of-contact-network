use std::sync::LazyLock;

use prometheus::{
    register_int_counter, register_int_gauge, IntCounter, IntGauge,
};

static PROOFS_CREATED: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("poi_proofs_created_total", "Total number of contact proofs created")
        .expect("failed to register proofs_created counter")
});

static PROOFS_VERIFIED: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("poi_proofs_verified_total", "Total number of proofs verified")
        .expect("failed to register proofs_verified counter")
});

static PROOFS_FAILED: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("poi_proofs_failed_total", "Total number of proofs that failed verification")
        .expect("failed to register proofs_failed counter")
});

static PROOFS_REJECTED: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("poi_proofs_rejected_total", "Total number of proofs rejected")
        .expect("failed to register proofs_rejected counter")
});

static ACTIVE_PROOFS: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("poi_active_proofs", "Current number of active proofs")
        .expect("failed to register active_proofs gauge")
});

static CHAIN_LENGTH: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("poi_chain_length", "Current proof chain length")
        .expect("failed to register chain_length gauge")
});

pub fn increment_proofs_created() {
    PROOFS_CREATED.inc();
}

pub fn increment_proofs_verified() {
    PROOFS_VERIFIED.inc();
}

pub fn increment_proofs_failed() {
    PROOFS_FAILED.inc();
}

pub fn increment_proofs_rejected() {
    PROOFS_REJECTED.inc();
}

pub fn set_active_proofs(count: i64) {
    ACTIVE_PROOFS.set(count);
}

pub fn set_chain_length(length: i64) {
    CHAIN_LENGTH.set(length);
}

pub fn get_proofs_created() -> u64 {
    PROOFS_CREATED.get()
}

pub fn get_proofs_verified() -> u64 {
    PROOFS_VERIFIED.get()
}

pub fn get_proofs_failed() -> u64 {
    PROOFS_FAILED.get()
}

pub fn get_proofs_rejected() -> u64 {
    PROOFS_REJECTED.get()
}

pub fn get_active_proofs() -> i64 {
    ACTIVE_PROOFS.get()
}

#[derive(Debug, Clone)]
pub struct ProofMetrics {
    pub proofs_created: u64,
    pub proofs_verified: u64,
    pub proofs_failed: u64,
    pub proofs_rejected: u64,
    pub active_proofs: i64,
    pub chain_length: i64,
}

impl ProofMetrics {
    pub fn new() -> Self {
        ProofMetrics {
            proofs_created: 0,
            proofs_verified: 0,
            proofs_failed: 0,
            proofs_rejected: 0,
            active_proofs: 0,
            chain_length: 0,
        }
    }

    pub fn collect() -> Self {
        ProofMetrics {
            proofs_created: get_proofs_created(),
            proofs_verified: get_proofs_verified(),
            proofs_failed: get_proofs_failed(),
            proofs_rejected: PROOFS_REJECTED.get(),
            active_proofs: get_active_proofs(),
            chain_length: CHAIN_LENGTH.get(),
        }
    }

    pub fn snapshot(&self) -> ProofMetricsSnapshot {
        ProofMetricsSnapshot {
            proofs_created: self.proofs_created,
            proofs_verified: self.proofs_verified,
            proofs_failed: self.proofs_failed,
            proofs_rejected: self.proofs_rejected,
            active_proofs: self.active_proofs,
            chain_length: self.chain_length,
        }
    }
}

impl Default for ProofMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProofMetricsSnapshot {
    pub proofs_created: u64,
    pub proofs_verified: u64,
    pub proofs_failed: u64,
    pub proofs_rejected: u64,
    pub active_proofs: i64,
    pub chain_length: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = ProofMetrics::new();
        assert_eq!(metrics.proofs_created, 0);
        assert_eq!(metrics.proofs_verified, 0);
        assert_eq!(metrics.active_proofs, 0);
    }

    #[test]
    fn test_increment_proofs_created() {
        let before = get_proofs_created();
        increment_proofs_created();
        assert_eq!(get_proofs_created(), before + 1);
    }

    #[test]
    fn test_increment_proofs_verified() {
        let before = get_proofs_verified();
        increment_proofs_verified();
        assert_eq!(get_proofs_verified(), before + 1);
    }

    #[test]
    fn test_increment_proofs_failed() {
        let before = get_proofs_failed();
        increment_proofs_failed();
        assert_eq!(get_proofs_failed(), before + 1);
    }

    #[test]
    fn test_set_active_proofs() {
        set_active_proofs(42);
        assert_eq!(get_active_proofs(), 42);
    }

    #[test]
    fn test_collect() {
        increment_proofs_created();
        let metrics = ProofMetrics::collect();
        assert!(metrics.proofs_created > 0);
    }

    #[test]
    fn test_snapshot() {
        let metrics = ProofMetrics::new();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.proofs_created, 0);
    }

    #[test]
    fn test_metrics_default() {
        let metrics = ProofMetrics::default();
        assert_eq!(metrics.proofs_created, 0);
    }
}
