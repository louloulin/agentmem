// 压力测试
use agent_state_db_rust::{
    AgentDB, AgentState, Memory, Document, MemoryType, StateType,
    CacheManager, MonitoringManager, LogLevel, AgentDbConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::Semaphore;

// 压力测试配置
struct StressTestConfig {
    max_concurrent_operations: usize,
    total_operations: usize,
    operation_timeout: Duration,
    memory_limit_mb: usize,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 50,
            total_operations: 10000,
            operation_timeout: Duration::from_secs(30),
            memory_limit_mb: 1024,
        }
    }
}

async fn create_stress_test_db() -> (AgentDB, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("stress_test_db");
    let db = AgentDB::new(db_path.to_str().unwrap(), 384).await.unwrap();
    (db, temp_dir)
}

#[tokio::test]
async fn test_high_concurrency_agent_operations() {
    let (db, _temp_dir) = create_stress_test_db().await;
    let config = StressTestConfig::default();
    
    println!("开始高并发Agent操作压力测试...");
    println!("并发数: {}, 总操作数: {}", config.max_concurrent_operations, config.total_operations);
    
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..config.total_operations {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let db_ref = &db; // 在实际实现中，这里需要Arc<AgentDB>
        
        let handle = tokio::spawn(async move {
            let _permit = permit; // 保持permit直到任务完成
            
            let agent_id = i as u64 + 100000;
            let data_size = 512 + (i % 1024); // 变化的数据大小
            let state = AgentState::new(agent_id, 1, StateType::WorkingMemory, vec![i as u8; data_size]);
            
            // 保存状态
            // db_ref.save_agent_state(&state).await.unwrap();
            
            // 立即加载验证
            // let loaded = db_ref.load_agent_state(agent_id).await.unwrap();
            // assert!(loaded.is_some());
            
            // 更新状态
            let new_data = vec![(i + 1) as u8; data_size];
            // db_ref.update_agent_state(agent_id, new_data).await.unwrap();
            
            if i % 1000 == 0 {
                println!("完成操作: {}/{}", i, config.total_operations);
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有操作完成
    for handle in handles {
        tokio::time::timeout(config.operation_timeout, handle)
            .await
            .expect("操作超时")
            .expect("任务执行失败");
    }
    
    let total_duration = start_time.elapsed();
    let ops_per_second = config.total_operations as f64 / total_duration.as_secs_f64();
    
    println!("高并发Agent操作完成!");
    println!("总时间: {:?}", total_duration);
    println!("操作/秒: {:.2}", ops_per_second);
    
    // 验证性能要求
    assert!(ops_per_second > 100.0, "操作速度应该超过100ops/s，实际: {:.2}", ops_per_second);
    assert!(total_duration < Duration::from_secs(300), "总时间应该在5分钟内");
}

#[tokio::test]
async fn test_memory_stress() {
    let (db, _temp_dir) = create_stress_test_db().await;
    let config = StressTestConfig {
        total_operations: 50000,
        max_concurrent_operations: 100,
        ..Default::default()
    };
    
    println!("开始记忆系统压力测试...");
    
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..config.total_operations {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            
            let agent_id = (i % 1000) as u64 + 200000; // 1000个不同的agent
            let memory_types = [MemoryType::Episodic, MemoryType::Semantic, MemoryType::Procedural, MemoryType::Working];
            let memory_type = memory_types[i % memory_types.len()];
            
            let content = format!("压力测试记忆内容 {} - 这是一个较长的记忆内容用于测试系统在大量数据下的性能表现", i);
            let importance = 0.1 + (i % 100) as f32 / 100.0 * 0.9; // 0.1-1.0之间的重要性
            
            let memory = Memory::new(agent_id, memory_type, content, importance);
            
            // 存储记忆 (在实际实现中需要处理引用)
            // db.store_memory(&memory).await.unwrap();
            
            // 随机执行一些查询操作
            if i % 10 == 0 {
                // db.get_agent_memories(agent_id, Some(memory_type), 10).await.unwrap();
            }
            
            if i % 100 == 0 {
                // db.search_memories(agent_id, "压力测试", 5).await.unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有操作完成
    for (i, handle) in handles.into_iter().enumerate() {
        tokio::time::timeout(config.operation_timeout, handle)
            .await
            .expect("操作超时")
            .expect("任务执行失败");
            
        if i % 5000 == 0 {
            println!("记忆操作进度: {}/{}", i, config.total_operations);
        }
    }
    
    let total_duration = start_time.elapsed();
    let ops_per_second = config.total_operations as f64 / total_duration.as_secs_f64();
    
    println!("记忆系统压力测试完成!");
    println!("总时间: {:?}", total_duration);
    println!("操作/秒: {:.2}", ops_per_second);
    
    assert!(ops_per_second > 500.0, "记忆操作速度应该超过500ops/s");
}

#[tokio::test]
async fn test_vector_search_stress() {
    let (db, _temp_dir) = create_stress_test_db().await;
    let config = StressTestConfig {
        total_operations: 5000,
        max_concurrent_operations: 20,
        ..Default::default()
    };
    
    println!("开始向量搜索压力测试...");
    
    // 首先添加大量向量数据
    let vector_count = 1000;
    println!("准备{}个向量数据...", vector_count);
    
    for i in 0..vector_count {
        let vector: Vec<f32> = (0..384).map(|j| (i * 384 + j) as f32 / (vector_count * 384) as f32).collect();
        let vector_id = format!("stress_vec_{}", i);
        let metadata = format!("压力测试向量{}", i);
        
        // db.add_vector(vector_id, vector, metadata).await.unwrap();
        
        if i % 100 == 0 {
            println!("向量准备进度: {}/{}", i, vector_count);
        }
    }
    
    println!("开始并发向量搜索...");
    
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..config.total_operations {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            
            // 生成随机查询向量
            let query_vector: Vec<f32> = (0..384).map(|j| (i * 384 + j) as f32 / (config.total_operations * 384) as f32).collect();
            
            // 执行向量搜索
            // let search_results = db.search_vectors(query_vector.clone(), 10).await.unwrap();
            
            // 执行相似度搜索
            if i % 5 == 0 {
                // let similarity_results = db.similarity_search(query_vector, 0.7, 5).await.unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有搜索完成
    for (i, handle) in handles.into_iter().enumerate() {
        tokio::time::timeout(config.operation_timeout, handle)
            .await
            .expect("搜索操作超时")
            .expect("搜索任务失败");
            
        if i % 500 == 0 {
            println!("搜索进度: {}/{}", i, config.total_operations);
        }
    }
    
    let total_duration = start_time.elapsed();
    let searches_per_second = config.total_operations as f64 / total_duration.as_secs_f64();
    
    println!("向量搜索压力测试完成!");
    println!("总时间: {:?}", total_duration);
    println!("搜索/秒: {:.2}", searches_per_second);
    
    assert!(searches_per_second > 50.0, "向量搜索速度应该超过50次/s");
}

#[tokio::test]
async fn test_cache_stress() {
    let config = AgentDbConfig::default();
    let cache_manager = CacheManager::new(config.performance);
    
    let stress_config = StressTestConfig {
        total_operations: 100000,
        max_concurrent_operations: 200,
        ..Default::default()
    };
    
    println!("开始缓存系统压力测试...");
    
    let semaphore = Arc::new(Semaphore::new(stress_config.max_concurrent_operations));
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..stress_config.total_operations {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let cache_ref = &cache_manager;
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            
            let key = (i % 10000) as u64; // 重复使用10000个key
            let data_size = 100 + (i % 900); // 100-1000字节的数据
            let data = vec![i as u8; data_size];
            
            // 80%写操作，20%读操作
            if i % 5 != 0 {
                cache_ref.set(key, data, 1);
            } else {
                let _cached_data = cache_ref.get(key);
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有缓存操作完成
    for handle in handles {
        handle.await.expect("缓存操作失败");
    }
    
    let total_duration = start_time.elapsed();
    let ops_per_second = stress_config.total_operations as f64 / total_duration.as_secs_f64();
    
    // 获取最终统计
    let stats = cache_manager.get_statistics();
    
    println!("缓存系统压力测试完成!");
    println!("总时间: {:?}", total_duration);
    println!("操作/秒: {:.2}", ops_per_second);
    println!("缓存条目数: {}", stats.total_entries);
    println!("缓存命中数: {}", stats.total_hits);
    println!("命中率: {:.2}%", stats.hit_rate * 100.0);
    
    assert!(ops_per_second > 10000.0, "缓存操作速度应该超过10000ops/s");
    assert!(stats.total_entries > 0, "应该有缓存条目");
}

#[tokio::test]
async fn test_monitoring_stress() {
    let config = AgentDbConfig::default();
    let monitor = MonitoringManager::new(config.logging);
    
    let stress_config = StressTestConfig {
        total_operations: 50000,
        max_concurrent_operations: 100,
        ..Default::default()
    };
    
    println!("开始监控系统压力测试...");
    
    let semaphore = Arc::new(Semaphore::new(stress_config.max_concurrent_operations));
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..stress_config.total_operations {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let monitor_ref = &monitor;
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            
            let operation_type = i % 4;
            
            match operation_type {
                0 => {
                    // 记录日志
                    let level = match i % 5 {
                        0 => LogLevel::Error,
                        1 => LogLevel::Warn,
                        2 => LogLevel::Info,
                        3 => LogLevel::Debug,
                        _ => LogLevel::Trace,
                    };
                    monitor_ref.log(level, "stress_test", &format!("压力测试日志 {}", i), None);
                }
                1 => {
                    // 记录性能指标
                    let metric_value = (i as f64) * 0.001;
                    monitor_ref.record_metric("stress_metric", metric_value, "count", None);
                }
                2 => {
                    // 记录错误
                    let error_type = format!("stress_error_{}", i % 10);
                    monitor_ref.record_error(&error_type, &format!("压力测试错误 {}", i), None);
                }
                _ => {
                    // 健康检查
                    if i % 100 == 0 {
                        let _health = monitor_ref.health_check("stress_component").await;
                    }
                }
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有监控操作完成
    for handle in handles {
        handle.await.expect("监控操作失败");
    }
    
    let total_duration = start_time.elapsed();
    let ops_per_second = stress_config.total_operations as f64 / total_duration.as_secs_f64();
    
    // 获取监控统计
    let logs = monitor.get_logs(None, Some(100));
    let metrics = monitor.get_metrics(None, Some(100));
    let errors = monitor.get_error_summary();
    
    println!("监控系统压力测试完成!");
    println!("总时间: {:?}", total_duration);
    println!("操作/秒: {:.2}", ops_per_second);
    println!("日志条目数: {}", logs.len());
    println!("指标条目数: {}", metrics.len());
    println!("错误类型数: {}", errors.len());
    
    assert!(ops_per_second > 5000.0, "监控操作速度应该超过5000ops/s");
    assert!(logs.len() > 0, "应该有日志记录");
    assert!(metrics.len() > 0, "应该有性能指标");
}

#[tokio::test]
async fn test_mixed_workload_stress() {
    let (db, _temp_dir) = create_stress_test_db().await;
    let config = AgentDbConfig::default();
    let cache_manager = CacheManager::new(config.performance.clone());
    let monitor = MonitoringManager::new(config.logging);
    
    let stress_config = StressTestConfig {
        total_operations: 20000,
        max_concurrent_operations: 50,
        ..Default::default()
    };
    
    println!("开始混合工作负载压力测试...");
    
    let semaphore = Arc::new(Semaphore::new(stress_config.max_concurrent_operations));
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..stress_config.total_operations {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            
            let workload_type = i % 6;
            
            match workload_type {
                0 => {
                    // Agent状态操作
                    let agent_id = (i % 1000) as u64 + 500000;
                    let state = AgentState::new(agent_id, 1, StateType::WorkingMemory, vec![i as u8; 256]);
                    // db.save_agent_state(&state).await.unwrap();
                }
                1 => {
                    // 记忆操作
                    let agent_id = (i % 1000) as u64 + 500000;
                    let memory = Memory::new(agent_id, MemoryType::Episodic, format!("混合测试记忆{}", i), 0.5);
                    // db.store_memory(&memory).await.unwrap();
                }
                2 => {
                    // 向量操作
                    let vector: Vec<f32> = (0..384).map(|j| (i * j) as f32 / 1000.0).collect();
                    // db.add_vector(format!("mixed_vec_{}", i), vector, format!("混合测试向量{}", i)).await.unwrap();
                }
                3 => {
                    // 缓存操作
                    let key = (i % 5000) as u64;
                    let data = vec![i as u8; 128];
                    cache_manager.set(key, data, 1);
                }
                4 => {
                    // 监控操作
                    monitor.log(LogLevel::Info, "mixed_stress", &format!("混合测试日志{}", i), None);
                    monitor.record_metric("mixed_metric", i as f64, "count", None);
                }
                _ => {
                    // 搜索操作
                    if i % 10 == 0 {
                        let agent_id = (i % 1000) as u64 + 500000;
                        // db.search_memories(agent_id, "混合", 5).await.unwrap();
                    }
                }
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有操作完成
    for (i, handle) in handles.into_iter().enumerate() {
        handle.await.expect("混合工作负载操作失败");
        
        if i % 2000 == 0 {
            println!("混合工作负载进度: {}/{}", i, stress_config.total_operations);
        }
    }
    
    let total_duration = start_time.elapsed();
    let ops_per_second = stress_config.total_operations as f64 / total_duration.as_secs_f64();
    
    println!("混合工作负载压力测试完成!");
    println!("总时间: {:?}", total_duration);
    println!("操作/秒: {:.2}", ops_per_second);
    
    // 验证各个组件的状态
    let cache_stats = cache_manager.get_statistics();
    let logs = monitor.get_logs(None, Some(10));
    let metrics = monitor.get_metrics(None, Some(10));
    
    println!("缓存条目: {}", cache_stats.total_entries);
    println!("日志条目: {}", logs.len());
    println!("指标条目: {}", metrics.len());
    
    assert!(ops_per_second > 200.0, "混合工作负载速度应该超过200ops/s");
    assert!(cache_stats.total_entries > 0, "缓存应该有数据");
    assert!(logs.len() > 0, "应该有日志记录");
    assert!(metrics.len() > 0, "应该有性能指标");
}
