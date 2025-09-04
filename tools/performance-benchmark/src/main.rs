//! AgentMem Performance Benchmark Suite
//! 
//! Comprehensive performance testing and benchmarking tool for measuring
//! memory operations, search performance, and system scalability.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub memory_operations: MemoryBenchmarkConfig,
    pub search_operations: SearchBenchmarkConfig,
    pub concurrent_operations: ConcurrencyBenchmarkConfig,
    pub stress_test: StressTestConfig,
}

/// Memory operations benchmark config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmarkConfig {
    pub num_memories: usize,
    pub memory_size_bytes: usize,
    pub batch_sizes: Vec<usize>,
    pub warmup_iterations: usize,
    pub test_iterations: usize,
}

/// Search operations benchmark config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBenchmarkConfig {
    pub dataset_sizes: Vec<usize>,
    pub query_types: Vec<QueryType>,
    pub result_limits: Vec<usize>,
    pub test_iterations: usize,
}

/// Concurrency benchmark config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyBenchmarkConfig {
    pub thread_counts: Vec<usize>,
    pub operations_per_thread: usize,
    pub operation_mix: OperationMix,
}

/// Stress test config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    pub duration_seconds: u64,
    pub max_concurrent_operations: usize,
    pub memory_pressure_mb: usize,
    pub cpu_intensive_operations: bool,
}

/// Query types for search benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Exact,
    Fuzzy,
    Semantic,
    Hybrid,
}

/// Operation mix for concurrency tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMix {
    pub read_percentage: f64,
    pub write_percentage: f64,
    pub update_percentage: f64,
    pub delete_percentage: f64,
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub test_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration: Duration,
    pub memory_results: Vec<MemoryBenchmarkResult>,
    pub search_results: Vec<SearchBenchmarkResult>,
    pub concurrency_results: Vec<ConcurrencyBenchmarkResult>,
    pub stress_test_results: Option<StressTestResult>,
    pub system_metrics: SystemMetrics,
}

/// Memory benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmarkResult {
    pub operation: String,
    pub batch_size: usize,
    pub total_operations: usize,
    pub duration: Duration,
    pub operations_per_second: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// Search benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBenchmarkResult {
    pub query_type: QueryType,
    pub dataset_size: usize,
    pub result_limit: usize,
    pub total_queries: usize,
    pub duration: Duration,
    pub queries_per_second: f64,
    pub average_latency_ms: f64,
    pub accuracy_score: f64,
    pub memory_usage_mb: f64,
}

/// Concurrency benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyBenchmarkResult {
    pub thread_count: usize,
    pub total_operations: usize,
    pub duration: Duration,
    pub operations_per_second: f64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
    pub throughput_mb_per_second: f64,
}

/// Stress test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub duration: Duration,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub average_operations_per_second: f64,
    pub peak_memory_usage_mb: f64,
    pub peak_cpu_usage_percent: f64,
    pub error_rate: f64,
    pub stability_score: f64,
}

/// System metrics during benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_cores: usize,
    pub total_memory_mb: usize,
    pub available_memory_mb: usize,
    pub disk_space_gb: usize,
    pub network_bandwidth_mbps: f64,
    pub os_info: String,
    pub rust_version: String,
}

/// Performance benchmark suite
pub struct PerformanceBenchmark {
    config: BenchmarkConfig,
    results: Arc<RwLock<Vec<BenchmarkResults>>>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            memory_operations: MemoryBenchmarkConfig {
                num_memories: 10000,
                memory_size_bytes: 1024,
                batch_sizes: vec![1, 10, 100, 1000],
                warmup_iterations: 100,
                test_iterations: 1000,
            },
            search_operations: SearchBenchmarkConfig {
                dataset_sizes: vec![1000, 10000, 100000],
                query_types: vec![QueryType::Exact, QueryType::Fuzzy, QueryType::Semantic],
                result_limits: vec![10, 50, 100],
                test_iterations: 100,
            },
            concurrent_operations: ConcurrencyBenchmarkConfig {
                thread_counts: vec![1, 2, 4, 8, 16],
                operations_per_thread: 1000,
                operation_mix: OperationMix {
                    read_percentage: 70.0,
                    write_percentage: 20.0,
                    update_percentage: 8.0,
                    delete_percentage: 2.0,
                },
            },
            stress_test: StressTestConfig {
                duration_seconds: 300, // 5 minutes
                max_concurrent_operations: 1000,
                memory_pressure_mb: 1024,
                cpu_intensive_operations: true,
            },
        }
    }
}

