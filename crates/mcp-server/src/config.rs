#[derive(Debug, Clone)]
pub struct McpConfig {
    pub bind_addr: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub server_name: String,
    pub server_version: String,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:3100".to_string(),
            api_url: "http://127.0.0.1:3000".to_string(),
            api_key: None,
            server_name: "poi-mcp-server".to_string(),
            server_version: "0.1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert_eq!(config.bind_addr, "127.0.0.1:3100");
        assert_eq!(config.api_url, "http://127.0.0.1:3000");
        assert!(config.api_key.is_none());
        assert_eq!(config.server_name, "poi-mcp-server");
        assert_eq!(config.server_version, "0.1.0");
    }

    #[test]
    fn test_mcp_config_custom() {
        let config = McpConfig {
            bind_addr: "0.0.0.0:9090".into(),
            api_url: "https://api.example.com".into(),
            api_key: Some("secret".into()),
            server_name: "custom".into(),
            server_version: "2.0.0".into(),
        };
        assert_eq!(config.bind_addr, "0.0.0.0:9090");
        assert_eq!(config.api_key, Some("secret".into()));
        assert_eq!(config.server_name, "custom");
    }
}
