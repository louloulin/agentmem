//! Mem0 compatibility client implementation

use crate::{
    config::Mem0Config,
    error::{Mem0Error, Result},
    types::{
        AddMemoryRequest, DeleteMemoryResponse, Memory, MemoryFilter, MemoryHistory,
        MemorySearchResult, SearchMemoryRequest, UpdateMemoryRequest,
    },
    utils::{
        calculate_importance_score, generate_memory_id,
        sanitize_metadata, validate_memory_content, validate_user_id,
    },
};

// Simplified implementation without complex dependencies for now
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Mem0 compatibility client
pub struct Mem0Client {
    /// Configuration
    config: Mem0Config,

    /// In-memory storage for demonstration
    memories: Arc<DashMap<String, Memory>>,

    /// Memory history cache
    history_cache: Arc<DashMap<String, Vec<MemoryHistory>>>,
}

impl Mem0Client {
    /// Create a new Mem0Client with default configuration
    pub async fn new() -> Result<Self> {
        let config = Mem0Config::default();
        Self::with_config(config).await
    }
    
    /// Create a new Mem0Client with custom configuration
    pub async fn with_config(config: Mem0Config) -> Result<Self> {
        config.validate()?;

        info!("Initializing Mem0Client with config: {:?}", config);

        // For now, create a simple in-memory implementation
        // In a full implementation, this would initialize the actual AgentMem components

        info!("Mem0Client initialized successfully");

        Ok(Self {
            config,
            memories: Arc::new(DashMap::new()),
            history_cache: Arc::new(DashMap::new()),
        })
    }
    
    /// Add a new memory
    pub async fn add(
        &self,
        user_id: &str,
        memory: &str,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        self.add_with_options(AddMemoryRequest {
            memory: memory.to_string(),
            user_id: user_id.to_string(),
            agent_id: None,
            run_id: None,
            metadata: metadata.unwrap_or_default(),
        }).await
    }
    
    /// Add a new memory with full options
    pub async fn add_with_options(&self, request: AddMemoryRequest) -> Result<String> {
        validate_user_id(&request.user_id)?;
        validate_memory_content(&request.memory)?;

        let mut metadata = request.metadata;
        sanitize_metadata(&mut metadata);

        let memory_id = generate_memory_id();
        let importance = calculate_importance_score(&request.memory, &metadata);

        let memory = Memory {
            id: memory_id.clone(),
            memory: request.memory,
            user_id: request.user_id,
            agent_id: request.agent_id,
            run_id: request.run_id,
            metadata,
            score: Some(importance),
            created_at: Utc::now(),
            updated_at: None,
        };

        self.memories.insert(memory_id.clone(), memory);

        debug!("Added memory with ID: {}", memory_id);
        Ok(memory_id)
    }
    
    /// Search for memories
    pub async fn search(
        &self,
        query: &str,
        user_id: &str,
        filters: Option<MemoryFilter>,
    ) -> Result<MemorySearchResult> {
        self.search_with_options(SearchMemoryRequest {
            query: query.to_string(),
            user_id: user_id.to_string(),
            filters,
            limit: None,
        }).await
    }
    
    /// Search for memories with full options
    pub async fn search_with_options(&self, request: SearchMemoryRequest) -> Result<MemorySearchResult> {
        validate_user_id(&request.user_id)?;

        if request.query.is_empty() {
            return Err(Mem0Error::InvalidContent {
                reason: "Search query cannot be empty".to_string(),
            });
        }

        let limit = request.limit
            .or_else(|| request.filters.as_ref().and_then(|f| f.limit))
            .unwrap_or(self.config.memory.default_search_limit);

        // Simple text-based search for demonstration
        let query_lower = request.query.to_lowercase();
        let mut matching_memories: Vec<Memory> = self.memories
            .iter()
            .filter(|entry| {
                let memory = entry.value();
                // Filter by user_id
                if memory.user_id != request.user_id {
                    return false;
                }

                // Apply additional filters if provided
                if let Some(filters) = &request.filters {
                    if let Some(agent_id) = &filters.agent_id {
                        if memory.agent_id.as_ref() != Some(agent_id) {
                            return false;
                        }
                    }
                    if let Some(run_id) = &filters.run_id {
                        if memory.run_id.as_ref() != Some(run_id) {
                            return false;
                        }
                    }
                }

                // Simple text matching
                memory.memory.to_lowercase().contains(&query_lower)
            })
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by score (descending) and limit results
        matching_memories.sort_by(|a, b| {
            b.score.unwrap_or(0.0).partial_cmp(&a.score.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)
        });
        matching_memories.truncate(limit);

        let total = matching_memories.len();

        debug!("Found {} memories for query: {}", total, request.query);

        Ok(MemorySearchResult {
            memories: matching_memories,
            total,
            metadata: std::collections::HashMap::new(),
        })
    }
    
    /// Get a specific memory by ID
    pub async fn get(&self, memory_id: &str, user_id: &str) -> Result<Memory> {
        validate_user_id(user_id)?;

        let memory = self.memories
            .get(memory_id)
            .ok_or_else(|| Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            })?;

        // Check if the memory belongs to the user
        if memory.user_id != user_id {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        }

