//! Anthropic Claude LLM提供商实现

use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Anthropic API请求结构
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

/// Anthropic消息格式
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API响应结构
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

/// Anthropic内容结构
#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Anthropic使用统计
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic提供商实现
pub struct AnthropicProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
}

impl AnthropicProvider {
    /// 创建新的Anthropic提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        if config.api_key.is_none() {
            return Err(AgentMemError::config_error("Anthropic API key is required"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Anthropic可能需要更长时间
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {}", e))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());

        Ok(Self {
            config,
            client,
            base_url,
        })
    }

    /// 将Message转换为Anthropic格式，并提取系统消息
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_message = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role {
                agent_mem_traits::MessageRole::System => {
                    // Anthropic将系统消息单独处理
                    system_message = Some(msg.content.clone());
                }
                agent_mem_traits::MessageRole::User => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                agent_mem_traits::MessageRole::Assistant => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        (system_message, anthropic_messages)
    }

    /// 构建API请求
    fn build_request(&self, messages: &[Message]) -> AnthropicRequest {
        let (system, anthropic_messages) = self.convert_messages(messages);

        AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            messages: anthropic_messages,
            system,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("Anthropic API key not configured"))?;

        let request = self.build_request(messages);
        let url = format!("{}/v1/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::llm_error(format!(
                "Anthropic API error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {}", e))
        })?;

        if anthropic_response.content.is_empty() {
            return Err(AgentMemError::llm_error("No content in Anthropic response"));
        }

        // 合并所有文本内容
        let content = anthropic_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("");

        if content.is_empty() {
            return Err(AgentMemError::llm_error(
                "No text content in Anthropic response",
            ));
        }

        Ok(content)
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        // 流式生成的实现（简化版本）
        Err(AgentMemError::llm_error(
            "Streaming not implemented for Anthropic provider",
        ))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "anthropic".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            supports_streaming: false, // 暂时不支持
            supports_functions: false, // Claude不支持函数调用
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error("Anthropic API key is required"));
        }

        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }

        // 验证模型名称是否为已知的Anthropic模型
        let known_models = [
            "claude-3-opus-20240229",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
            "claude-2.1",
            "claude-2.0",
            "claude-instant-1.2",
        ];

        if !known_models.contains(&self.config.model.as_str()) {
            // 警告但不阻止，因为Anthropic可能会发布新模型
            eprintln!("Warning: Unknown Anthropic model: {}", self.config.model);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::MessageRole;

    #[test]
    fn test_anthropic_provider_creation() {
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = AnthropicProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_anthropic_provider_no_api_key() {
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: None,
            ..Default::default()
        };

        let provider = AnthropicProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_convert_messages() {
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = AnthropicProvider::new(config).unwrap();

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant".to_string(),
                timestamp: None,
            },
            Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
                timestamp: None,
            },
        ];

        let (system, anthropic_messages) = provider.convert_messages(&messages);
        assert_eq!(system, Some("You are a helpful assistant".to_string()));
        assert_eq!(anthropic_messages.len(), 1);
        assert_eq!(anthropic_messages[0].role, "user");
    }

    #[test]
    fn test_validate_config() {
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = AnthropicProvider::new(config).unwrap();
        assert!(provider.validate_config().is_ok());
    }
}
