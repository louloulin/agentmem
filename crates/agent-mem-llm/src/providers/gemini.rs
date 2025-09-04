//! Google Gemini LLM提供商实现

use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

/// Google Gemini提供商实现
pub struct GeminiProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
}

impl GeminiProvider {
    /// 创建新的Gemini提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        if config.api_key.is_none() {
            return Err(AgentMemError::config_error("Google AI API key is required"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {}", e))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());

        Ok(Self {
            config,
            client,
            base_url,
        })
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    async fn generate(&self, _messages: &[Message]) -> Result<String> {
        // Google Gemini的实现
        // 这里提供一个基础框架，实际实现需要根据Gemini API的规范
        Err(AgentMemError::llm_error(
            "Gemini provider not fully implemented yet",
        ))
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        Err(AgentMemError::llm_error(
            "Streaming not implemented for Gemini provider",
        ))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "gemini".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(8192),
            supports_streaming: false,
            supports_functions: true,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error("Google AI API key is required"));
        }

        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }

        // 验证模型名称是否为已知的Gemini模型
        let known_models = [
            "gemini-pro",
            "gemini-pro-vision",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
        ];

        if !known_models.contains(&self.config.model.as_str()) {
            eprintln!("Warning: Unknown Gemini model: {}", self.config.model);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_creation() {
        let config = LLMConfig {
            provider: "gemini".to_string(),
            model: "gemini-pro".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = GeminiProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_gemini_provider_no_api_key() {
        let config = LLMConfig {
            provider: "gemini".to_string(),
            model: "gemini-pro".to_string(),
            api_key: None,
            ..Default::default()
        };

        let provider = GeminiProvider::new(config);
        assert!(provider.is_err());
    }
}
