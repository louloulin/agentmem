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
pub mod hierarchy;
pub mod hierarchical_service;
pub mod conflict_resolver;
pub mod llm_optimizer;
pub mod hierarchy_manager;
pub mod importance_scorer;
pub mod adaptive_strategy;
pub mod context_aware_search;
pub mod monitoring;
pub mod logging;
pub mod security;

pub use manager::MemoryManager;
pub use lifecycle::MemoryLifecycle;
pub use types::*;
pub use operations::*;
pub use history::MemoryHistory;
pub use hierarchy::*;
pub use hierarchical_service::{
    HierarchicalMemoryService, HierarchicalMemoryRecord, HierarchicalServiceConfig,
    ConflictResolutionStrategy, MemoryInheritanceRule, InheritanceType,
    HierarchicalSearchFilters
};
pub use conflict_resolver::{
    ConflictResolver, ConflictResolverConfig, ConflictResolution, ConflictDetection, ConflictType
};
pub use llm_optimizer::{
    LlmOptimizer, LlmOptimizationConfig, OptimizationStrategy, PromptTemplateType,
    PromptTemplate, OptimizedLlmResponse, LlmPerformanceMetrics, LlmProvider
};
pub use hierarchy_manager::{
    HierarchyManager, HierarchyManagerConfig, HierarchyNode, HierarchyStatistics
};
pub use importance_scorer::{
    AdvancedImportanceScorer, ImportanceScorerConfig, ImportanceFactors,
    MemoryUsageStats, ScoringContext, AccessType
};
pub use adaptive_strategy::{
    AdaptiveStrategyManager, AdaptiveStrategyConfig, MemoryStrategy,
    StrategyPerformance, StrategyRecommendation, StrategyParameters
};
pub use context_aware_search::{
    ContextAwareSearchEngine, ContextAwareSearchConfig, ContextualSearchQuery,
    SearchStrategy, ResultPreferences, ContextualSearchResult, SearchAnalytics
};
pub use monitoring::{
    MonitoringSystem, MonitoringConfig, MetricPoint, MetricType, HealthStatus,
    ComponentStatus, AlertRule, AlertCondition, AlertSeverity, Alert, PerformanceProfile, SystemInfo
};
pub use logging::{
    LoggingSystem, LoggingConfig, LogLevel, LogEntry, AuditEntry, AuditEventType,
    AuditResult, SecurityEntry, SecurityEventType, SecuritySeverity, ComplianceExport
};
pub use security::{
    SecuritySystem, SecurityConfig, Permission, Role, UserAccount, Session as SecuritySession,
    AccessControlEntry, ThreatRule, ThreatRuleType, ThreatSeverity, ThreatAction, ThreatIncident
};

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::{Session, Message, MemoryProvider};
    use tokio_test;

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let manager = MemoryManager::new();
        assert!(true); // Basic creation test
    }

    #[tokio::test]
    async fn test_add_and_get_memory() {
        let manager = MemoryManager::new();
        let session = Session::new()
            .with_agent_id(Some("test-agent".to_string()))
            .with_user_id(Some("test-user".to_string()));

        // Test direct memory addition instead of using MemoryProvider trait
        let memory_id = manager.add_memory(
            "test-agent".to_string(),
            Some("test-user".to_string()),
            "I love playing tennis".to_string(),
            None,
            None,
            None,
        ).await.unwrap();

        let retrieved = manager.get_memory(&memory_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "I love playing tennis");
    }

    #[tokio::test]
    async fn test_search_memories() {
        let manager = MemoryManager::new();

        // Add some memories directly
        let _id1 = manager.add_memory(
            "test-agent".to_string(),
            None,
            "I love playing tennis".to_string(),
            None,
            None,
            None,
        ).await.unwrap();

        let _id2 = manager.add_memory(
            "test-agent".to_string(),
            None,
            "I enjoy reading books".to_string(),
            None,
            None,
            None,
        ).await.unwrap();

        let _id3 = manager.add_memory(
            "test-agent".to_string(),
            None,
            "Tennis is my favorite sport".to_string(),
            None,
            None,
            None,
        ).await.unwrap();

        // Search for tennis-related memories
        let query = crate::types::MemoryQuery::new("test-agent".to_string())
            .with_text_query("tennis".to_string())
            .with_limit(10);
        let results = manager.search_memories(query).await.unwrap();
        assert!(results.len() >= 2); // Should find at least 2 tennis-related memories
    }

    #[tokio::test]
    async fn test_update_memory() {
        let manager = MemoryManager::new();

        let memory_id = manager.add_memory(
            "test-agent".to_string(),
            None,
            "Original content".to_string(),
            None,
            None,
            None,
        ).await.unwrap();

        // Update the memory
        manager.update_memory(
            &memory_id,
            Some("Updated content".to_string()),
            None,
            None,
        ).await.unwrap();

        // Verify the update
        let retrieved = manager.get_memory(&memory_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Updated content");
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let manager = MemoryManager::new();

        let memory_id = manager.add_memory(
            "test-agent".to_string(),
            None,
            "To be deleted".to_string(),
            None,
            None,
            None,
        ).await.unwrap();

        // Delete the memory
        manager.delete_memory(&memory_id).await.unwrap();

        // Verify deletion
        let retrieved = manager.get_memory(&memory_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_types() {
        let memory = Memory::new(
            "agent1".to_string(),
            Some("user1".to_string()),
            MemoryType::Semantic,
            "Test semantic memory".to_string(),
            0.8,
        );

        assert_eq!(memory.memory_type, MemoryType::Semantic);
        assert_eq!(memory.importance, 0.8);
        assert_eq!(memory.content, "Test semantic memory");
    }

    #[tokio::test]
    async fn test_memory_lifecycle() {
        let mut lifecycle = MemoryLifecycle::with_default_config();
        let memory = Memory::new(
            "agent1".to_string(),
            None,
            MemoryType::Working,
            "Test memory".to_string(),
            0.5,
        );

        // Register memory
        lifecycle.register_memory(&memory).unwrap();
        assert!(lifecycle.is_accessible(&memory.id));

        // Archive memory
        lifecycle.archive_memory(&memory.id).unwrap();
        assert!(lifecycle.is_accessible(&memory.id)); // Still accessible when archived

        // Delete memory
        lifecycle.delete_memory(&memory.id).unwrap();
        assert!(!lifecycle.is_accessible(&memory.id)); // Not accessible when deleted
    }
}
