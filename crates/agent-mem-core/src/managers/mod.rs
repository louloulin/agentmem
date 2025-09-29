//! Memory managers module
//!
//! 专门的记忆管理器，实现不同类型记忆的特化管理

pub mod core_memory;
pub mod resource_memory;
pub mod knowledge_vault;
pub mod contextual_memory;

pub use core_memory::{
    CoreMemoryManager, CoreMemoryBlock, CoreMemoryBlockType,
    CoreMemoryConfig, CoreMemoryStats
};

pub use resource_memory::{
    ResourceMemoryManager, ResourceMetadata, ResourceType,
    ResourceStorageConfig, ResourceStorageStats
};

pub use knowledge_vault::{
    KnowledgeVaultManager, KnowledgeVaultConfig, KnowledgeEntry, SensitivityLevel,
    AccessPermission, UserPermissions, AuditLogEntry, AuditAction, KnowledgeVaultStats
};

pub use contextual_memory::{
    ContextualMemoryManager, ContextualMemoryConfig, ContextState, EnvironmentType,
    LocationInfo, TemporalInfo, TimeOfDay, Season, UserState, ActivityState,
    DeviceInfo, NetworkInfo, ContextCorrelation, CorrelationType,
    EnvironmentChangeEvent, ChangeType, ContextualMemoryStats
};
