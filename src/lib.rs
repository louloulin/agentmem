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
pub mod ffi;

#[cfg(test)]
pub mod tests;

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

// 导入必要的依赖
use lancedb::connect;

// 主要的集成数据库结构
pub struct AgentDatabase {
    pub agent_state_db: AgentStateDB,
    pub memory_manager: MemoryManager,
    pub vector_engine: Option<AdvancedVectorEngine>,
    pub security_manager: Option<SecurityManager>,
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
