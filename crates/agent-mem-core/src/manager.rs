//! Core memory manager implementation

use crate::{
    history::{HistoryConfig, MemoryHistory},
    lifecycle::{LifecycleConfig, MemoryLifecycle},
    operations::{InMemoryOperations, MemoryOperations},
    types::{Memory, MemoryQuery, MemorySearchResult, MemoryStats, MemoryType},
};
use agent_mem_config::MemoryConfig;
use agent_mem_traits::{
    AgentMemError, HistoryEntry, MemoryEvent, MemoryItem, MemoryProvider, Message, Result, Session,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core memory manager
pub struct MemoryManager {
    /// Memory operations backend
    operations: Arc<RwLock<Box<dyn MemoryOperations + Send + Sync>>>,
    /// Memory lifecycle manager
    lifecycle: Arc<RwLock<MemoryLifecycle>>,
    /// Memory history tracker
    history: Arc<RwLock<MemoryHistory>>,
    /// Configuration
    config: MemoryConfig,
}

impl MemoryManager {
    /// Create a new memory manager with default configuration
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    /// Create a new memory manager with custom configuration
    pub fn with_config(config: MemoryConfig) -> Self {
        let operations: Box<dyn MemoryOperations + Send + Sync> =
            Box::new(InMemoryOperations::new());
        let lifecycle = MemoryLifecycle::with_default_config();
        let history = MemoryHistory::with_default_config();

        Self {
            operations: Arc::new(RwLock::new(operations)),
            lifecycle: Arc::new(RwLock::new(lifecycle)),
            history: Arc::new(RwLock::new(history)),
            config,
        }
    }

    /// Create a new memory manager with custom lifecycle and history configs
    pub fn with_custom_configs(
        memory_config: MemoryConfig,
        lifecycle_config: LifecycleConfig,
        history_config: HistoryConfig,
    ) -> Self {
        let operations: Box<dyn MemoryOperations + Send + Sync> =
            Box::new(InMemoryOperations::new());
        let lifecycle = MemoryLifecycle::new(lifecycle_config);
        let history = MemoryHistory::new(history_config);

        Self {
            operations: Arc::new(RwLock::new(operations)),
            lifecycle: Arc::new(RwLock::new(lifecycle)),
            history: Arc::new(RwLock::new(history)),
            config: memory_config,
        }
    }

    /// Add a new memory
    pub async fn add_memory(
        &self,
        agent_id: String,
        user_id: Option<String>,
        content: String,
        memory_type: Option<MemoryType>,
        importance: Option<f32>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        let mut memory = Memory::new(
            agent_id,
            user_id,
            memory_type.unwrap_or(MemoryType::Episodic),
            content,
            importance.unwrap_or(0.5),
        );

        if let Some(metadata) = metadata {
            for (key, value) in metadata {
                memory.add_metadata(key, value);
            }
        }

        // Register with lifecycle manager
        {
            let mut lifecycle = self.lifecycle.write().await;
            lifecycle.register_memory(&memory)?;
        }

        // Record creation in history
        {
            let mut history = self.history.write().await;
            history.record_creation(&memory)?;
        }

        // Store the memory
        let mut operations = self.operations.write().await;
        operations.create_memory(memory).await
    }

    /// Get a memory by ID
    pub async fn get_memory(&self, memory_id: &str) -> Result<Option<Memory>> {
        // Check if memory is accessible
        {
            let lifecycle = self.lifecycle.read().await;
            if !lifecycle.is_accessible(memory_id) {
                return Ok(None);
            }
        }

        // Get the memory first
        let mut memory = {
            let operations = self.operations.read().await;
            operations.get_memory(memory_id).await?
        };

        if let Some(ref mut mem) = memory {
            // Record access
            mem.access();

            // Update lifecycle
            {
                let mut lifecycle = self.lifecycle.write().await;
                lifecycle.record_access(memory_id)?;
            }

            // Record in history (if enabled)
            {
                let mut history = self.history.write().await;
                history.record_access(mem)?;
            }

            // Update the memory in storage
            {
                let mut operations = self.operations.write().await;
                operations.update_memory(mem.clone()).await?;
            }
        }

        Ok(memory)
    }

    /// Update an existing memory
    pub async fn update_memory(
        &self,
        memory_id: &str,
        new_content: Option<String>,
        new_importance: Option<f32>,
        new_metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<()> {
        let operations = self.operations.read().await;
        let mut memory = operations
            .get_memory(memory_id)
            .await?
            .ok_or_else(|| AgentMemError::memory_error("Memory not found"))?;

        let old_content = memory.content.clone();
        let old_importance = memory.importance;
        let old_version = memory.version;

        // Update fields
        if let Some(content) = new_content {
            memory.update_content(content);
        }

        if let Some(importance) = new_importance {
            memory.importance = importance.clamp(0.0, 1.0);
        }

        if let Some(metadata) = new_metadata {
            for (key, value) in metadata {
                memory.add_metadata(key, value);
            }
        }

        // Record changes in history
        {
            let mut history = self.history.write().await;

            if memory.content != old_content {
                history.record_content_update(&memory, &old_content, None)?;
            }

            if memory.importance != old_importance {
                history.record_importance_change(&memory, old_importance)?;
            }
        }

        // Record lifecycle update
        {
            let mut lifecycle = self.lifecycle.write().await;
            lifecycle.record_update(memory_id, old_version, memory.version)?;
        }

        // Update in storage
        drop(operations);
        let mut operations = self.operations.write().await;
        operations.update_memory(memory).await
    }

    /// Delete a memory
    pub async fn delete_memory(&self, memory_id: &str) -> Result<bool> {
        // Mark as deleted in lifecycle
        {
            let mut lifecycle = self.lifecycle.write().await;
            lifecycle.delete_memory(memory_id)?;
        }

        // Delete from storage
        let mut operations = self.operations.write().await;
        operations.delete_memory(memory_id).await
    }

    /// Search memories
    pub async fn search_memories(&self, query: MemoryQuery) -> Result<Vec<MemorySearchResult>> {
        let operations = self.operations.read().await;
        operations.search_memories(query).await
    }

    /// Get all memories for an agent
    pub async fn get_agent_memories(
        &self,
        agent_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Memory>> {
        let operations = self.operations.read().await;
        operations.get_agent_memories(agent_id, limit).await
    }

    /// Get memories by type
    pub async fn get_memories_by_type(
        &self,
        agent_id: &str,
        memory_type: MemoryType,
    ) -> Result<Vec<Memory>> {
        let operations = self.operations.read().await;
        operations.get_memories_by_type(agent_id, memory_type).await
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self, agent_id: Option<&str>) -> Result<MemoryStats> {
        let operations = self.operations.read().await;
        operations.get_memory_stats(agent_id).await
    }

    /// Apply automatic lifecycle policies
    pub async fn apply_auto_policies(&self) -> Result<Vec<String>> {
        let operations = self.operations.read().await;
        let all_memories = operations.get_agent_memories("", None).await?; // Get all memories
        drop(operations);

        let mut lifecycle = self.lifecycle.write().await;
        lifecycle.apply_auto_policies(&all_memories)
    }

    /// Clean up expired memories and old history
    pub async fn cleanup(&self) -> Result<(usize, usize)> {
        // Clean up history
        let history_cleaned = {
            let mut history = self.history.write().await;
            history.cleanup_old_entries()
        };

        // Clean up lifecycle events
        let lifecycle_cleaned = {
            let mut lifecycle = self.lifecycle.write().await;
            lifecycle.cleanup_old_events(30 * 24 * 3600); // 30 days
            0 // Return 0 for now, could implement actual cleanup count
        };

        Ok((history_cleaned, lifecycle_cleaned))
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MemoryProvider for MemoryManager {
    async fn add(&self, messages: &[Message], session: &Session) -> Result<Vec<MemoryItem>> {
        let mut results = Vec::new();

        for message in messages {
            let memory_id = self
                .add_memory(
                    session
                        .agent_id
                        .clone()
                        .unwrap_or_else(|| "default".to_string()),
                    session.user_id.clone(),
                    message.content.clone(),
                    None, // Use default memory type
                    None, // Use default importance
                    None, // No additional metadata
                )
                .await?;

            if let Some(memory) = self.get_memory(&memory_id).await? {
                results.push(memory.into());
            }
        }

        Ok(results)
    }

    async fn get(&self, memory_id: &str) -> Result<Option<MemoryItem>> {
        let memory = self.get_memory(memory_id).await?;
        Ok(memory.map(|m| m.into()))
    }

    async fn search(
        &self,
        query: &str,
        session: &Session,
        limit: usize,
    ) -> Result<Vec<MemoryItem>> {
        let mut memory_query = MemoryQuery::new(
            session
                .agent_id
                .clone()
                .unwrap_or_else(|| "default".to_string()),
        )
        .with_text_query(query.to_string())
        .with_limit(limit);

        if let Some(ref user_id) = session.user_id {
            memory_query = memory_query.with_user_id(user_id.clone());
        }

        let results = self.search_memories(memory_query).await?;
        Ok(results.into_iter().map(|r| r.memory.into()).collect())
    }

    async fn update(&self, memory_id: &str, data: &str) -> Result<()> {
        self.update_memory(
            memory_id,
            Some(data.to_string()),
            None, // Don't update importance through this interface
            None, // No metadata updates
        )
        .await
    }

    async fn delete(&self, memory_id: &str) -> Result<()> {
        self.delete_memory(memory_id).await?;
        Ok(())
    }

    async fn history(&self, memory_id: &str) -> Result<Vec<HistoryEntry>> {
        let history = self.history.read().await;
        if let Some(entries) = history.get_memory_history(memory_id) {
            let items: Vec<HistoryEntry> = entries
                .iter()
                .map(|entry| {
                    let event = match entry.change_type {
                        crate::history::ChangeType::Created => MemoryEvent::Create,
                        crate::history::ChangeType::ContentUpdated
                        | crate::history::ChangeType::ImportanceChanged
                        | crate::history::ChangeType::MetadataUpdated => MemoryEvent::Update,
                        crate::history::ChangeType::Deprecated => MemoryEvent::Delete,
                        _ => MemoryEvent::Update,
                    };

                    HistoryEntry {
                        id: format!("{}_{}", entry.memory_id, entry.version),
                        memory_id: entry.memory_id.clone(),
                        event,
                        timestamp: chrono::DateTime::from_timestamp(entry.timestamp, 0)
                            .unwrap_or_else(|| chrono::Utc::now()),
                        data: Some(serde_json::json!({
                            "content": entry.content,
                            "change_type": entry.change_type.to_string(),
                            "version": entry.version
                        })),
                    }
                })
                .collect();
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_all(&self, session: &Session) -> Result<Vec<MemoryItem>> {
        let agent_id = session.agent_id.as_deref().unwrap_or("default");
        let memories = self.get_agent_memories(agent_id, None).await?;
        Ok(memories.into_iter().map(|m| m.into()).collect())
    }

    async fn reset(&self) -> Result<()> {
        // This is a destructive operation, typically used for testing
        // For now, we'll just return Ok since we don't have a reset implementation
        // TODO: Implement actual reset functionality if needed
        Ok(())
    }
}
