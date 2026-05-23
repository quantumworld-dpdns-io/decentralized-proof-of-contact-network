use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::error::NetworkError;
use crate::message::PeerId;

#[derive(Clone)]
pub struct TokenBucket {
    capacity: u64,
    tokens: Arc<AtomicU64>,
    refill_rate: u64,
    refill_interval: Duration,
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucket {
    pub fn new(capacity: u64, refill_per_sec: u64) -> Self {
        Self {
            capacity,
            tokens: Arc::new(AtomicU64::new(capacity)),
            refill_rate: refill_per_sec,
            refill_interval: Duration::from_secs(1),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub async fn try_consume(&self, count: u64) -> Result<(), NetworkError> {
        self.refill().await;
        let current = self.tokens.load(Ordering::Acquire);
        if current < count {
            return Err(NetworkError::RateLimited);
        }
        self.tokens.fetch_sub(count, Ordering::Release);
        Ok(())
    }

    async fn refill(&self) {
        let mut last = self.last_refill.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);
        if elapsed >= self.refill_interval {
            let elapsed_secs = elapsed.as_secs().max(1);
            let add = elapsed_secs.saturating_mul(self.refill_rate);
            self.tokens
                .fetch_update(Ordering::Release, Ordering::Acquire, |t| {
                    Some((t + add).min(self.capacity))
                })
                .ok();
            *last = now;
        }
    }

    pub fn available_tokens(&self) -> u64 {
        self.tokens.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    pub global: TokenBucket,
    per_peer: Arc<Mutex<HashMap<PeerId, TokenBucket>>>,
    per_peer_capacity: u64,
    per_peer_rate: u64,
}

impl RateLimiter {
    pub fn new(global_capacity: u64, global_rate: u64, per_peer_capacity: u64, per_peer_rate: u64) -> Self {
        Self {
            global: TokenBucket::new(global_capacity, global_rate),
            per_peer: Arc::new(Mutex::new(HashMap::new())),
            per_peer_capacity,
            per_peer_rate,
        }
    }

    pub async fn check_message(&self, peer_id: PeerId) -> Result<(), NetworkError> {
        self.global.try_consume(1).await?;
        let mut peers = self.per_peer.lock().await;
        let bucket = peers
            .entry(peer_id)
            .or_insert_with(|| TokenBucket::new(self.per_peer_capacity, self.per_peer_rate));
        bucket.try_consume(1).await
    }

    pub async fn peer_bucket(&self, peer_id: PeerId) -> TokenBucket {
        let mut peers = self.per_peer.lock().await;
        peers
            .entry(peer_id)
            .or_insert_with(|| TokenBucket::new(self.per_peer_capacity, self.per_peer_rate))
            .clone()
    }

    pub async fn remove_peer(&self, peer_id: PeerId) {
        let mut peers = self.per_peer.lock().await;
        peers.remove(&peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket_basic() {
        let bucket = TokenBucket::new(10, 10);
        assert_eq!(bucket.available_tokens(), 10);
        assert!(bucket.try_consume(5).await.is_ok());
        assert_eq!(bucket.available_tokens(), 5);
    }

    #[tokio::test]
    async fn test_token_bucket_rate_limited() {
        let bucket = TokenBucket::new(3, 10);
        assert!(bucket.try_consume(3).await.is_ok());
        assert!(bucket.try_consume(1).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_global() {
        let limiter = RateLimiter::new(5, 10, 5, 10);
        let peer = PeerId::new();
        for _ in 0..5 {
            assert!(limiter.check_message(peer).await.is_ok());
        }
        assert!(limiter.check_message(peer).await.is_err());
    }
}
