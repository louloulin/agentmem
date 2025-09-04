//! # Agent Memory Traits
//! 
//! Core traits and abstractions for the AgentMem memory platform.
//! This crate defines the fundamental interfaces that all components must implement.

pub mod memory;
pub mod llm;
pub mod storage;
pub mod embedder;
pub mod session;
pub mod error;
pub mod types;

// Re-export main traits
pub use memory::MemoryProvider;
pub use llm::LLMProvider;
pub use storage::{VectorStore, GraphStore, KeyValueStore, HistoryStore};
pub use embedder::Embedder;
pub use session::SessionManager;
pub use error::{AgentMemError, Result};
pub use types::*;
