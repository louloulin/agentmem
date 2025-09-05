// 性能监控模块
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

// 移除未使用的导入

// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_latency: HashMap<String, Duration>,
    pub throughput: HashMap<String, f64>,
    pub memory_usage: u64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub active_connections: u32,
}

// 性能诊断
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDiagnostics {
    pub bottlenecks: Vec<String>,
    pub recommendations: Vec<String>,
    pub health_score: f64,
}

// 性能监控器
pub struct PerformanceMonitor {
    start_time: Instant,
    metrics: PerformanceMetrics,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: PerformanceMetrics {
                operation_latency: HashMap::new(),
                throughput: HashMap::new(),
                memory_usage: 0,
                cache_hit_rate: 0.0,
                error_rate: 0.0,
                active_connections: 0,
            },
        }
    }

    pub fn record_operation(&mut self, operation: &str, duration: Duration) {
        self.metrics.operation_latency.insert(operation.to_string(), duration);
    }

    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    pub fn diagnose(&self) -> PerformanceDiagnostics {
        PerformanceDiagnostics {
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
            health_score: 1.0,
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
