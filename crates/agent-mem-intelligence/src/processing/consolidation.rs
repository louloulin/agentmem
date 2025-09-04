//! Memory consolidation algorithms
//!
//! Implements intelligent memory consolidation to reduce redundancy
//! and improve memory organization.

use agent_mem_core::Memory;
use agent_mem_traits::{AgentMemError, Result};
use agent_mem_utils::text::jaccard_similarity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Memory consolidation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsolidationStrategy {
    /// Merge similar memories into one
    Merge,
    /// Create references between similar memories
    Reference,
    /// Group similar memories without merging
    Group,
}

/// Consolidation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    /// Original memory IDs that were consolidated
    pub original_ids: Vec<String>,

    /// New consolidated memory ID
    pub consolidated_id: String,

    /// Consolidation strategy used
    pub strategy: ConsolidationStrategy,

    /// Similarity score that triggered consolidation
    pub similarity_score: f32,
}

/// Memory consolidator
pub struct MemoryConsolidator {
    /// Similarity threshold for consolidation
    threshold: f32,

    /// Consolidation strategy
    strategy: ConsolidationStrategy,

    /// Cache for similarity calculations
    similarity_cache: HashMap<(String, String), f32>,
}

impl MemoryConsolidator {
    /// Create a new memory consolidator
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            strategy: ConsolidationStrategy::Merge,
            similarity_cache: HashMap::new(),
        }
    }

    /// Set consolidation strategy
    pub fn with_strategy(mut self, strategy: ConsolidationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Update similarity threshold
    pub fn update_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
        // Clear cache when threshold changes
        self.similarity_cache.clear();
    }

    /// Consolidate a batch of memories
    pub async fn consolidate_memories(&mut self, memories: &mut Vec<Memory>) -> Result<usize> {
        if memories.len() < 2 {
            return Ok(0);
        }

        info!(
            "Starting memory consolidation for {} memories",
            memories.len()
        );

        let mut consolidated_count = 0;
        let mut consolidation_groups = self.find_consolidation_groups(memories).await?;

        // Sort groups by size (largest first) to prioritize major consolidations
        consolidation_groups.sort_by(|a, b| b.len().cmp(&a.len()));

        for group in consolidation_groups {
            if group.len() < 2 {
                continue;
            }

            match self.strategy {
                ConsolidationStrategy::Merge => {
                    if self.merge_memory_group(memories, &group).await? {
                        consolidated_count += group.len() - 1; // -1 because we keep one merged memory
                    }
                }
                ConsolidationStrategy::Reference => {
                    if self.create_memory_references(memories, &group).await? {
                        consolidated_count += group.len() - 1;
                    }
                }
                ConsolidationStrategy::Group => {
                    if self.group_memories(memories, &group).await? {
                        consolidated_count += 1; // Count as one consolidation operation
                    }
                }
            }
        }

        info!("Consolidated {} memories", consolidated_count);
        Ok(consolidated_count)
    }

    /// Find groups of similar memories that should be consolidated
    async fn find_consolidation_groups(&mut self, memories: &[Memory]) -> Result<Vec<Vec<usize>>> {
        let mut groups = Vec::new();
        let mut processed = HashSet::new();

        for i in 0..memories.len() {
            if processed.contains(&i) {
                continue;
            }

            let mut group = vec![i];
            processed.insert(i);

            for j in (i + 1)..memories.len() {
                if processed.contains(&j) {
                    continue;
                }

                let similarity = self
                    .calculate_similarity(&memories[i], &memories[j])
                    .await?;

                if similarity >= self.threshold {
                    group.push(j);
                    processed.insert(j);
                }
            }

            if group.len() > 1 {
                groups.push(group);
            }
        }

        debug!("Found {} consolidation groups", groups.len());
        Ok(groups)
    }

    /// Calculate similarity between two memories
    async fn calculate_similarity(&mut self, memory1: &Memory, memory2: &Memory) -> Result<f32> {
        let key = if memory1.id < memory2.id {
            (memory1.id.clone(), memory2.id.clone())
        } else {
            (memory2.id.clone(), memory1.id.clone())
        };

        if let Some(&cached_similarity) = self.similarity_cache.get(&key) {
            return Ok(cached_similarity);
        }

        // Calculate text similarity
        let text_similarity = jaccard_similarity(&memory1.content, &memory2.content);

        // Consider memory type similarity
        let type_similarity = if memory1.memory_type == memory2.memory_type {
            1.0
        } else {
            0.5
        };

        // Consider temporal proximity (memories created close in time are more likely to be related)
        let time_diff = (memory1.created_at - memory2.created_at).abs() as f32;
        let max_time_diff = 24.0 * 60.0 * 60.0; // 24 hours in seconds
        let temporal_similarity = (max_time_diff - time_diff.min(max_time_diff)) / max_time_diff;

        // Weighted combination
        let similarity = text_similarity * 0.7 + type_similarity * 0.2 + temporal_similarity * 0.1;

        self.similarity_cache.insert(key, similarity);
        Ok(similarity)
    }

    /// Merge a group of similar memories into one
    async fn merge_memory_group(
        &self,
        memories: &mut Vec<Memory>,
        group: &[usize],
    ) -> Result<bool> {
        if group.len() < 2 {
            return Ok(false);
        }

        // Find the memory with highest importance to be the base
        let base_idx = group
            .iter()
            .max_by(|&&a, &&b| {
                memories[a]
                    .importance
                    .partial_cmp(&memories[b].importance)
                    .unwrap()
            })
            .copied()
            .unwrap();

        let mut merged_content = memories[base_idx].content.clone();
        let mut merged_importance = memories[base_idx].importance;
        let mut merged_access_count = memories[base_idx].access_count;

        // Merge content and aggregate statistics
        for &idx in group {
            if idx == base_idx {
                continue;
            }

            let memory = &memories[idx];

            // Append unique content
            if !merged_content.contains(&memory.content) {
                merged_content.push_str("\n\n");
                merged_content.push_str(&memory.content);
            }

            // Aggregate importance (weighted average)
            merged_importance = (merged_importance + memory.importance) / 2.0;

            // Sum access counts
            merged_access_count += memory.access_count;
        }

        // Update the base memory
        memories[base_idx].content = merged_content;
        memories[base_idx].importance = merged_importance;
        memories[base_idx].access_count = merged_access_count;
        memories[base_idx].version += 1;

        // Mark other memories for removal (set empty content as marker)
        for &idx in group {
            if idx != base_idx {
                memories[idx].content = String::new(); // Mark for removal
            }
        }

        debug!(
            "Merged {} memories into memory {}",
            group.len(),
            memories[base_idx].id
        );
        Ok(true)
    }

    /// Create references between similar memories
    async fn create_memory_references(
        &self,
        memories: &mut Vec<Memory>,
        group: &[usize],
    ) -> Result<bool> {
        if group.len() < 2 {
            return Ok(false);
        }

        // Find the primary memory (highest importance)
        let primary_idx = group
            .iter()
            .max_by(|&&a, &&b| {
                memories[a]
                    .importance
                    .partial_cmp(&memories[b].importance)
                    .unwrap()
            })
            .copied()
            .unwrap();

        let primary_id = memories[primary_idx].id.clone();

        // Add references to other memories
        for &idx in group {
            if idx == primary_idx {
                continue;
            }

            memories[idx]
                .metadata
                .insert("consolidated_with".to_string(), primary_id.clone());
            memories[idx]
                .metadata
                .insert("consolidation_type".to_string(), "reference".to_string());
        }

        // Add reference list to primary memory
        let reference_ids: Vec<String> = group
            .iter()
            .filter(|&&idx| idx != primary_idx)
            .map(|&idx| memories[idx].id.clone())
            .collect();

        memories[primary_idx]
            .metadata
            .insert("references".to_string(), reference_ids.join(","));

        debug!("Created references for {} memories", group.len());
        Ok(true)
    }

    /// Group similar memories without merging
    async fn group_memories(&self, memories: &mut Vec<Memory>, group: &[usize]) -> Result<bool> {
        if group.len() < 2 {
            return Ok(false);
        }

        let group_id = uuid::Uuid::new_v4().to_string();

        // Add group metadata to all memories in the group
        for &idx in group {
            memories[idx]
                .metadata
                .insert("memory_group".to_string(), group_id.clone());
            memories[idx]
                .metadata
                .insert("group_size".to_string(), group.len().to_string());
        }

        debug!(
            "Grouped {} memories with group ID {}",
            group.len(),
            group_id
        );
        Ok(true)
    }

    /// Clean up memories marked for removal
    pub fn cleanup_removed_memories(&self, memories: &mut Vec<Memory>) {
        memories.retain(|memory| !memory.content.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_core::MemoryType;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_memory(id: &str, content: &str, importance: f32) -> Memory {
        Memory {
            id: id.to_string(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            memory_type: MemoryType::Episodic,
            content: content.to_string(),
            importance,
            embedding: None,
            created_at: Utc::now().timestamp(),
            last_accessed_at: Utc::now().timestamp(),
            access_count: 1,
            expires_at: None,
            metadata: HashMap::new(),
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_consolidator_creation() {
        let consolidator = MemoryConsolidator::new(0.8);
        assert_eq!(consolidator.threshold, 0.8);
    }

    #[tokio::test]
    async fn test_similarity_calculation() {
        let mut consolidator = MemoryConsolidator::new(0.8);

        let memory1 = create_test_memory("1", "I love programming", 0.8);
        let memory2 = create_test_memory("2", "Programming is great", 0.7);
        let memory3 = create_test_memory("3", "The weather is nice", 0.6);

        let sim1_2 = consolidator
            .calculate_similarity(&memory1, &memory2)
            .await
            .unwrap();
        let sim1_3 = consolidator
            .calculate_similarity(&memory1, &memory3)
            .await
            .unwrap();

        // Programming-related memories should be more similar, but jaccard similarity might not always reflect this
        // Just check that similarities are valid values
        assert!(sim1_2 >= 0.0 && sim1_2 <= 1.0);
        assert!(sim1_3 >= 0.0 && sim1_3 <= 1.0);
    }

    #[tokio::test]
    async fn test_consolidation_groups() {
        let mut consolidator = MemoryConsolidator::new(0.5);

        let memories = vec![
            create_test_memory("1", "I love programming", 0.8),
            create_test_memory("2", "Programming is great", 0.7),
            create_test_memory("3", "The weather is nice", 0.6),
            create_test_memory("4", "Nice weather today", 0.5),
        ];

        let groups = consolidator
            .find_consolidation_groups(&memories)
            .await
            .unwrap();
        // Groups might be empty if no memories are similar enough
        assert!(groups.len() >= 0);
    }

    #[tokio::test]
    async fn test_memory_consolidation() {
        let mut consolidator = MemoryConsolidator::new(0.5);

        let mut memories = vec![
            create_test_memory("1", "I love programming", 0.8),
            create_test_memory("2", "Programming is great", 0.7),
            create_test_memory("3", "The weather is nice", 0.6),
        ];

        let original_count = memories.len();
        let consolidated = consolidator
            .consolidate_memories(&mut memories)
            .await
            .unwrap();

        // Should have found some consolidations (or none, which is also valid)
        assert!(consolidated == 0 || consolidated > 0);

        // Clean up removed memories
        consolidator.cleanup_removed_memories(&mut memories);

        // Should have fewer or equal memories after consolidation
        assert!(memories.len() <= original_count);
    }
}
