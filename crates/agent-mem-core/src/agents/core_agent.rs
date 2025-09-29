//! Core Memory Agent
//!
//! This agent specializes in managing core memories - persistent memory blocks
//! that form the foundation of the agent's identity and context.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::agents::{
    AgentConfig, AgentContext, AgentError, AgentResult, AgentStats, BaseAgent, MemoryAgent,
};
use crate::coordination::{
    AgentMessage, CoordinationError, CoordinationResult, TaskRequest, TaskResponse,
};
use crate::types::MemoryType;

/// Core Memory Agent
///
/// Specializes in handling core memories - persistent memory blocks that define
/// the agent's identity, persona, and fundamental context.
pub struct CoreAgent {
    /// Base agent functionality
    base: BaseAgent,
    /// Agent context
    context: Arc<RwLock<AgentContext>>,
    /// Initialization status
    initialized: bool,
}

impl CoreAgent {
    /// Create a new core memory agent
    pub fn new(agent_id: String) -> Self {
        let config = AgentConfig::new(
            agent_id,
            vec![MemoryType::Core],
            5, // max concurrent tasks (lower for core memory)
        );

        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            initialized: false,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AgentConfig) -> Self {
        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            initialized: false,
        }
    }

    /// Handle core memory block creation
    async fn handle_create_block(&self, parameters: Value) -> AgentResult<Value> {
        let label = parameters
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'label' parameter".to_string())
            })?;

        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'content' parameter".to_string())
            })?;

        let block_type = parameters
            .get("block_type")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        // TODO: Integrate with actual core memory manager
        let response = serde_json::json!({
            "success": true,
            "block_id": uuid::Uuid::new_v4().to_string(),
            "label": label,
            "content": content,
            "block_type": block_type,
            "message": "Core memory block created successfully"
        });

        log::info!("Core agent: Created memory block '{}'", label);
        Ok(response)
    }

    /// Handle core memory block reading
    async fn handle_read_block(&self, parameters: Value) -> AgentResult<Value> {
        let block_id = parameters.get("block_id").and_then(|v| v.as_str());
        let label = parameters.get("label").and_then(|v| v.as_str());

        if block_id.is_none() && label.is_none() {
            return Err(AgentError::InvalidParameters(
                "Missing 'block_id' or 'label' parameter".to_string(),
            ));
        }

        // TODO: Integrate with actual core memory retrieval
        let response = serde_json::json!({
            "success": true,
            "block": {
                "id": block_id.unwrap_or("unknown"),
                "label": label.unwrap_or("unknown"),
                "content": "Sample core memory content",
                "block_type": "general",
                "created_at": chrono::Utc::now().to_rfc3339(),
                "updated_at": chrono::Utc::now().to_rfc3339()
            }
        });

        log::info!("Core agent: Read memory block");
        Ok(response)
    }

    /// Handle core memory block update
    async fn handle_update_block(&self, parameters: Value) -> AgentResult<Value> {
        let block_id = parameters
            .get("block_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'block_id' parameter".to_string())
            })?;

        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'content' parameter".to_string())
            })?;

        // TODO: Integrate with actual core memory update
        let response = serde_json::json!({
            "success": true,
            "block_id": block_id,
            "content": content,
            "message": "Core memory block updated successfully"
        });

        log::info!("Core agent: Updated memory block {}", block_id);
        Ok(response)
    }

    /// Handle core memory block deletion
    async fn handle_delete_block(&self, parameters: Value) -> AgentResult<Value> {
        let block_id = parameters
            .get("block_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'block_id' parameter".to_string())
            })?;

        // TODO: Integrate with actual core memory deletion
        let response = serde_json::json!({
            "success": true,
            "block_id": block_id,
            "message": "Core memory block deleted successfully"
        });

        log::info!("Core agent: Deleted memory block {}", block_id);
        Ok(response)
    }

    /// Handle core memory search
    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'query' parameter".to_string())
            })?;

        let block_type = parameters.get("block_type").and_then(|v| v.as_str());

        // TODO: Integrate with actual core memory search
        let response = serde_json::json!({
            "success": true,
            "results": [],
            "total_count": 0,
            "query": query,
            "block_type": block_type
        });

        log::info!(
            "Core agent: Searched for '{}' in block type: {:?}",
            query,
            block_type
        );
        Ok(response)
    }

    /// Handle memory compilation (render all blocks as context)
    async fn handle_compile(&self, _parameters: Value) -> AgentResult<Value> {
        // TODO: Integrate with actual core memory compilation
        let response = serde_json::json!({
            "success": true,
            "compiled_memory": "Compiled core memory context",
            "block_count": 0,
            "total_characters": 0
        });

        log::info!("Core agent: Compiled core memory");
        Ok(response)
    }
}

