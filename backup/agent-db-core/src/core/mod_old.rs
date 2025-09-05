// 核心数据结构和类型定义 - 简化版本
pub mod types;
pub mod error;

// 重新导出核心类型，避免冲突
pub use types::{StateType, AgentState, Memory, MemoryType, Document, DocumentChunk, SearchResult, RAGContext, DatabaseConfig};
pub use error::{AgentDbError, CAgentDbErrorCode};