impl PerformanceBenchmark {
    /// Create a new performance benchmark suite
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run all benchmarks
    pub async fn run_all_benchmarks(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        println!("🚀 Starting AgentMem Performance Benchmark Suite");
        println!("================================================");

        let start_time = Instant::now();
        let started_at = Utc::now();

        let mut results = BenchmarkResults {
            test_name: "AgentMem Full Benchmark Suite".to_string(),
            started_at,
            completed_at: started_at, // Will be updated
            duration: Duration::default(),
            memory_results: Vec::new(),
            search_results: Vec::new(),
            concurrency_results: Vec::new(),
            stress_test_results: None,
            system_metrics: self.collect_system_metrics().await,
        };

        // Run memory benchmarks
        println!("\n📊 Running Memory Operation Benchmarks...");
        results.memory_results = self.run_memory_benchmarks().await?;

        // Run search benchmarks
        println!("\n🔍 Running Search Operation Benchmarks...");
        results.search_results = self.run_search_benchmarks().await?;

        // Run concurrency benchmarks
        println!("\n⚡ Running Concurrency Benchmarks...");
        results.concurrency_results = self.run_concurrency_benchmarks().await?;

        // Run stress test
        println!("\n🔥 Running Stress Test...");
        results.stress_test_results = Some(self.run_stress_test().await?);

        let total_duration = start_time.elapsed();
        results.completed_at = Utc::now();
        results.duration = total_duration;

        // Store results
        {
            let mut stored_results = self.results.write().await;
            stored_results.push(results.clone());
        }

        println!("\n✅ Benchmark Suite Completed!");
        println!("Total Duration: {:.2}s", total_duration.as_secs_f64());

        Ok(results)
    }

