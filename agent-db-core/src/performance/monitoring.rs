// 监控和日志模块
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::types::AgentDbError;
use crate::config::LoggingConfig;

// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: i64,
    pub tags: HashMap<String, String>,
}

// 错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub error_id: String,
    pub error_type: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub context: HashMap<String, String>,
    pub timestamp: i64,
    pub count: u64,
}

// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub response_time: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

// 监控管理器
pub struct MonitoringManager {
    config: LoggingConfig,
    logs: Arc<Mutex<Vec<LogEntry>>>,
    metrics: Arc<Mutex<Vec<PerformanceMetric>>>,
    errors: Arc<Mutex<HashMap<String, ErrorInfo>>>,
    health_checks: Arc<Mutex<Vec<HealthCheckResult>>>,
    start_time: Instant,
}

impl MonitoringManager {
    pub fn new(config: LoggingConfig) -> Self {
        Self {
            config,
            logs: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(HashMap::new())),
            health_checks: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    // 记录日志
    pub fn log(&self, level: LogLevel, module: &str, message: &str, metadata: Option<HashMap<String, String>>) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now().timestamp(),
            level,
            module: module.to_string(),
            message: message.to_string(),
            metadata: metadata.unwrap_or_default(),
        };

        // 控制台输出
        if self.config.console_enabled {
            println!("[{}] {} - {}: {}", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                entry.level.to_string(),
                entry.module,
                entry.message
            );
        }

        // 存储日志
        let mut logs = self.logs.lock().unwrap();
        logs.push(entry);

