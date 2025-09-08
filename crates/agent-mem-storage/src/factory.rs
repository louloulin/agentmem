//! 存储工厂模式实现

use crate::backends::{
    ChromaStore, ElasticsearchStore, FaissStore, LanceDBStore, MemoryVectorStore,
    MilvusStore, MongoDBStore, PineconeStore, QdrantStore, WeaviateStore
};
use agent_mem_traits::{AgentMemError, Result, VectorStore, VectorStoreConfig};
use async_trait::async_trait;
use std::sync::Arc;

/// 存储提供商枚举，包装不同的存储实现
pub enum VectorStoreEnum {
    Memory(MemoryVectorStore),
    Chroma(ChromaStore),
    #[cfg(feature = "faiss")]
    Faiss(FaissStore),
    #[cfg(feature = "mongodb")]
    MongoDB(MongoDBStore),
    #[cfg(feature = "qdrant")]
    Qdrant(QdrantStore),
    #[cfg(feature = "pinecone")]
    Pinecone(PineconeStore),
    #[cfg(feature = "elasticsearch")]
    Elasticsearch(ElasticsearchStore),
    #[cfg(feature = "lancedb")]
    LanceDB(LanceDBStore),
    #[cfg(feature = "milvus")]
    Milvus(MilvusStore),
    #[cfg(feature = "weaviate")]
    Weaviate(WeaviateStore),
}

