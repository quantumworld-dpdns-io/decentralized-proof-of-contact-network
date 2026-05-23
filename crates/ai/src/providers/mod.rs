#[cfg(feature = "ollama")]
pub mod ollama;

#[cfg(any(
    feature = "lm-studio",
    feature = "vllm",
    feature = "sglang",
    feature = "llamacpp"
))]
pub mod openai_compat;

#[cfg(feature = "lm-studio")]
pub mod lm_studio;

#[cfg(feature = "vllm")]
pub mod vllm;

#[cfg(feature = "sglang")]
pub mod sglang;

#[cfg(feature = "llamacpp")]
pub mod llamacpp;

use crate::config::AiProvider;
use crate::error::{AiError, Result};
use crate::provider::AiProvider as AiProviderTrait;

pub fn create_provider(config: &crate::config::AiConfig) -> Result<Box<dyn AiProviderTrait>> {
    match config.provider {
        #[cfg(feature = "ollama")]
        AiProvider::Ollama => Ok(Box::new(ollama::OllamaProvider::new(config))),
        #[cfg(feature = "lm-studio")]
        AiProvider::LMStudio => Ok(Box::new(lm_studio::LMStudioProvider::new(config))),
        #[cfg(feature = "vllm")]
        AiProvider::Vllm => Ok(Box::new(vllm::VllmProvider::new(config))),
        #[cfg(feature = "sglang")]
        AiProvider::Sglang => Ok(Box::new(sglang::SglangProvider::new(config))),
        #[cfg(feature = "llamacpp")]
        AiProvider::Llamacpp => Ok(Box::new(llamacpp::LlamacppProvider::new(config))),
        #[allow(unreachable_patterns)]
        _ => Err(AiError::ProviderError(format!(
            "Provider '{}' is not supported (feature not enabled)",
            config.provider.as_str()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider_ollama() {
        let mut config = crate::config::AiConfig::default();
        config.provider = AiProvider::Ollama;
        let result = create_provider(&config);
        #[cfg(feature = "ollama")]
        assert!(result.is_ok());
        #[cfg(not(feature = "ollama"))]
        assert!(result.is_err());
    }
}