        Ok(memory.clone())
    }
    
    /// Update a memory
    pub async fn update(
        &self,
        memory_id: &str,
        user_id: &str,
        request: UpdateMemoryRequest,
    ) -> Result<Memory> {
        validate_user_id(user_id)?;

        // Get existing memory
        let mut memory = self.memories
            .get_mut(memory_id)
            .ok_or_else(|| Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            })?;

        // Check if the memory belongs to the user
        if memory.user_id != user_id {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        }

        // Update content if provided
        if let Some(new_content) = request.memory {
            validate_memory_content(&new_content)?;
            memory.memory = new_content;
        }

        // Update metadata if provided
        if let Some(mut new_metadata) = request.metadata {
            sanitize_metadata(&mut new_metadata);
            memory.metadata = new_metadata;
        }

        // Update timestamps and importance
        memory.updated_at = Some(Utc::now());
        memory.score = Some(calculate_importance_score(&memory.memory, &memory.metadata));

        let updated_memory = memory.clone();

        debug!("Updated memory with ID: {}", memory_id);
        Ok(updated_memory)
    }

    /// Delete a memory
    pub async fn delete(&self, memory_id: &str, user_id: &str) -> Result<DeleteMemoryResponse> {
        validate_user_id(user_id)?;

        // Check if memory exists and belongs to user
        if let Some(memory) = self.memories.get(memory_id) {
            if memory.user_id != user_id {
                return Err(Mem0Error::MemoryNotFound {
                    id: memory_id.to_string(),
                });
            }
        } else {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        }

        // Remove the memory
        self.memories.remove(memory_id);

        debug!("Deleted memory with ID: {}", memory_id);
        Ok(DeleteMemoryResponse {
            success: true,
            message: Some("Memory deleted successfully".to_string()),
        })
    }

    /// Get all memories for a user
    pub async fn get_all(&self, user_id: &str, filters: Option<MemoryFilter>) -> Result<Vec<Memory>> {
        validate_user_id(user_id)?;

        let limit = filters
            .as_ref()
            .and_then(|f| f.limit)
            .unwrap_or(1000); // Default large limit for get_all

        let mut memories: Vec<Memory> = self.memories
            .iter()
            .filter(|entry| {
                let memory = entry.value();
                // Filter by user_id
                if memory.user_id != user_id {
                    return false;
                }

                // Apply additional filters if provided
                if let Some(filters) = &filters {
                    if let Some(agent_id) = &filters.agent_id {
                        if memory.agent_id.as_ref() != Some(agent_id) {
                            return false;
                        }
                    }
                    if let Some(run_id) = &filters.run_id {
                        if memory.run_id.as_ref() != Some(run_id) {
                            return false;
                        }
                    }
                    if let Some(created_after) = &filters.created_after {
                        if memory.created_at < *created_after {
                            return false;
                        }
                    }
                    if let Some(created_before) = &filters.created_before {
                        if memory.created_at > *created_before {
                            return false;
                        }
                    }
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by creation time (newest first) and limit
        memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        memories.truncate(limit);

        debug!("Retrieved {} memories for user: {}", memories.len(), user_id);
        Ok(memories)
    }

    /// Get memory history for a specific memory
    pub async fn get_history(&self, memory_id: &str, user_id: &str) -> Result<Vec<MemoryHistory>> {
        validate_user_id(user_id)?;

        // Check if history is cached
        if let Some(history) = self.history_cache.get(memory_id) {
            return Ok(history.clone());
        }

        // For now, return empty history as AgentMem doesn't have built-in history tracking
        // In a full implementation, you would integrate with AgentMem's lifecycle events
        warn!("Memory history not implemented yet for memory: {}", memory_id);
        Ok(Vec::new())
    }

    /// Delete all memories for a user
    pub async fn delete_all(&self, user_id: &str) -> Result<DeleteMemoryResponse> {
        validate_user_id(user_id)?;

        // Collect memory IDs to delete
        let memory_ids: Vec<String> = self.memories
            .iter()
            .filter(|entry| entry.value().user_id == user_id)
            .map(|entry| entry.key().clone())
            .collect();

        let deleted_count = memory_ids.len();

        // Delete each memory
        for memory_id in memory_ids {
            self.memories.remove(&memory_id);
        }

        debug!("Deleted {} memories for user: {}", deleted_count, user_id);
        Ok(DeleteMemoryResponse {
            success: true,
            message: Some(format!("Deleted {} memories", deleted_count)),
        })
    }

    /// Get memory statistics for a user
    pub async fn get_stats(&self, user_id: &str) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        validate_user_id(user_id)?;

        let user_memories: Vec<Memory> = self.memories
            .iter()
            .filter(|entry| entry.value().user_id == user_id)
            .map(|entry| entry.value().clone())
            .collect();

        let total_memories = user_memories.len();
        let avg_importance = if total_memories > 0 {
            user_memories.iter().map(|m| m.score.unwrap_or(0.0)).sum::<f32>() / total_memories as f32
        } else {
            0.0
        };

        // Count memories by agent_id
        let mut agent_counts = std::collections::HashMap::new();
        for memory in &user_memories {
            let agent_id = memory.agent_id.as_deref().unwrap_or("default");
            *agent_counts.entry(agent_id.to_string()).or_insert(0) += 1;
        }

        let mut stats = std::collections::HashMap::new();
        stats.insert("total_memories".to_string(), serde_json::Value::Number(total_memories.into()));
        stats.insert("average_importance".to_string(), serde_json::Value::Number(
            serde_json::Number::from_f64(avg_importance as f64).unwrap_or_else(|| 0.into())
        ));
        stats.insert("agent_counts".to_string(), serde_json::to_value(agent_counts)?);

        debug!("Generated stats for user: {}", user_id);
        Ok(stats)
    }

    /// Reset the client (clear all data)
    pub async fn reset(&self) -> Result<()> {
        warn!("Reset operation requested - this will clear all data");

        // Clear all memories
        self.memories.clear();

        // Clear history cache
        self.history_cache.clear();

        info!("Client reset completed");
        Ok(())
    }
}