#[async_trait]
impl VectorStore for VectorStoreEnum {
    async fn add_vectors(&self, vectors: Vec<agent_mem_traits::VectorData>) -> Result<Vec<String>> {
        match self {
            VectorStoreEnum::Memory(store) => store.add_vectors(vectors).await,
            VectorStoreEnum::Chroma(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "faiss")]
            VectorStoreEnum::Faiss(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "mongodb")]
            VectorStoreEnum::MongoDB(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "qdrant")]
            VectorStoreEnum::Qdrant(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "pinecone")]
            VectorStoreEnum::Pinecone(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "elasticsearch")]
            VectorStoreEnum::Elasticsearch(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "lancedb")]
            VectorStoreEnum::LanceDB(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "milvus")]
            VectorStoreEnum::Milvus(store) => store.add_vectors(vectors).await,
            #[cfg(feature = "weaviate")]
            VectorStoreEnum::Weaviate(store) => store.add_vectors(vectors).await,
        }
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<agent_mem_traits::VectorSearchResult>> {
        match self {
            VectorStoreEnum::Memory(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            VectorStoreEnum::Chroma(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "faiss")]
            VectorStoreEnum::Faiss(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "mongodb")]
            VectorStoreEnum::MongoDB(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "qdrant")]
            VectorStoreEnum::Qdrant(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "pinecone")]
            VectorStoreEnum::Pinecone(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "elasticsearch")]
            VectorStoreEnum::Elasticsearch(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "lancedb")]
            VectorStoreEnum::LanceDB(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "milvus")]
            VectorStoreEnum::Milvus(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
            #[cfg(feature = "weaviate")]
            VectorStoreEnum::Weaviate(store) => {
                store.search_vectors(query_vector, limit, threshold).await
            }
        }
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        match self {
            VectorStoreEnum::Memory(store) => store.delete_vectors(ids).await,
            VectorStoreEnum::Chroma(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "faiss")]
            VectorStoreEnum::Faiss(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "mongodb")]
            VectorStoreEnum::MongoDB(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "qdrant")]
            VectorStoreEnum::Qdrant(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "pinecone")]
            VectorStoreEnum::Pinecone(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "elasticsearch")]
            VectorStoreEnum::Elasticsearch(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "lancedb")]
            VectorStoreEnum::LanceDB(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "milvus")]
            VectorStoreEnum::Milvus(store) => store.delete_vectors(ids).await,
            #[cfg(feature = "weaviate")]
            VectorStoreEnum::Weaviate(store) => store.delete_vectors(ids).await,
        }
    }

    async fn update_vectors(&self, vectors: Vec<agent_mem_traits::VectorData>) -> Result<()> {
        match self {
            VectorStoreEnum::Memory(store) => store.update_vectors(vectors).await,
            VectorStoreEnum::Chroma(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "faiss")]
            VectorStoreEnum::Faiss(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "mongodb")]
            VectorStoreEnum::MongoDB(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "qdrant")]
            VectorStoreEnum::Qdrant(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "pinecone")]
            VectorStoreEnum::Pinecone(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "elasticsearch")]
            VectorStoreEnum::Elasticsearch(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "lancedb")]
            VectorStoreEnum::LanceDB(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "milvus")]
            VectorStoreEnum::Milvus(store) => store.update_vectors(vectors).await,
            #[cfg(feature = "weaviate")]
            VectorStoreEnum::Weaviate(store) => store.update_vectors(vectors).await,
        }
    }

    async fn get_vector(&self, id: &str) -> Result<Option<agent_mem_traits::VectorData>> {
        match self {
            VectorStoreEnum::Memory(store) => store.get_vector(id).await,
            VectorStoreEnum::Chroma(store) => store.get_vector(id).await,
            #[cfg(feature = "faiss")]
            VectorStoreEnum::Faiss(store) => store.get_vector(id).await,
            #[cfg(feature = "mongodb")]
            VectorStoreEnum::MongoDB(store) => store.get_vector(id).await,
            #[cfg(feature = "qdrant")]
            VectorStoreEnum::Qdrant(store) => store.get_vector(id).await,
            #[cfg(feature = "pinecone")]
            VectorStoreEnum::Pinecone(store) => store.get_vector(id).await,
            #[cfg(feature = "elasticsearch")]
            VectorStoreEnum::Elasticsearch(store) => store.get_vector(id).await,
            #[cfg(feature = "lancedb")]
            VectorStoreEnum::LanceDB(store) => store.get_vector(id).await,
            #[cfg(feature = "milvus")]
            VectorStoreEnum::Milvus(store) => store.get_vector(id).await,
            #[cfg(feature = "weaviate")]
            VectorStoreEnum::Weaviate(store) => store.get_vector(id).await,
        }
    }

    async fn count_vectors(&self) -> Result<usize> {
        match self {
            VectorStoreEnum::Memory(store) => store.count_vectors().await,
            VectorStoreEnum::Chroma(store) => store.count_vectors().await,
            #[cfg(feature = "faiss")]
            VectorStoreEnum::Faiss(store) => store.count_vectors().await,
            #[cfg(feature = "mongodb")]
            VectorStoreEnum::MongoDB(store) => store.count_vectors().await,
            #[cfg(feature = "qdrant")]
            VectorStoreEnum::Qdrant(store) => store.count_vectors().await,
            #[cfg(feature = "pinecone")]
            VectorStoreEnum::Pinecone(store) => store.count_vectors().await,
            #[cfg(feature = "elasticsearch")]
            VectorStoreEnum::Elasticsearch(store) => store.count_vectors().await,
            #[cfg(feature = "lancedb")]
            VectorStoreEnum::LanceDB(store) => store.count_vectors().await,
            #[cfg(feature = "milvus")]
            VectorStoreEnum::Milvus(store) => store.count_vectors().await,
            #[cfg(feature = "weaviate")]
            VectorStoreEnum::Weaviate(store) => store.count_vectors().await,
        }
    }

    async fn clear(&self) -> Result<()> {
        match self {
            VectorStoreEnum::Memory(store) => store.clear().await,
            VectorStoreEnum::Chroma(store) => store.clear().await,
            #[cfg(feature = "faiss")]
            VectorStoreEnum::Faiss(store) => store.clear().await,
            #[cfg(feature = "mongodb")]
            VectorStoreEnum::MongoDB(store) => store.clear().await,
            #[cfg(feature = "qdrant")]
            VectorStoreEnum::Qdrant(store) => store.clear().await,
            #[cfg(feature = "pinecone")]
            VectorStoreEnum::Pinecone(store) => store.clear().await,
            #[cfg(feature = "elasticsearch")]
            VectorStoreEnum::Elasticsearch(store) => store.clear().await,
            #[cfg(feature = "lancedb")]
            VectorStoreEnum::LanceDB(store) => store.clear().await,
            #[cfg(feature = "milvus")]
            VectorStoreEnum::Milvus(store) => store.clear().await,
            #[cfg(feature = "weaviate")]
            VectorStoreEnum::Weaviate(store) => store.clear().await,
        }
    }
}

/// 存储工厂，用于创建不同的存储后端实例
pub struct StorageFactory;

