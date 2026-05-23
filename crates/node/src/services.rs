use crate::error::ServiceError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Service: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self) -> Result<(), ServiceError>;
    async fn stop(&self) -> Result<(), ServiceError>;
    fn dependencies(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

pub struct ServiceRegistry {
    services: HashMap<&'static str, Arc<dyn Service>>,
    start_order: Vec<&'static str>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            start_order: Vec::new(),
        }
    }

    pub fn register(&mut self, service: Arc<dyn Service>) -> Result<(), ServiceError> {
        let name = service.name();
        if self.services.contains_key(name) {
            return Err(ServiceError::AlreadyRegistered(name.to_string()));
        }
        for dep in service.dependencies() {
            if !self.services.contains_key(dep) {
                return Err(ServiceError::UnsatisfiedDependency {
                    service: name.to_string(),
                    dep: dep.to_string(),
                });
            }
        }
        self.start_order.push(name);
        self.services.insert(name, service);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<Arc<dyn Service>> {
        self.services.get(name).cloned()
    }

    pub async fn start_all(&self) -> Result<(), ServiceError> {
        for name in &self.start_order {
            let service = self.services.get(name).unwrap();
            tracing::info!("starting service: {}", name);
            service.start().await?;
            tracing::info!("service started: {}", name);
        }
        Ok(())
    }

    pub async fn stop_all(&self) -> Result<(), ServiceError> {
        for name in self.start_order.iter().rev() {
            let service = self.services.get(name).unwrap();
            tracing::info!("stopping service: {}", name);
            service.stop().await?;
            tracing::info!("service stopped: {}", name);
        }
        Ok(())
    }

    pub fn service_count(&self) -> usize {
        self.services.len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockService {
        name: &'static str,
        deps: Vec<&'static str>,
        started: std::sync::atomic::AtomicBool,
    }

    impl MockService {
        fn new(name: &'static str, deps: Vec<&'static str>) -> Self {
            Self {
                name,
                deps,
                started: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    #[async_trait]
    impl Service for MockService {
        fn name(&self) -> &'static str {
            self.name
        }

        fn dependencies(&self) -> Vec<&'static str> {
            self.deps.clone()
        }

        async fn start(&self) -> Result<(), ServiceError> {
            self.started.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn stop(&self) -> Result<(), ServiceError> {
            self.started.store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_and_resolve() {
        let mut registry = ServiceRegistry::new();
        let service = Arc::new(MockService::new("test", vec![]));
        registry.register(service.clone()).unwrap();
        let resolved = registry.resolve("test").unwrap();
        assert_eq!(resolved.name(), "test");
    }

    #[tokio::test]
    async fn test_double_register_fails() {
        let mut registry = ServiceRegistry::new();
        let service = Arc::new(MockService::new("dup", vec![]));
        registry.register(service.clone()).unwrap();
        let dup = Arc::new(MockService::new("dup", vec![]));
        assert!(registry.register(dup).is_err());
    }

    #[tokio::test]
    async fn test_unsatisfied_dependency() {
        let mut registry = ServiceRegistry::new();
        let service = Arc::new(MockService::new("dependent", vec!["missing"]));
        assert!(registry.register(service).is_err());
    }

    #[tokio::test]
    async fn test_start_stop_all() {
        let mut registry = ServiceRegistry::new();
        let s1 = Arc::new(MockService::new("svc1", vec![]));
        let s2 = Arc::new(MockService::new("svc2", vec![]));
        registry.register(s1).unwrap();
        registry.register(s2).unwrap();

        registry.start_all().await.unwrap();
        assert_eq!(registry.service_count(), 2);

        registry.stop_all().await.unwrap();
    }
}
