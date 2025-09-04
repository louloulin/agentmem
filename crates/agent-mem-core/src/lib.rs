//! # Agent Memory Core
//! 
//! Core memory management for the AgentMem memory platform.
//! 
//! This crate provides the core memory management functionality including:
//! - Memory lifecycle management
//! - Memory types and operations
//! - CRUD operations for memories
//! - History tracking and versioning

pub mod manager;
pub mod lifecycle;
pub mod types;
pub mod operations;
pub mod history;

pub use manager::MemoryManager;
pub use lifecycle::MemoryLifecycle;
pub use types::*;
pub use operations::*;
pub use history::MemoryHistory;
