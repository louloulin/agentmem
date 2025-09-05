//! DeepSeek LLM Provider
//!
//! DeepSeek API 集成，提供高质量的中英文语言模型服务

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use agent_mem_traits::{AgentMemError, Result};

/// DeepSeek API 配置
#[derive(Debug, Clone)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout: Duration,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            timeout: Duration::from_secs(30),
        }
    }
}

/// DeepSeek API 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekMessage {
    pub role: String,
    pub content: String,
}

/// DeepSeek API 请求体
#[derive(Debug, Serialize)]
pub struct DeepSeekRequest {
    pub model: String,
    pub messages: Vec<DeepSeekMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

/// DeepSeek API 响应
#[derive(Debug, Deserialize)]
pub struct DeepSeekResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<DeepSeekChoice>,
    pub usage: DeepSeekUsage,
}

#[derive(Debug, Deserialize)]
pub struct DeepSeekChoice {
    pub index: u32,
    pub message: DeepSeekMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeepSeekUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// DeepSeek LLM 提供商
pub struct DeepSeekProvider {
    config: DeepSeekConfig,
    client: Client,
}

impl DeepSeekProvider {
    /// 创建新的 DeepSeek 提供商实例
    pub fn new(config: DeepSeekConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| AgentMemError::LLMError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// 创建带有 API 密钥的 DeepSeek 提供商
    pub fn with_api_key(api_key: String) -> Result<Self> {
        let config = DeepSeekConfig {
            api_key,
            ..Default::default()
        };
        Self::new(config)
    }

    /// 发送聊天完成请求
    pub async fn chat_completion(&self, messages: Vec<DeepSeekMessage>) -> Result<DeepSeekResponse> {
        let request = DeepSeekRequest {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::LLMError(format!(
                "DeepSeek API error {}: {}",
                status, error_text
            )));
        }

        let deepseek_response: DeepSeekResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to parse response: {}", e)))?;

        Ok(deepseek_response)
    }

    /// 生成文本响应
    pub async fn generate_text(&self, prompt: &str) -> Result<String> {
        let messages = vec![DeepSeekMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let response = self.chat_completion(messages).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(AgentMemError::LLMError(
                "No response choices returned".to_string(),
            ))
        }
    }

    /// 生成结构化 JSON 响应
    pub async fn generate_json<T>(&self, prompt: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json_prompt = format!(
            "{}\n\nPlease respond with valid JSON only, no additional text or formatting.",
            prompt
        );

        let response_text = self.generate_text(&json_prompt).await?;

        // 尝试清理响应文本，移除可能的代码块标记
        let cleaned_text = response_text
            .trim()
            .strip_prefix("```json")
            .unwrap_or(&response_text)
            .strip_suffix("```")
            .unwrap_or(&response_text)
            .trim();

        serde_json::from_str(cleaned_text)
            .map_err(|e| AgentMemError::LLMError(format!("Failed to parse JSON response: {}", e)))
    }

    /// 生成系统和用户消息的响应
    pub async fn generate_with_system(&self, system: &str, user: &str) -> Result<String> {
        let messages = vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: user.to_string(),
            },
        ];

        let response = self.chat_completion(messages).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(AgentMemError::LLMError(
                "No response choices returned".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deepseek_config_default() {
        let config = DeepSeekConfig::default();
        assert_eq!(config.base_url, "https://api.deepseek.com/v1");
        assert_eq!(config.model, "deepseek-chat");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 4096);
    }

    #[tokio::test]
    async fn test_deepseek_provider_creation() {
        let config = DeepSeekConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = DeepSeekProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_deepseek_with_api_key() {
        let provider = DeepSeekProvider::with_api_key("test-key".to_string());
        assert!(provider.is_ok());
    }
}
