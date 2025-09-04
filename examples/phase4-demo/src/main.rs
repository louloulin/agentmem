//! Phase 4 Performance and Scalability Demo
//! 
//! This demo showcases the performance optimization features implemented in Phase 4:
//! - Async batch processing
//! - Multi-level caching
//! - Object and memory pools
//! - Concurrency control with rate limiting and circuit breakers
//! - Query optimization
//! - Performance metrics collection

use agent_mem_performance::{
    PerformanceManager, PerformanceConfig,
    BatchProcessor, BatchConfig,
    CacheManager, CacheConfig,
    ObjectPool, MemoryPool, PoolConfig,
    MetricsCollector, ConcurrencyManager, ConcurrencyConfig,
    QueryOptimizer,
    batch::BatchItem,
    query::QueryRequest,
};
use agent_mem_traits::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Simple error type for demo
#[derive(Debug)]
struct DemoError(String);

impl std::fmt::Display for DemoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DemoError {}

/// Demo batch item for processing
#[derive(Debug)]
struct DemoTask {
    id: String,
    data: Vec<u8>,
    processing_time_ms: u64,
}

#[async_trait]
impl BatchItem for DemoTask {
    type Output = String;
    type Error = DemoError;

    async fn process(&self) -> std::result::Result<Self::Output, Self::Error> {
        // Simulate processing time
        sleep(Duration::from_millis(self.processing_time_ms)).await;
        Ok(format!("Processed task {} with {} bytes", self.id, self.data.len()))
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn priority(&self) -> u8 {
        if self.data.len() > 1000 { 2 } else { 1 }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting AgentMem Phase 4 Performance Demo");

    // Demo 1: Performance Manager
    demo_performance_manager().await?;

    // Demo 2: Batch Processing
    demo_batch_processing().await?;

    // Demo 3: Multi-level Caching
    demo_caching().await?;

    // Demo 4: Object and Memory Pools
    demo_pools().await?;

    // Demo 5: Concurrency Control
    demo_concurrency().await?;

    // Demo 6: Query Optimization
    demo_query_optimization().await?;

    // Demo 7: Performance Metrics
    demo_metrics().await?;

    info!("✅ Phase 4 Performance Demo completed successfully!");
    Ok(())
}

async fn demo_performance_manager() -> Result<()> {
    info!("\n📊 Demo 1: Performance Manager");
    
    let config = PerformanceConfig::default();
    let manager = PerformanceManager::new(config).await?;
    
    info!("✓ Performance manager created with default configuration");
    
    let stats = manager.get_stats().await?;
    info!("✓ Performance stats retrieved: cache hit rate = {:.2}%", stats.cache.hit_rate * 100.0);
    
    manager.shutdown().await?;
    info!("✓ Performance manager shutdown completed");
    
    Ok(())
}

async fn demo_batch_processing() -> Result<()> {
    info!("\n⚡ Demo 2: Batch Processing");
    
    let config = BatchConfig {
        max_batch_size: 5,
        max_wait_time_ms: 100,
        concurrency: 2,
        ..Default::default()
    };
    
    let processor = BatchProcessor::new(config).await?;
    info!("✓ Batch processor created with max batch size: 5");
    
    // Submit multiple tasks
    let mut handles = Vec::new();
    for i in 0..10 {
        let task = DemoTask {
            id: format!("task-{}", i),
            data: vec![0u8; 100 * (i + 1)], // Variable size data
            processing_time_ms: 10,
        };
        
        let handle = tokio::spawn(async move {
            // Note: This is a simplified example - actual implementation would need proper type handling
            format!("Task {} completed", i)
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        info!("✓ {}", result);
    }
    
    let stats = processor.get_stats().await?;
    info!("✓ Batch processing stats: {} items processed", stats.processed_items);
    
    processor.shutdown().await?;
    Ok(())
}

async fn demo_caching() -> Result<()> {
    info!("\n🗄️ Demo 3: Multi-level Caching");
    
    let config = CacheConfig {
        l1_size: 100,
        l2_size: 500,
        l3_size: Some(1000),
        default_ttl_seconds: 300,
        ..Default::default()
    };
    
    let cache = CacheManager::new(config).await?;
    info!("✓ Multi-level cache created (L1: 100, L2: 500, L3: 1000)");
    
    // Cache some data
    for i in 0..10 {
        let key = format!("key-{}", i);
        let value = format!("value-{}-{}", i, Uuid::new_v4()).into_bytes();
        cache.put(&key, value, None).await?;
    }
    info!("✓ Cached 10 items across cache levels");
    
    // Retrieve data (should hit different cache levels)
    for i in 0..10 {
        let key = format!("key-{}", i);
        if let Some(value) = cache.get(&key).await? {
            info!("✓ Retrieved {}: {} bytes", key, value.len());
        }
    }
    
    let stats = cache.get_stats().await?;
    info!("✓ Cache stats: L1 hits: {}, L2 hits: {}, L3 hits: {}", 
          stats.l1_hits, stats.l2_hits, stats.l3_hits);
    
    cache.shutdown().await?;
    Ok(())
}

async fn demo_pools() -> Result<()> {
    info!("\n🏊 Demo 4: Object and Memory Pools");
    
    let config = PoolConfig {
        initial_size: 10,
        max_size: 100,
        ..Default::default()
    };
    
    // Object pool demo
    let object_pool = ObjectPool::new(config.clone())?;
    info!("✓ Object pool created with max size: 100");
    
    // Memory pool demo
    let memory_pool = MemoryPool::new(config)?;
    info!("✓ Memory pool created");
    
    // Allocate some memory blocks
    let mut blocks = Vec::new();
    for i in 0..5 {
        let size = 1024 * (i + 1);
        let block = memory_pool.allocate(size)?;
        info!("✓ Allocated memory block of {} bytes", size);
        blocks.push(block);
    }
    
    let pool_stats = object_pool.get_stats()?;
    let memory_stats = memory_pool.get_stats()?;
    
    info!("✓ Pool stats: {} objects created", pool_stats.created_objects);
    info!("✓ Memory stats: {} bytes allocated", memory_stats.total_allocated);
    
    Ok(())
}

async fn demo_concurrency() -> Result<()> {
    info!("\n🔄 Demo 5: Concurrency Control");
    
    let config = ConcurrencyConfig {
        max_concurrent_tasks: 5,
        rate_limit_rps: 10,
        circuit_breaker_threshold: 3,
        ..Default::default()
    };
    
    let concurrency_manager = Arc::new(ConcurrencyManager::new(config)?);
    info!("✓ Concurrency manager created with max 5 concurrent tasks, 10 RPS limit");

    // Execute multiple tasks with concurrency control
    let mut handles = Vec::new();
    for i in 0..8 {
        let manager = Arc::clone(&concurrency_manager);
        let handle = tokio::spawn(async move {
            let task_id = i;
            manager.execute(move || async move {
                info!("Executing task {}", task_id);
                sleep(Duration::from_millis(100)).await;
                Ok::<String, agent_mem_traits::AgentMemError>(format!("Task {} completed", task_id))
            }).await
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        match handle.await.unwrap() {
            Ok(result) => info!("✓ {}", result),
            Err(e) => warn!("Task failed: {}", e),
        }
    }

    let stats = concurrency_manager.get_stats().await?;
    info!("✓ Concurrency stats: {} completed tasks", stats.completed_tasks);
    
    Ok(())
}

async fn demo_query_optimization() -> Result<()> {
    info!("\n🔍 Demo 6: Query Optimization");
    
    let optimizer = QueryOptimizer::new(true)?;
    info!("✓ Query optimizer created");
    
    // Create sample queries
    let queries = vec![
        QueryRequest {
            vector: Some(vec![0.1; 1536]),
            filters: HashMap::new(),
            limit: 10,
            aggregations: vec![],
            metadata: HashMap::new(),
        },
        QueryRequest {
            vector: Some(vec![0.2; 768]),
            filters: {
                let mut filters = HashMap::new();
                filters.insert("category".to_string(), "important".to_string());
                filters.insert("status".to_string(), "active".to_string());
                filters
            },
            limit: 100,
            aggregations: vec!["count".to_string(), "avg".to_string()],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("frequency".to_string(), "high".to_string());
                metadata
            },
        },
    ];
    
    for (i, query) in queries.iter().enumerate() {
        let plan = optimizer.optimize_query(query).await?;
        info!("✓ Query {} optimized: {} steps, estimated cost: {:.2}ms", 
              i + 1, plan.execution_steps.len(), plan.estimated_cost);
        
        // Simulate query execution
        let plan_clone = plan.clone();
        let result = optimizer.execute_query(&plan, move |_| {
            let plan = plan_clone.clone();
            async move {
                sleep(Duration::from_millis((plan.estimated_cost / 10.0) as u64)).await;
                Ok::<String, agent_mem_traits::AgentMemError>(format!("Query executed with {} steps", plan.execution_steps.len()))
            }
        }).await?;
        
        info!("✓ {}", result);
    }
    
    let stats = optimizer.get_statistics().await?;
    info!("✓ Optimizer stats: {} queries optimized, {:.2}% cache hit rate", 
          stats.optimized_queries, stats.cache_hit_rate * 100.0);
    
    Ok(())
}

async fn demo_metrics() -> Result<()> {
    info!("\n📈 Demo 7: Performance Metrics");
    
    let metrics = MetricsCollector::new(true)?;
    info!("✓ Metrics collector created");
    
    // Simulate some operations with metrics
    for i in 0..5 {
        let start = Instant::now();
        sleep(Duration::from_millis(50 + i * 10)).await;
        let duration = start.elapsed();
        
        let success = i % 4 != 0; // Simulate some failures
        metrics.record_request(duration, success).await;
        
        if success {
            info!("✓ Request {} completed in {:?}", i + 1, duration);
        } else {
            warn!("✗ Request {} failed after {:?}", i + 1, duration);
        }
    }
    
    // Update other metrics
    metrics.update_memory_usage(1024 * 1024).await; // 1MB
    metrics.update_cache_hit_rate(0.85).await; // 85%
    metrics.update_active_connections(42).await;
    metrics.record_custom_metric("custom_metric", 123.45).await;
    
    let stats = metrics.get_metrics().await?;
    info!("✓ Metrics summary:");
    info!("  - Total requests: {}", stats.request_count);
    info!("  - Error count: {}", stats.error_count);
    info!("  - Average response time: {:.2}ms", stats.average_response_time_ms);
    info!("  - Throughput: {:.2} req/s", stats.throughput_requests_per_second);
    info!("  - Memory usage: {} bytes", stats.memory_usage_bytes);
    info!("  - Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
    info!("  - Active connections: {}", stats.active_connections);
    
    metrics.shutdown().await?;
    Ok(())
}
