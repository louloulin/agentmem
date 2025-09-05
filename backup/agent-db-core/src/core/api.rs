// 简化的 API 模块
use std::sync::Arc;
use lancedb::Connection;

use crate::core::{AgentDbError, AgentState, Memory, Document};
use crate::agent_state::AgentStateDB;
use crate::memory::MemoryManager;
use crate::rag::RAGEngine;
use crate::vector::AdvancedVectorEngine;

/// Agent数据库的主要API接口
pub struct AgentDB {
    state_db: AgentStateDB,
    memory_manager: MemoryManager,
    rag_engine: Option<RAGEngine>,
    vector_engine: Option<AdvancedVectorEngine>,
    connection: Arc<Connection>,
}

impl AgentDB {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = lancedb::connect(db_path).execute().await?;
        let connection = Arc::new(connection);
        
        let state_db = AgentStateDB::new(db_path).await?;
        let memory_manager = MemoryManager::new(connection.clone());
        
        Ok(Self {
            state_db,
            memory_manager,
            rag_engine: None,
            vector_engine: None,
            connection,
        })
    }

    pub async fn save_agent_state(&self, state: &AgentState) -> Result<(), AgentDbError> {
        self.state_db.save_state(state).await
    }

    pub async fn load_agent_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentDbError> {
        self.state_db.load_state(agent_id).await
    }

    pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentDbError> {
        self.memory_manager.store_memory(memory).await
    }

    pub async fn get_memories(&self, agent_id: u64) -> Result<Vec<Memory>, AgentDbError> {
        self.memory_manager.get_memories_by_agent(agent_id).await
    }
}
