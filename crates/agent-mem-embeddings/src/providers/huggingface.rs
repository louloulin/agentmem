//! HuggingFace嵌入提供商实现

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;

/// HuggingFace嵌入提供商
/// 注意：这是一个基础实现框架，实际的HuggingFace集成需要更多的依赖和实现
pub struct HuggingFaceEmbedder {
    config: EmbeddingConfig,
}

impl HuggingFaceEmbedder {
    /// 创建新的HuggingFace嵌入器实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        // 验证模型名称
        if config.model.is_empty() {
            return Err(AgentMemError::config_error(
                "HuggingFace model name is required",
            ));
        }

        Ok(Self { config })
    }

    /// 模拟嵌入生成（实际实现需要集成HuggingFace transformers）
    async fn generate_mock_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        // 这里返回一个模拟的嵌入向量
        // 实际实现应该使用HuggingFace的transformers库
        let embedding = vec![0.1; self.config.dimension];
        Ok(embedding)
    }
}

#[async_trait]
impl Embedder for HuggingFaceEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // 目前返回模拟嵌入，实际实现需要集成HuggingFace模型
        self.generate_mock_embedding(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        // 简单的顺序处理，实际实现应该支持批量处理
        for text in texts {
            let embedding = self.embed(text).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn provider_name(&self) -> &str {
        "huggingface"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    async fn health_check(&self) -> Result<bool> {
        // 简单的健康检查，实际实现应该检查模型是否可用
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_huggingface_embedder_creation() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let result = HuggingFaceEmbedder::new(config).await;
        assert!(result.is_ok());

        let embedder = result.unwrap();
        assert_eq!(embedder.provider_name(), "huggingface");
        assert_eq!(
            embedder.model_name(),
            "sentence-transformers/all-MiniLM-L6-v2"
        );
        assert_eq!(embedder.dimension(), 768);
    }

    #[tokio::test]
    async fn test_huggingface_embedder_empty_model() {
        let config = EmbeddingConfig {
            provider: "huggingface".to_string(),
            model: "".to_string(),
            ..Default::default()
        };

        let result = HuggingFaceEmbedder::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_embed_single_text() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let embedder = HuggingFaceEmbedder::new(config).await.unwrap();

        let result = embedder.embed("test text").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let embedder = HuggingFaceEmbedder::new(config).await.unwrap();

        let texts = vec![
            "first text".to_string(),
            "second text".to_string(),
            "third text".to_string(),
        ];

        let result = embedder.embed_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 768);
        assert_eq!(embeddings[1].len(), 768);
        assert_eq!(embeddings[2].len(), 768);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let embedder = HuggingFaceEmbedder::new(config).await.unwrap();

        let result = embedder.health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
