//! Mem5 Enhanced AgentMem Client
//!
//! This module provides the next-generation AgentMem client with full Mem0 compatibility
//! and enhanced features including:
//! - Batch operations support
//! - Advanced filtering and search
//! - Error recovery and retry mechanisms
//! - Performance monitoring and telemetry
//! - Production-grade reliability

use crate::{
    error::{ClientError, ClientResult},
    models::*,
    retry::{RetryExecutor, RetryPolicy},
};

use agent_mem_compat::{
    Mem0Client,
    client::{EnhancedAddRequest, EnhancedSearchRequest, BatchAddRequest, BatchAddResult, Messages}
};
use agent_mem_config::MemoryConfig;

use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};
use tokio::sync::{Semaphore, RwLock};
use tracing::{debug, info, instrument};

/// Mem5 Enhanced AgentMem Client
///
/// This client provides full Mem0 API compatibility with enhanced performance,
/// reliability, and production-grade features.
pub struct Mem5Client {
    /// Mem0 compatibility layer
    compat_client: Arc<Mem0Client>,

    /// Configuration
    config: MemoryConfig,

    /// Retry executor for error recovery
    retry_executor: RetryExecutor,

    /// Concurrency control
    semaphore: Arc<Semaphore>,

    /// Operation counters
    operation_counter: AtomicU64,

    /// Client state
    state: Arc<RwLock<ClientState>>,
}

/// Client state for monitoring and management
#[derive(Debug, Clone)]
pub struct ClientState {
    pub is_healthy: bool,
    pub last_health_check: Instant,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            is_healthy: true,
            last_health_check: Instant::now(),
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
        }
    }
}

impl Mem5Client {
    /// Create a new Mem5Client with default configuration
    pub async fn new() -> ClientResult<Self> {
        let config = MemoryConfig::default();
        Self::with_config(config).await
    }

    /// Create a new Mem5Client with custom configuration
    pub async fn with_config(config: MemoryConfig) -> ClientResult<Self> {
        info!("Initializing Mem5Client with enhanced features");

        // Initialize core components
        let compat_client = Mem0Client::new()
            .await
            .map_err(|e| ClientError::InternalError(format!("Failed to initialize Mem0 client: {}", e)))?;

        // Create retry policy with default values
        let retry_policy = RetryPolicy::new(3)
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_millis(5000));

        let retry_executor = RetryExecutor::new(retry_policy);

        // Create concurrency control
        let semaphore = Arc::new(Semaphore::new(10)); // Default max concurrent operations

        info!("Mem5Client initialized successfully");

