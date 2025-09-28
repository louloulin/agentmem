//! AgentMem Core - Memory Management Engine
//!
//! This crate provides the core memory management functionality for AgentMem,
//! including hierarchical memory architecture, intelligent memory processing,
//! and advanced search capabilities.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod client;
pub mod collaboration;
pub mod compression;
pub mod conflict;
pub mod context;
pub mod engine;
/// Graph-based memory management and reasoning capabilities
pub mod graph_memory;
pub mod hierarchy;
pub mod history;
pub mod intelligence;
pub mod lifecycle;
pub mod manager;
pub mod operations;
pub mod search;
pub mod security;
pub mod storage;
pub mod tenant;
pub mod types;

// Re-export core types
pub use engine::{MemoryEngine, MemoryEngineConfig};
pub use hierarchy::{HierarchyManager, MemoryLevel};

// Re-export from traits
pub use agent_mem_traits::{
    AgentMemError, MemoryItem as Memory, MemoryType, Result as MemoryResult, Session,
};

/// Core error types
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Search error
    #[error("Search error: {0}")]
    Search(String),

    /// Hierarchy error
    #[error("Hierarchy error: {0}")]
    Hierarchy(String),

    /// Intelligence error
    #[error("Intelligence error: {0}")]
    Intelligence(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Core result type
pub type CoreResult<T> = Result<T, CoreError>;
