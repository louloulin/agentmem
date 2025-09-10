//! Memory configuration

use agent_mem_traits::{LLMConfig, VectorStoreConfig};
use serde::{Deserialize, Serialize};

/// Main configuration for memory management
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    /// LLM provider configuration
    pub llm: LLMConfig,

    /// Vector store configuration
    pub vector_store: VectorStoreConfig,

    /// Graph store configuration (optional)
    pub graph_store: Option<GraphStoreConfig>,

    /// Embedder configuration
    pub embedder: EmbedderConfig,

    /// Session configuration
    pub session: SessionConfig,

    /// Intelligence configuration
    pub intelligence: IntelligenceConfig,

    /// Performance configuration
    pub performance: PerformanceConfig,
}



/// Graph store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoreConfig {
    pub provider: String,
    pub uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

impl Default for GraphStoreConfig {
    fn default() -> Self {
        Self {
            provider: "neo4j".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: None,
            password: None,
            database: Some("neo4j".to_string()),
        }
    }
}

/// Embedder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub dimension: usize,
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-ada-002".to_string(),
            api_key: None,
            base_url: None,
            dimension: 1536,
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub default_user_id: Option<String>,
    pub default_agent_id: Option<String>,
    pub session_timeout_seconds: u64,
    pub enable_telemetry: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_user_id: None,
            default_agent_id: None,
            session_timeout_seconds: 3600, // 1 hour
            enable_telemetry: true,
        }
    }
}

/// Intelligence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub similarity_threshold: f32,
    pub clustering_threshold: f32,
    pub enable_conflict_detection: bool,
    pub enable_memory_summarization: bool,
    pub importance_scoring: bool,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.8,
            clustering_threshold: 0.7,
            enable_conflict_detection: true,
            enable_memory_summarization: true,
            importance_scoring: true,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of retry attempts for failed operations
    pub retry_attempts: Option<u32>,
    /// Base delay in milliseconds for exponential backoff
    pub base_delay_ms: Option<u64>,
    /// Maximum delay in milliseconds for exponential backoff
    pub max_delay_ms: Option<u64>,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: Option<usize>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            retry_attempts: Some(3),
            base_delay_ms: Some(100),
            max_delay_ms: Some(5000),
            max_concurrent_operations: Some(10),
        }
    }
}
