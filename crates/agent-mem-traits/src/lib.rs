//! # Agent Memory Traits
//!
//! Core traits and abstractions for the AgentMem memory platform.
//! This crate defines the fundamental interfaces that all components must implement.

pub mod embedder;
pub mod error;
pub mod llm;
pub mod memory;
pub mod session;
pub mod storage;
pub mod types;

// Re-export main traits
pub use embedder::Embedder;
pub use error::{AgentMemError, Result};
pub use llm::{LLMProvider, ModelInfo};
pub use memory::MemoryProvider;
pub use session::SessionManager;
pub use storage::{
    EmbeddingVectorStore, GraphResult, GraphStore, HistoryStore, KeyValueStore, LegacyVectorStore,
    VectorStore, VectorStoreStats,
};
pub use types::*;
