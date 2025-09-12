//! # AgentMem Mem0 Compatibility Layer
//!
//! This crate provides a compatibility layer that allows AgentMem to be used as a drop-in
//! replacement for Mem0. It implements the Mem0 API surface while leveraging AgentMem's
//! advanced memory management capabilities.
//!
//! ## Features
//!
//! - **Drop-in Replacement**: Compatible with existing Mem0 code
//! - **Enhanced Performance**: Leverages AgentMem's optimized storage and retrieval
//! - **Advanced Memory Types**: Supports episodic, semantic, procedural, and working memory
//! - **Intelligent Processing**: Automatic importance scoring and memory consolidation
//! - **Flexible Storage**: Multiple vector database backends
//!
//! ## Usage
//!
//! ```rust,no_run
//! use agent_mem_compat::Mem0Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Mem0Client::new().await?;
//!     
//!     // Add a memory
//!     let memory_id = client.add("user123", "I love pizza", None).await?;
//!     
//!     // Search memories
//!     let memories = client.search("food preferences", "user123", None).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod context_aware;
pub mod graph_memory;
pub mod procedural_memory;
pub mod types;
pub mod error;
pub mod utils;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use client::Mem0Client;
pub use config::Mem0Config;
pub use context_aware::{
    ContextAwareManager, ContextAwareConfig, ContextInfo, ContextPattern,
    ContextAwareSearchRequest, ContextAwareSearchResult, ContextLearningResult,
};
pub use graph_memory::{GraphMemoryManager, GraphMemoryConfig, FusedMemory};
pub use procedural_memory::{
    ProceduralMemoryManager, ProceduralMemoryConfig, Workflow, WorkflowStep, WorkflowExecution,
    TaskChain, Task, StepType, StepStatus, ExecutionStatus, ChainStatus, TaskStatus, TaskPriority,
    StepExecutionResult, TaskExecutionResult,
};
pub use types::{
    AddMemoryRequest, BatchAddResult, BatchDeleteItem, BatchDeleteRequest, BatchDeleteResult,
    BatchUpdateItem, BatchUpdateRequest, BatchUpdateResult, ChangeType, DeleteMemoryResponse,
    FilterOperation, Memory, MemoryFilter, MemoryHistory, MemorySearchResult,
    MemorySearchResultItem, SearchMemoryRequest, SortField, SortOrder, UpdateMemoryRequest,
};
pub use error::{Mem0Error, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Mem0 API compatibility version
pub const MEM0_API_VERSION: &str = "1.0.0";
