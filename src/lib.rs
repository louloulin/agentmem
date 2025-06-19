<<<<<<< HEAD
// Agent状态数据库库 - 模块化实现
//! Agent状态数据库
//!
//! 这是一个基于LanceDB的Agent状态管理系统，提供：
//! - Agent状态持久化
//! - 记忆管理
//! - RAG（检索增强生成）功能
//! - 向量搜索
//! - 统一的API接口
//!
//! ## 模块结构
//!
//! - `types`: 核心类型定义和错误处理
//! - `database`: Agent状态数据库核心功能
//! - `memory`: 记忆管理系统
//! - `rag`: 检索增强生成引擎
//! - `vector`: 向量搜索引擎
//! - `api`: 统一的高级API接口
//!
//! ## 使用示例
//!
//! ```rust
//! use agent_state_db::{AgentDB, AgentState, StateType, Memory, MemoryType, Document};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建数据库实例
//!     let db = AgentDB::new("./agent_db", 384).await?;
//!
//!     // 保存Agent状态
//!     let state = AgentState::new(1, 1, StateType::WorkingMemory, b"test data".to_vec());
//!     db.save_agent_state(&state).await?;
//!
//!     // 存储记忆
//!     let memory = Memory::new(1, MemoryType::Episodic, "重要的对话".to_string(), 0.8);
//!     db.store_memory(&memory).await?;
//!
//!     // 添加文档
//!     let mut doc = Document::new("测试文档".to_string(), "这是一个测试文档的内容".to_string());
//!     doc.chunk_document(100, 20)?;
//!     db.add_document(doc).await?;
//!
//!     Ok(())
//! }
//! ```

// 模块声明
pub mod types;
pub mod config;
pub mod utils;
pub mod database;
pub mod memory;
pub mod rag;
pub mod vector;
pub mod api;
pub mod ffi;
pub mod performance;
pub mod monitoring;

// 重新导出主要类型和API
pub use types::*;
pub use config::{AgentDbConfig, ConfigManager};
pub use api::AgentDB;
pub use database::AgentStateDB;
pub use memory::MemoryManager;
pub use rag::RAGEngine;
pub use vector::{VectorSearchEngine, AdvancedVectorEngine};
pub use performance::{CacheManager, ConnectionPool, BatchOperationManager, MemoryManager as PerfMemoryManager};
pub use monitoring::{MonitoringManager, LogLevel, HealthStatus};

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取库版本信息
pub fn version() -> &'static str {
    VERSION
}

