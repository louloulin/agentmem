
// AgentDB Core - Rust 核心引擎
// 模块化架构的 Rust 核心模块

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

// 重新导出核心类型
pub use core::{
    AgentDbError, AgentState, StateType, Memory, MemoryType,
    Document, DocumentChunk, SearchResult, RAGContext
};
pub use core::config::DatabaseConfig;
pub use agent_state::AgentStateDB;
pub use memory::MemoryManager;
pub use vector::{AdvancedVectorEngine, VectorSearchResult, VectorIndexConfig};
pub use security::{SecurityManager, User, Permission, AccessToken};
pub use performance::{PerformanceMonitor, PerformanceMetrics};
pub use distributed::{AgentNetworkManager, AgentNode, DistributedStateManager, NetworkStatus};
pub use realtime::{RealTimeStreamProcessor, StreamDataItem, StreamQueryProcessor};
pub use rag::RAGEngine;

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
        // 使用简化的实现，不依赖 LanceDB
        let agent_state_db = AgentStateDB::new(&config.path).await?;
        let mut memory_manager = MemoryManager::new(&config.path);
        memory_manager.init().await?;

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
        let mut vector_engine = AdvancedVectorEngine::new(&self.config.path, config);
        vector_engine.init().await?;
        self.vector_engine = Some(vector_engine);
        Ok(self)
    }

    pub fn with_security_manager(mut self) -> Self {
        self.security_manager = Some(SecurityManager::new());
        self
    }

    pub async fn with_rag_engine(mut self) -> Result<Self, AgentDbError> {
        self.rag_engine = Some(RAGEngine::new(&self.config.path).await?);
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

    pub async fn hybrid_search_documents(&self, text_query: &str, query_embedding: Vec<f32>, _alpha: f32, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        if let Some(ref engine) = self.rag_engine {
            // 简化实现：结合文本搜索和语义搜索
            let mut text_results = engine.search_by_text(text_query, limit / 2).await?;
            let mut semantic_results = engine.semantic_search(query_embedding, limit / 2).await?;

            // 合并结果并去重
            text_results.append(&mut semantic_results);
            text_results.truncate(limit);

            Ok(text_results)
        } else {
            Err(AgentDbError::Internal("RAG engine not initialized".to_string()))
        }
    }

    pub async fn build_context(&self, query: &str, search_results: Vec<SearchResult>, _max_tokens: usize) -> Result<RAGContext, AgentDbError> {
        // 简化实现：构建 RAG 上下文
        let context = search_results
            .iter()
            .map(|result| result.content.clone())
            .collect::<Vec<String>>()
            .join("\n\n");

        let confidence = if search_results.is_empty() {
            0.0
        } else {
            search_results.iter().map(|r| r.score).sum::<f32>() / search_results.len() as f32
        };

        Ok(RAGContext {
            query: query.to_string(),
            context,
            sources: search_results,
            confidence,
        })
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
        path: db_path.to_string(),
        max_connections: 10,
        connection_timeout_ms: 5000,
        retry_attempts: 3,
        backup_enabled: false,
        backup_interval_hours: 24,
    };
    AgentDatabase::new(config).await
}

pub async fn create_database_with_config(config: DatabaseConfig) -> Result<AgentDatabase, AgentDbError> {
    AgentDatabase::new(config).await
}

// 添加单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    async fn create_test_database() -> (AgentDatabase, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let db = create_database(db_path).await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_agent_state_operations() {
        let (db, _temp_dir) = create_test_database().await;

        let state = AgentState::new(
            1001,
            1,
            StateType::WorkingMemory,
            b"test data".to_vec(),
        );

        // 测试保存状态
        assert!(db.save_agent_state(&state).await.is_ok());

        // 测试加载状态
        let loaded_state = db.load_agent_state(1001).await.unwrap();
        assert!(loaded_state.is_some());
        assert_eq!(loaded_state.unwrap().agent_id, 1001);
    }

    #[tokio::test]
    async fn test_memory_operations() {
        let (db, _temp_dir) = create_test_database().await;

        let memory = Memory::new(
            1001,
            MemoryType::Episodic,
            "Test memory content".to_string(),
            0.8,
        );

        // 测试存储记忆
        assert!(db.store_memory(&memory).await.is_ok());

        // 测试获取记忆
        let memories = db.get_memories(1001).await.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].content, "Test memory content");
    }

    #[tokio::test]
    async fn test_vector_operations() {
        let (mut db, _temp_dir) = create_test_database().await;

        // 初始化向量引擎
        let vector_config = vector::VectorIndexConfig {
            dimension: 3,
            metric: "cosine".to_string(),
            index_type: "flat".to_string(),
            ef_construction: 200,
            m: 16,
        };
        db = db.with_vector_engine(vector_config).await.unwrap();

        let vector = vec![1.0, 2.0, 3.0];
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "test".to_string());

        // 测试添加向量
        assert!(db.add_vector(1, vector.clone(), metadata).await.is_ok());

        // 测试搜索向量
        let results = db.search_vectors(&vector, 5).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_rag_operations() {
        let (mut db, _temp_dir) = create_test_database().await;

        // 初始化 RAG 引擎
        db = db.with_rag_engine().await.unwrap();

        let document = Document::new(
            "Test Document".to_string(),
            "This is a test document for RAG functionality.".to_string(),
        );

        // 测试文档索引
        assert!(db.index_document(&document).await.is_ok());

        // 测试文档搜索
        let results = db.search_documents("test", 5).await.unwrap();
        assert!(!results.is_empty());
    }
}

