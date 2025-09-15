//! AgentMem Core - Memory Management Engine
//! 
//! This crate provides the core memory management functionality for AgentMem,
//! including hierarchical memory architecture, intelligent memory processing,
//! and advanced search capabilities.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod engine;
pub mod hierarchy;
pub mod intelligence;
pub mod search;
pub mod storage;
pub mod conflict;
pub mod lifecycle;
pub mod context;
pub mod manager;
pub mod history;
pub mod operations;
pub mod types;
pub mod client;
pub mod tenant;

// Re-export core types
pub use engine::{MemoryEngine, MemoryEngineConfig};
pub use hierarchy::{MemoryLevel, HierarchyManager};

// Re-export from traits
pub use agent_mem_traits::{
    MemoryItem as Memory, MemoryType, Session, AgentMemError, Result as MemoryResult,
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
