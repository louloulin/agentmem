//! 本地嵌入提供商实现

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use std::path::Path;

/// 本地嵌入提供商
/// 注意：这是一个基础实现框架，实际的本地模型集成需要更多的依赖和实现
pub struct LocalEmbedder {
    config: EmbeddingConfig,
    model_path: String,
}

impl LocalEmbedder {
    /// 创建新的本地嵌入器实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        let model_path = config
            .get_model_path()
            .ok_or_else(|| AgentMemError::config_error("Local model path is required"))?
            .to_string();

        // 验证模型路径是否存在
        if !Path::new(&model_path).exists() {
            return Err(AgentMemError::config_error(format!(
                "Model path does not exist: {}",
                model_path
            )));
        }

        Ok(Self { config, model_path })
    }

    /// 加载本地模型（模拟实现）
    async fn load_model(&self) -> Result<()> {
        // 这里应该实际加载本地模型
        // 实际实现可能使用candle、ort（ONNX Runtime）或其他推理框架
        println!("Loading local model from: {}", self.model_path);
        Ok(())
    }

    /// 模拟嵌入生成（实际实现需要使用本地模型推理）
    async fn generate_embedding_with_model(&self, _text: &str) -> Result<Vec<f32>> {
        // 这里返回一个模拟的嵌入向量
        // 实际实现应该使用加载的本地模型进行推理
        let embedding = (0..self.config.dimension)
            .map(|i| (i as f32 * 0.001) % 1.0)
            .collect();
        Ok(embedding)
    }

    /// 批量处理文本（优化版本）
    async fn process_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        // 实际实现应该支持真正的批量推理以提高效率
        for text in texts {
            let embedding = self.generate_embedding_with_model(text).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }
}

#[async_trait]
impl Embedder for LocalEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.generate_embedding_with_model(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // 将文本分批处理以避免内存问题
        let batch_size = self.config.batch_size;
        let mut all_embeddings = Vec::new();

        for chunk in texts.chunks(batch_size) {
            let batch_embeddings = self.process_batch(chunk).await?;
            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn provider_name(&self) -> &str {
        "local"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    async fn health_check(&self) -> Result<bool> {
        // 检查模型路径是否仍然存在
        if !Path::new(&self.model_path).exists() {
            return Ok(false);
        }

        // 尝试生成一个测试嵌入
        match self.embed("health check").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_embedder_creation_missing_path() {
        let config = EmbeddingConfig::local("/nonexistent/path", 384);
        let result = LocalEmbedder::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_embedder_creation_with_valid_path() {
        // 创建临时文件作为模型路径
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 384);
        let result = LocalEmbedder::new(config).await;
        assert!(result.is_ok());

        let embedder = result.unwrap();
        assert_eq!(embedder.provider_name(), "local");
        assert_eq!(embedder.model_name(), "local");
        assert_eq!(embedder.dimension(), 384);
    }

    #[tokio::test]
    async fn test_embed_single_text() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 384);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let result = embedder.embed("test text").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 256);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let texts = vec![
            "first text".to_string(),
            "second text".to_string(),
            "third text".to_string(),
        ];

        let result = embedder.embed_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 256);
        assert_eq!(embeddings[1].len(), 256);
        assert_eq!(embeddings[2].len(), 256);
    }

    #[tokio::test]
    async fn test_embed_empty_batch() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 128);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let result = embedder.embed_batch(&[]).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 128);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let result = embedder.health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_health_check_missing_model() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 128);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        // 删除模型文件
        std::fs::remove_file(&model_path).unwrap();

        let result = embedder.health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}
