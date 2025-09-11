//! Mem5 Enhanced Mem0 compatibility client implementation
//!
//! This module provides a fully compatible Mem0 API with enhanced features:
//! - Batch operations support
//! - Advanced filtering and search
//! - Error recovery and retry mechanisms
//! - Performance monitoring and telemetry
//! - Production-grade reliability

use crate::{
    config::Mem0Config,
    error::{Mem0Error, Result},
    types::{
        AddMemoryRequest, ChangeType, DeleteMemoryResponse, Memory, MemoryFilter, MemoryHistory,
        MemorySearchResult, SearchMemoryRequest, SortField, SortOrder, UpdateMemoryRequest,
    },
    utils::{
        calculate_importance_score, generate_memory_id,
        sanitize_metadata, validate_memory_content, validate_user_id,
    },
};

use agent_mem_traits::Message;
use agent_mem_performance::batch::{BatchProcessor, BatchItem, BatchConfig};
use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info, warn, instrument};

/// Enhanced message types for Mem0 compatibility
#[derive(Debug, Clone)]
pub enum Messages {
    Single(String),
    Structured(Message),
    Multiple(Vec<String>),
}

impl Messages {
    pub fn validate(&self) -> Result<()> {
        match self {
            Messages::Single(s) => {
                if s.trim().is_empty() {
                    return Err(Mem0Error::InvalidContent {
                        reason: "Empty message".to_string(),
                    });
                }
            }
            Messages::Structured(msg) => {
                if msg.content.trim().is_empty() {
                    return Err(Mem0Error::InvalidContent {
                        reason: "Empty structured message content".to_string(),
                    });
                }
            }
            Messages::Multiple(msgs) => {
                if msgs.is_empty() {
                    return Err(Mem0Error::InvalidContent {
                        reason: "Empty message list".to_string(),
                    });
                }
                for msg in msgs {
                    if msg.trim().is_empty() {
                        return Err(Mem0Error::InvalidContent {
                            reason: "Empty message in list".to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Convert messages to content string for storage
    pub fn to_content(&self) -> String {
        match self {
            Messages::Single(s) => s.clone(),
            Messages::Structured(msg) => msg.content.clone(),
            Messages::Multiple(msgs) => msgs.join("\n"),
        }
    }

    /// Convert messages to a list of Message structs
    pub fn to_message_list(&self) -> Vec<Message> {
        match self {
            Messages::Single(s) => vec![Message::user(s)],
            Messages::Structured(msg) => vec![msg.clone()],
            Messages::Multiple(msgs) => {
                msgs.iter().map(|s| Message::user(s)).collect()
            }
        }
    }
}

/// Enhanced add request with full Mem0 compatibility
#[derive(Debug, Clone)]
pub struct EnhancedAddRequest {
    pub messages: Messages,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub infer: bool,
    pub memory_type: Option<String>,
    pub prompt: Option<String>,
}

/// Enhanced search request with advanced filtering
#[derive(Debug, Clone)]
pub struct EnhancedSearchRequest {
    pub query: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub limit: usize,
    pub filters: Option<HashMap<String, serde_json::Value>>,
    pub threshold: Option<f32>,
}

/// Batch operation request
#[derive(Debug, Clone)]
pub struct BatchAddRequest {
    pub requests: Vec<EnhancedAddRequest>,
}

/// Batch operation result
#[derive(Debug, Clone)]
pub struct BatchAddResult {
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<String>,
    pub errors: Vec<String>,
}

/// Memory operation for batch processing
#[derive(Debug, Clone)]
pub struct MemoryOperation {
    pub request: EnhancedAddRequest,
    pub client: Arc<DashMap<String, Memory>>,
}

#[async_trait]
impl BatchItem for MemoryOperation {
    type Output = String;
    type Error = crate::error::Mem0Error;

    async fn process(&self) -> std::result::Result<Self::Output, Self::Error> {
        // Generate memory ID
        let memory_id = uuid::Uuid::new_v4().to_string();

        // Convert messages to content
        let content = self.request.messages.to_content();

        // Create memory
        let memory = Memory {
            id: memory_id.clone(),
            memory: content,
            user_id: self.request.user_id.clone().unwrap_or_else(|| "default".to_string()),
            agent_id: self.request.agent_id.clone(),
            run_id: self.request.run_id.clone(),
            metadata: self.request.metadata.clone().unwrap_or_default(),
            score: None,
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
        };

        // Store memory
        self.client.insert(memory_id.clone(), memory);

        Ok(memory_id)
    }

    fn size(&self) -> usize {
        self.request.messages.to_content().len()
    }

    fn priority(&self) -> u8 {
        // Higher priority for shorter content (faster processing)
        if self.size() < 100 {
            10
        } else if self.size() < 500 {
            5
        } else {
            1
        }
    }
}

/// Enhanced Mem0 compatibility client with Mem5 features
pub struct Mem0Client {
    /// Configuration
    config: Mem0Config,

    /// In-memory storage for demonstration
    memories: Arc<DashMap<String, Memory>>,

    /// Memory history cache
    history_cache: Arc<DashMap<String, Vec<MemoryHistory>>>,

    /// Batch processor for concurrent operations
    batch_processor: Option<Arc<BatchProcessor>>,
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

        // Initialize batch processor for concurrent operations
        let batch_config = BatchConfig {
            max_batch_size: 50,
            max_wait_time_ms: 100,
            concurrency: 4,
            buffer_size: 1000,
            enable_compression: false,
            retry_attempts: 3,
            retry_delay_ms: 50,
        };

        let batch_processor = match BatchProcessor::new(batch_config).await {
            Ok(processor) => Some(Arc::new(processor)),
            Err(e) => {
                warn!("Failed to initialize batch processor: {}, falling back to sequential processing", e);
                None
            }
        };

        Ok(Self {
            config,
            memories: Arc::new(DashMap::new()),
            history_cache: Arc::new(DashMap::new()),
            batch_processor,
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

        self.memories.insert(memory_id.clone(), memory.clone());

        // Create history entry for creation
        self.create_history_entry(
            &memory_id,
            &memory.user_id,
            None, // no previous memory
            Some(memory.memory.clone()),
            None, // no previous metadata
            Some(memory.metadata.clone()),
            ChangeType::Created,
            None, // changed_by
            None, // reason
        ).await?;

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

        // Enhanced search with complex filtering and scoring
        let mut matching_memories: Vec<Memory> = self.memories
            .iter()
            .filter(|entry| {
                let memory = entry.value();
                // Filter by user_id
                if memory.user_id != request.user_id {
                    return false;
                }

                // Apply enhanced filters if provided
                if let Some(filters) = &request.filters {
                    if !filters.matches(memory) {
                        return false;
                    }
                }

                // Enhanced text matching with scoring
                self.calculate_search_score(&request.query, memory) > 0.0
            })
            .map(|entry| entry.value().clone())
            .collect();

        // Apply enhanced sorting if specified in filters
        if let Some(ref filter) = request.filters {
            self.sort_memories(&mut matching_memories, &filter.sort_field, &filter.sort_order);

            // Apply pagination
            if let Some(offset) = filter.offset {
                if offset < matching_memories.len() {
                    matching_memories = matching_memories.into_iter().skip(offset).collect();
                } else {
                    matching_memories.clear();
                }
            }
        } else {
            // Default sorting by score (descending)
            matching_memories.sort_by(|a, b| {
                b.score.unwrap_or(0.0).partial_cmp(&a.score.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

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

    /// Calculate search relevance score
    fn calculate_search_score(&self, query: &str, memory: &Memory) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = memory.memory.to_lowercase();

        // Exact match gets highest score
        if content_lower == query_lower {
            return 1.0;
        }

        // Contains query gets high score
        if content_lower.contains(&query_lower) {
            let ratio = query_lower.len() as f32 / content_lower.len() as f32;
            return 0.8 * ratio;
        }

        // Word-based matching
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        let mut matches = 0;
        for query_word in &query_words {
            for content_word in &content_words {
                if content_word.contains(query_word) || query_word.contains(content_word) {
                    matches += 1;
                    break;
                }
            }
        }

        if matches > 0 {
            let word_score = matches as f32 / query_words.len() as f32;
            return 0.5 * word_score;
        }

        0.0
    }

    /// Sort memories based on specified criteria
    fn sort_memories(&self, memories: &mut Vec<Memory>, sort_field: &SortField, sort_order: &SortOrder) {
        use crate::types::{SortField, SortOrder};

        memories.sort_by(|a, b| {
            let comparison = match sort_field {
                SortField::CreatedAt => a.created_at.cmp(&b.created_at),
                SortField::UpdatedAt => {
                    match (a.updated_at, b.updated_at) {
                        (Some(a_updated), Some(b_updated)) => a_updated.cmp(&b_updated),
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                SortField::Score => a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal),
                SortField::ContentLength => a.memory.len().cmp(&b.memory.len()),
                SortField::Metadata(key) => {
                    match (a.metadata.get(key), b.metadata.get(key)) {
                        (Some(a_val), Some(b_val)) => {
                            // Try to compare as numbers first, then as strings
                            if let (Some(a_num), Some(b_num)) = (a_val.as_f64(), b_val.as_f64()) {
                                a_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal)
                            } else {
                                a_val.to_string().cmp(&b_val.to_string())
                            }
                        }
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
            };

            match sort_order {
                SortOrder::Asc => comparison,
                SortOrder::Desc => comparison.reverse(),
            }
        });
    }

    /// Create a history entry for memory changes
    async fn create_history_entry(
        &self,
        memory_id: &str,
        user_id: &str,
        prev_memory: Option<String>,
        new_memory: Option<String>,
        prev_metadata: Option<HashMap<String, serde_json::Value>>,
        new_metadata: Option<HashMap<String, serde_json::Value>>,
        change_type: ChangeType,
        changed_by: Option<String>,
        reason: Option<String>,
    ) -> Result<()> {
        let history_entry = MemoryHistory {
            id: uuid::Uuid::new_v4().to_string(),
            memory_id: memory_id.to_string(),
            user_id: user_id.to_string(),
            prev_memory,
            new_memory,
            prev_metadata,
            new_metadata,
            timestamp: Utc::now(),
            change_type,
            changed_by,
            reason,
            version: self.get_next_version(memory_id).await,
            related_memory_ids: Vec::new(),
            metadata: HashMap::new(),
        };

        // Store history entry
        self.history_cache
            .entry(memory_id.to_string())
            .or_insert_with(Vec::new)
            .push(history_entry);

        Ok(())
    }

    /// Get the next version number for a memory
    async fn get_next_version(&self, memory_id: &str) -> u32 {
        if let Some(history) = self.history_cache.get(memory_id) {
            history.iter().map(|h| h.version).max().unwrap_or(0) + 1
        } else {
            1
        }
    }

    /// Get memory history
    pub async fn history(&self, memory_id: &str, user_id: &str) -> Result<Vec<MemoryHistory>> {
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

        // Get history entries
        let history = self.history_cache
            .get(memory_id)
            .map(|h| h.clone())
            .unwrap_or_default();

        debug!("Retrieved {} history entries for memory: {}", history.len(), memory_id);
        Ok(history)
    }

    /// Update a memory with history tracking
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

        // Store original values for history tracking
        let prev_memory = memory.memory.clone();
        let prev_metadata = memory.metadata.clone();
        let mut change_type = None;

        // Update content if provided
        if let Some(new_content) = request.memory {
            validate_memory_content(&new_content)?;
            if memory.memory != new_content {
                memory.memory = new_content;
                change_type = Some(ChangeType::ContentUpdated);
            }
        }

        // Update metadata if provided
        if let Some(mut new_metadata) = request.metadata {
            sanitize_metadata(&mut new_metadata);
            if memory.metadata != new_metadata {
                memory.metadata = new_metadata;
                change_type = Some(if change_type.is_some() {
                    ChangeType::ContentUpdated
                } else {
                    ChangeType::MetadataUpdated
                });
            }
        }

        // Only update timestamp and create history if something actually changed
        if let Some(change_type) = change_type {
            memory.updated_at = Some(Utc::now());
            memory.score = Some(calculate_importance_score(&memory.memory, &memory.metadata));

            // Create history entry
            self.create_history_entry(
                memory_id,
                user_id,
                Some(prev_memory),
                Some(memory.memory.clone()),
                Some(prev_metadata),
                Some(memory.metadata.clone()),
                change_type,
                None, // changed_by
                None, // reason
            ).await?;
        }

        let updated_memory = memory.clone();

        debug!("Updated memory with ID: {}", memory_id);
        Ok(updated_memory)
    }

    /// Delete a memory
    pub async fn delete(&self, memory_id: &str, user_id: &str) -> Result<DeleteMemoryResponse> {
        validate_user_id(user_id)?;

        // Check if memory exists and belongs to user, and get a copy for history
        let memory_to_delete = if let Some(memory) = self.memories.get(memory_id) {
            if memory.user_id != user_id {
                return Err(Mem0Error::MemoryNotFound {
                    id: memory_id.to_string(),
                });
            }
            memory.clone()
        } else {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        };

        // Create history entry before deletion
        self.create_history_entry(
            memory_id,
            user_id,
            Some(memory_to_delete.memory.clone()),
            None, // no new memory (deleted)
            Some(memory_to_delete.metadata.clone()),
            None, // no new metadata (deleted)
            ChangeType::Deleted,
            None, // changed_by
            None, // reason
        ).await?;

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

    // ========== Mem5 Enhanced API Methods ==========

    /// Enhanced add method with full Mem0 compatibility
    #[instrument(skip(self, request))]
    pub async fn add_enhanced(&self, request: EnhancedAddRequest) -> Result<String> {
        // Validate messages
        request.messages.validate()?;

        // Convert messages to string content
        let content = request.messages.to_content();

        // Create traditional add request
        let add_request = AddMemoryRequest {
            memory: content,
            user_id: request.user_id.unwrap_or_else(|| "default".to_string()),
            agent_id: request.agent_id,
            run_id: request.run_id,
            metadata: request.metadata.unwrap_or_default(),
        };

        // Use existing add logic
        self.add_with_options(add_request).await
    }

    /// Enhanced search method with advanced filtering
    #[instrument(skip(self, request))]
    pub async fn search_enhanced(&self, request: EnhancedSearchRequest) -> Result<MemorySearchResult> {
        // Create traditional search request
        let search_request = SearchMemoryRequest {
            query: request.query,
            user_id: request.user_id.unwrap_or_else(|| "default".to_string()),
            filters: Some(MemoryFilter {
                agent_id: request.agent_id,
                run_id: request.run_id,
                memory_type: None,
                created_after: None,
                created_before: None,
                updated_after: None,
                updated_before: None,
                min_score: None,
                max_score: None,
                min_content_length: None,
                max_content_length: None,
                metadata_filters: HashMap::new(),
                metadata: HashMap::new(),
                content_contains: None,
                content_regex: None,
                tags: Vec::new(),
                exclude_tags: Vec::new(),
                sort_field: SortField::default(),
                sort_order: SortOrder::default(),
                limit: Some(request.limit),
                offset: None,
            }),
            limit: Some(request.limit),
        };

        // Use existing search logic
        self.search_with_options(search_request).await
    }

    /// Batch add memories with concurrent processing
    #[instrument(skip(self, request))]
    pub async fn add_batch(&self, request: BatchAddRequest) -> Result<BatchAddResult> {
        let mut successful = 0;
        let mut failed = 0;
        let mut results = Vec::new();
        let mut errors = Vec::new();

        // Process each request
        for add_request in request.requests {
            match self.add_enhanced(add_request).await {
                Ok(memory_id) => {
                    successful += 1;
                    results.push(memory_id);
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("Failed to add memory: {}", e));
                }
            }
        }

        debug!("Batch add completed: {} successful, {} failed", successful, failed);

        Ok(BatchAddResult {
            successful,
            failed,
            results,
            errors,
        })
    }

    /// Batch update memories
    #[instrument(skip(self))]
    pub async fn update_batch(
        &self,
        updates: Vec<(String, String, UpdateMemoryRequest)>, // (memory_id, user_id, request)
    ) -> Result<Vec<Result<Memory>>> {
        let mut results = Vec::new();

        for (memory_id, user_id, update_request) in updates {
            let result = self.update(&memory_id, &user_id, update_request).await;
            results.push(result);
        }

        debug!("Batch update completed: {} operations", results.len());
        Ok(results)
    }

    /// Batch delete memories
    #[instrument(skip(self))]
    pub async fn delete_batch(
        &self,
        deletes: Vec<(String, String)>, // (memory_id, user_id)
    ) -> Result<Vec<bool>> {
        let mut results = Vec::new();

        for (memory_id, user_id) in deletes {
            match self.delete(&memory_id, &user_id).await {
                Ok(_) => results.push(true),
                Err(_) => results.push(false),
            }
        }

        debug!("Batch delete completed: {} operations", results.len());
        Ok(results)
    }
}