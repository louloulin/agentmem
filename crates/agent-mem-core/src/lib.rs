//! AgentMem Core - Memory Management Engine
//!
//! This crate provides the core memory management functionality for AgentMem,
//! including hierarchical memory architecture, intelligent memory processing,
//! and advanced search capabilities.

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Specialized memory agents for different cognitive memory types
pub mod agents;
pub mod client;
pub mod collaboration;
pub mod compression;
pub mod conflict;
pub mod context;
/// Multi-agent coordination and orchestration
pub mod coordination;
pub mod engine;
/// Graph-based memory management and reasoning capabilities
pub mod graph_memory;
pub mod hierarchy;
pub mod history;
pub mod integration;
pub mod intelligence;
pub mod lifecycle;
pub mod manager;
/// Specialized memory managers for different memory types
pub mod managers;
pub mod operations;
/// Active retrieval system with topic extraction, intelligent routing, and context synthesis
pub mod retrieval;
pub mod search;
pub mod security;
pub mod storage;
pub mod tenant;
pub mod types;

// Re-export core types
pub use engine::{MemoryEngine, MemoryEngineConfig};
pub use hierarchy::{HierarchyManager, MemoryLevel};
pub use managers::{
    ActivityState, ChangeType, ContextCorrelation, ContextState, ContextualMemoryConfig,
    ContextualMemoryManager, ContextualMemoryStats, CoreMemoryBlock, CoreMemoryBlockType,
    CoreMemoryConfig, CoreMemoryManager, CoreMemoryStats, CorrelationType, DeviceInfo,
    EnvironmentChangeEvent, EnvironmentType, LocationInfo, NetworkInfo, ResourceMemoryManager,
    ResourceMetadata, ResourceStorageConfig, ResourceStorageStats, ResourceType, Season,
    TemporalInfo, TimeOfDay, UserState,
};

// Re-export coordination and agents modules
pub use agents::{
    AgentConfig, AgentStats, BaseAgent, ContextualAgent, CoreAgent, EpisodicAgent, KnowledgeAgent,
    MemoryAgent, ProceduralAgent, ResourceAgent, SemanticAgent, WorkingAgent,
};
pub use coordination::{
    AgentMessage, AgentStatus, CoordinationError, CoordinationResult, CoordinationStats,
    LoadBalancingStrategy, MessageType, MetaMemoryConfig, MetaMemoryManager, TaskRequest,
    TaskResponse,
};

// Re-export retrieval modules
pub use retrieval::{
    ActiveRetrievalConfig, ActiveRetrievalSystem, ConflictResolution, ContextSynthesizer,
    ContextSynthesizerConfig, ExtractedTopic, RetrievalRequest, RetrievalResponse, RetrievalRouter,
    RetrievalRouterConfig, RetrievalStats, RetrievalStrategy, RetrievedMemory, RouteDecision,
    RoutingResult, SynthesisResult, TopicCategory, TopicExtractor, TopicExtractorConfig,
    TopicHierarchy,
};

// Re-export integration modules
pub use integration::{
    ComponentHealth, HealthStatus, SystemConfig, SystemIntegrationManager, SystemState,
    SystemStatus,
};

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

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

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
