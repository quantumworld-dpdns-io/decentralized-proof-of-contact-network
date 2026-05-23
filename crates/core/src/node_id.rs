use crate::types::NodeId;
use uuid::Uuid;

pub fn generate_node_id() -> NodeId {
    NodeId(format!("poi-{}", Uuid::new_v4()))
}

pub fn validate_node_id(id: &NodeId) -> bool {
    if !id.0.starts_with("poi-") {
        return false;
    }
    let uuid_part = &id.0[4..];
    Uuid::parse_str(uuid_part).is_ok()
}

impl NodeId {
    pub fn new() -> Self {
        generate_node_id()
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_node_id() {
        let id = generate_node_id();
        assert!(id.0.starts_with("poi-"));
        assert_eq!(id.0.len(), 40); // "poi-" (4) + UUID (36)
    }

    #[test]
    fn test_validate_valid_node_id() {
        let id = generate_node_id();
        assert!(validate_node_id(&id));
    }

    #[test]
    fn test_validate_invalid_node_id() {
        let invalid = NodeId("invalid".to_string());
        assert!(!validate_node_id(&invalid));

        let no_prefix = NodeId("550e8400-e29b-41d4-a716-446655440000".to_string());
        assert!(!validate_node_id(&no_prefix));

        let bad_uuid = NodeId("poi-not-a-uuid".to_string());
        assert!(!validate_node_id(&bad_uuid));
    }

    #[test]
    fn test_node_id_new() {
        let id = NodeId::new();
        assert!(validate_node_id(&id));
    }

    #[test]
    fn test_node_id_default() {
        let id = NodeId::default();
        assert!(validate_node_id(&id));
    }
}
