use crate::error::RuntimeError;
use std::num::NonZeroUsize;
use tokio::runtime::{self, Runtime};

pub struct RuntimeManager {
    runtime: Option<Runtime>,
    worker_count: usize,
}

fn default_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(4)
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            runtime: None,
            worker_count: default_worker_count(),
        }
    }

    pub fn with_workers(worker_count: usize) -> Self {
        Self {
            runtime: None,
            worker_count,
        }
    }

    pub fn start(&mut self) -> Result<(), RuntimeError> {
        if self.runtime.is_some() {
            return Err(RuntimeError::AlreadyRunning);
        }
        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(self.worker_count)
            .enable_all()
            .build()
            .map_err(|e| RuntimeError::SpawnError(e.to_string()))?;
        self.runtime = Some(rt);
        Ok(())
    }

    pub fn block_on<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.runtime
            .as_ref()
            .expect("Runtime not started")
            .block_on(f)
    }

    pub fn spawn<F>(&self, f: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime
            .as_ref()
            .expect("Runtime not started")
            .spawn(f)
    }

    pub fn shutdown(&mut self) {
        if let Some(rt) = self.runtime.take() {
            rt.shutdown_background();
        }
    }

    pub fn is_running(&self) -> bool {
        self.runtime.is_some()
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_start_stop() {
        let mut mgr = RuntimeManager::with_workers(2);
        assert!(!mgr.is_running());
        mgr.start().unwrap();
        assert!(mgr.is_running());

        let result = mgr.block_on(async { 42 });
        assert_eq!(result, 42);

        mgr.shutdown();
        assert!(!mgr.is_running());
    }

    #[test]
    fn test_double_start_fails() {
        let mut mgr = RuntimeManager::with_workers(1);
        mgr.start().unwrap();
        assert!(mgr.start().is_err());
    }

    #[test]
    fn test_spawn_and_await() {
        let mut mgr = RuntimeManager::with_workers(1);
        mgr.start().unwrap();

        let handle = mgr.spawn(async {
            "hello from spawned task"
        });
        let result = mgr.block_on(async { handle.await.unwrap() });
        assert_eq!(result, "hello from spawned task");
    }
}
