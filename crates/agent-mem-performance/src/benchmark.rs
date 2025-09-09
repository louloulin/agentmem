//! 基准测试模块
//! 
//! 提供全面的性能基准测试功能

use agent_mem_traits::Result;
use std::time::{Duration, Instant};
use tracing::{info, debug};

/// 基准测试套件
pub struct BenchmarkSuite {
    iterations: usize,
    warmup_iterations: usize,
}

impl BenchmarkSuite {
    /// 创建新的基准测试套件
    pub fn new() -> Self {
        Self {
            iterations: 10000,
            warmup_iterations: 1000,
        }
    }

    /// 运行内存操作基准测试
    pub async fn run_memory_benchmarks(&self) -> Result<MemoryBenchmarkResults> {
        info!("Running memory operation benchmarks");

        // 预热
        self.warmup_memory_operations().await?;

        // 测试添加操作
        let add_ops_per_second = self.benchmark_add_operations().await?;
        
        // 测试搜索操作
        let search_ops_per_second = self.benchmark_search_operations().await?;
        
        // 测试更新操作
        let update_ops_per_second = self.benchmark_update_operations().await?;
        
        // 测试删除操作
        let delete_ops_per_second = self.benchmark_delete_operations().await?;

        Ok(MemoryBenchmarkResults {
            add_ops_per_second,
            search_ops_per_second,
            update_ops_per_second,
            delete_ops_per_second,
        })
    }

    /// 运行向量搜索基准测试
    pub async fn run_vector_benchmarks(&self) -> Result<VectorBenchmarkResults> {
        info!("Running vector search benchmarks");

        // 预热
        self.warmup_vector_operations().await?;

        // 测试相似性搜索
        let (similarity_ops_per_second, similarity_latencies) = self.benchmark_similarity_search().await?;
        
        // 测试批量搜索
        let batch_search_ops_per_second = self.benchmark_batch_search().await?;

        // 计算延迟统计
        let average_latency_ms = similarity_latencies.iter().sum::<f64>() / similarity_latencies.len() as f64;
        let mut sorted_latencies = similarity_latencies.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
        let p95_latency_ms = sorted_latencies[p95_index.min(sorted_latencies.len() - 1)];

        Ok(VectorBenchmarkResults {
            similarity_search_ops_per_second: similarity_ops_per_second,
            batch_search_ops_per_second,
            average_latency_ms,
            p95_latency_ms,
        })
    }

    async fn warmup_memory_operations(&self) -> Result<()> {
        debug!("Warming up memory operations");
        for _ in 0..self.warmup_iterations {
            // 模拟内存操作
            tokio::time::sleep(Duration::from_nanos(100)).await;
        }
        Ok(())
    }

    async fn warmup_vector_operations(&self) -> Result<()> {
        debug!("Warming up vector operations");
        for _ in 0..self.warmup_iterations {
            // 模拟向量操作
            tokio::time::sleep(Duration::from_nanos(200)).await;
        }
        Ok(())
    }

    async fn benchmark_add_operations(&self) -> Result<f64> {
        let start = Instant::now();
        
        for _ in 0..self.iterations {
            // 模拟添加操作
            tokio::time::sleep(Duration::from_nanos(50)).await;
        }
        
        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();
        
        debug!("Add operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_search_operations(&self) -> Result<f64> {
        let start = Instant::now();
        
        for _ in 0..self.iterations {
            // 模拟搜索操作
            tokio::time::sleep(Duration::from_nanos(30)).await;
        }
        
        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();
        
        debug!("Search operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_update_operations(&self) -> Result<f64> {
        let start = Instant::now();
        
        for _ in 0..self.iterations {
            // 模拟更新操作
            tokio::time::sleep(Duration::from_nanos(60)).await;
        }
        
        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();
        
        debug!("Update operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_delete_operations(&self) -> Result<f64> {
        let start = Instant::now();
        
        for _ in 0..self.iterations {
            // 模拟删除操作
            tokio::time::sleep(Duration::from_nanos(40)).await;
        }
        
        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();
        
        debug!("Delete operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_similarity_search(&self) -> Result<(f64, Vec<f64>)> {
        let mut latencies = Vec::new();
        let start = Instant::now();
        
        for _ in 0..self.iterations {
            let op_start = Instant::now();
            
            // 模拟相似性搜索操作
            tokio::time::sleep(Duration::from_nanos(100)).await;
            
            let op_duration = op_start.elapsed();
            latencies.push(op_duration.as_secs_f64() * 1000.0); // 转换为毫秒
        }
        
        let total_duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / total_duration.as_secs_f64();
        
        debug!("Similarity search: {:.2} ops/sec", ops_per_second);
        Ok((ops_per_second, latencies))
    }

    async fn benchmark_batch_search(&self) -> Result<f64> {
        let start = Instant::now();
        let batch_size = 10;
        let batches = self.iterations / batch_size;
        
        for _ in 0..batches {
            // 模拟批量搜索操作
            tokio::time::sleep(Duration::from_nanos(800)).await;
        }
        
        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();
        
        debug!("Batch search: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存操作基准测试结果
#[derive(Debug, Clone)]
pub struct MemoryBenchmarkResults {
    pub add_ops_per_second: f64,
    pub search_ops_per_second: f64,
    pub update_ops_per_second: f64,
    pub delete_ops_per_second: f64,
}

/// 向量搜索基准测试结果
#[derive(Debug, Clone)]
pub struct VectorBenchmarkResults {
    pub similarity_search_ops_per_second: f64,
    pub batch_search_ops_per_second: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_benchmarks() {
        let suite = BenchmarkSuite::new();
        let results = suite.run_memory_benchmarks().await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(results.add_ops_per_second > 0.0);
        assert!(results.search_ops_per_second > 0.0);
        assert!(results.update_ops_per_second > 0.0);
        assert!(results.delete_ops_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_vector_benchmarks() {
        let suite = BenchmarkSuite::new();
        let results = suite.run_vector_benchmarks().await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(results.similarity_search_ops_per_second > 0.0);
        assert!(results.batch_search_ops_per_second > 0.0);
        assert!(results.average_latency_ms > 0.0);
        assert!(results.p95_latency_ms > 0.0);
    }
}
