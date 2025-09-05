//! Intelligence module - Importance scoring and conflict resolution

use crate::Memory;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};


/// Intelligence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    /// Importance scoring weights
    pub importance_weights: ImportanceWeights,
    
    /// Conflict detection sensitivity
    pub conflict_sensitivity: f64,
    
    /// Auto-resolution threshold
    pub auto_resolution_threshold: f64,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            importance_weights: ImportanceWeights::default(),
            conflict_sensitivity: 0.8,
            auto_resolution_threshold: 0.9,
        }
    }
}

/// Importance scoring weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceWeights {
    /// Recency weight
    pub recency: f64,
    
    /// Frequency weight
    pub frequency: f64,
    
    /// Content relevance weight
    pub relevance: f64,
    
    /// User interaction weight
    pub interaction: f64,
}

impl Default for ImportanceWeights {
    fn default() -> Self {
        Self {
            recency: 0.3,
            frequency: 0.2,
            relevance: 0.3,
            interaction: 0.2,
        }
    }
}

/// Importance factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceFactors {
    /// Recency score (0.0 to 1.0)
    pub recency_score: f64,
    
    /// Frequency score (0.0 to 1.0)
    pub frequency_score: f64,
    
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    
    /// Interaction score (0.0 to 1.0)
    pub interaction_score: f64,
    
    /// Final weighted score
    pub final_score: f64,
}

/// Importance scorer trait
#[async_trait]
pub trait ImportanceScorer: Send + Sync {
    /// Calculate importance factors for a memory
    async fn calculate_importance(&self, memory: &Memory) -> crate::CoreResult<ImportanceFactors>;
    
    /// Update importance based on access patterns
    async fn update_importance(&self, memory_id: &str, access_type: AccessType) -> crate::CoreResult<f64>;
}

/// Access type for importance updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessType {
    /// Memory was read
    Read,
    
    /// Memory was updated
    Update,
    
    /// Memory was referenced in context
    Reference,
    
    /// Memory was used in decision making
    Decision,
}

/// Default importance scorer implementation
pub struct DefaultImportanceScorer {
    config: IntelligenceConfig,
}

impl DefaultImportanceScorer {
    /// Create new default importance scorer
    pub fn new(config: IntelligenceConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ImportanceScorer for DefaultImportanceScorer {
    async fn calculate_importance(&self, memory: &Memory) -> crate::CoreResult<ImportanceFactors> {
        let now = chrono::Utc::now();
        
        // Calculate recency score (more recent = higher score)
        let age_hours = (now - memory.created_at).num_hours() as f64;
        let recency_score = (1.0 / (1.0 + age_hours / 24.0)).max(0.0).min(1.0);
        
        // Calculate frequency score based on metadata (placeholder)
        let frequency_score = 0.5; // TODO: Implement based on access patterns

        // Calculate relevance score (placeholder - would use semantic analysis)
        let relevance_score = memory.score.unwrap_or(0.5) as f64; // Use existing score as base
        
        // Calculate interaction score based on metadata
        let interaction_score = if memory.metadata.contains_key("user_interaction") {
            0.8
        } else {
            0.3
        };
        
        // Calculate final weighted score
        let weights = &self.config.importance_weights;
        let final_score = (recency_score * weights.recency
            + frequency_score * weights.frequency
            + relevance_score * weights.relevance
            + interaction_score * weights.interaction)
            .max(0.0)
            .min(1.0);
        
        Ok(ImportanceFactors {
            recency_score,
            frequency_score,
            relevance_score,
            interaction_score,
            final_score,
        })
    }
    
    async fn update_importance(&self, _memory_id: &str, _access_type: AccessType) -> crate::CoreResult<f64> {
        // TODO: Implement importance updates based on access patterns
        Ok(0.5)
    }
}

/// Conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Conflicting memory IDs
    pub memory_ids: Vec<String>,
    
    /// Conflict type
    pub conflict_type: ConflictType,
    
    /// Confidence score
    pub confidence: f64,
    
    /// Suggested resolution
    pub suggested_resolution: ResolutionStrategy,
}

/// Types of conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Contradictory information
    Contradiction,
    
