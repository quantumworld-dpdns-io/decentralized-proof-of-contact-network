use async_trait::async_trait;

use crate::error::Result;
use crate::provider::{AiProvider, ChatConfig, ChatMessage, ChatResponse};
use super::openai_compat::OpenAiCompatibleProvider;

make_openai_compat_provider!(LMStudioProvider, "http://localhost:1234");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiConfig;

    #[test]
    fn test_lm_studio_construction() {
        let config = AiConfig::default();
        let provider = LMStudioProvider::new(&config);
        assert_eq!(provider.0.model(), "llama3.2");
    }
}
