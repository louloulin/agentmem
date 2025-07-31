
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

// 测试模块已移动到 tests/ 目录

// 重新导出核心类型
pub use core::{
    AgentDbError, AgentState, StateType, Memory, MemoryType,
    DatabaseConfig
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
use std::sync::Arc;
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
        let connection = connect(&config.path).execute().await?;
        let agent_state_db = AgentStateDB::new(&config.path).await?;
        let memory_manager = MemoryManager::new(Arc::new(connection.clone()));

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
        let connection = connect(&self.config.path).execute().await?;
        self.vector_engine = Some(AdvancedVectorEngine::new(Arc::new(connection), config));
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
        path: db_path.to_string(),
        ..Default::default()
    };
    AgentDatabase::new(config).await
}

pub async fn create_database_with_config(config: DatabaseConfig) -> Result<AgentDatabase, AgentDbError> {
    AgentDatabase::new(config).await
}

