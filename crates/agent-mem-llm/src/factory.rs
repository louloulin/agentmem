//! LLM工厂模式实现

use crate::providers::{OpenAIProvider, AnthropicProvider};
#[cfg(feature = "azure")]
use crate::providers::AzureProvider;
#[cfg(feature = "gemini")]
use crate::providers::GeminiProvider;
#[cfg(feature = "ollama")]
use crate::providers::OllamaProvider;
use crate::providers::{ClaudeProvider, CohereProvider, MistralProvider, PerplexityProvider};

use agent_mem_traits::{LLMProvider, LLMConfig, Result, AgentMemError, ModelInfo, Message};
use async_trait::async_trait;
use std::sync::Arc;

/// LLM提供商枚举，包装不同的提供商实现
pub enum LLMProviderEnum {
    #[cfg(feature = "openai")]
    OpenAI(OpenAIProvider),
    #[cfg(feature = "anthropic")]
    Anthropic(AnthropicProvider),
    #[cfg(feature = "azure")]
    Azure(AzureProvider),
    #[cfg(feature = "gemini")]
    Gemini(GeminiProvider),
    #[cfg(feature = "ollama")]
    Ollama(OllamaProvider),
    Claude(ClaudeProvider),
    Cohere(CohereProvider),
    Mistral(MistralProvider),
    Perplexity(PerplexityProvider),
}

#[async_trait]
impl LLMProvider for LLMProviderEnum {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.generate(messages).await,
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.generate(messages).await,
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.generate(messages).await,
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.generate(messages).await,
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.generate(messages).await,
            LLMProviderEnum::Claude(provider) => provider.generate(messages).await,
            LLMProviderEnum::Cohere(provider) => provider.generate(messages).await,
            LLMProviderEnum::Mistral(provider) => provider.generate(messages).await,
            LLMProviderEnum::Perplexity(provider) => provider.generate(messages).await,
        }
    }

    async fn generate_stream(&self, messages: &[Message]) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Claude(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Cohere(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Mistral(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Perplexity(provider) => provider.generate_stream(messages).await,
        }
    }

    fn get_model_info(&self) -> ModelInfo {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.get_model_info(),
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.get_model_info(),
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.get_model_info(),
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.get_model_info(),
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.get_model_info(),
            LLMProviderEnum::Claude(provider) => provider.get_model_info(),
            LLMProviderEnum::Cohere(provider) => provider.get_model_info(),
            LLMProviderEnum::Mistral(provider) => provider.get_model_info(),
            LLMProviderEnum::Perplexity(provider) => provider.get_model_info(),
        }
    }

    fn validate_config(&self) -> Result<()> {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.validate_config(),
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.validate_config(),
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.validate_config(),
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.validate_config(),
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.validate_config(),
            LLMProviderEnum::Claude(provider) => provider.validate_config(),
            LLMProviderEnum::Cohere(provider) => provider.validate_config(),
            LLMProviderEnum::Mistral(provider) => provider.validate_config(),
            LLMProviderEnum::Perplexity(provider) => provider.validate_config(),
        }
    }
}

/// LLM工厂，用于创建不同的LLM提供商实例
pub struct LLMFactory;

impl LLMFactory {
    /// 根据配置创建LLM提供商实例
    pub fn create_provider(config: &LLMConfig) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let provider_enum = match config.provider.as_str() {
            "openai" => {
                #[cfg(feature = "openai")]
                {
                    let provider = OpenAIProvider::new(config.clone())?;
                    LLMProviderEnum::OpenAI(provider)
                }
                #[cfg(not(feature = "openai"))]
                {
                    return Err(AgentMemError::unsupported_provider("OpenAI feature not enabled"));
                }
            }
            "anthropic" => {
                #[cfg(feature = "anthropic")]
                {
                    let provider = AnthropicProvider::new(config.clone())?;
                    LLMProviderEnum::Anthropic(provider)
                }
                #[cfg(not(feature = "anthropic"))]
                {
                    return Err(AgentMemError::unsupported_provider("Anthropic feature not enabled"));
                }
            }
            "azure" => {
                #[cfg(feature = "azure")]
                {
                    let provider = AzureProvider::new(config.clone())?;
                    LLMProviderEnum::Azure(provider)
                }
                #[cfg(not(feature = "azure"))]
                {
                    return Err(AgentMemError::unsupported_provider("Azure feature not enabled"));
                }
            }
            "gemini" => {
                #[cfg(feature = "gemini")]
                {
                    let provider = GeminiProvider::new(config.clone())?;
                    LLMProviderEnum::Gemini(provider)
                }
                #[cfg(not(feature = "gemini"))]
                {
                    return Err(AgentMemError::unsupported_provider("Gemini feature not enabled"));
                }
            }
            "ollama" => {
                #[cfg(feature = "ollama")]
                {
                    let provider = OllamaProvider::new(config.clone())?;
                    LLMProviderEnum::Ollama(provider)
                }
                #[cfg(not(feature = "ollama"))]
                {
                    return Err(AgentMemError::unsupported_provider("Ollama feature not enabled"));
                }
            }
            "claude" => {
                let provider = ClaudeProvider::new(config.clone())?;
                LLMProviderEnum::Claude(provider)
            }
            "cohere" => {
                let provider = CohereProvider::new(config.clone())?;
                LLMProviderEnum::Cohere(provider)
            }
            "mistral" => {
                let provider = MistralProvider::new(config.clone())?;
                LLMProviderEnum::Mistral(provider)
            }
            "perplexity" => {
                let provider = PerplexityProvider::new(config.clone())?;
                LLMProviderEnum::Perplexity(provider)
            }
            _ => return Err(AgentMemError::unsupported_provider(&config.provider)),
        };

