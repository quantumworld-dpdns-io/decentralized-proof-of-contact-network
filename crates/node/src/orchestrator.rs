use crate::error::RuntimeError;
use poi_core::ContactProof;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofStatus {
    Pending,
    Signed,
    Stored,
    Gossiped,
    Verified,
    Archived,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ProofItem {
    pub proof: ContactProof,
    pub priority: u32,
    pub retry_count: u32,
    pub next_retry: Option<chrono::DateTime<chrono::Utc>>,
    pub status: ProofStatus,
}

impl Eq for ProofItem {}

impl PartialEq for ProofItem {
    fn eq(&self, other: &Self) -> bool {
        self.proof.id() == other.proof.id()
    }
}

impl PartialOrd for ProofItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProofItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.retry_count.cmp(&self.retry_count))
    }
}

pub struct ProofOrchestrator {
    queue: Arc<Mutex<BinaryHeap<ProofItem>>>,
    running: AtomicBool,
    max_retries: u32,
    batch_size: usize,
    processed_count: Arc<std::sync::atomic::AtomicU64>,
    failed_count: Arc<std::sync::atomic::AtomicU64>,
}

impl ProofOrchestrator {
    pub fn new(max_retries: u32, batch_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            running: AtomicBool::new(false),
            max_retries,
            batch_size,
            processed_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            failed_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn submit(&self, proof: ContactProof, priority: u32) {
        let mut queue = self.queue.lock().expect("queue lock poisoned");
        queue.push(ProofItem {
            proof,
            priority,
            retry_count: 0,
            next_retry: None,
            status: ProofStatus::Pending,
        });
        tracing::debug!("proof submitted to orchestrator queue");
    }

    pub fn start(&self, runtime: &crate::RuntimeManager) -> Result<(), RuntimeError> {
        self.running.store(true, AtomicOrdering::SeqCst);
        let queue = self.queue.clone();
        let running = Arc::new(AtomicBool::new(true));
        let max_retries = self.max_retries;
        let batch_size = self.batch_size;
        let processed = self.processed_count.clone();
        let failed = self.failed_count.clone();

        runtime.spawn(async move {
            tracing::info!("orchestrator started");

            while running.load(AtomicOrdering::SeqCst) {
                let batch = {
                    let mut q = queue.lock().expect("queue lock poisoned");
                    let mut batch = Vec::with_capacity(batch_size);
                    for _ in 0..batch_size {
                        if let Some(item) = q.pop() {
                            batch.push(item);
                        } else {
                            break;
                        }
                    }
                    batch
                };

                for item in batch {
                    match process_proof_lifecycle(&item).await {
                        Ok(()) => {
                            processed.fetch_add(1, AtomicOrdering::SeqCst);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "proof {:?} processing failed: {e}",
                                item.proof.id()
                            );
                            let mut q = queue.lock().expect("queue lock poisoned");
                            if item.retry_count < max_retries {
                                let mut retried = item.clone();
                                retried.retry_count += 1;
                                let delay = 2u64.pow(retried.retry_count.min(10));
                                retried.next_retry = Some(
                                    chrono::Utc::now()
                                        + chrono::Duration::seconds(delay as i64),
                                );
                                q.push(retried);
                            } else {
                                failed.fetch_add(1, AtomicOrdering::SeqCst);
                                tracing::error!(
                                    "proof {:?} failed after {max_retries} retries",
                                    item.proof.id()
                                );
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            tracing::info!("orchestrator stopped");
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), RuntimeError> {
        self.running.store(false, AtomicOrdering::SeqCst);
        tracing::info!("orchestrator stop requested");
        Ok(())
    }

    pub fn queue_size(&self) -> usize {
        self.queue.lock().expect("queue lock poisoned").len()
    }

    pub fn processed_count(&self) -> u64 {
        self.processed_count.load(AtomicOrdering::SeqCst)
    }

    pub fn failed_count(&self) -> u64 {
        self.failed_count.load(AtomicOrdering::SeqCst)
    }
}

async fn process_proof_lifecycle(item: &ProofItem) -> Result<(), String> {
    tracing::debug!(
        "processing proof {:?}: priority={}, retry={}",
        item.proof.id(),
        item.priority,
        item.retry_count
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use poi_core::{ContactProofExt, NodeId, OrbitalWindow, ProofMetadata, WindowType};

    fn make_proof() -> ContactProof {
        let window = OrbitalWindow::new(
            Utc::now(),
            Utc::now() + Duration::hours(1),
            WindowType::Standard,
        );
        let metadata = ProofMetadata {
            protocol_version: "1.0".to_string(),
            chain_position: None,
            confidence_score: 1.0,
            proof_purpose: "test".to_string(),
        };
        ContactProof::new(
            NodeId::new(),
            NodeId::new(),
            window,
            metadata,
        )
    }

    #[test]
    fn test_submit_and_queue_size() {
        let orch = ProofOrchestrator::new(3, 10);
        assert_eq!(orch.queue_size(), 0);
        orch.submit(make_proof(), 1);
        assert_eq!(orch.queue_size(), 1);
    }

    #[test]
    fn test_priority_ordering() {
        let mut heap = BinaryHeap::new();
        heap.push(ProofItem {
            proof: make_proof(),
            priority: 1,
            retry_count: 0,
            next_retry: None,
            status: ProofStatus::Pending,
        });
        heap.push(ProofItem {
            proof: make_proof(),
            priority: 10,
            retry_count: 0,
            next_retry: None,
            status: ProofStatus::Pending,
        });
        assert_eq!(heap.pop().unwrap().priority, 10);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let orch = ProofOrchestrator::new(3, 5);
        let mut rt = crate::RuntimeManager::with_workers(1);
        rt.start().unwrap();
        orch.start(&rt).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        orch.stop().unwrap();
    }

    #[test]
    fn test_proof_item_ord() {
        let high = ProofItem {
            proof: make_proof(),
            priority: 100,
            retry_count: 0,
            next_retry: None,
            status: ProofStatus::Pending,
        };
        let low = ProofItem {
            proof: make_proof(),
            priority: 1,
            retry_count: 0,
            next_retry: None,
            status: ProofStatus::Pending,
        };
        assert!(high > low);
        assert!(low < high);
    }
}
