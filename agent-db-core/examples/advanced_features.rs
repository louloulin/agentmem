// 高级功能示例
use agent_state_db::{
    AgentDB, AgentState, Memory, MemoryType, StateType,
    CacheManager, MonitoringManager, LogLevel,
    AgentDbConfig, ConfigManager,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 配置管理示例
    println!("=== 配置管理示例 ===");
    let mut config = AgentDbConfig::default();
    config.vector.dimension = 512;
    config.performance.cache_size_mb = 1024;
    config.logging.level = "debug".to_string();
    
    println!("配置验证: {:?}", config.validate());
    
    // 2. 性能优化示例
    println!("\n=== 性能优化示例 ===");
    let cache_manager = CacheManager::new(config.performance.clone());
    
    // 缓存操作
    let query_hash = 12345u64;
    let test_data = vec![1, 2, 3, 4, 5];
    cache_manager.set(query_hash, test_data.clone(), 5);
    
    if let Some(cached_data) = cache_manager.get(query_hash) {
        println!("缓存命中: {:?}", cached_data);
    }
    
    let stats = cache_manager.get_statistics();
    println!("缓存统计: 条目数={}, 命中数={}", stats.total_entries, stats.total_hits);
    
    // 3. 监控系统示例
    println!("\n=== 监控系统示例 ===");
    let monitor = MonitoringManager::new(config.logging.clone());
    
    // 记录日志
    monitor.log(LogLevel::Info, "example", "系统启动", None);
    monitor.log(LogLevel::Debug, "example", "调试信息", None);
    
    // 记录性能指标
    monitor.record_metric("response_time", 0.123, "seconds", None);
    monitor.record_metric("memory_usage", 512.0, "MB", None);
    
    // 记录错误
    monitor.record_error("connection_error", "数据库连接失败", None);
    
    // 获取监控数据
    let logs = monitor.get_logs(Some(LogLevel::Info), Some(10));
    println!("日志条目数: {}", logs.len());
    
    let metrics = monitor.get_metrics(None, Some(10));
    println!("性能指标数: {}", metrics.len());
    
    let errors = monitor.get_error_summary();
    println!("错误统计: {}", errors.len());
    
    // 4. 高级API示例
    println!("\n=== 高级API示例 ===");
    let db = AgentDB::new("./example_db", 384).await?;
    
    // 批量操作
    let states = vec![
        AgentState::new(100, 1, StateType::WorkingMemory, vec![1, 2, 3]),
        AgentState::new(101, 1, StateType::LongTermMemory, vec![4, 5, 6]),
        AgentState::new(102, 1, StateType::Context, vec![7, 8, 9]),
    ];
    
    let batch_results = db.batch_save_agent_states(states).await?;
    println!("批量保存结果: {} 个操作", batch_results.len());
    
    // 批量记忆操作
    let memories = vec![
        Memory::new(100, MemoryType::Episodic, "记忆1".to_string(), 0.8),
        Memory::new(100, MemoryType::Semantic, "记忆2".to_string(), 0.9),
        Memory::new(101, MemoryType::Procedural, "记忆3".to_string(), 0.7),
    ];
    
    let memory_results = db.batch_store_memories(memories).await?;
    println!("批量记忆存储结果: {} 个操作", memory_results.len());
    
    // 系统健康检查
    let health = db.get_system_health().await?;
    println!("系统健康状态: {:?}", health);
    
    // Agent行为模式分析
    let patterns = db.analyze_agent_patterns(100).await?;
    println!("Agent 100 行为模式: {:?}", patterns);
    
    // 5. 流式处理示例
    println!("\n=== 流式处理示例 ===");
    let mut processed_count = 0;
    
    db.stream_memories(100, |memory| {
        processed_count += 1;
        println!("处理记忆: {} (重要性: {})", memory.content, memory.importance);
        Ok(())
    }).await?;
    
    println!("流式处理完成，共处理 {} 条记忆", processed_count);
    
    // 6. 健康检查示例
    println!("\n=== 健康检查示例 ===");
    let health_result = monitor.health_check("database").await;
    println!("数据库健康检查: {:?}", health_result);
    
    let health_result = monitor.health_check("memory").await;
    println!("内存健康检查: {:?}", health_result);
    
    // 7. 监控数据导出
    println!("\n=== 监控数据导出 ===");
    let monitoring_data = monitor.export_monitoring_data()?;
    println!("监控数据导出完成，数据大小: {} 字节", monitoring_data.len());
    
    // 8. 配置管理器示例
    println!("\n=== 配置管理器示例 ===");
    let mut config_manager = ConfigManager::new();
    
    // 更新配置
    let mut new_config = AgentDbConfig::default();
    new_config.vector.dimension = 768;
    config_manager.update_config(new_config)?;
    
    let current_config = config_manager.get_config();
    println!("当前向量维度: {}", current_config.vector.dimension);
    
    println!("\n=== 示例完成 ===");
    println!("所有高级功能演示完成！");
    
    Ok(())
}

// 辅助函数：演示错误处理
async fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = MonitoringManager::new(AgentDbConfig::default().logging);
    
    // 模拟一些错误情况
    monitor.record_error("validation_error", "输入参数无效", None);
    monitor.record_error("network_error", "网络连接超时", None);
    monitor.record_error("validation_error", "输入参数无效", None); // 重复错误
    
    let error_summary = monitor.get_error_summary();
    for error in error_summary {
        println!("错误类型: {}, 消息: {}, 次数: {}", 
                error.error_type, error.message, error.count);
    }
    
    Ok(())
}

// 辅助函数：演示性能监控
async fn demonstrate_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = MonitoringManager::new(AgentDbConfig::default().logging);
    
    // 模拟一些性能指标
    for i in 0..10 {
        let response_time = 0.1 + (i as f64 * 0.01);
        monitor.record_metric("api_response_time", response_time, "seconds", None);
        
        let memory_usage = 100.0 + (i as f64 * 10.0);
        monitor.record_metric("memory_usage", memory_usage, "MB", None);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    let metrics = monitor.get_metrics(Some("api_response_time"), Some(5));
    println!("最近5次API响应时间:");
    for metric in metrics {
        println!("  时间: {}, 值: {} {}", 
                metric.timestamp, metric.value, metric.unit);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_advanced_features_example() {
        // 测试示例代码的基本功能
        let config = AgentDbConfig::default();
        assert_eq!(config.vector.dimension, 384);
        
        let cache_manager = CacheManager::new(config.performance.clone());
        let stats = cache_manager.get_statistics();
        assert_eq!(stats.total_entries, 0);
        
        let monitor = MonitoringManager::new(config.logging);
        monitor.log(LogLevel::Info, "test", "测试消息", None);
        let logs = monitor.get_logs(None, Some(1));
        assert_eq!(logs.len(), 1);
    }
}
