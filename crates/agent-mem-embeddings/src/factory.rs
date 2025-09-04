//! 嵌入模型工厂模式实现

use crate::config::EmbeddingConfig;
use crate::providers::{OpenAIEmbedder, HuggingFaceEmbedder, LocalEmbedder, AnthropicEmbedder, CohereEmbedder};
use agent_mem_traits::{Embedder, Result, AgentMemError};
use async_trait::async_trait;
use std::sync::Arc;

/// 嵌入提供商枚举，包装不同的嵌入实现
pub enum EmbedderEnum {
    OpenAI(OpenAIEmbedder),
    #[cfg(feature = "huggingface")]
    HuggingFace(HuggingFaceEmbedder),
    #[cfg(feature = "local")]
    Local(LocalEmbedder),
}

#[async_trait]
impl Embedder for EmbedderEnum {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.embed(text).await,
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.embed(text).await,
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.embed(text).await,
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.embed_batch(texts).await,
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.embed_batch(texts).await,
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.embed_batch(texts).await,
        }
    }

    fn dimension(&self) -> usize {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.dimension(),
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.dimension(),
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.dimension(),
        }
    }

    fn provider_name(&self) -> &str {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.provider_name(),
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.provider_name(),
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.provider_name(),
        }
    }

    fn model_name(&self) -> &str {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.model_name(),
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.model_name(),
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.model_name(),
        }
    }

    async fn health_check(&self) -> Result<bool> {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.health_check().await,
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.health_check().await,
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.health_check().await,
        }
    }
}

/// 嵌入工厂，用于创建不同的嵌入提供商实例
pub struct EmbeddingFactory;

impl EmbeddingFactory {
    /// 根据配置创建嵌入器实例
    pub async fn create_embedder(config: &EmbeddingConfig) -> Result<Arc<dyn Embedder + Send + Sync>> {
        // 验证配置
        config.validate()?;

        let embedder_enum = match config.provider.as_str() {
            "openai" => {
                let embedder = OpenAIEmbedder::new(config.clone()).await?;
                EmbedderEnum::OpenAI(embedder)
            }
            "huggingface" => {
                #[cfg(feature = "huggingface")]
                {
                    let embedder = HuggingFaceEmbedder::new(config.clone()).await?;
                    EmbedderEnum::HuggingFace(embedder)
                }
                #[cfg(not(feature = "huggingface"))]
                {
                    return Err(AgentMemError::unsupported_provider("HuggingFace feature not enabled"));
                }
            }
            "local" => {
                #[cfg(feature = "local")]
                {
                    let embedder = LocalEmbedder::new(config.clone()).await?;
                    EmbedderEnum::Local(embedder)
                }
                #[cfg(not(feature = "local"))]
                {
                    return Err(AgentMemError::unsupported_provider("Local feature not enabled"));
                }
            }
            _ => return Err(AgentMemError::unsupported_provider(&config.provider)),
        };

        Ok(Arc::new(embedder_enum))
    }

    /// 获取支持的嵌入提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        let mut providers = Vec::new();
        
        providers.push("openai");
        
        #[cfg(feature = "huggingface")]
        providers.push("huggingface");
        
        #[cfg(feature = "local")]
        providers.push("local");
        
        providers
    }

    /// 检查提供商是否受支持
    pub fn is_provider_supported(provider: &str) -> bool {
        Self::supported_providers().contains(&provider)
    }

    /// 创建默认的OpenAI嵌入器
    pub async fn create_openai_embedder(api_key: String) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::openai(Some(api_key));
        Self::create_embedder(&config).await
    }

    /// 创建OpenAI 3-small嵌入器
    pub async fn create_openai_3_small(api_key: String) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::openai_3_small(Some(api_key));
        Self::create_embedder(&config).await
    }

    /// 创建OpenAI 3-large嵌入器
    pub async fn create_openai_3_large(api_key: String) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::openai_3_large(Some(api_key));
        Self::create_embedder(&config).await
    }

    /// 创建HuggingFace嵌入器
    #[cfg(feature = "huggingface")]
    pub async fn create_huggingface_embedder(model: &str) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::huggingface(model);
        Self::create_embedder(&config).await
    }

    /// 创建本地嵌入器
    #[cfg(feature = "local")]
    pub async fn create_local_embedder(model_path: &str, dimension: usize) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::local(model_path, dimension);
        Self::create_embedder(&config).await
    }

    /// 从环境变量创建嵌入器
    pub async fn from_env() -> Result<Arc<dyn Embedder + Send + Sync>> {
        let provider = std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string());
        
        match provider.as_str() {
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY")
                    .map_err(|_| AgentMemError::config_error("OPENAI_API_KEY environment variable not set"))?;
                Self::create_openai_embedder(api_key).await
            }
            "huggingface" => {
                #[cfg(feature = "huggingface")]
                {
                    let model = std::env::var("HUGGINGFACE_MODEL")
                        .unwrap_or_else(|_| "sentence-transformers/all-MiniLM-L6-v2".to_string());
                    Self::create_huggingface_embedder(&model).await
                }
                #[cfg(not(feature = "huggingface"))]
                {
                    Err(AgentMemError::unsupported_provider("HuggingFace feature not enabled"))
                }
            }
            "local" => {
                #[cfg(feature = "local")]
                {
                    let model_path = std::env::var("LOCAL_MODEL_PATH")
                        .map_err(|_| AgentMemError::config_error("LOCAL_MODEL_PATH environment variable not set"))?;
                    let dimension = std::env::var("LOCAL_MODEL_DIMENSION")
                        .unwrap_or_else(|_| "768".to_string())
                        .parse::<usize>()
                        .map_err(|_| AgentMemError::config_error("Invalid LOCAL_MODEL_DIMENSION"))?;
                    Self::create_local_embedder(&model_path, dimension).await
                }
                #[cfg(not(feature = "local"))]
                {
                    Err(AgentMemError::unsupported_provider("Local feature not enabled"))
                }
            }
            _ => Err(AgentMemError::unsupported_provider(&provider)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_providers() {
        let providers = EmbeddingFactory::supported_providers();
        assert!(!providers.is_empty());
        assert!(providers.contains(&"openai"));
    }

    #[test]
    fn test_is_provider_supported() {
        assert!(EmbeddingFactory::is_provider_supported("openai"));
        assert!(!EmbeddingFactory::is_provider_supported("unsupported_provider"));
    }

    #[test]
    fn test_create_embedder_unsupported() {
        let config = EmbeddingConfig {
            provider: "unsupported".to_string(),
            ..Default::default()
        };
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(EmbeddingFactory::create_embedder(&config));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_openai_embedder_no_key() {
        let config = EmbeddingConfig::openai(None);
        let result = EmbeddingFactory::create_embedder(&config).await;
        assert!(result.is_err());
    }
}
