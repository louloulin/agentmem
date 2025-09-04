//! Azure OpenAI LLM提供商实现

use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

/// Azure OpenAI提供商实现
pub struct AzureProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
    api_version: String,
}

impl AzureProvider {
    /// 创建新的Azure OpenAI提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        if config.api_key.is_none() {
            return Err(AgentMemError::config_error(
                "Azure OpenAI API key is required",
            ));
        }

        if config.base_url.is_none() {
            return Err(AgentMemError::config_error(
                "Azure OpenAI endpoint URL is required",
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {}", e))
            })?;

        let base_url = config.base_url.clone().unwrap();
        let api_version = "2023-12-01-preview".to_string(); // 默认API版本

        Ok(Self {
            config,
            client,
            base_url,
            api_version,
        })
    }

    /// 设置API版本
    pub fn with_api_version(mut self, api_version: &str) -> Self {
        self.api_version = api_version.to_string();
        self
    }
}

#[async_trait]
impl LLMProvider for AzureProvider {
    async fn generate(&self, _messages: &[Message]) -> Result<String> {
        // Azure OpenAI的实现与OpenAI类似，但使用不同的认证和端点
        // 这里提供一个基础框架，实际实现需要根据Azure OpenAI的API规范
        Err(AgentMemError::llm_error(
            "Azure OpenAI provider not fully implemented yet",
        ))
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        Err(AgentMemError::llm_error(
            "Streaming not implemented for Azure provider",
        ))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "azure".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            supports_streaming: false,
            supports_functions: true,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error(
                "Azure OpenAI API key is required",
            ));
        }

        if self.config.base_url.is_none() {
            return Err(AgentMemError::config_error(
                "Azure OpenAI endpoint URL is required",
            ));
        }

        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Deployment name is required"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_provider_creation() {
        let config = LLMConfig {
            provider: "azure".to_string(),
            model: "gpt-35-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://your-resource.openai.azure.com".to_string()),
            ..Default::default()
        };

        let provider = AzureProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_azure_provider_missing_config() {
        let config = LLMConfig {
            provider: "azure".to_string(),
            model: "gpt-35-turbo".to_string(),
            api_key: None,
            base_url: None,
            ..Default::default()
        };

        let provider = AzureProvider::new(config);
        assert!(provider.is_err());
    }
}
