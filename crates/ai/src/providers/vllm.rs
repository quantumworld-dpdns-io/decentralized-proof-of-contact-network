use async_trait::async_trait;

use crate::error::Result;
use crate::provider::{AiProvider, ChatConfig, ChatMessage, ChatResponse};
use super::openai_compat::OpenAiCompatibleProvider;

make_openai_compat_provider!(VllmProvider, "http://localhost:8000");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiConfig;

    #[test]
    fn test_vllm_construction() {
        let config = AiConfig::default();
        let provider = VllmProvider::new(&config);
        assert_eq!(provider.0.model(), "llama3.2");
    }
}
