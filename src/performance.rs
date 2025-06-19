<<<<<<< HEAD
// 性能优化模块
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use lancedb::Connection;

use crate::types::AgentDbError;
use crate::config::PerformanceConfig;

// 查询缓存结构
#[derive(Debug, Clone)]
pub struct QueryCache {
    pub cache_id: String,
    pub query_hash: u64,
    pub result_data: Vec<u8>,
    pub result_count: usize,
    pub hit_count: u64,
    pub created_at: i64,
    pub last_accessed: i64,
    pub expires_at: i64,
}

// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub total_hits: u64,
    pub total_size: usize,
    pub expired_entries: usize,
    pub hit_rate: f64,
    pub memory_usage: usize,
}

// 连接池统计
#[derive(Debug, Clone)]
pub struct ConnectionPoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub average_wait_time: f64,
}

// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub query_count: u64,
    pub average_query_time: f64,
    pub cache_hit_rate: f64,
    pub memory_usage: usize,
    pub cpu_usage: f64,
    pub throughput: f64,
}

// 缓存管理器
pub struct CacheManager {
    cache: Arc<RwLock<HashMap<u64, QueryCache>>>,
    config: PerformanceConfig,
    stats: Arc<Mutex<CacheStatistics>>,
}

impl CacheManager {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(CacheStatistics {
                total_entries: 0,
                total_hits: 0,
                total_size: 0,
                expired_entries: 0,
                hit_rate: 0.0,
                memory_usage: 0,
            })),
        }
    }

    // 获取缓存结果
    pub fn get(&self, query_hash: u64) -> Option<Vec<u8>> {
        let current_time = chrono::Utc::now().timestamp();
        
        let mut cache = self.cache.write().unwrap();
        if let Some(entry) = cache.get_mut(&query_hash) {
            if entry.expires_at > current_time {
                entry.hit_count += 1;
                entry.last_accessed = current_time;
                
                // 更新统计
                let mut stats = self.stats.lock().unwrap();
                stats.total_hits += 1;
                
                return Some(entry.result_data.clone());
            } else {
                // 缓存过期，删除
                cache.remove(&query_hash);
            }
        }
        
        None
    }

    // 设置缓存
    pub fn set(&self, query_hash: u64, data: Vec<u8>, result_count: usize) {
        let current_time = chrono::Utc::now().timestamp();
        let ttl = 3600; // 1小时TTL
        
        let mut cache = self.cache.write().unwrap();
        
        // 检查缓存大小限制
        if cache.len() >= self.config.cache_size_mb * 1024 / 4 { // 简化的大小估算
            self.evict_lru(&mut cache);
        }
        
        let entry = QueryCache {
            cache_id: format!("cache_{}", query_hash),
            query_hash,
            result_data: data.clone(),
            result_count,
            hit_count: 0,
            created_at: current_time,
            last_accessed: current_time,
            expires_at: current_time + ttl,
        };
        
        cache.insert(query_hash, entry);
        
        // 更新统计
        let mut stats = self.stats.lock().unwrap();
        stats.total_entries = cache.len();
        stats.total_size += data.len();
        stats.memory_usage = stats.total_size;
    }

    // LRU淘汰策略
    fn evict_lru(&self, cache: &mut HashMap<u64, QueryCache>) {
        if let Some((&oldest_hash, _)) = cache.iter()
            .min_by_key(|(_, entry)| entry.last_accessed) {
            cache.remove(&oldest_hash);
        }
    }

    // 清理过期缓存
    pub fn cleanup_expired(&self) {
        let current_time = chrono::Utc::now().timestamp();
        let mut cache = self.cache.write().unwrap();
        
        let expired_keys: Vec<u64> = cache.iter()
            .filter(|(_, entry)| entry.expires_at < current_time)
            .map(|(&key, _)| key)
            .collect();
        
        for key in expired_keys {
            cache.remove(&key);
        }
        
        // 更新统计
        let mut stats = self.stats.lock().unwrap();
        stats.total_entries = cache.len();
        stats.expired_entries = 0; // 已清理
    }

    // 获取缓存统计
    pub fn get_statistics(&self) -> CacheStatistics {
        let cache = self.cache.read().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        stats.total_entries = cache.len();
        stats.hit_rate = if stats.total_entries > 0 {
            stats.total_hits as f64 / stats.total_entries as f64
        } else {
            0.0
        };
        
        stats.clone()
    }
}