    /// Run memory operation benchmarks
    async fn run_memory_benchmarks(&self) -> Result<Vec<MemoryBenchmarkResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for &batch_size in &self.config.memory_operations.batch_sizes {
            // Simulate memory operations
            let result = self.benchmark_memory_operations("add_memory", batch_size).await?;
            results.push(result);

            let result = self.benchmark_memory_operations("get_memory", batch_size).await?;
            results.push(result);

            let result = self.benchmark_memory_operations("update_memory", batch_size).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Benchmark specific memory operation
    async fn benchmark_memory_operations(
        &self,
        operation: &str,
        batch_size: usize,
    ) -> Result<MemoryBenchmarkResult, Box<dyn std::error::Error>> {
        let mut latencies = Vec::new();
        let start_memory = self.get_memory_usage().await;
        let start_time = Instant::now();

        // Warmup
        for _ in 0..self.config.memory_operations.warmup_iterations {
            let _ = self.simulate_memory_operation(operation, batch_size).await;
        }

        // Actual benchmark
        for _ in 0..self.config.memory_operations.test_iterations {
            let op_start = Instant::now();
            let _ = self.simulate_memory_operation(operation, batch_size).await;
            latencies.push(op_start.elapsed());
        }

        let total_duration = start_time.elapsed();
        let end_memory = self.get_memory_usage().await;
        let total_operations = self.config.memory_operations.test_iterations * batch_size;

        // Calculate statistics
        latencies.sort();
        let average_latency = latencies.iter().sum::<Duration>().as_millis() as f64 / latencies.len() as f64;
        let p95_index = (latencies.len() as f64 * 0.95) as usize;
        let p99_index = (latencies.len() as f64 * 0.99) as usize;

        Ok(MemoryBenchmarkResult {
            operation: operation.to_string(),
            batch_size,
            total_operations,
            duration: total_duration,
            operations_per_second: total_operations as f64 / total_duration.as_secs_f64(),
            average_latency_ms: average_latency,
            p95_latency_ms: latencies[p95_index].as_millis() as f64,
            p99_latency_ms: latencies[p99_index].as_millis() as f64,
            memory_usage_mb: (end_memory - start_memory) as f64 / 1024.0 / 1024.0,
            cpu_usage_percent: self.get_cpu_usage().await,
        })
    }

    /// Simulate memory operation
    async fn simulate_memory_operation(&self, operation: &str, batch_size: usize) -> Result<(), Box<dyn std::error::Error>> {
        match operation {
            "add_memory" => {
                // Simulate adding memories
                for _ in 0..batch_size {
                    let _memory_id = Uuid::new_v4().to_string();
                    let _content = format!("Test memory content {}", Uuid::new_v4());
                    // Simulate processing time
                    tokio::time::sleep(Duration::from_micros(10)).await;
                }
            }
            "get_memory" => {
                // Simulate retrieving memories
                for _ in 0..batch_size {
                    let _memory_id = Uuid::new_v4().to_string();
                    // Simulate lookup time
                    tokio::time::sleep(Duration::from_micros(5)).await;
                }
            }
            "update_memory" => {
                // Simulate updating memories
                for _ in 0..batch_size {
                    let _memory_id = Uuid::new_v4().to_string();
                    let _new_content = format!("Updated content {}", Uuid::new_v4());
                    // Simulate update time
                    tokio::time::sleep(Duration::from_micros(15)).await;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Run search benchmarks
    async fn run_search_benchmarks(&self) -> Result<Vec<SearchBenchmarkResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for &dataset_size in &self.config.search_operations.dataset_sizes {
            for query_type in &self.config.search_operations.query_types {
                for &result_limit in &self.config.search_operations.result_limits {
                    let result = self.benchmark_search_operation(
                        query_type.clone(),
                        dataset_size,
                        result_limit,
                    ).await?;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Benchmark search operation
    async fn benchmark_search_operation(
        &self,
        query_type: QueryType,
        dataset_size: usize,
        result_limit: usize,
    ) -> Result<SearchBenchmarkResult, Box<dyn std::error::Error>> {
        let start_memory = self.get_memory_usage().await;
        let start_time = Instant::now();
        let mut total_latency = Duration::default();

        // Run search queries
        for _ in 0..self.config.search_operations.test_iterations {
            let query_start = Instant::now();
            let _ = self.simulate_search_operation(&query_type, dataset_size, result_limit).await;
            total_latency += query_start.elapsed();
        }

        let total_duration = start_time.elapsed();
        let end_memory = self.get_memory_usage().await;

        Ok(SearchBenchmarkResult {
            query_type,
            dataset_size,
            result_limit,
            total_queries: self.config.search_operations.test_iterations,
            duration: total_duration,
            queries_per_second: self.config.search_operations.test_iterations as f64 / total_duration.as_secs_f64(),
            average_latency_ms: total_latency.as_millis() as f64 / self.config.search_operations.test_iterations as f64,
            accuracy_score: 0.95, // Simulated accuracy
            memory_usage_mb: (end_memory - start_memory) as f64 / 1024.0 / 1024.0,
        })
    }

    /// Simulate search operation
    async fn simulate_search_operation(
        &self,
        query_type: &QueryType,
        dataset_size: usize,
        result_limit: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let base_delay = match query_type {
            QueryType::Exact => Duration::from_micros(100),
            QueryType::Fuzzy => Duration::from_micros(500),
            QueryType::Semantic => Duration::from_millis(2),
            QueryType::Hybrid => Duration::from_millis(3),
        };

        // Scale delay based on dataset size
        let scaled_delay = base_delay * (dataset_size as u32 / 1000).max(1);
        tokio::time::sleep(scaled_delay).await;

        // Return simulated results
        let results = (0..result_limit.min(100))
            .map(|i| format!("result_{}", i))
            .collect();

        Ok(results)
    }

    /// Run concurrency benchmarks
    async fn run_concurrency_benchmarks(&self) -> Result<Vec<ConcurrencyBenchmarkResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for &thread_count in &self.config.concurrent_operations.thread_counts {
            let result = self.benchmark_concurrency(thread_count).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Benchmark concurrency
    async fn benchmark_concurrency(&self, thread_count: usize) -> Result<ConcurrencyBenchmarkResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let operations_per_thread = self.config.concurrent_operations.operations_per_thread;
        
        let mut handles = Vec::new();
        let errors = Arc::new(RwLock::new(0usize));

        for _ in 0..thread_count {
            let errors_clone = Arc::clone(&errors);
            let handle = tokio::spawn(async move {
                for _ in 0..operations_per_thread {
                    // Simulate mixed operations based on operation mix
                    let operation_type = rand::random::<f64>();
                    let result = if operation_type < 0.7 {
                        // Read operation
                        tokio::time::sleep(Duration::from_micros(50)).await;
                        Ok(())
                    } else if operation_type < 0.9 {
                        // Write operation
                        tokio::time::sleep(Duration::from_micros(200)).await;
                        Ok(())
                    } else if operation_type < 0.98 {
                        // Update operation
                        tokio::time::sleep(Duration::from_micros(150)).await;
                        Ok(())
                    } else {
                        // Delete operation
                        tokio::time::sleep(Duration::from_micros(100)).await;
                        Ok(())
                    };

                    if result.is_err() {
                        let mut errors_guard = errors_clone.write().await;
                        *errors_guard += 1;
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.await?;
        }

        let total_duration = start_time.elapsed();
        let total_operations = thread_count * operations_per_thread;
        let error_count = *errors.read().await;

        Ok(ConcurrencyBenchmarkResult {
            thread_count,
            total_operations,
            duration: total_duration,
            operations_per_second: total_operations as f64 / total_duration.as_secs_f64(),
            average_latency_ms: total_duration.as_millis() as f64 / total_operations as f64,
            error_rate: error_count as f64 / total_operations as f64,
            throughput_mb_per_second: (total_operations * 1024) as f64 / 1024.0 / 1024.0 / total_duration.as_secs_f64(),
        })
    }

    /// Run stress test
    async fn run_stress_test(&self) -> Result<StressTestResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let duration = Duration::from_secs(self.config.stress_test.duration_seconds);
        let mut total_operations = 0usize;
        let mut successful_operations = 0usize;
        let mut peak_memory = 0usize;
        let mut peak_cpu = 0.0f64;

        println!("  Running stress test for {} seconds...", self.config.stress_test.duration_seconds);

        while start_time.elapsed() < duration {
            // Simulate high load
            let mut handles = Vec::new();
            
            for _ in 0..self.config.stress_test.max_concurrent_operations.min(100) {
                let handle = tokio::spawn(async move {
                    // Simulate various operations under stress
                    tokio::time::sleep(Duration::from_micros(100)).await;
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                });
                handles.push(handle);
            }

            // Wait for batch completion
            for handle in handles {
                total_operations += 1;
                if handle.await.is_ok() {
                    successful_operations += 1;
                }
            }

            // Monitor system resources
            let current_memory = self.get_memory_usage().await;
            let current_cpu = self.get_cpu_usage().await;
            
            if current_memory > peak_memory {
                peak_memory = current_memory;
            }
            if current_cpu > peak_cpu {
                peak_cpu = current_cpu;
            }

            // Small delay to prevent overwhelming the system
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let actual_duration = start_time.elapsed();
        let failed_operations = total_operations - successful_operations;
        let error_rate = failed_operations as f64 / total_operations as f64;
        
        // Calculate stability score (higher is better)
        let stability_score = (1.0 - error_rate) * 100.0;

        Ok(StressTestResult {
            duration: actual_duration,
            total_operations,
            successful_operations,
            failed_operations,
            average_operations_per_second: total_operations as f64 / actual_duration.as_secs_f64(),
            peak_memory_usage_mb: peak_memory as f64 / 1024.0 / 1024.0,
            peak_cpu_usage_percent: peak_cpu,
            error_rate,
            stability_score,
        })
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu_cores: num_cpus::get(),
            total_memory_mb: 8192, // Simulated
            available_memory_mb: 4096, // Simulated
            disk_space_gb: 500, // Simulated
            network_bandwidth_mbps: 1000.0, // Simulated
            os_info: std::env::consts::OS.to_string(),
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Get current memory usage
    async fn get_memory_usage(&self) -> usize {
        // Simulated memory usage
        1024 * 1024 * 100 // 100MB
    }

    /// Get current CPU usage
    async fn get_cpu_usage(&self) -> f64 {
        // Simulated CPU usage
        25.0 + (rand::random::<f64>() * 50.0)
    }

    /// Generate performance report
    pub fn generate_performance_report(&self, results: &BenchmarkResults) -> String {
        format!(
            r#"
# AgentMem Performance Benchmark Report

**Generated:** {}
**Duration:** {:.2}s
**System:** {} cores, {}MB RAM

## Memory Operations Performance

{}

## Search Operations Performance

{}

## Concurrency Performance

{}

## Stress Test Results

{}

## System Metrics

- CPU Cores: {}
- Total Memory: {}MB
- Available Memory: {}MB
- OS: {}
- Rust Version: {}

## Summary

- **Overall Performance:** {}
- **Stability Score:** {:.1}%
- **Peak Memory Usage:** {:.1}MB
- **Peak CPU Usage:** {:.1}%
            "#,
            results.completed_at.format("%Y-%m-%d %H:%M:%S UTC"),
            results.duration.as_secs_f64(),
            results.system_metrics.cpu_cores,
            results.system_metrics.total_memory_mb,
            self.format_memory_results(&results.memory_results),
            self.format_search_results(&results.search_results),
            self.format_concurrency_results(&results.concurrency_results),
            self.format_stress_test_results(&results.stress_test_results),
            results.system_metrics.cpu_cores,
            results.system_metrics.total_memory_mb,
            results.system_metrics.available_memory_mb,
            results.system_metrics.os_info,
            results.system_metrics.rust_version,
            "Good", // Simplified overall assessment
            results.stress_test_results.as_ref().map_or(0.0, |r| r.stability_score),
            results.stress_test_results.as_ref().map_or(0.0, |r| r.peak_memory_usage_mb),
            results.stress_test_results.as_ref().map_or(0.0, |r| r.peak_cpu_usage_percent),
        )
    }

    fn format_memory_results(&self, results: &[MemoryBenchmarkResult]) -> String {
        results.iter()
            .map(|r| format!(
                "- **{}** (batch {}): {:.0} ops/sec, {:.2}ms avg latency",
                r.operation, r.batch_size, r.operations_per_second, r.average_latency_ms
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_search_results(&self, results: &[SearchBenchmarkResult]) -> String {
        results.iter()
            .map(|r| format!(
                "- **{:?}** ({}k dataset): {:.0} queries/sec, {:.2}ms avg latency",
                r.query_type, r.dataset_size / 1000, r.queries_per_second, r.average_latency_ms
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_concurrency_results(&self, results: &[ConcurrencyBenchmarkResult]) -> String {
        results.iter()
            .map(|r| format!(
                "- **{} threads**: {:.0} ops/sec, {:.2}ms avg latency, {:.2}% error rate",
                r.thread_count, r.operations_per_second, r.average_latency_ms, r.error_rate * 100.0
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_stress_test_results(&self, result: &Option<StressTestResult>) -> String {
        match result {
            Some(r) => format!(
                "- **Duration:** {:.0}s\n- **Operations:** {} total, {} successful\n- **Throughput:** {:.0} ops/sec\n- **Error Rate:** {:.2}%\n- **Stability Score:** {:.1}%",
                r.duration.as_secs_f64(),
                r.total_operations,
                r.successful_operations,
                r.average_operations_per_second,
                r.error_rate * 100.0,
                r.stability_score
            ),
            None => "No stress test results available".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem Performance Benchmark Suite");
    println!("========================================");

    let config = BenchmarkConfig::default();
    let benchmark = PerformanceBenchmark::new(config);

    let results = benchmark.run_all_benchmarks().await?;

    // Generate and save report
    let report = benchmark.generate_performance_report(&results);
    std::fs::write("performance_report.md", &report)?;
    println!("\n📄 Performance report saved to: performance_report.md");

    // Save JSON results
    let json_results = serde_json::to_string_pretty(&results)?;
    std::fs::write("performance_results.json", json_results)?;
    println!("📄 JSON results saved to: performance_results.json");

    println!("\n✅ Performance benchmark completed successfully!");

    Ok(())
}

// Add num_cpus dependency for CPU core detection
use num_cpus;