/// 初始化日志系统（可选）
pub fn init_logging() {
    env_logger::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[tokio::test]
    async fn test_basic_functionality() {
        // 基本功能测试
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");

        let db = AgentDB::new(db_path.to_str().unwrap(), 384).await.unwrap();

        // 测试Agent状态
        let state = AgentState::new(1, 1, StateType::WorkingMemory, b"test".to_vec());
        db.save_agent_state(&state).await.unwrap();

        let loaded_state = db.load_agent_state(1).await.unwrap();
        assert!(loaded_state.is_some());

        // 测试记忆
        let memory = Memory::new(1, MemoryType::Episodic, "test memory".to_string(), 0.5);
        db.store_memory(&memory).await.unwrap();

        let memories = db.get_agent_memories(1, None, 10).await.unwrap();
        assert!(!memories.is_empty());

        // 测试文档
        let mut doc = Document::new("Test Doc".to_string(), "Test content".to_string());
        doc.chunk_document(50, 10).unwrap();
        db.add_document(doc).await.unwrap();

        let docs = db.list_documents(10).await.unwrap();
        assert!(!docs.is_empty());
    }

    #[test]
    fn test_config_functionality() {
        // 测试配置管理
        let config = AgentDbConfig::default();
        assert_eq!(config.vector.dimension, 384);
        assert_eq!(config.database.path, "./agent_db");

        // 测试配置验证
        assert!(config.validate().is_ok());

        // 测试配置管理器
        let mut manager = ConfigManager::new();
        let new_config = AgentDbConfig::default();
        assert!(manager.update_config(new_config).is_ok());
    }

    #[test]
    fn test_utils_functionality() {
        // 测试文本工具
        let text = "Hello World! This is a test.";
        let tokens = utils::text::tokenize(text);
        assert!(!tokens.is_empty());

        let similarity = utils::text::jaccard_similarity("hello world", "hello earth");
        assert!(similarity > 0.0 && similarity < 1.0);

        // 测试向量工具
        let mut vector = vec![1.0, 2.0, 3.0];
        utils::vector::normalize(&mut vector);
        let norm = utils::vector::l2_norm(&vector);
        assert!((norm - 1.0).abs() < 1e-6);

        // 测试时间工具
        let timestamp = utils::time::current_timestamp();
        assert!(timestamp > 0);

        // 测试序列化工具
        let data = vec![1, 2, 3, 4, 5];
        let json = utils::serialization::to_json(&data).unwrap();
        let restored: Vec<i32> = utils::serialization::from_json(&json).unwrap();
        assert_eq!(data, restored);

        // 测试哈希工具
        let hash1 = utils::hash::hash_string("test");
        let hash2 = utils::hash::hash_string("test");
        assert_eq!(hash1, hash2);

        let uuid = utils::hash::generate_uuid();
        assert!(!uuid.is_empty());
    }

    #[test]
    fn test_performance_functionality() {
        // 测试缓存管理器
        let config = config::PerformanceConfig::default();
        let cache_manager = performance::CacheManager::new(config.clone());

        // 测试缓存设置和获取
        let query_hash = 12345u64;
        let test_data = vec![1, 2, 3, 4, 5];
        cache_manager.set(query_hash, test_data.clone(), 5);

        let cached_data = cache_manager.get(query_hash);
        assert!(cached_data.is_some());
        assert_eq!(cached_data.unwrap(), test_data);

        // 测试缓存统计
        let stats = cache_manager.get_statistics();
        assert!(stats.total_entries > 0);

        // 测试批量操作管理器
        let batch_manager = performance::BatchOperationManager::new(config.clone());
        batch_manager.add_operation("test_op".to_string(), vec![1, 2, 3]);
        assert_eq!(batch_manager.pending_count(), 1);

        // 测试内存管理器
        let memory_manager = performance::MemoryManager::new(config);
        assert!(memory_manager.allocate(1024).is_ok());
        assert_eq!(memory_manager.get_memory_usage(), 1024);
        memory_manager.deallocate(512);
        assert_eq!(memory_manager.get_memory_usage(), 512);
    }

    #[test]
    fn test_monitoring_functionality() {
        // 测试监控管理器
        let config = config::LoggingConfig::default();
        let monitor = monitoring::MonitoringManager::new(config);

        // 测试日志记录
        monitor.log(
            monitoring::LogLevel::Info,
            "test_module",
            "Test message",
            None,
        );

        let logs = monitor.get_logs(Some(monitoring::LogLevel::Info), Some(10));
        assert!(!logs.is_empty());
        assert_eq!(logs[0].message, "Test message");

        // 测试性能指标记录
        monitor.record_metric("test_metric", 42.0, "count", None);
        let metrics = monitor.get_metrics(Some("test_metric"), Some(10));
        assert!(!metrics.is_empty());
        assert_eq!(metrics[0].value, 42.0);

        // 测试错误记录
        monitor.record_error("test_error", "Test error message", None);
        let errors = monitor.get_error_summary();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].message, "Test error message");

        // 测试运行时间
        std::thread::sleep(std::time::Duration::from_millis(1));
        let uptime = monitor.get_uptime();
        assert!(uptime.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_advanced_api_functionality() {
        let db = AgentDB::new("./test_advanced_db", 384).await.unwrap();

        // 测试批量操作
        let states = vec![
            AgentState::new(10, 1, StateType::WorkingMemory, vec![1, 2, 3]),
            AgentState::new(11, 1, StateType::LongTermMemory, vec![4, 5, 6]),
        ];

        let results = db.batch_save_agent_states(states).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        // 测试系统健康状态
        let health = db.get_system_health().await.unwrap();
        assert!(health.contains_key("database"));
        assert!(health.contains_key("total_agents"));

        // 测试Agent行为模式分析
        let patterns = db.analyze_agent_patterns(10).await.unwrap();
        assert!(patterns.contains_key("state_changes"));
    }
}
=======
// Agent状态数据库 - 基于LanceDB的Rust实现
// 模块化架构

// 核心模块
pub mod core;
pub mod agent_state;
pub mod memory;
pub mod vector;
pub mod security;
pub mod performance;
pub mod distributed;
pub mod realtime;
pub mod rag;
pub mod ffi;

// 测试模块
#[cfg(test)]
mod tests_new_features;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
mod benchmark;

#[cfg(test)]
mod stress_test;

// 重新导出核心类型
pub use core::{
    AgentDbError, AgentState, StateType, Memory, MemoryType,
    DatabaseConfig, QueryResult, PaginationParams, SortOrder
};
pub use agent_state::AgentStateDB;
pub use memory::{MemoryManager, MemoryStatistics};
pub use vector::{AdvancedVectorEngine, VectorSearchResult, SimilarityAlgorithm};
pub use security::{SecurityManager, User, Role, Permission, AccessToken};
pub use performance::{PerformanceMonitor, PerformanceMetrics, PerformanceDiagnostics};
pub use distributed::{AgentNetworkManager, AgentNode, DistributedStateManager, MessageRouter};
pub use realtime::{RealTimeStreamProcessor, StreamDataItem, StreamDataType, StreamQueryProcessor};
pub use rag::{RAGEngine, Document, DocumentChunk, SearchResult, RAGContext};

// 导入必要的依赖
use lancedb::connect;

// 主要的集成数据库结构
pub struct AgentDatabase {
    pub agent_state_db: AgentStateDB,
    pub memory_manager: MemoryManager,
    pub vector_engine: Option<AdvancedVectorEngine>,
    pub security_manager: Option<SecurityManager>,
    pub rag_engine: Option<RAGEngine>,
    pub config: DatabaseConfig,
}

