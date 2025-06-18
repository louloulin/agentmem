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
pub mod database;
pub mod memory;
pub mod rag;
pub mod vector;
pub mod api;

// 重新导出主要类型和API
pub use types::*;
pub use api::AgentDB;
pub use database::AgentStateDB;
pub use memory::MemoryManager;
pub use rag::RAGEngine;
pub use vector::VectorSearchEngine;

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
}