        Ok(Arc::new(provider_enum))
    }

    /// 获取支持的提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        let mut providers = Vec::new();

        #[cfg(feature = "openai")]
        providers.push("openai");

        #[cfg(feature = "anthropic")]
        providers.push("anthropic");

        #[cfg(feature = "azure")]
        providers.push("azure");

        #[cfg(feature = "gemini")]
        providers.push("gemini");

        #[cfg(feature = "ollama")]
        providers.push("ollama");

        // New providers (always available)
        providers.push("claude");
        providers.push("cohere");
        providers.push("mistral");
        providers.push("perplexity");

        providers
    }

    /// 检查提供商是否受支持
    pub fn is_provider_supported(provider: &str) -> bool {
        Self::supported_providers().contains(&provider)
    }

    /// 创建默认的OpenAI提供商（如果启用）
    #[cfg(feature = "openai")]
    pub fn create_openai_provider(api_key: &str) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建默认的Anthropic提供商（如果启用）
    #[cfg(feature = "anthropic")]
    pub fn create_anthropic_provider(api_key: &str) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建本地Ollama提供商（如果启用）
    #[cfg(feature = "ollama")]
    pub fn create_ollama_provider(base_url: Option<&str>, model: &str) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "ollama".to_string(),
            model: model.to_string(),
            base_url: Some(base_url.unwrap_or("http://localhost:11434").to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Claude提供商
    pub fn create_claude_provider(api_key: &str, model: Option<&str>) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "claude".to_string(),
            model: model.unwrap_or("claude-3-haiku-20240307").to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Cohere提供商
    pub fn create_cohere_provider(api_key: &str, model: Option<&str>) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "cohere".to_string(),
            model: model.unwrap_or("command-r").to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Mistral提供商
    pub fn create_mistral_provider(api_key: &str, model: Option<&str>) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "mistral".to_string(),
            model: model.unwrap_or("mistral-small-latest").to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Perplexity提供商
    pub fn create_perplexity_provider(api_key: &str, model: Option<&str>) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "perplexity".to_string(),
            model: model.unwrap_or("llama-3.1-sonar-small-128k-chat").to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_providers() {
        let providers = LLMFactory::supported_providers();
        assert!(!providers.is_empty());
        
        // 检查默认启用的提供商
        #[cfg(feature = "openai")]
        assert!(providers.contains(&"openai"));
        
        #[cfg(feature = "anthropic")]
        assert!(providers.contains(&"anthropic"));
    }

    #[test]
    fn test_is_provider_supported() {
        #[cfg(feature = "openai")]
        assert!(LLMFactory::is_provider_supported("openai"));
        
        #[cfg(feature = "anthropic")]
        assert!(LLMFactory::is_provider_supported("anthropic"));
        
        assert!(!LLMFactory::is_provider_supported("unsupported_provider"));
    }

    #[test]
    fn test_create_provider_unsupported() {
        let config = LLMConfig {
            provider: "unsupported".to_string(),
            ..Default::default()
        };
        
        let result = LLMFactory::create_provider(&config);
        assert!(result.is_err());
    }

    #[cfg(feature = "openai")]
    #[test]
    fn test_create_openai_provider() {
        let result = LLMFactory::create_openai_provider("test-key");
        assert!(result.is_ok());
    }

    #[cfg(feature = "anthropic")]
    #[test]
    fn test_create_anthropic_provider() {
        let result = LLMFactory::create_anthropic_provider("test-key");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_claude_provider() {
        let result = LLMFactory::create_claude_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_cohere_provider() {
        let result = LLMFactory::create_cohere_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_mistral_provider() {
        let result = LLMFactory::create_mistral_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_perplexity_provider() {
        let result = LLMFactory::create_perplexity_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_providers_supported() {
        let providers = LLMFactory::supported_providers();
        assert!(providers.contains(&"claude"));
        assert!(providers.contains(&"cohere"));
        assert!(providers.contains(&"mistral"));
        assert!(providers.contains(&"perplexity"));
    }
}