impl AgentDatabase {
    pub async fn new(config: DatabaseConfig) -> Result<Self, AgentDbError> {
        let connection = connect(&config.db_path).execute().await?;
        let agent_state_db = AgentStateDB::new(&config.db_path).await?;
        let memory_manager = MemoryManager::new(connection.clone());

        Ok(Self {
            agent_state_db,
            memory_manager,
            vector_engine: None,
            security_manager: None,
            rag_engine: None,
            config,
        })
    }

    pub async fn with_vector_engine(mut self, config: vector::VectorIndexConfig) -> Result<Self, AgentDbError> {
        let connection = connect(&self.config.db_path).execute().await?;
        self.vector_engine = Some(AdvancedVectorEngine::new(connection, config));
        Ok(self)
    }

    pub fn with_security_manager(mut self) -> Self {
        self.security_manager = Some(SecurityManager::new());
        self
    }

    pub async fn with_rag_engine(mut self) -> Result<Self, AgentDbError> {
        self.rag_engine = Some(RAGEngine::new(&self.config.db_path).await?);
        Ok(self)
    }

    // Agent状态操作
    pub async fn save_agent_state(&self, state: &AgentState) -> Result<(), AgentDbError> {
        self.agent_state_db.save_state(state).await
    }

    pub async fn load_agent_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentDbError> {
        self.agent_state_db.load_state(agent_id).await
    }

    // 记忆操作
    pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentDbError> {
        self.memory_manager.store_memory(memory).await
    }

    pub async fn get_memories(&self, agent_id: u64) -> Result<Vec<Memory>, AgentDbError> {
        self.memory_manager.get_memories_by_agent(agent_id).await
    }

    // 向量操作
    pub async fn add_vector(&self, id: u64, vector: Vec<f32>, metadata: std::collections::HashMap<String, String>) -> Result<(), AgentDbError> {
        if let Some(ref engine) = self.vector_engine {
            engine.add_vector(id, vector, metadata).await
        } else {
            Err(AgentDbError::Internal("Vector engine not initialized".to_string()))
        }
    }

    pub async fn search_vectors(&self, query: &[f32], limit: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        if let Some(ref engine) = self.vector_engine {
            engine.search_vectors(query, limit).await
        } else {
            Err(AgentDbError::Internal("Vector engine not initialized".to_string()))
        }
    }

    // 向量状态操作
    pub async fn save_vector_state(&self, state: &AgentState, embedding: Vec<f32>) -> Result<(), AgentDbError> {
        self.agent_state_db.save_vector_state(state, embedding).await
    }

    pub async fn vector_search_states(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        self.agent_state_db.vector_search(query_embedding, limit).await
    }

    pub async fn search_by_agent_and_similarity(&self, agent_id: u64, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        self.agent_state_db.search_by_agent_and_similarity(agent_id, query_embedding, limit).await
    }

    // RAG操作
    pub async fn index_document(&self, document: &Document) -> Result<String, AgentDbError> {
        if let Some(ref engine) = self.rag_engine {
            engine.index_document(document).await
        } else {
            Err(AgentDbError::Internal("RAG engine not initialized".to_string()))
        }
    }

    pub async fn search_documents(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        if let Some(ref engine) = self.rag_engine {
            engine.search_by_text(query, limit).await
        } else {
            Err(AgentDbError::Internal("RAG engine not initialized".to_string()))
        }
    }

    pub async fn semantic_search_documents(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        if let Some(ref engine) = self.rag_engine {
            engine.semantic_search(query_embedding, limit).await
        } else {
            Err(AgentDbError::Internal("RAG engine not initialized".to_string()))
        }
    }

    pub async fn hybrid_search_documents(&self, text_query: &str, query_embedding: Vec<f32>, alpha: f32, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        if let Some(ref engine) = self.rag_engine {
            engine.hybrid_search(text_query, query_embedding, alpha, limit).await
        } else {
            Err(AgentDbError::Internal("RAG engine not initialized".to_string()))
        }
    }

    pub async fn build_context(&self, query: &str, search_results: Vec<SearchResult>, max_tokens: usize) -> Result<RAGContext, AgentDbError> {
        if let Some(ref engine) = self.rag_engine {
            engine.build_context(query, search_results, max_tokens).await
        } else {
            Err(AgentDbError::Internal("RAG engine not initialized".to_string()))
        }
    }

    pub async fn get_document(&self, doc_id: &str) -> Result<Option<Document>, AgentDbError> {
        if let Some(ref engine) = self.rag_engine {
            engine.get_document(doc_id).await
        } else {
            Err(AgentDbError::Internal("RAG engine not initialized".to_string()))
        }
    }
}

// 便利函数
pub async fn create_database(db_path: &str) -> Result<AgentDatabase, AgentDbError> {
    let config = DatabaseConfig {
        db_path: db_path.to_string(),
        ..Default::default()
    };
    AgentDatabase::new(config).await
}

pub async fn create_database_with_config(config: DatabaseConfig) -> Result<AgentDatabase, AgentDbError> {
    AgentDatabase::new(config).await
}
>>>>>>> origin/feature-module