        // 限制日志数量
        if logs.len() > 10000 {
            logs.drain(0..1000); // 删除最旧的1000条
        }
    }

    // 记录性能指标
    pub fn record_metric(&self, name: &str, value: f64, unit: &str, tags: Option<HashMap<String, String>>) {
        let metric = PerformanceMetric {
            metric_name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            tags: tags.unwrap_or_default(),
        };

        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(metric);

        // 限制指标数量
        if metrics.len() > 50000 {
            metrics.drain(0..5000); // 删除最旧的5000条
        }
    }

    // 记录错误
    pub fn record_error(&self, error_type: &str, message: &str, context: Option<HashMap<String, String>>) {
        let error_id = format!("{}_{}", error_type, crate::utils::hash::hash_string(message));
        
        let mut errors = self.errors.lock().unwrap();
        
        if let Some(existing_error) = errors.get_mut(&error_id) {
            existing_error.count += 1;
            existing_error.timestamp = chrono::Utc::now().timestamp();
        } else {
            let error_info = ErrorInfo {
                error_id: error_id.clone(),
                error_type: error_type.to_string(),
                message: message.to_string(),
                stack_trace: None,
                context: context.unwrap_or_default(),
                timestamp: chrono::Utc::now().timestamp(),
                count: 1,
            };
            errors.insert(error_id, error_info);
        }
    }

    // 健康检查
    pub async fn health_check(&self, component: &str) -> HealthCheckResult {
        let start_time = Instant::now();
        
        let (status, message) = match component {
            "database" => self.check_database_health().await,
            "memory" => self.check_memory_health(),
            "cache" => self.check_cache_health(),
            "vector_engine" => self.check_vector_engine_health(),
            _ => (HealthStatus::Unknown, "Unknown component".to_string()),
        };

        let response_time = start_time.elapsed().as_secs_f64();

        let result = HealthCheckResult {
            component: component.to_string(),
            status,
            message,
            response_time,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 存储健康检查结果
        let mut health_checks = self.health_checks.lock().unwrap();
        health_checks.push(result.clone());

        // 限制健康检查记录数量
        if health_checks.len() > 1000 {
            health_checks.drain(0..100);
        }

        result
    }

    // 数据库健康检查
    async fn check_database_health(&self) -> (HealthStatus, String) {
        // 简化的健康检查逻辑
        (HealthStatus::Healthy, "Database is operational".to_string())
    }

    // 内存健康检查
    fn check_memory_health(&self) -> (HealthStatus, String) {
        // 获取系统内存使用情况
        let memory_usage = self.get_memory_usage_percentage();
        
        if memory_usage > 90.0 {
            (HealthStatus::Critical, format!("High memory usage: {:.1}%", memory_usage))
        } else if memory_usage > 75.0 {
            (HealthStatus::Warning, format!("Moderate memory usage: {:.1}%", memory_usage))
        } else {
            (HealthStatus::Healthy, format!("Memory usage: {:.1}%", memory_usage))
        }
    }

    // 缓存健康检查
    fn check_cache_health(&self) -> (HealthStatus, String) {
        // 简化的缓存健康检查
        (HealthStatus::Healthy, "Cache is operational".to_string())
    }

    // 向量引擎健康检查
    fn check_vector_engine_health(&self) -> (HealthStatus, String) {
        // 简化的向量引擎健康检查
        (HealthStatus::Healthy, "Vector engine is operational".to_string())
    }

    // 获取内存使用百分比
    fn get_memory_usage_percentage(&self) -> f64 {
        // 简化实现，实际应用中需要获取真实的系统内存信息
        50.0
    }

    // 获取系统运行时间
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    // 获取日志
    pub fn get_logs(&self, level: Option<LogLevel>, limit: Option<usize>) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap();
        let mut filtered_logs: Vec<LogEntry> = logs.iter()
            .filter(|log| {
                if let Some(filter_level) = level {
                    log.level == filter_level
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // 按时间戳倒序排列
        filtered_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            filtered_logs.truncate(limit);
        }

        filtered_logs
    }

    // 获取性能指标
    pub fn get_metrics(&self, metric_name: Option<&str>, limit: Option<usize>) -> Vec<PerformanceMetric> {
        let metrics = self.metrics.lock().unwrap();
        let mut filtered_metrics: Vec<PerformanceMetric> = metrics.iter()
            .filter(|metric| {
                if let Some(name) = metric_name {
                    metric.metric_name == name
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // 按时间戳倒序排列
        filtered_metrics.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            filtered_metrics.truncate(limit);
        }

        filtered_metrics
    }

    // 获取错误统计
    pub fn get_error_summary(&self) -> Vec<ErrorInfo> {
        let errors = self.errors.lock().unwrap();
        let mut error_list: Vec<ErrorInfo> = errors.values().cloned().collect();
        
        // 按错误次数倒序排列
        error_list.sort_by(|a, b| b.count.cmp(&a.count));
        
        error_list
    }

    // 获取最近的健康检查结果
    pub fn get_latest_health_checks(&self) -> Vec<HealthCheckResult> {
        let health_checks = self.health_checks.lock().unwrap();
        let mut latest_checks: Vec<HealthCheckResult> = health_checks.iter()
            .cloned()
            .collect();

        // 按时间戳倒序排列
        latest_checks.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // 只返回最近的检查结果
        latest_checks.truncate(20);
        
        latest_checks
    }

    // 清理旧数据
    pub fn cleanup_old_data(&self, retention_days: i64) {
        let cutoff_time = chrono::Utc::now().timestamp() - (retention_days * 24 * 3600);

        // 清理旧日志
        {
            let mut logs = self.logs.lock().unwrap();
            logs.retain(|log| log.timestamp > cutoff_time);
        }

        // 清理旧指标
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.retain(|metric| metric.timestamp > cutoff_time);
        }

        // 清理旧健康检查
        {
            let mut health_checks = self.health_checks.lock().unwrap();
            health_checks.retain(|check| check.timestamp > cutoff_time);
        }
    }

    // 导出监控数据
    pub fn export_monitoring_data(&self) -> Result<String, AgentDbError> {
        let data = serde_json::json!({
            "logs": self.get_logs(None, Some(1000)),
            "metrics": self.get_metrics(None, Some(1000)),
            "errors": self.get_error_summary(),
            "health_checks": self.get_latest_health_checks(),
            "uptime_seconds": self.get_uptime().as_secs(),
        });

        serde_json::to_string_pretty(&data).map_err(AgentDbError::Serde)
    }
}