// 连接池管理器
pub struct ConnectionPool {
    connections: Arc<Mutex<Vec<Arc<Connection>>>>,
    semaphore: Arc<Semaphore>,
    config: PerformanceConfig,
    stats: Arc<Mutex<ConnectionPoolStats>>,
}

impl ConnectionPool {
    pub async fn new(db_path: &str, config: PerformanceConfig) -> Result<Self, AgentDbError> {
        let max_connections = config.parallel_workers;
        let semaphore = Arc::new(Semaphore::new(max_connections));
        
        // 预创建一些连接
        let mut connections = Vec::new();
        for _ in 0..max_connections.min(5) {
            let conn = lancedb::connect(db_path).execute().await?;
            connections.push(Arc::new(conn));
        }
        
        let idle_count = connections.len();

        Ok(Self {
            connections: Arc::new(Mutex::new(connections)),
            semaphore,
            config,
            stats: Arc::new(Mutex::new(ConnectionPoolStats {
                total_connections: max_connections,
                active_connections: 0,
                idle_connections: idle_count,
                total_requests: 0,
                failed_requests: 0,
                average_wait_time: 0.0,
            })),
        })
    }

    // 获取连接
    pub async fn get_connection(&self) -> Result<Arc<Connection>, AgentDbError> {
        let start_time = Instant::now();
        
        // 获取信号量许可
        let _permit = self.semaphore.acquire().await
            .map_err(|e| AgentDbError::Internal(format!("Failed to acquire connection: {}", e)))?;
        
        let wait_time = start_time.elapsed().as_secs_f64();
        
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_requests += 1;
            stats.active_connections += 1;
            stats.average_wait_time = (stats.average_wait_time + wait_time) / 2.0;
        }
        
        // 尝试从池中获取连接
        let mut connections = self.connections.lock().unwrap();
        if let Some(conn) = connections.pop() {
            return Ok(conn);
        }
        
        // 如果池中没有连接，创建新连接
        drop(connections); // 释放锁
        let conn = lancedb::connect("./default_db").execute().await?;
        Ok(Arc::new(conn))
    }

    // 归还连接
    pub fn return_connection(&self, connection: Arc<Connection>) {
        let mut connections = self.connections.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        if connections.len() < self.config.parallel_workers {
            connections.push(connection);
            stats.idle_connections = connections.len();
        }
        
        stats.active_connections = stats.active_connections.saturating_sub(1);
    }

    // 获取连接池统计
    pub fn get_statistics(&self) -> ConnectionPoolStats {
        self.stats.lock().unwrap().clone()
    }
}

// 批量操作管理器
pub struct BatchOperationManager {
    config: PerformanceConfig,
    pending_operations: Arc<Mutex<Vec<BatchOperation>>>,
}

#[derive(Debug, Clone)]
pub struct BatchOperation {
    pub operation_type: String,
    pub data: Vec<u8>,
    pub timestamp: i64,
}

impl BatchOperationManager {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            pending_operations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // 添加批量操作
    pub fn add_operation(&self, operation_type: String, data: Vec<u8>) {
        let operation = BatchOperation {
            operation_type,
            data,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let mut operations = self.pending_operations.lock().unwrap();
        operations.push(operation);
        
        // 如果达到批量大小，触发执行
        if operations.len() >= self.config.batch_size {
            // 这里可以触发批量执行逻辑
            self.flush_operations();
        }
    }

    // 刷新操作
    pub fn flush_operations(&self) {
        let mut operations = self.pending_operations.lock().unwrap();
        if !operations.is_empty() {
            // 执行批量操作
            log::info!("Executing {} batch operations", operations.len());
            operations.clear();
        }
    }

    // 获取待处理操作数量
    pub fn pending_count(&self) -> usize {
        self.pending_operations.lock().unwrap().len()
    }
}

// 内存管理器
pub struct MemoryManager {
    config: PerformanceConfig,
    allocated_memory: Arc<Mutex<usize>>,
    gc_interval: Duration,
    last_gc: Arc<Mutex<Instant>>,
}

impl MemoryManager {
    pub fn new(config: PerformanceConfig) -> Self {
        let gc_interval_ms = config.gc_interval_ms;
        Self {
            config,
            allocated_memory: Arc::new(Mutex::new(0)),
            gc_interval: Duration::from_millis(gc_interval_ms),
            last_gc: Arc::new(Mutex::new(Instant::now())),
        }
    }

