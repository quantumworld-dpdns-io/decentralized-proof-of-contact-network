use crate::error::RuntimeError;
use chrono::{DateTime, Duration, Utc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Open,
    Closed,
}

pub struct OrbitalWindowScheduler {
    window_duration: Duration,
    check_interval: Duration,
    running: AtomicBool,
    window_state: Arc<Mutex<WindowState>>,
    windows_opened: Arc<std::sync::atomic::AtomicU64>,
    contacts_detected: Arc<std::sync::atomic::AtomicU64>,
}

impl OrbitalWindowScheduler {
    pub fn new(window_duration: Duration, check_interval: Duration) -> Self {
        Self {
            window_duration,
            check_interval,
            running: AtomicBool::new(false),
            window_state: Arc::new(Mutex::new(WindowState::Closed)),
            windows_opened: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            contacts_detected: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub async fn window_state(&self) -> WindowState {
        *self.window_state.lock().await
    }

    pub fn start(&self, runtime: &crate::RuntimeManager) -> Result<(), RuntimeError> {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let window_duration = self.window_duration;
        let check_interval = self.check_interval;
        let window_state = self.window_state.clone();
        let windows_opened = self.windows_opened.clone();
        let contacts_detected = self.contacts_detected.clone();

        runtime.spawn(async move {
            let mut next_window_open: DateTime<Utc> = Utc::now();
            tracing::info!("scheduler started");

            while running.load(Ordering::SeqCst) {
                let now = Utc::now();

                if now >= next_window_open {
                    {
                        let mut state = window_state.lock().await;
                        *state = WindowState::Open;
                    }
                    windows_opened.fetch_add(1, Ordering::SeqCst);
                    tracing::info!(
                        "contact window opened (total: {})",
                        windows_opened.load(Ordering::SeqCst)
                    );

                    tokio::time::sleep(tokio::time::Duration::from_secs(
                        window_duration.num_seconds().max(1) as u64,
                    ))
                    .await;

                    {
                        let mut state = window_state.lock().await;
                        *state = WindowState::Closed;
                    }
                    tracing::info!("contact window closed");

                    next_window_open = next_window_open
                        + window_duration * 2;
                }

                // Simulate contact detection during open window
                let is_open = *window_state.lock().await == WindowState::Open;
                if is_open {
                    contacts_detected.fetch_add(1, Ordering::SeqCst);
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(
                    check_interval.num_seconds().max(1) as u64,
                ))
                .await;
            }

            tracing::info!("scheduler stopped");
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), RuntimeError> {
        self.running.store(false, Ordering::SeqCst);
        tracing::info!("scheduler stop requested");
        Ok(())
    }

    pub fn windows_opened(&self) -> u64 {
        self.windows_opened.load(Ordering::SeqCst)
    }

    pub fn contacts_detected(&self) -> u64 {
        self.contacts_detected.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_window_state_initial() {
        let scheduler =
            OrbitalWindowScheduler::new(Duration::minutes(5), Duration::seconds(30));
        assert_eq!(scheduler.window_state().await, WindowState::Closed);
    }

    #[ignore = "requires refactoring to avoid nested tokio runtime (RuntimeManager inside #[tokio::test])"]
    #[tokio::test]
    async fn test_windows_opened_count() {
        let scheduler =
            OrbitalWindowScheduler::new(Duration::seconds(1), Duration::milliseconds(100));
        let mut rt = crate::RuntimeManager::with_workers(1);
        rt.start().unwrap();

        scheduler.start(&rt).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        scheduler.stop().unwrap();

        assert!(scheduler.windows_opened() > 0);
    }

    #[test]
    fn test_window_state_equality() {
        assert_eq!(WindowState::Open, WindowState::Open);
        assert_eq!(WindowState::Closed, WindowState::Closed);
        assert_ne!(WindowState::Open, WindowState::Closed);
    }
}