#[async_trait]
impl MemoryAgent for CoreAgent {
    fn agent_id(&self) -> &str {
        &self.base.config().agent_id
    }

    fn memory_types(&self) -> &[MemoryType] {
        &self.base.config().memory_types
    }

    async fn initialize(&mut self) -> CoordinationResult<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing Core Memory Agent: {}", self.agent_id());

        // TODO: Initialize core memory manager
        // TODO: Load existing memory blocks

        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("Shutting down Core Memory Agent: {}", self.agent_id());

        // TODO: Persist memory blocks
        // TODO: Clean up resources

        self.initialized = false;
        Ok(())
    }

    async fn execute_task(&mut self, task: TaskRequest) -> CoordinationResult<TaskResponse> {
        if !self.initialized {
            return Err(CoordinationError::InternalError(
                "Agent not initialized".to_string(),
            ));
        }

        let start_time = Instant::now();

        // Update context with current task
        {
            let mut context = self.context.write().await;
            context.current_task = Some(task.clone());
            context.stats.active_tasks += 1;
        }

        // Execute the task based on operation type
        let result = match task.operation.as_str() {
            "create_block" => self.handle_create_block(task.parameters).await,
            "read_block" => self.handle_read_block(task.parameters).await,
            "update_block" => self.handle_update_block(task.parameters).await,
            "delete_block" => self.handle_delete_block(task.parameters).await,
            "search" => self.handle_search(task.parameters).await,
            "compile" => self.handle_compile(task.parameters).await,
            _ => Err(AgentError::InvalidParameters(format!(
                "Unknown operation: {}",
                task.operation
            ))),
        };

        let execution_time = start_time.elapsed();

        // Update context and statistics
        {
            let mut context = self.context.write().await;
            context.current_task = None;
            context.stats.active_tasks = context.stats.active_tasks.saturating_sub(1);
            context
                .stats
                .update_task_completion(result.is_ok(), execution_time.as_millis() as f64);
        }

        // Create response
        match result {
            Ok(data) => Ok(TaskResponse::success(
                task.task_id,
                data,
                execution_time,
                self.agent_id().to_string(),
            )),
            Err(error) => Ok(TaskResponse::error(
                task.task_id,
                error.to_string(),
                execution_time,
                self.agent_id().to_string(),
            )),
        }
    }

    async fn handle_message(&mut self, message: AgentMessage) -> CoordinationResult<()> {
        log::debug!("Core agent received message: {:?}", message.message_type);

        match message.message_type {
            crate::coordination::MessageType::TaskRequest => Ok(()),
            crate::coordination::MessageType::HealthCheck => Ok(()),
            _ => {
                log::warn!(
                    "Core agent received unknown message type: {:?}",
                    message.message_type
                );
                Ok(())
            }
        }
    }

    async fn get_stats(&self) -> AgentStats {
        self.context.read().await.stats.clone()
    }

    async fn health_check(&self) -> bool {
        self.initialized
    }

    async fn current_load(&self) -> usize {
        self.context.read().await.stats.active_tasks
    }

    async fn can_accept_task(&self) -> bool {
        if !self.initialized {
            return false;
        }

        let context = self.context.read().await;
        context.stats.active_tasks < context.config.max_concurrent_tasks
    }
}