    // 分配内存
    pub fn allocate(&self, size: usize) -> Result<(), AgentDbError> {
        let mut allocated = self.allocated_memory.lock().unwrap();
        
        if *allocated + size > self.config.memory_limit_mb * 1024 * 1024 {
            return Err(AgentDbError::Internal("Memory limit exceeded".to_string()));
        }
        
        *allocated += size;
        Ok(())
    }

    // 释放内存
    pub fn deallocate(&self, size: usize) {
        let mut allocated = self.allocated_memory.lock().unwrap();
        *allocated = allocated.saturating_sub(size);
    }

    // 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        *self.allocated_memory.lock().unwrap()
    }

    // 垃圾回收
    pub fn gc(&self) {
        let mut last_gc = self.last_gc.lock().unwrap();
        let now = Instant::now();
        
        if now.duration_since(*last_gc) >= self.gc_interval {
            log::info!("Performing garbage collection");
            // 这里可以实现具体的GC逻辑
            *last_gc = now;
        }
    }
}
=======
// Performance 模块 - 性能监控和诊断
// 从 lib.rs 自动拆分生成

use std::sync::{Arc, RwLock, Mutex};
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

// 性能指标结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkIO,
    pub database_stats: DatabaseStats,
    pub query_performance: QueryPerformance,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub avg_query_time_ms: f64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformance {
    pub total_queries: u64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_qps: f64,
}

