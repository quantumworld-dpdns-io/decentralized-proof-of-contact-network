use crate::error::StateError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Init,
    Syncing,
    Active,
    Degraded,
    Shutdown,
}

impl NodeState {
    pub fn can_transition_to(self, target: NodeState) -> bool {
        matches!(
            (self, target),
            (NodeState::Init, NodeState::Syncing)
                | (NodeState::Init, NodeState::Shutdown)
                | (NodeState::Syncing, NodeState::Active)
                | (NodeState::Syncing, NodeState::Degraded)
                | (NodeState::Active, NodeState::Degraded)
                | (NodeState::Active, NodeState::Shutdown)
                | (NodeState::Degraded, NodeState::Active)
                | (NodeState::Degraded, NodeState::Shutdown)
        )
    }
}

pub struct StateManager {
    state: RwLock<NodeState>,
    state_path: Option<PathBuf>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(NodeState::Init),
            state_path: None,
        }
    }

    pub fn with_persistence(path: PathBuf) -> Self {
        let initial_state = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or(NodeState::Init)
        } else {
            NodeState::Init
        };

        Self {
            state: RwLock::new(initial_state),
            state_path: Some(path),
        }
    }

    pub fn current(&self) -> NodeState {
        *self.state.read().expect("state lock poisoned")
    }

    pub fn transition(&self, target: NodeState) -> Result<NodeState, StateError> {
        let current = self.current();
        if !current.can_transition_to(target) {
            return Err(StateError::InvalidTransition {
                from: current,
                to: target,
            });
        }
        {
            let mut state = self.state.write().expect("state lock poisoned");
            *state = target;
        }
        if let Some(path) = &self.state_path {
            if let Err(e) = persist_to_file(path, target) {
                tracing::warn!("failed to persist state: {e}");
            }
        }
        Ok(target)
    }
}

fn persist_to_file(path: &PathBuf, state: NodeState) -> Result<(), StateError> {
    let content =
        serde_json::to_string(&state).map_err(|e| StateError::PersistenceError(e.to_string()))?;
    std::fs::write(path, &content)
        .map_err(|e| StateError::PersistenceError(e.to_string()))?;
    Ok(())
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let sm = StateManager::new();
        assert_eq!(sm.current(), NodeState::Init);

        sm.transition(NodeState::Syncing).unwrap();
        assert_eq!(sm.current(), NodeState::Syncing);

        sm.transition(NodeState::Active).unwrap();
        assert_eq!(sm.current(), NodeState::Active);

        sm.transition(NodeState::Degraded).unwrap();
        assert_eq!(sm.current(), NodeState::Degraded);

        sm.transition(NodeState::Active).unwrap();
        assert_eq!(sm.current(), NodeState::Active);

        sm.transition(NodeState::Shutdown).unwrap();
        assert_eq!(sm.current(), NodeState::Shutdown);
    }

    #[test]
    fn test_invalid_transition() {
        let sm = StateManager::new();
        assert!(sm.transition(NodeState::Active).is_err());
        assert!(sm.transition(NodeState::Degraded).is_err());
    }

    #[test]
    fn test_can_transition_to() {
        assert!(NodeState::Init.can_transition_to(NodeState::Syncing));
        assert!(NodeState::Init.can_transition_to(NodeState::Shutdown));
        assert!(!NodeState::Init.can_transition_to(NodeState::Active));
        assert!(!NodeState::Init.can_transition_to(NodeState::Degraded));
        assert!(NodeState::Syncing.can_transition_to(NodeState::Active));
        assert!(NodeState::Syncing.can_transition_to(NodeState::Degraded));
        assert!(!NodeState::Syncing.can_transition_to(NodeState::Init));
        assert!(NodeState::Active.can_transition_to(NodeState::Degraded));
        assert!(NodeState::Active.can_transition_to(NodeState::Shutdown));
        assert!(!NodeState::Active.can_transition_to(NodeState::Init));
        assert!(!NodeState::Active.can_transition_to(NodeState::Syncing));
    }

    #[test]
    fn test_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");

        let sm = StateManager::with_persistence(path.clone());
        assert_eq!(sm.current(), NodeState::Init);
        sm.transition(NodeState::Syncing).unwrap();
        drop(sm);

        let sm2 = StateManager::with_persistence(path);
        assert_eq!(sm2.current(), NodeState::Syncing);
    }

    #[test]
    fn test_serde_roundtrip() {
        let states = [
            NodeState::Init,
            NodeState::Syncing,
            NodeState::Active,
            NodeState::Degraded,
            NodeState::Shutdown,
        ];
        for &state in &states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: NodeState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, deserialized);
        }
    }
}
