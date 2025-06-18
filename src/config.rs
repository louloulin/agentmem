// 配置管理模块
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::types::{AgentDbError, VectorIndexType};

// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDbConfig {
    pub database: DatabaseConfig,
    pub vector: VectorConfig,
    pub memory: MemoryConfig,
    pub rag: RAGConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}

// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub retry_attempts: u32,
    pub backup_enabled: bool,
    pub backup_interval_hours: u64,
}

// 向量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    pub dimension: usize,
    pub index_type: VectorIndexType,
    pub similarity_threshold: f32,
    pub max_vectors_per_batch: usize,
    pub hnsw_config: HNSWConfig,
    pub ivf_config: IVFConfig,
}

// HNSW配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HNSWConfig {
    pub max_level: usize,
    pub max_connections: usize,
    pub ef_construction: usize,
    pub ml: f32,
}

// IVF配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IVFConfig {
    pub num_clusters: usize,
    pub num_probes: usize,
    pub training_sample_ratio: f32,
}

// 记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_memories_per_agent: usize,
    pub decay_factor: f32,
    pub similarity_threshold: f32,
    pub compression_enabled: bool,
    pub merge_similar_memories: bool,
    pub importance_threshold: f32,
}

// RAG配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_context_tokens: usize,
    pub relevance_threshold: f32,
    pub hybrid_search_alpha: f32,
    pub rerank_enabled: bool,
}

// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub cache_size_mb: usize,
    pub batch_size: usize,
    pub parallel_workers: usize,
    pub memory_limit_mb: usize,
    pub gc_interval_ms: u64,
}

// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub file_path: Option<String>,
    pub max_file_size_mb: usize,
    pub max_files: usize,
    pub console_enabled: bool,
}

impl Default for AgentDbConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            vector: VectorConfig::default(),
            memory: MemoryConfig::default(),
            rag: RAGConfig::default(),
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "./agent_db".to_string(),
            max_connections: 10,
            connection_timeout_ms: 5000,
            retry_attempts: 3,
            backup_enabled: false,
            backup_interval_hours: 24,
        }
    }
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            dimension: 384,
            index_type: VectorIndexType::Flat,
            similarity_threshold: 0.7,
            max_vectors_per_batch: 1000,
            hnsw_config: HNSWConfig::default(),
            ivf_config: IVFConfig::default(),
        }
    }
}

impl Default for HNSWConfig {
    fn default() -> Self {
        Self {
            max_level: 16,
            max_connections: 16,
            ef_construction: 200,
            ml: 1.0 / (2.0_f32).ln(),
        }
    }
}

impl Default for IVFConfig {
    fn default() -> Self {
        Self {
            num_clusters: 100,
            num_probes: 10,
            training_sample_ratio: 0.1,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memories_per_agent: 10000,
            decay_factor: 0.01,
            similarity_threshold: 0.8,
            compression_enabled: true,
            merge_similar_memories: true,
            importance_threshold: 0.1,
        }
    }
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 100,
            max_context_tokens: 4000,
            relevance_threshold: 0.5,
            hybrid_search_alpha: 0.7,
            rerank_enabled: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_size_mb: 512,
            batch_size: 100,
            parallel_workers: num_cpus::get(),
            memory_limit_mb: 2048,
            gc_interval_ms: 60000,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: true,
            file_path: Some("./logs/agent_db.log".to_string()),
            max_file_size_mb: 100,
            max_files: 5,
            console_enabled: true,
        }
    }
}

impl AgentDbConfig {
    // 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, AgentDbError> {
        let content = fs::read_to_string(path)
            .map_err(|e| AgentDbError::Internal(format!("Failed to read config file: {}", e)))?;
        
        let config: AgentDbConfig = serde_json::from_str(&content)
            .map_err(|e| AgentDbError::Serde(e))?;
        