    /// Duplicate content
    Duplicate,
    
    /// Outdated information
    Outdated,
    
    /// Inconsistent metadata
    Inconsistent,
}

/// Resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Keep the most recent memory
    KeepRecent,
    
    /// Keep the most important memory
    KeepImportant,
    
    /// Merge memories
    Merge,
    
    /// Mark as conflicted for manual review
    ManualReview,
}

/// Conflict resolver trait
#[async_trait]
pub trait ConflictResolver: Send + Sync {
    /// Detect conflicts between memories
    async fn detect_conflicts(&self, memories: &[Memory]) -> crate::CoreResult<Vec<ConflictInfo>>;
    
    /// Auto-resolve conflicts based on strategy
    async fn auto_resolve_conflicts(&self, conflicts: &[ConflictInfo], memories: &[Memory]) -> crate::CoreResult<Vec<Memory>>;
}

/// Default conflict resolver implementation
pub struct DefaultConflictResolver {
    config: IntelligenceConfig,
}

impl DefaultConflictResolver {
    /// Create new default conflict resolver
    pub fn new(config: IntelligenceConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ConflictResolver for DefaultConflictResolver {
    async fn detect_conflicts(&self, memories: &[Memory]) -> crate::CoreResult<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();
        
        // Simple duplicate detection based on content similarity
        for (i, memory1) in memories.iter().enumerate() {
            for memory2 in memories.iter().skip(i + 1) {
                if self.are_similar(&memory1.content, &memory2.content) {
                    conflicts.push(ConflictInfo {
                        memory_ids: vec![memory1.id.clone(), memory2.id.clone()],
                        conflict_type: ConflictType::Duplicate,
                        confidence: 0.8,
                        suggested_resolution: ResolutionStrategy::KeepImportant,
                    });
                }
            }
        }
        
        Ok(conflicts)
    }
    
    async fn auto_resolve_conflicts(&self, conflicts: &[ConflictInfo], memories: &[Memory]) -> crate::CoreResult<Vec<Memory>> {
        let mut resolved_memories = memories.to_vec();
        let mut to_remove = Vec::new();
        
        for conflict in conflicts {
            if conflict.confidence >= self.config.auto_resolution_threshold {
                match conflict.suggested_resolution {
                    ResolutionStrategy::KeepImportant => {
                        // Find the memory with highest importance and remove others
                        let mut best_memory = None;
                        let mut best_importance = -1.0;
                        
                        for memory_id in &conflict.memory_ids {
                            if let Some(memory) = memories.iter().find(|m| m.id == *memory_id) {
                                let importance = memory.score.unwrap_or(0.0) as f64;
                                if importance > best_importance {
                                    best_importance = importance;
                                    best_memory = Some(memory_id.clone());
                                }
                            }
                        }
                        
                        // Mark others for removal
                        for memory_id in &conflict.memory_ids {
                            if Some(memory_id.clone()) != best_memory {
                                to_remove.push(memory_id.clone());
                            }
                        }
                    }
                    _ => {
                        // For other strategies, mark for manual review
                        continue;
                    }
                }
            }
        }
        
        // Remove conflicted memories
        resolved_memories.retain(|m| !to_remove.contains(&m.id));
        
        Ok(resolved_memories)
    }
}

impl DefaultConflictResolver {
    /// Check if two content strings are similar
    fn are_similar(&self, content1: &str, content2: &str) -> bool {
        // Simple similarity check - in practice would use more sophisticated methods
        let similarity = self.calculate_similarity(content1, content2);
        similarity > 0.8
    }
    
    /// Calculate similarity between two strings
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        // Simple Jaccard similarity
        let words1: std::collections::HashSet<&str> = s1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = s2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}
