//! AgentMem Performance Optimization Module
//! 
//! This module provides performance optimization components including:
//! - Async batch processing
//! - Memory pools and object pools
//! - Multi-level caching
//! - Query optimization
//! - Concurrency control

pub mod batch;
pub mod cache;
pub mod pool;
pub mod metrics;
pub mod concurrency;
pub mod query;

// Re-export main types
pub use batch::{BatchProcessor, BatchConfig, BatchResult};
pub use cache::{CacheManager, CacheConfig, CacheStats};
pub use pool::{ObjectPool, MemoryPool, PoolConfig};
pub use metrics::{PerformanceMetrics, MetricsCollector};
pub use concurrency::{ConcurrencyManager, ConcurrencyConfig};
pub use query::{QueryOptimizer, QueryPlan, OptimizationHint};

use agent_mem_traits::{Result, AgentMemError};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Batch processing configuration
    pub batch: BatchConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Object pool configuration
    pub pool: PoolConfig,
    /// Concurrency configuration
    pub concurrency: ConcurrencyConfig,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable query optimization
    pub enable_query_optimization: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch: BatchConfig::default(),
            cache: CacheConfig::default(),
            pool: PoolConfig::default(),
            concurrency: ConcurrencyConfig::default(),
            enable_metrics: true,
            enable_query_optimization: true,
        }
    }
}

/// Main performance manager
pub struct PerformanceManager {
    config: PerformanceConfig,
    batch_processor: Arc<BatchProcessor>,
    cache_manager: Arc<CacheManager>,
    object_pool: Arc<ObjectPool>,
    memory_pool: Arc<MemoryPool>,
    metrics_collector: Arc<MetricsCollector>,
    concurrency_manager: Arc<ConcurrencyManager>,
    query_optimizer: Arc<QueryOptimizer>,
}

impl PerformanceManager {
    /// Create a new performance manager
    pub async fn new(config: PerformanceConfig) -> Result<Self> {
        let batch_processor = Arc::new(BatchProcessor::new(config.batch.clone()).await?);
        let cache_manager = Arc::new(CacheManager::new(config.cache.clone()).await?);
        let object_pool = Arc::new(ObjectPool::new(config.pool.clone())?);
        let memory_pool = Arc::new(MemoryPool::new(config.pool.clone())?);
        let metrics_collector = Arc::new(MetricsCollector::new(config.enable_metrics)?);
        let concurrency_manager = Arc::new(ConcurrencyManager::new(config.concurrency.clone())?);
        let query_optimizer = Arc::new(QueryOptimizer::new(config.enable_query_optimization)?);

        Ok(Self {
            config,
            batch_processor,
            cache_manager,
            object_pool,
            memory_pool,
            metrics_collector,
            concurrency_manager,
            query_optimizer,
        })
    }

    /// Get batch processor
    pub fn batch_processor(&self) -> Arc<BatchProcessor> {
        Arc::clone(&self.batch_processor)
    }

    /// Get cache manager
    pub fn cache_manager(&self) -> Arc<CacheManager> {
        Arc::clone(&self.cache_manager)
    }

    /// Get object pool
    pub fn object_pool(&self) -> Arc<ObjectPool> {
        Arc::clone(&self.object_pool)
    }

    /// Get memory pool
    pub fn memory_pool(&self) -> Arc<MemoryPool> {
        Arc::clone(&self.memory_pool)
    }

    /// Get metrics collector
    pub fn metrics_collector(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.metrics_collector)
    }

    /// Get concurrency manager
    pub fn concurrency_manager(&self) -> Arc<ConcurrencyManager> {
        Arc::clone(&self.concurrency_manager)
    }

    /// Get query optimizer
    pub fn query_optimizer(&self) -> Arc<QueryOptimizer> {
        Arc::clone(&self.query_optimizer)
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> Result<PerformanceStats> {
        let cache_stats = self.cache_manager.get_stats().await?;
        let batch_stats = self.batch_processor.get_stats().await?;
        let pool_stats = self.object_pool.get_stats()?;
        let memory_stats = self.memory_pool.get_stats()?;
        let concurrency_stats = self.concurrency_manager.get_stats().await?;

        Ok(PerformanceStats {
            cache: cache_stats,
            batch: batch_stats,
            pool: pool_stats,
            memory: memory_stats,
            concurrency: concurrency_stats,
        })
    }

    /// Shutdown the performance manager
    pub async fn shutdown(&self) -> Result<()> {
        self.batch_processor.shutdown().await?;
        self.cache_manager.shutdown().await?;
        self.metrics_collector.shutdown().await?;
        Ok(())
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub cache: CacheStats,
    pub batch: BatchResult,
    pub pool: PoolStats,
    pub memory: MemoryStats,
    pub concurrency: ConcurrencyStats,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_objects: usize,
    pub available_objects: usize,
    pub borrowed_objects: usize,
    pub created_objects: u64,
    pub recycled_objects: u64,
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_used: usize,
    pub total_free: usize,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

/// Concurrency statistics
#[derive(Debug, Clone)]
pub struct ConcurrencyStats {
    pub active_tasks: usize,
    pub queued_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_task_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_manager_creation() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_performance_stats() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config).await.unwrap();
        let stats = manager.get_stats().await;
        assert!(stats.is_ok());
    }

    #[tokio::test]
    async fn test_performance_manager_shutdown() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config).await.unwrap();
        let result = manager.shutdown().await;
        assert!(result.is_ok());
    }
}