        config.validate()?;
        Ok(config)
    }

    // 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), AgentDbError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AgentDbError::Serde(e))?;
        
        // 确保目录存在
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AgentDbError::Internal(format!("Failed to create config directory: {}", e)))?;
        }
        
        fs::write(path, content)
            .map_err(|e| AgentDbError::Internal(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }

    // 从环境变量加载配置
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // 数据库配置
        if let Ok(path) = std::env::var("AGENT_DB_PATH") {
            config.database.path = path;
        }
        
        // 向量配置
        if let Ok(dim) = std::env::var("AGENT_DB_VECTOR_DIMENSION") {
            if let Ok(dimension) = dim.parse::<usize>() {
                config.vector.dimension = dimension;
            }
        }
        
        // 日志配置
        if let Ok(level) = std::env::var("AGENT_DB_LOG_LEVEL") {
            config.logging.level = level;
        }
        
        config
    }

    // 验证配置
    pub fn validate(&self) -> Result<(), AgentDbError> {
        // 验证向量维度
        if self.vector.dimension == 0 {
            return Err(AgentDbError::InvalidArgument("Vector dimension must be greater than 0".to_string()));
        }
        
        // 验证相似性阈值
        if self.vector.similarity_threshold < 0.0 || self.vector.similarity_threshold > 1.0 {
            return Err(AgentDbError::InvalidArgument("Similarity threshold must be between 0.0 and 1.0".to_string()));
        }
        
        // 验证记忆配置
        if self.memory.max_memories_per_agent == 0 {
            return Err(AgentDbError::InvalidArgument("Max memories per agent must be greater than 0".to_string()));
        }
        
        // 验证RAG配置
        if self.rag.chunk_size == 0 {
            return Err(AgentDbError::InvalidArgument("Chunk size must be greater than 0".to_string()));
        }
        
        // 验证性能配置
        if self.performance.parallel_workers == 0 {
            return Err(AgentDbError::InvalidArgument("Parallel workers must be greater than 0".to_string()));
        }
        
        Ok(())
    }

    // 合并配置
    pub fn merge(&mut self, other: &AgentDbConfig) {
        // 这里可以实现配置合并逻辑
        // 暂时简单覆盖
        *self = other.clone();
    }

    // 获取配置值
    pub fn get_string(&self, key: &str) -> Option<String> {
        match key {
            "database.path" => Some(self.database.path.clone()),
            "logging.level" => Some(self.logging.level.clone()),
            _ => None,
        }
    }

    pub fn get_usize(&self, key: &str) -> Option<usize> {
        match key {
            "vector.dimension" => Some(self.vector.dimension),
            "memory.max_memories_per_agent" => Some(self.memory.max_memories_per_agent),
            "rag.chunk_size" => Some(self.rag.chunk_size),
            "performance.parallel_workers" => Some(self.performance.parallel_workers),
            _ => None,
        }
    }

    pub fn get_f32(&self, key: &str) -> Option<f32> {
        match key {
            "vector.similarity_threshold" => Some(self.vector.similarity_threshold),
            "memory.decay_factor" => Some(self.memory.decay_factor),
            "rag.hybrid_search_alpha" => Some(self.rag.hybrid_search_alpha),
            _ => None,
        }
    }
}

// 配置管理器
pub struct ConfigManager {
    config: AgentDbConfig,
    config_path: Option<String>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: AgentDbConfig::default(),
            config_path: None,
        }
    }

    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self, AgentDbError> {
        let config = AgentDbConfig::from_file(&path)?;
        Ok(Self {
            config,
            config_path: Some(path.as_ref().to_string_lossy().to_string()),
        })
    }

    pub fn get_config(&self) -> &AgentDbConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: AgentDbConfig) -> Result<(), AgentDbError> {
        new_config.validate()?;
        self.config = new_config;
        
        // 如果有配置文件路径，自动保存
        if let Some(ref path) = self.config_path {
            self.config.save_to_file(path)?;
        }
        
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), AgentDbError> {
        if let Some(ref path) = self.config_path {
            self.config = AgentDbConfig::from_file(path)?;
        }
        Ok(())
    }
}
