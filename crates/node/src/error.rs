use thiserror::Error;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Startup error: {0}")]
    Startup(#[from] StartupError),
    #[error("Shutdown error: {0}")]
    Shutdown(#[from] ShutdownError),
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("State error: {0}")]
    State(#[from] StateError),
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum StartupError {
    #[error("Failed to initialize runtime: {0}")]
    RuntimeInit(String),
    #[error("Failed to load configuration: {0}")]
    ConfigLoad(String),
    #[error("Failed to start service '{service}': {cause}")]
    ServiceStart { service: String, cause: String },
    #[error("Failed to initialize state: {0}")]
    StateInit(String),
    #[error("Failed to bind address {addr}: {cause}")]
    BindError { addr: String, cause: String },
}

#[derive(Error, Debug)]
pub enum ShutdownError {
    #[error("Failed to stop service '{service}': {cause}")]
    ServiceStop { service: String, cause: String },
    #[error("Graceful shutdown timed out")]
    Timeout,
    #[error("Runtime shutdown error: {0}")]
    RuntimeShutdown(String),
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service '{0}' not found")]
    NotFound(String),
    #[error("Service '{0}' already registered")]
    AlreadyRegistered(String),
    #[error("Service '{0}' failed: {1}")]
    ServiceFailure(String, String),
    #[error("Dependency '{dep}' not satisfied for service '{service}'")]
    UnsatisfiedDependency { service: String, dep: String },
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file {path}: {cause}")]
    IoError { path: String, cause: String },
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    #[error("Unsupported config format: {0}")]
    UnsupportedFormat(String),
    #[error("Missing required config field: {0}")]
    MissingField(String),
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Invalid state transition")]
    InvalidTransition { from: crate::state::NodeState, to: crate::state::NodeState },
    #[error("State persistence error: {0}")]
    PersistenceError(String),
    #[error("State recovery error: {0}")]
    RecoveryError(String),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Runtime not initialized")]
    NotInitialized,
    #[error("Runtime already running")]
    AlreadyRunning,
    #[error("Runtime shut down")]
    ShutDown,
    #[error("Failed to spawn task: {0}")]
    SpawnError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NodeError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(err.to_string().contains("file not found"));

        let err = StartupError::RuntimeInit("oops".into());
        assert!(err.to_string().contains("oops"));

        let err = ConfigError::UnsupportedFormat("xml".into());
        assert!(err.to_string().contains("xml"));

        let err = StateError::PersistenceError("disk full".into());
        assert!(err.to_string().contains("disk full"));
    }
}