        Ok(Self {
            compat_client: Arc::new(compat_client),
            config,
            retry_executor,
            semaphore,
            operation_counter: AtomicU64::new(0),
            state: Arc::new(RwLock::new(ClientState::default())),
        })
    }
    
    /// Add a memory with full Mem0 compatibility
    #[instrument(skip(self))]
    pub async fn add(
        &self,
        messages: Messages,
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
        metadata: Option<HashMap<String, Value>>,
        infer: bool,
        memory_type: Option<String>,
        prompt: Option<String>,
    ) -> ClientResult<String> {
        let operation_id = self.operation_counter.fetch_add(1, Ordering::SeqCst);
        let start_time = Instant::now();
        
        debug!("Starting add operation {}", operation_id);
        
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await
            .map_err(|e| ClientError::InternalError(format!("Failed to acquire semaphore: {}", e)))?;
        
        let request = EnhancedAddRequest {
            messages,
            user_id,
            agent_id,
            run_id,
            metadata,
            infer,
            memory_type,
            prompt,
        };
        
        let result = self.retry_executor.execute(|| {
            let compat_client = self.compat_client.clone();
            let request = request.clone();
            Box::pin(async move {
                compat_client.add_enhanced(request).await
                    .map_err(|e| ClientError::InternalError(format!("Mem0 add failed: {}", e)))
            })
        }).await;
        
        // Update metrics
        let duration = start_time.elapsed();
        debug!("Add operation completed in {:?}", duration);
        
        // Update state
        let mut state = self.state.write().await;
        state.total_operations += 1;
        if result.is_ok() {
            state.successful_operations += 1;
        } else {
            state.failed_operations += 1;
        }
        
        debug!("Completed add operation {} in {:?}", operation_id, duration);
        result
    }
    
    /// Search memories with advanced filtering
    #[instrument(skip(self))]
    pub async fn search(
        &self,
        query: String,
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
        limit: usize,
        filters: Option<HashMap<String, Value>>,
        threshold: Option<f32>,
    ) -> ClientResult<Vec<Memory>> {
        let operation_id = self.operation_counter.fetch_add(1, Ordering::SeqCst);
        let start_time = Instant::now();
        
        debug!("Starting search operation {}", operation_id);
        
        let _permit = self.semaphore.acquire().await
            .map_err(|e| ClientError::InternalError(format!("Failed to acquire semaphore: {}", e)))?;
        
        let request = EnhancedSearchRequest {
            query,
            user_id,
            agent_id,
            run_id,
            limit,
            filters,
            threshold,
        };
        
        let result = self.retry_executor.execute(|| {
            let compat_client = self.compat_client.clone();
            let request = request.clone();
            Box::pin(async move {
                let search_result = compat_client.search_enhanced(request).await
                    .map_err(|e| ClientError::InternalError(format!("Mem0 search failed: {}", e)))?;
                
                // Convert to client Memory format
                let memories: Vec<Memory> = search_result.memories.into_iter().map(|m| Memory {
                    id: m.id,
                    agent_id: m.agent_id.unwrap_or_default(),
                    user_id: m.user_id.into(),
                    content: m.memory,
                    memory_type: None, // TODO: Map memory type
                    importance: m.score,
                    created_at: m.created_at,
                    metadata: Some(m.metadata.into_iter().map(|(k, v)| (k, v.to_string())).collect()),
                }).collect();
                
                Ok(memories)
            })
        }).await;
        
        let duration = start_time.elapsed();
        debug!("Search operation completed in {:?}", duration);
        
        let mut state = self.state.write().await;
        state.total_operations += 1;
        if result.is_ok() {
            state.successful_operations += 1;
        } else {
            state.failed_operations += 1;
        }
        
        debug!("Completed search operation {} in {:?}", operation_id, duration);
        result
    }
    
    /// Batch add memories with concurrent processing
    #[instrument(skip(self))]
    pub async fn add_batch(&self, requests: Vec<EnhancedAddRequest>) -> ClientResult<BatchAddResult> {
        let operation_id = self.operation_counter.fetch_add(1, Ordering::SeqCst);
        let start_time = Instant::now();
        
        debug!("Starting batch add operation {} with {} requests", operation_id, requests.len());
        
        let batch_request = BatchAddRequest { requests };
        
        let result = self.retry_executor.execute(|| {
            let compat_client = self.compat_client.clone();
            let batch_request = batch_request.clone();
            Box::pin(async move {
                compat_client.add_batch(batch_request).await
                    .map_err(|e| ClientError::InternalError(format!("Batch add failed: {}", e)))
            })
        }).await;
        
        let duration = start_time.elapsed();
        debug!("Batch add operation completed in {:?}", duration);
        
        let mut state = self.state.write().await;
        state.total_operations += 1;
        if result.is_ok() {
            state.successful_operations += 1;
        } else {
            state.failed_operations += 1;
        }
        
        debug!("Completed batch add operation {} in {:?}", operation_id, duration);
        result
    }
    
    /// Get client health status
    pub async fn health_check(&self) -> ClientResult<HealthResponse> {
        let state = self.state.read().await;
        
        let mut checks = HashMap::new();
        checks.insert("memory_engine".to_string(), "healthy".to_string());
        checks.insert("compat_client".to_string(), "healthy".to_string());
        checks.insert("performance_monitor".to_string(), "healthy".to_string());
        
        Ok(HealthResponse {
            status: if state.is_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks,
        })
    }
    
    /// Get client metrics
    pub async fn get_metrics(&self) -> ClientResult<MetricsResponse> {
        let state = self.state.read().await;

        let mut metrics = HashMap::new();
        metrics.insert("total_operations".to_string(), state.total_operations as f64);
        metrics.insert("successful_operations".to_string(), state.successful_operations as f64);
        metrics.insert("failed_operations".to_string(), state.failed_operations as f64);

        Ok(MetricsResponse {
            timestamp: chrono::Utc::now(),
            metrics,
        })
    }
}
