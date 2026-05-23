use poi_core::ContactProof;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct CacheEntry {
    proof: ContactProof,
    inserted_at: Instant,
}

struct CacheInner {
    map: HashMap<uuid::Uuid, CacheEntry>,
    order: VecDeque<uuid::Uuid>,
    hits: u64,
    misses: u64,
}

pub struct ProofCache {
    max_capacity: usize,
    ttl: Duration,
    inner: Mutex<CacheInner>,
}

impl ProofCache {
    pub fn new(max_capacity: usize, ttl: Duration) -> Self {
        Self {
            max_capacity,
            ttl,
            inner: Mutex::new(CacheInner {
                map: HashMap::new(),
                order: VecDeque::new(),
                hits: 0,
                misses: 0,
            }),
        }
    }

    pub fn get(&self, id: &uuid::Uuid) -> Option<ContactProof> {
        let mut inner = self.inner.lock().expect("cache lock poisoned");

        let proof = {
            let entry = inner.map.get(id)?;
            if entry.inserted_at.elapsed() > self.ttl {
                inner.map.remove(id);
                inner.order.retain(|k| k != id);
                inner.misses += 1;
                return None;
            }
            entry.proof.clone()
        };

        inner.hits += 1;
        inner.order.retain(|k| k != id);
        inner.order.push_back(*id);
        Some(proof)
    }

    pub fn insert(&self, proof: ContactProof) {
        let mut inner = self.inner.lock().expect("cache lock poisoned");
        let id = proof.id.0;

        if !inner.map.contains_key(&id) && inner.map.len() >= self.max_capacity {
            if let Some(evict_id) = inner.order.pop_front() {
                inner.map.remove(&evict_id);
            }
        }

        inner.map.insert(
            id,
            CacheEntry {
                proof,
                inserted_at: Instant::now(),
            },
        );
        inner.order.retain(|k| *k != id);
        inner.order.push_back(id);
    }

    pub fn remove(&self, id: &uuid::Uuid) -> bool {
        let mut inner = self.inner.lock().expect("cache lock poisoned");
        let existed = inner.map.remove(id).is_some();
        if existed {
            inner.order.retain(|k| k != id);
        }
        existed
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().expect("cache lock poisoned");
        inner.map.clear();
        inner.order.clear();
    }

    pub fn contains(&self, id: &uuid::Uuid) -> bool {
        let inner = self.inner.lock().expect("cache lock poisoned");
        inner.map.contains_key(id)
    }

    pub fn stats(&self) -> CacheStats {
        let inner = self.inner.lock().expect("cache lock poisoned");
        CacheStats {
            size: inner.map.len(),
            capacity: self.max_capacity,
            hits: inner.hits,
            misses: inner.misses,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration as ChronoDuration, Utc};
    use poi_core::{ContactProofExt, NodeId, OrbitalWindow, ProofMetadata, WindowType};

    fn make_proof(id_val: u64) -> ContactProof {
        let proof_id = poi_core::ProofId(uuid::Uuid::from_u64_pair(id_val, 0));
        let window = OrbitalWindow::new(
            Utc::now(),
            Utc::now() + ChronoDuration::hours(1),
            WindowType::Standard,
        );
        let metadata = ProofMetadata {
            protocol_version: "1.0".to_string(),
            chain_position: None,
            confidence_score: 1.0,
            proof_purpose: "test".to_string(),
        };
        // Need to reconstruct with the specific id
        let mut proof = ContactProof::new(
            NodeId::new(),
            NodeId::new(),
            window,
            metadata,
        );
        proof.id = proof_id;
        proof
    }

    #[test]
    fn test_insert_and_get() {
        let cache = ProofCache::new(10, Duration::from_secs(60));
        let proof = make_proof(1);
        let id = proof.id.0;

        assert!(cache.get(&id).is_none());
        cache.insert(proof.clone());
        let retrieved = cache.get(&id).unwrap();
        assert_eq!(retrieved.id.0, id);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = ProofCache::new(2, Duration::from_secs(60));
        cache.insert(make_proof(1));
        cache.insert(make_proof(2));
        cache.insert(make_proof(3));

        assert!(!cache.contains(&uuid::Uuid::from_u64_pair(1, 0)));
        assert!(cache.contains(&uuid::Uuid::from_u64_pair(2, 0)));
        assert!(cache.contains(&uuid::Uuid::from_u64_pair(3, 0)));
    }

    #[test]
    fn test_lru_order() {
        let cache = ProofCache::new(2, Duration::from_secs(60));
        cache.insert(make_proof(1));
        cache.insert(make_proof(2));

        // Access 1 to make it recently used
        cache.get(&uuid::Uuid::from_u64_pair(1, 0));

        // Insert 3 should evict 2 (LRU)
        cache.insert(make_proof(3));

        assert!(cache.contains(&uuid::Uuid::from_u64_pair(1, 0)));
        assert!(!cache.contains(&uuid::Uuid::from_u64_pair(2, 0)));
        assert!(cache.contains(&uuid::Uuid::from_u64_pair(3, 0)));
    }

    #[test]
    fn test_ttl_expiry() {
        let cache = ProofCache::new(10, Duration::from_millis(10));
        let proof = make_proof(1);
        let id = proof.id.0;
        cache.insert(proof);
        assert!(cache.get(&id).is_some());

        std::thread::sleep(Duration::from_millis(20));
        assert!(cache.get(&id).is_none());
    }

    #[test]
    fn test_stats() {
        let cache = ProofCache::new(5, Duration::from_secs(60));
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert!((stats.hit_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remove() {
        let cache = ProofCache::new(5, Duration::from_secs(60));
        let proof = make_proof(1);
        let id = proof.id.0;
        cache.insert(proof);
        assert!(cache.contains(&id));
        assert!(cache.remove(&id));
        assert!(!cache.contains(&id));
        assert!(!cache.remove(&id)); // already removed
    }

    #[test]
    fn test_clear() {
        let cache = ProofCache::new(5, Duration::from_secs(60));
        cache.insert(make_proof(1));
        cache.insert(make_proof(2));
        assert_eq!(cache.stats().size, 2);
        cache.clear();
        assert_eq!(cache.stats().size, 0);
    }

    #[test]
    fn test_capacity_zero() {
        let cache = ProofCache::new(0, Duration::from_secs(60));
        cache.insert(make_proof(1));
        // With capacity 0, nothing should be stored
        assert_eq!(cache.stats().size, 0);
    }
}
