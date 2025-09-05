//! Memory Engine - Core orchestration and management

use crate::{
    hierarchy::{HierarchyManager, DefaultHierarchyManager, HierarchyConfig, MemoryLevel},
    intelligence::{ImportanceScorer, DefaultImportanceScorer, ConflictResolver, DefaultConflictResolver, IntelligenceConfig},
};
use crate::{Memory, hierarchy::MemoryScope};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Memory engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEngineConfig {
    /// Hierarchy configuration
    pub hierarchy: HierarchyConfig,
    
    /// Intelligence configuration
    pub intelligence: IntelligenceConfig,
    
    /// Enable automatic memory processing
    pub auto_processing: bool,
    
    /// Processing interval in seconds
    pub processing_interval_seconds: u64,
    
    /// Maximum memories to process in one batch
    pub max_batch_size: usize,
}

impl Default for MemoryEngineConfig {
    fn default() -> Self {
        Self {
            hierarchy: HierarchyConfig::default(),
            intelligence: IntelligenceConfig::default(),
            auto_processing: true,
            processing_interval_seconds: 300, // 5 minutes
            max_batch_size: 100,
        }
    }
}

/// Core memory engine
pub struct MemoryEngine {
    config: MemoryEngineConfig,
    hierarchy_manager: Arc<dyn HierarchyManager>,
    importance_scorer: Arc<dyn ImportanceScorer>,
    conflict_resolver: Arc<dyn ConflictResolver>,
}

impl MemoryEngine {
    /// Create new memory engine with default implementations
    pub fn new(config: MemoryEngineConfig) -> Self {
        let hierarchy_manager = Arc::new(DefaultHierarchyManager::new(config.hierarchy.clone()));
        let importance_scorer = Arc::new(DefaultImportanceScorer::new(config.intelligence.clone()));
        let conflict_resolver = Arc::new(DefaultConflictResolver::new(config.intelligence.clone()));
        
        Self {
            config,
            hierarchy_manager,
            importance_scorer,
            conflict_resolver,
        }
    }
    
    /// Add memory with full processing
    pub async fn add_memory(&self, mut memory: Memory) -> crate::CoreResult<String> {
        // Calculate importance if auto-processing is enabled
        if self.config.auto_processing {
            let importance_factors = self.importance_scorer.calculate_importance(&memory).await?;
            memory.importance = importance_factors.final_score;
            
            debug!("Calculated importance {} for memory {}", memory.importance, memory.id);
        }
        
        // Add to hierarchy
        let hierarchical_memory = self.hierarchy_manager.add_memory(memory).await?;
        
        info!("Added memory {} to engine", hierarchical_memory.memory.id);
        Ok(hierarchical_memory.memory.id)
    }
    
    /// Get memory by ID
    pub async fn get_memory(&self, id: &str) -> crate::CoreResult<Option<Memory>> {
        if let Some(hierarchical_memory) = self.hierarchy_manager.get_memory(id).await? {
            Ok(Some(hierarchical_memory.memory))
        } else {
            Ok(None)
        }
    }
    
    /// Update memory with reprocessing
    pub async fn update_memory(&self, mut memory: Memory) -> crate::CoreResult<Memory> {
        // Recalculate importance if auto-processing is enabled
        if self.config.auto_processing {
            let importance_factors = self.importance_scorer.calculate_importance(&memory).await?;
            memory.importance = importance_factors.final_score;
        }
        
        // Get current hierarchical memory
        if let Some(mut hierarchical_memory) = self.hierarchy_manager.get_memory(&memory.id).await? {
            hierarchical_memory.memory = memory;
            
            // Update in hierarchy (may trigger level changes)
            let updated = self.hierarchy_manager.update_memory(hierarchical_memory).await?;
            
            info!("Updated memory {}", updated.memory.id);
            Ok(updated.memory)
        } else {
            Err(crate::CoreError::Storage(format!("Memory {} not found", memory.id)))
        }
    }
    
    /// Remove memory
    pub async fn remove_memory(&self, id: &str) -> crate::CoreResult<bool> {
        let removed = self.hierarchy_manager.remove_memory(id).await?;
        
        if removed {
            info!("Removed memory {}", id);
        }
        
        Ok(removed)
    }
    
    /// Search memories with intelligent ranking
    pub async fn search_memories(&self, _query: &str, _scope: Option<MemoryScope>, _limit: Option<usize>) -> crate::CoreResult<Vec<Memory>> {
        // TODO: Implement intelligent search
        // For now, return empty results
        warn!("Search not yet implemented");
        Ok(Vec::new())
    }
    
    /// Process memories for conflicts and optimization
    pub async fn process_memories(&self) -> crate::CoreResult<ProcessingReport> {
        let mut report = ProcessingReport::default();
        
        if !self.config.auto_processing {
            return Ok(report);
        }
        
        // Get all memories from all levels
        let mut all_memories = Vec::new();
        
        for level in [MemoryLevel::Strategic, MemoryLevel::Tactical, MemoryLevel::Operational, MemoryLevel::Contextual] {
            let level_memories = self.hierarchy_manager.get_memories_at_level(level).await?;
            all_memories.extend(level_memories.into_iter().map(|hm| hm.memory));
        }
        
        report.total_memories = all_memories.len();
        
        // Detect conflicts
        let conflicts = self.conflict_resolver.detect_conflicts(&all_memories).await?;
        report.conflicts_detected = conflicts.len();
        
        // Auto-resolve conflicts
        if !conflicts.is_empty() {
            let resolved_memories = self.conflict_resolver.auto_resolve_conflicts(&conflicts, &all_memories).await?;
            report.conflicts_resolved = all_memories.len() - resolved_memories.len();
            
            // Update resolved memories back to hierarchy
            for memory in resolved_memories {
                if let Err(e) = self.update_memory(memory).await {
                    warn!("Failed to update resolved memory: {}", e);
                    report.errors += 1;
                }
            }
        }
        
        info!("Processing completed: {:?}", report);
        Ok(report)
    }
    
    /// Get engine statistics
    pub async fn get_statistics(&self) -> crate::CoreResult<EngineStatistics> {
        let hierarchy_stats = self.hierarchy_manager.get_hierarchy_stats().await?;
        
        Ok(EngineStatistics {
            total_memories: hierarchy_stats.memories_by_level.values().sum(),
            memories_by_level: hierarchy_stats.memories_by_level,
            avg_importance_by_level: hierarchy_stats.avg_importance_by_level,
            inheritance_relationships: hierarchy_stats.inheritance_relationships,
            level_utilization: hierarchy_stats.level_utilization,
        })
    }
}

/// Memory processing report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessingReport {
    /// Total memories processed
    pub total_memories: usize,
    
    /// Conflicts detected
    pub conflicts_detected: usize,
    
    /// Conflicts resolved
    pub conflicts_resolved: usize,
    
    /// Memories promoted
    pub memories_promoted: usize,
    
    /// Memories demoted
    pub memories_demoted: usize,
    
    /// Processing errors
    pub errors: usize,
    
    /// Processing duration in milliseconds
    pub duration_ms: u64,
}

/// Engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatistics {
    /// Total number of memories
    pub total_memories: usize,
    
    /// Memories by hierarchy level
    pub memories_by_level: std::collections::HashMap<MemoryLevel, usize>,
    
    /// Average importance by level
    pub avg_importance_by_level: std::collections::HashMap<MemoryLevel, f64>,
    
    /// Number of inheritance relationships
    pub inheritance_relationships: usize,
    
    /// Level utilization ratios
    pub level_utilization: std::collections::HashMap<MemoryLevel, f64>,
}