// 性能监控器
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    query_times: Arc<Mutex<Vec<f64>>>,
    start_time: Instant,
    query_count: AtomicU64,
    error_count: AtomicU64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            query_times: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            query_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    pub fn record_query(&self, duration_ms: f64, success: bool) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        
        if !success {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // 记录查询时间
        if let Ok(mut times) = self.query_times.lock() {
            times.push(duration_ms);
            
            // 保持最近1000个查询的记录
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        let mut metrics = self.metrics.read().unwrap().clone();
        
        // 更新查询性能统计
        if let Ok(times) = self.query_times.lock() {
            if !times.is_empty() {
                let total_queries = self.query_count.load(Ordering::Relaxed);
                let avg_latency = times.iter().sum::<f64>() / times.len() as f64;
                
                let mut sorted_times = times.clone();
                sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
                let p99_index = (sorted_times.len() as f64 * 0.99) as usize;
                
                let p95_latency = sorted_times.get(p95_index).copied().unwrap_or(0.0);
                let p99_latency = sorted_times.get(p99_index).copied().unwrap_or(0.0);
                
                let elapsed_secs = self.start_time.elapsed().as_secs_f64();
                let throughput = if elapsed_secs > 0.0 {
                    total_queries as f64 / elapsed_secs
                } else {
                    0.0
                };

                metrics.query_performance = QueryPerformance {
                    total_queries,
                    avg_latency_ms: avg_latency,
                    p95_latency_ms: p95_latency,
                    p99_latency_ms: p99_latency,
                    throughput_qps: throughput,
                };
            }
        }

        metrics.timestamp = chrono::Utc::now().timestamp();
        metrics
    }

    pub fn update_system_metrics(&self) {
        // 这里可以集成系统监控库来获取实际的系统指标
        // 为了简化，我们使用模拟数据
        
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.cpu_usage = self.get_cpu_usage();
            metrics.memory_usage = self.get_memory_usage();
            metrics.disk_usage = self.get_disk_usage();
            metrics.network_io = self.get_network_io();
            metrics.database_stats = self.get_database_stats();
        }
    }

    fn get_cpu_usage(&self) -> f64 {
        // 模拟CPU使用率
        rand::random::<f64>() * 100.0
    }

    fn get_memory_usage(&self) -> f64 {
        // 模拟内存使用率
        rand::random::<f64>() * 100.0
    }

    fn get_disk_usage(&self) -> f64 {
        // 模拟磁盘使用率
        rand::random::<f64>() * 100.0
    }

    fn get_network_io(&self) -> NetworkIO {
        NetworkIO {
            bytes_sent: rand::random::<u64>() % 1000000,
            bytes_received: rand::random::<u64>() % 1000000,
            packets_sent: rand::random::<u64>() % 10000,
            packets_received: rand::random::<u64>() % 10000,
        }
    }

    fn get_database_stats(&self) -> DatabaseStats {
        let total_queries = self.query_count.load(Ordering::Relaxed);
        let failed_queries = self.error_count.load(Ordering::Relaxed);
        let successful_queries = total_queries - failed_queries;

        DatabaseStats {
            total_queries,
            successful_queries,
            failed_queries,
            avg_query_time_ms: 0.0, // 将在get_metrics中计算
            cache_hit_rate: rand::random::<f64>(),
            active_connections: rand::random::<u32>() % 100,
        }
    }

    pub fn get_error_rate(&self) -> f64 {
        let total = self.query_count.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);
        
        if total > 0 {
            errors as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn reset_metrics(&self) {
        self.query_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        
        if let Ok(mut times) = self.query_times.lock() {
            times.clear();
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_io: NetworkIO {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
            },
            database_stats: DatabaseStats {
                total_queries: 0,
                successful_queries: 0,
                failed_queries: 0,
                avg_query_time_ms: 0.0,
                cache_hit_rate: 0.0,
                active_connections: 0,
            },
            query_performance: QueryPerformance {
                total_queries: 0,
                avg_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                throughput_qps: 0.0,
            },
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

// 性能诊断器
pub struct PerformanceDiagnostics {
    monitor: Arc<PerformanceMonitor>,
    thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_cpu_usage: f64,
    pub max_memory_usage: f64,
    pub max_query_latency_ms: f64,
    pub min_cache_hit_rate: f64,
    pub max_error_rate: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_cpu_usage: 80.0,
            max_memory_usage: 85.0,
            max_query_latency_ms: 1000.0,
            min_cache_hit_rate: 0.8,
            max_error_rate: 0.05,
        }
    }
}

impl PerformanceDiagnostics {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            monitor,
            thresholds: PerformanceThresholds::default(),
        }
    }

    pub fn diagnose(&self) -> Vec<PerformanceIssue> {
        let metrics = self.monitor.get_metrics();
        let mut issues = Vec::new();

        // 检查CPU使用率
        if metrics.cpu_usage > self.thresholds.max_cpu_usage {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::System,
                description: format!("High CPU usage: {:.1}%", metrics.cpu_usage),
                recommendation: "Consider optimizing queries or scaling resources".to_string(),
            });
        }

        // 检查内存使用率
        if metrics.memory_usage > self.thresholds.max_memory_usage {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::System,
                description: format!("High memory usage: {:.1}%", metrics.memory_usage),
                recommendation: "Consider increasing memory or optimizing data structures".to_string(),
            });
        }

        // 检查查询延迟
        if metrics.query_performance.avg_latency_ms > self.thresholds.max_query_latency_ms {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Database,
                description: format!("High query latency: {:.1}ms", metrics.query_performance.avg_latency_ms),
                recommendation: "Optimize queries, add indexes, or scale database".to_string(),
            });
        }

        // 检查缓存命中率
        if metrics.database_stats.cache_hit_rate < self.thresholds.min_cache_hit_rate {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Database,
                description: format!("Low cache hit rate: {:.1}%", metrics.database_stats.cache_hit_rate * 100.0),
                recommendation: "Increase cache size or optimize cache strategy".to_string(),
            });
        }

        // 检查错误率
        let error_rate = self.monitor.get_error_rate();
        if error_rate > self.thresholds.max_error_rate {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Application,
                description: format!("High error rate: {:.1}%", error_rate * 100.0),
                recommendation: "Investigate and fix application errors".to_string(),
            });
        }

        issues
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub enum IssueCategory {
    System,
    Database,
    Network,
    Application,
}
>>>>>>> origin/feature-module