impl StorageFactory {
    /// 根据配置创建向量存储实例
    pub async fn create_vector_store(
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let store_enum = match config.provider.as_str() {
            "memory" => {
                let store = MemoryVectorStore::new(config.clone()).await?;
                VectorStoreEnum::Memory(store)
            }
            "chroma" => {
                let store = ChromaStore::new(config.clone()).await?;
                VectorStoreEnum::Chroma(store)
            }
            "faiss" => {
                #[cfg(feature = "faiss")]
                {
                    use crate::backends::faiss::FaissConfig;
                    let faiss_config = FaissConfig {
                        dimension: config.dimension.unwrap_or(1536),
                        ..Default::default()
                    };
                    let store = FaissStore::new(faiss_config).await?;
                    VectorStoreEnum::Faiss(store)
                }
                #[cfg(not(feature = "faiss"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "FAISS feature not enabled",
                    ));
                }
            }
            "mongodb" => {
                #[cfg(feature = "mongodb")]
                {
                    use crate::backends::mongodb::MongoDBConfig;
                    let mongodb_config = MongoDBConfig {
                        connection_string: config.url.clone().unwrap_or_else(|| "mongodb://localhost:27017".to_string()),
                        database_name: config.table_name.clone(),
                        collection_name: config.collection_name.clone().unwrap_or_else(|| "vectors".to_string()),
                        ..Default::default()
                    };
                    let store = MongoDBStore::new(mongodb_config).await?;
                    VectorStoreEnum::MongoDB(store)
                }
                #[cfg(not(feature = "mongodb"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "MongoDB feature not enabled",
                    ));
                }
            }
            "qdrant" => {
                #[cfg(feature = "qdrant")]
                {
                    let store = QdrantStore::new(config.clone()).await?;
                    VectorStoreEnum::Qdrant(store)
                }
                #[cfg(not(feature = "qdrant"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Qdrant feature not enabled",
                    ));
                }
            }
            "pinecone" => {
                #[cfg(feature = "pinecone")]
                {
                    let store = PineconeStore::new(config.clone()).await?;
                    VectorStoreEnum::Pinecone(store)
                }
                #[cfg(not(feature = "pinecone"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Pinecone feature not enabled",
                    ));
                }
            }
            "elasticsearch" => {
                #[cfg(feature = "elasticsearch")]
                {
                    let store = ElasticsearchStore::new(config.clone()).await?;
                    VectorStoreEnum::Elasticsearch(store)
                }
                #[cfg(not(feature = "elasticsearch"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Elasticsearch feature not enabled",
                    ));
                }
            }
            "lancedb" => {
                #[cfg(feature = "lancedb")]
                {
                    let store = LanceDBStore::new(config.clone()).await?;
                    VectorStoreEnum::LanceDB(store)
                }
                #[cfg(not(feature = "lancedb"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "LanceDB feature not enabled",
                    ));
                }
            }
            "milvus" => {
                #[cfg(feature = "milvus")]
                {
                    let store = MilvusStore::new(config.clone()).await?;
                    VectorStoreEnum::Milvus(store)
                }
                #[cfg(not(feature = "milvus"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Milvus feature not enabled",
                    ));
                }
            }
            "weaviate" => {
                #[cfg(feature = "weaviate")]
                {
                    let store = WeaviateStore::new(config.clone()).await?;
                    VectorStoreEnum::Weaviate(store)
                }
                #[cfg(not(feature = "weaviate"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Weaviate feature not enabled",
                    ));
                }
            }
            _ => return Err(AgentMemError::unsupported_provider(&config.provider)),
        };

        Ok(Arc::new(store_enum))
    }

    /// 获取支持的存储提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        #[allow(unused_mut)]
        let mut providers = vec!["memory", "chroma"];

        #[cfg(feature = "qdrant")]
        providers.push("qdrant");

        #[cfg(feature = "pinecone")]
        providers.push("pinecone");

        #[cfg(feature = "elasticsearch")]
        providers.push("elasticsearch");

        #[cfg(feature = "lancedb")]
        providers.push("lancedb");

        #[cfg(feature = "milvus")]
        providers.push("milvus");

        #[cfg(feature = "weaviate")]
        providers.push("weaviate");

        providers
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
    pub async fn create_chroma_store(
        url: &str,
        collection_name: &str,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "chroma".to_string(),
            url: Some(url.to_string()),
            collection_name: Some(collection_name.to_string()),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }

    /// 创建Qdrant存储
    #[cfg(feature = "qdrant")]
    pub async fn create_qdrant_store(
        url: &str,
        collection_name: &str,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "qdrant".to_string(),
            url: Some(url.to_string()),
            collection_name: Some(collection_name.to_string()),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }

    /// 创建Pinecone存储
    #[cfg(feature = "pinecone")]
    pub async fn create_pinecone_store(
        api_key: &str,
        index_name: &str,
        environment: &str,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "pinecone".to_string(),
            api_key: Some(api_key.to_string()),
            index_name: Some(index_name.to_string()),
            url: Some(format!(
                "https://{}-{}.svc.{}.pinecone.io",
                index_name, "default", environment
            )),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }

    /// 创建Elasticsearch存储
    #[cfg(feature = "elasticsearch")]
    pub async fn create_elasticsearch_store(
        url: &str,
        index_name: &str,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "elasticsearch".to_string(),
            url: Some(url.to_string()),
            index_name: Some(index_name.to_string()),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }

    /// 创建LanceDB存储
    #[cfg(feature = "lancedb")]
    pub async fn create_lancedb_store(
        url: &str,
        table_name: &str,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "lancedb".to_string(),
            url: Some(url.to_string()),
            collection_name: Some(table_name.to_string()),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }

    /// 创建Milvus存储
    #[cfg(feature = "milvus")]
    pub async fn create_milvus_store(
        url: &str,
        collection_name: &str,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "milvus".to_string(),
            url: Some(url.to_string()),
            collection_name: Some(collection_name.to_string()),
            ..Default::default()
        };
        Self::create_vector_store(&config).await
    }

    /// 创建Weaviate存储
    #[cfg(feature = "weaviate")]
    pub async fn create_weaviate_store(
        url: &str,
        class_name: &str,
    ) -> Result<Arc<dyn VectorStore + Send + Sync>> {
        let config = VectorStoreConfig {
            provider: "weaviate".to_string(),
            url: Some(url.to_string()),
            collection_name: Some(class_name.to_string()),
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
        assert!(providers.contains(&"memory"));
        assert!(providers.contains(&"chroma"));

        #[cfg(feature = "qdrant")]
        assert!(providers.contains(&"qdrant"));

        #[cfg(feature = "pinecone")]
        assert!(providers.contains(&"pinecone"));

        #[cfg(feature = "elasticsearch")]
        assert!(providers.contains(&"elasticsearch"));

        #[cfg(feature = "lancedb")]
        assert!(providers.contains(&"lancedb"));

        #[cfg(feature = "milvus")]
        assert!(providers.contains(&"milvus"));

        #[cfg(feature = "weaviate")]
        assert!(providers.contains(&"weaviate"));
    }

    #[test]
    fn test_is_provider_supported() {
        assert!(StorageFactory::is_provider_supported("memory"));
        assert!(StorageFactory::is_provider_supported("chroma"));

        #[cfg(feature = "qdrant")]
        assert!(StorageFactory::is_provider_supported("qdrant"));

        #[cfg(feature = "pinecone")]
        assert!(StorageFactory::is_provider_supported("pinecone"));

        #[cfg(feature = "elasticsearch")]
        assert!(StorageFactory::is_provider_supported("elasticsearch"));

        #[cfg(feature = "lancedb")]
        assert!(StorageFactory::is_provider_supported("lancedb"));

        #[cfg(feature = "milvus")]
        assert!(StorageFactory::is_provider_supported("milvus"));

        #[cfg(feature = "weaviate")]
        assert!(StorageFactory::is_provider_supported("weaviate"));

        assert!(!StorageFactory::is_provider_supported(
            "unsupported_provider"
        ));
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

    #[tokio::test]
    async fn test_create_chroma_store() {
        // 只有在设置了环境变量时才运行真实的连接测试
        if std::env::var("CHROMA_TEST_ENABLED").is_ok() {
            let result = StorageFactory::create_chroma_store("http://localhost:8000", "test").await;
            assert!(result.is_ok());
        } else {
            // 模拟测试，验证配置创建
            let config = VectorStoreConfig {
                provider: "chroma".to_string(),
                url: Some("http://localhost:8000".to_string()),
                collection_name: Some("test".to_string()),
                ..Default::default()
            };

            // 验证配置正确
            assert_eq!(config.provider, "chroma");
            assert_eq!(config.url, Some("http://localhost:8000".to_string()));
            assert_eq!(config.collection_name, Some("test".to_string()));
        }
    }

    #[cfg(feature = "qdrant")]
    #[tokio::test]
    async fn test_create_qdrant_store() {
        let result = StorageFactory::create_qdrant_store("http://localhost:6333", "test").await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "pinecone")]
    #[tokio::test]
    async fn test_create_pinecone_store() {
        let result =
            StorageFactory::create_pinecone_store("test-key", "test-index", "us-east1-gcp").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_qdrant_config() {
        let config = VectorStoreConfig {
            provider: "qdrant".to_string(),
            url: Some("http://localhost:6333".to_string()),
            collection_name: Some("test".to_string()),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();

        #[cfg(feature = "qdrant")]
        {
            let result = rt.block_on(StorageFactory::create_vector_store(&config));
            assert!(result.is_ok());
        }

        #[cfg(not(feature = "qdrant"))]
        {
            let result = rt.block_on(StorageFactory::create_vector_store(&config));
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_create_pinecone_config() {
        let config = VectorStoreConfig {
            provider: "pinecone".to_string(),
            api_key: Some("test-key".to_string()),
            index_name: Some("test-index".to_string()),
            url: Some("https://test.pinecone.io".to_string()),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();

        #[cfg(feature = "pinecone")]
        {
            let result = rt.block_on(StorageFactory::create_vector_store(&config));
            assert!(result.is_ok());
        }

        #[cfg(not(feature = "pinecone"))]
        {
            let result = rt.block_on(StorageFactory::create_vector_store(&config));
            assert!(result.is_err());
        }
    }
}
