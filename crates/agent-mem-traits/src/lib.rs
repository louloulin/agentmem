//! # Agent Memory Traits
//!
//! Core traits and abstractions for the AgentMem memory platform.
//! This crate defines the fundamental interfaces that all components must implement.

pub mod batch;
pub mod embedder;
pub mod error;
pub mod llm;
pub mod memory;
pub mod session;
pub mod storage;
pub mod types;

// Re-export main traits
pub use batch::{
    AdvancedSearch, ArchiveCriteria, BatchMemoryOperations, ConfigurationProvider,
    HealthCheckProvider, MemoryLifecycle, MemoryStats, MemoryUpdate, RetryableOperations,
    TelemetryProvider,
};
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

// Re-export new Mem5 types
pub use types::{
    BatchResult, EnhancedAddRequest, EnhancedSearchRequest, FilterBuilder, HealthStatus,
    MemorySearchResult, Messages, MetadataBuilder, PerformanceReport, ProcessingOptions,
    ProcessingResult, SystemMetrics,
};
