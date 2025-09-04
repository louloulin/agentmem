//! Storage configuration extensions

use serde::{Deserialize, Serialize};
use agent_mem_traits::VectorStoreConfig as BaseVectorStoreConfig;

/// Extended storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Vector store configuration
    pub vector_store: VectorStoreConfig,
    
    /// Graph store configuration (optional)
    pub graph_store: Option<GraphStoreConfig>,
    
    /// Key-value store configuration
    pub kv_store: KeyValueStoreConfig,
    
    /// History store configuration
    pub history_store: HistoryStoreConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            vector_store: VectorStoreConfig::default(),
            graph_store: None,
            kv_store: KeyValueStoreConfig::default(),
            history_store: HistoryStoreConfig::default(),
        }
    }
}

/// Extended vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    #[serde(flatten)]
    pub base: BaseVectorStoreConfig,
    
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    
    /// Maximum connections in pool
    pub max_connections: u32,
    
    /// Enable connection pooling
    pub enable_pooling: bool,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            base: BaseVectorStoreConfig::default(),
            timeout_seconds: 30,
            max_connections: 10,
            enable_pooling: true,
            batch_size: 100,
        }
    }
}

impl VectorStoreConfig {
    pub fn lancedb() -> Self {
        Self {
            base: BaseVectorStoreConfig {
                provider: "lancedb".to_string(),
                path: "./data/vectors".to_string(),
                table_name: "memories".to_string(),
                dimension: 1536,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    pub fn pinecone(index_name: &str) -> Self {
        Self {
            base: BaseVectorStoreConfig {
                provider: "pinecone".to_string(),
                index_name: Some(index_name.to_string()),
                dimension: 1536,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    pub fn qdrant() -> Self {
        Self {
            base: BaseVectorStoreConfig {
                provider: "qdrant".to_string(),
                path: "http://localhost:6333".to_string(),
                table_name: "memories".to_string(),
                dimension: 1536,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Graph store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoreConfig {
    pub provider: String,
    pub uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub timeout_seconds: u64,
    pub max_connections: u32,
}

impl Default for GraphStoreConfig {
    fn default() -> Self {
        Self {
            provider: "neo4j".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: None,
            password: None,
            database: Some("neo4j".to_string()),
            timeout_seconds: 30,
            max_connections: 10,
        }
    }
}

/// Key-value store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueStoreConfig {
    pub provider: String,
    pub uri: String,
    pub password: Option<String>,
    pub database: u32,
    pub timeout_seconds: u64,
    pub max_connections: u32,
}

impl Default for KeyValueStoreConfig {
    fn default() -> Self {
        Self {
            provider: "redis".to_string(),
            uri: "redis://localhost:6379".to_string(),
            password: None,
            database: 0,
            timeout_seconds: 30,
            max_connections: 10,
        }
    }
}

/// History store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStoreConfig {
    pub provider: String,
    pub path: String,
    pub max_entries_per_memory: usize,
    pub cleanup_interval_hours: u64,
}

impl Default for HistoryStoreConfig {
    fn default() -> Self {
        Self {
            provider: "sqlite".to_string(),
            path: "./data/history.db".to_string(),
            max_entries_per_memory: 100,
            cleanup_interval_hours: 24,
        }
    }
}
