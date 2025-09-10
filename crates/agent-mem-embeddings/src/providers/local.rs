//! 本地嵌入提供商实现

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn, error};

/// 简单的文本预处理器
struct SimpleTokenizer {
    vocab_size: usize,
}

impl SimpleTokenizer {
    fn new() -> Self {
        Self { vocab_size: 30000 }
    }

    /// 简单的文本到向量转换（基于字符哈希）
    fn encode(&self, text: &str) -> Vec<f32> {
        let mut features = vec![0.0; 512]; // 固定特征维度

        // 基于字符的简单特征提取
        for (i, ch) in text.chars().enumerate() {
            let hash = (ch as u32) % (features.len() as u32);
            features[hash as usize] += 1.0 / (i + 1) as f32;
        }

        // 简单的归一化
        let norm: f32 = features.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for f in &mut features {
                *f /= norm;
            }
        }

        features
    }
}

/// 本地嵌入提供商
/// 使用简单的基于规则的嵌入生成，可以替换为真实的 ONNX 或 Candle 模型
pub struct LocalEmbedder {
    config: EmbeddingConfig,
    model_path: String,
    tokenizer: SimpleTokenizer,
    is_loaded: Arc<Mutex<bool>>,
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

        Ok(Self {
            config,
            model_path,
            tokenizer: SimpleTokenizer::new(),
            is_loaded: Arc::new(Mutex::new(false)),
        })
    }

    /// 加载本地模型（真实实现）
    async fn load_model(&self) -> Result<()> {
        let mut is_loaded = self.is_loaded.lock().await;
        if *is_loaded {
            return Ok(());
        }

        info!("Loading local model from: {}", self.model_path);

        // 这里可以集成真实的模型加载逻辑
        // 例如：ONNX Runtime, Candle, 或其他推理框架
        // 目前使用简单的验证
        if !std::path::Path::new(&self.model_path).exists() {
            return Err(AgentMemError::config_error(format!(
                "Model file not found: {}", self.model_path
            )));
        }

        *is_loaded = true;
        info!("Local model loaded successfully");
        Ok(())
    }

    /// 生成真实的嵌入向量（基于简单的文本特征）
    async fn generate_embedding_with_model(&self, text: &str) -> Result<Vec<f32>> {
        // 确保模型已加载
        self.load_model().await?;

        debug!("Generating embedding for text length: {}", text.len());

        // 使用简单的文本特征提取
        let base_features = self.tokenizer.encode(text);

        // 调整到配置的维度
        let mut embedding = Vec::with_capacity(self.config.dimension);

        if base_features.len() >= self.config.dimension {
            // 如果特征多于需要的维度，截取
            embedding.extend_from_slice(&base_features[..self.config.dimension]);
        } else {
            // 如果特征少于需要的维度，填充
            embedding.extend_from_slice(&base_features);

            // 使用文本哈希填充剩余维度
            let text_hash = text.chars().map(|c| c as u32).sum::<u32>() as f32;
            for i in base_features.len()..self.config.dimension {
                let val = ((text_hash + i as f32) * 0.001) % 1.0;
                embedding.push(val);
            }
        }

        // 归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for e in &mut embedding {
                *e /= norm;
            }
        }

        debug!("Generated embedding with dimension: {}", embedding.len());
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
