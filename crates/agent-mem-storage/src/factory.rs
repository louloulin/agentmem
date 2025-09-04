//! 存储工厂模式实现

use crate::backends::{MemoryVectorStore, ChromaStore};
use agent_mem_traits::{VectorStore, VectorStoreConfig, Result, AgentMemError};
use async_trait::async_trait;
use std::sync::Arc;

/// 存储提供商枚举，包装不同的存储实现
pub enum VectorStoreEnum {
    Memory(MemoryVectorStore),
    Chroma(ChromaStore),
}

#[async_trait]
impl VectorStore for VectorStoreEnum {
    async fn add_vectors(&self, vectors: Vec<agent_mem_traits::VectorData>) -> Result<Vec<String>> {
        match self {
            VectorStoreEnum::Memory(store) => store.add_vectors(vectors).await,
            VectorStoreEnum::Chroma(store) => store.add_vectors(vectors).await,
        }
    }

    async fn search_vectors(&self, query_vector: Vec<f32>, limit: usize, threshold: Option<f32>) -> Result<Vec<agent_mem_traits::VectorSearchResult>> {
        match self {
            VectorStoreEnum::Memory(store) => store.search_vectors(query_vector, limit, threshold).await,
            VectorStoreEnum::Chroma(store) => store.search_vectors(query_vector, limit, threshold).await,
        }
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        match self {
            VectorStoreEnum::Memory(store) => store.delete_vectors(ids).await,
            VectorStoreEnum::Chroma(store) => store.delete_vectors(ids).await,
        }
    }

    async fn update_vectors(&self, vectors: Vec<agent_mem_traits::VectorData>) -> Result<()> {
        match self {
            VectorStoreEnum::Memory(store) => store.update_vectors(vectors).await,
            VectorStoreEnum::Chroma(store) => store.update_vectors(vectors).await,
        }
    }

    async fn get_vector(&self, id: &str) -> Result<Option<agent_mem_traits::VectorData>> {
        match self {
            VectorStoreEnum::Memory(store) => store.get_vector(id).await,
            VectorStoreEnum::Chroma(store) => store.get_vector(id).await,
        }
    }

    async fn count_vectors(&self) -> Result<usize> {
        match self {
            VectorStoreEnum::Memory(store) => store.count_vectors().await,
            VectorStoreEnum::Chroma(store) => store.count_vectors().await,
        }
    }

    async fn clear(&self) -> Result<()> {
        match self {
            VectorStoreEnum::Memory(store) => store.clear().await,
            VectorStoreEnum::Chroma(store) => store.clear().await,
        }
    }
}

/// 存储工厂，用于创建不同的存储后端实例
pub struct StorageFactory;

impl StorageFactory {
    /// 根据配置创建向量存储实例
    pub async fn create_vector_store(config: &VectorStoreConfig) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let store_enum = match config.provider.as_str() {
            "memory" => {
                let store = MemoryVectorStore::new(config.clone()).await?;
                VectorStoreEnum::Memory(store)
            }
            "chroma" => {
                let store = ChromaStore::new(config.clone()).await?;
                VectorStoreEnum::Chroma(store)
            }
            _ => return Err(AgentMemError::unsupported_provider(&config.provider)),
        };

        Ok(Arc::new(store_enum))
    }

    /// 获取支持的存储提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        vec!["memory", "chroma"]
    }

    /// 检查提供商是否受支持
    pub fn is_provider_supported(provider: &str) -> bool {
        Self::supported_providers().contains(&provider)
    }

    /// 创建默认的内存存储
    pub async fn create_memory_store() -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "memory".to_string(),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }

    /// 创建Chroma存储
    pub async fn create_chroma_store(url: &str, collection_name: &str) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "chroma".to_string(),
            url: Some(url.to_string()),
            collection_name: Some(collection_name.to_string()),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_providers() {
        let providers = StorageFactory::supported_providers();
        assert_eq!(providers, vec!["memory", "chroma"]);
    }

    #[test]
    fn test_is_provider_supported() {
        assert!(StorageFactory::is_provider_supported("memory"));
        assert!(StorageFactory::is_provider_supported("chroma"));
        assert!(!StorageFactory::is_provider_supported("unsupported_provider"));
    }

    #[test]
    fn test_create_vector_store_unsupported() {
        let config = VectorStoreConfig {
            provider: "unsupported".to_string(),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(StorageFactory::create_vector_store(&config));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_memory_store() {
        let result = StorageFactory::create_memory_store().await;
        assert!(result.is_ok());
    }
}