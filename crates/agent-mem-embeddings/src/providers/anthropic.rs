//! Anthropic嵌入模型实现（模拟）
//! 注意：Anthropic目前主要专注于文本生成，这里提供一个模拟实现

use agent_mem_traits::{Embedder, Result, AgentMemError};
use crate::config::EmbeddingConfig;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Anthropic嵌入请求
#[derive(Debug, Serialize)]
struct AnthropicEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

/// Anthropic嵌入响应
#[derive(Debug, Deserialize)]
struct AnthropicEmbeddingResponse {
    data: Vec<AnthropicEmbeddingData>,
    model: String,
    usage: AnthropicUsage,
}

/// Anthropic嵌入数据
#[derive(Debug, Deserialize)]
struct AnthropicEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

/// Anthropic使用统计
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// Anthropic嵌入模型实现
pub struct AnthropicEmbedder {
    config: EmbeddingConfig,
    client: Client,
    api_key: String,
}

impl AnthropicEmbedder {
    /// 创建新的Anthropic嵌入器实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        let api_key = config.api_key.clone()
            .ok_or_else(|| AgentMemError::config_error("Anthropic API key is required"))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AgentMemError::network_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            client,
            api_key,
        })
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        // 注意：这是一个模拟实现，因为Anthropic目前不提供嵌入API
        // 在实际实现中，这里应该调用真实的API端点
        Ok(true)
    }
}

#[async_trait]
impl Embedder for AnthropicEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // 模拟实现：返回基于文本内容的确定性向量
        // 在实际实现中，这里应该调用Anthropic的嵌入API
        let mut embedding = vec![0.0; self.dimension()];
        
        // 基于文本内容生成确定性向量
        let bytes = text.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if i < embedding.len() {
                embedding[i] = (byte as f32) / 255.0;
            }
        }
        
        // 标准化向量
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // 模拟批量嵌入
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    async fn health_check(&self) -> Result<bool> {
        self.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_embedder_creation_no_api_key() {
        let config = EmbeddingConfig {
            provider: "anthropic".to_string(),
            model: "claude-embedding".to_string(),
            api_key: None,
            dimension: 1536,
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(AnthropicEmbedder::new(config));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_anthropic_embedder_creation_with_api_key() {
        let config = EmbeddingConfig {
            provider: "anthropic".to_string(),
            model: "claude-embedding".to_string(),
            api_key: Some("test-key".to_string()),
            dimension: 1536,
            ..Default::default()
        };

        let result = AnthropicEmbedder::new(config).await;
        assert!(result.is_ok());

        let embedder = result.unwrap();
        assert_eq!(embedder.provider_name(), "anthropic");
        assert_eq!(embedder.dimension(), 1536);
    }

    #[tokio::test]
    async fn test_embed_single_text() {
        let config = EmbeddingConfig {
            provider: "anthropic".to_string(),
            model: "claude-embedding".to_string(),
            api_key: Some("test-key".to_string()),
            dimension: 768,
            ..Default::default()
        };

        let embedder = AnthropicEmbedder::new(config).await.unwrap();
        let result = embedder.embed("Hello, world!").await;
        
        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 768);
        
        // 检查向量是否已标准化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let config = EmbeddingConfig {
            provider: "anthropic".to_string(),
            model: "claude-embedding".to_string(),
            api_key: Some("test-key".to_string()),
            dimension: 384,
            ..Default::default()
        };

        let embedder = AnthropicEmbedder::new(config).await.unwrap();
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
        ];
        
        let result = embedder.embed_batch(&texts).await;
        assert!(result.is_ok());
        
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = EmbeddingConfig {
            provider: "anthropic".to_string(),
            model: "claude-embedding".to_string(),
            api_key: Some("test-key".to_string()),
            dimension: 1536,
            ..Default::default()
        };

        let embedder = AnthropicEmbedder::new(config).await.unwrap();
        let result = embedder.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_deterministic_embedding() {
        let config = EmbeddingConfig {
            provider: "anthropic".to_string(),
            model: "claude-embedding".to_string(),
            api_key: Some("test-key".to_string()),
            dimension: 256,
            ..Default::default()
        };

        let embedder = AnthropicEmbedder::new(config).await.unwrap();
        
        // 相同的文本应该产生相同的嵌入
        let embedding1 = embedder.embed("test text").await.unwrap();
        let embedding2 = embedder.embed("test text").await.unwrap();
        
        assert_eq!(embedding1, embedding2);
        
        // 不同的文本应该产生不同的嵌入
        let embedding3 = embedder.embed("different text").await.unwrap();
        assert_ne!(embedding1, embedding3);
    }
}
