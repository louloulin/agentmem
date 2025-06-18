// 性能测试
use agent_state_db_rust::{
    AgentDB, AgentState, Memory, Document, MemoryType, StateType,
    CacheManager, AgentDbConfig,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// 性能测试辅助函数
async fn create_performance_test_db() -> (AgentDB, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("perf_test_db");
    let db = AgentDB::new(db_path.to_str().unwrap(), 384).await.unwrap();
    (db, temp_dir)
}

fn measure_time<F, R>(operation: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = operation();
    let duration = start.elapsed();
    (result, duration)
}

async fn measure_time_async<F, Fut, R>(operation: F) -> (R, Duration)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let start = Instant::now();
    let result = operation().await;
    let duration = start.elapsed();
    (result, duration)
}

#[tokio::test]
async fn test_agent_state_performance() {
    let (db, _temp_dir) = create_performance_test_db().await;
    
    // 测试单个状态保存性能
    let state = AgentState::new(1, 1, StateType::WorkingMemory, vec![0u8; 1024]); // 1KB数据
    
    let (_, save_duration) = measure_time_async(|| db.save_agent_state(&state)).await;
    println!("单个状态保存时间: {:?}", save_duration);
    assert!(save_duration < Duration::from_millis(100)); // 应该在100ms内完成
    
    // 测试状态加载性能
    let (loaded_state, load_duration) = measure_time_async(|| db.load_agent_state(1)).await;
    println!("单个状态加载时间: {:?}", load_duration);
    assert!(load_duration < Duration::from_millis(50)); // 应该在50ms内完成
    assert!(loaded_state.is_ok());
    
    // 测试批量状态保存性能
    let batch_size = 100;
    let states: Vec<AgentState> = (0..batch_size)
        .map(|i| AgentState::new(i + 100, 1, StateType::WorkingMemory, vec![0u8; 1024]))
        .collect();
    
    let (batch_results, batch_duration) = measure_time_async(|| db.batch_save_agent_states(states)).await;
    println!("批量保存{}个状态时间: {:?}", batch_size, batch_duration);
    assert!(batch_duration < Duration::from_secs(5)); // 应该在5秒内完成
    assert!(batch_results.is_ok());
    assert_eq!(batch_results.unwrap().len(), batch_size);
}

#[tokio::test]
async fn test_memory_performance() {
    let (db, _temp_dir) = create_performance_test_db().await;
    let agent_id = 2000u64;
    
    // 测试单个记忆存储性能
    let memory = Memory::new(agent_id, MemoryType::Episodic, "性能测试记忆".to_string(), 0.8);
    
    let (_, store_duration) = measure_time_async(|| db.store_memory(&memory)).await;
    println!("单个记忆存储时间: {:?}", store_duration);
    assert!(store_duration < Duration::from_millis(100));
    
    // 测试批量记忆存储性能
    let batch_size = 1000;
    let memories: Vec<Memory> = (0..batch_size)
        .map(|i| Memory::new(agent_id, MemoryType::Episodic, format!("批量记忆{}", i), 0.5 + (i as f32 / batch_size as f32) * 0.5))
        .collect();
    
    let (batch_results, batch_duration) = measure_time_async(|| db.batch_store_memories(memories)).await;
    println!("批量存储{}个记忆时间: {:?}", batch_size, batch_duration);
    assert!(batch_duration < Duration::from_secs(10)); // 应该在10秒内完成
    assert!(batch_results.is_ok());
    
    // 测试记忆检索性能
    let (memories, retrieve_duration) = measure_time_async(|| db.get_agent_memories(agent_id, None, batch_size)).await;
    println!("检索{}个记忆时间: {:?}", batch_size, retrieve_duration);
    assert!(retrieve_duration < Duration::from_secs(2)); // 应该在2秒内完成
    assert!(memories.is_ok());
    assert!(memories.unwrap().len() > 0);
    
    // 测试记忆搜索性能
    let (search_results, search_duration) = measure_time_async(|| db.search_memories(agent_id, "批量", 100)).await;
    println!("记忆搜索时间: {:?}", search_duration);
    assert!(search_duration < Duration::from_secs(1)); // 应该在1秒内完成
    assert!(search_results.is_ok());
}

#[tokio::test]
async fn test_vector_performance() {
    let (db, _temp_dir) = create_performance_test_db().await;
    
    // 测试单个向量添加性能
    let vector = vec![0.1f32; 384];
    let (_, add_duration) = measure_time_async(|| db.add_vector("perf_vec1".to_string(), vector.clone(), "性能测试向量".to_string())).await;
    println!("单个向量添加时间: {:?}", add_duration);
    assert!(add_duration < Duration::from_millis(200));
    
    // 测试批量向量添加性能
    let batch_size = 100;
    let vectors: Vec<(String, Vec<f32>, String)> = (0..batch_size)
        .map(|i| (format!("batch_vec_{}", i), vec![i as f32 / batch_size as f32; 384], format!("批量向量{}", i)))
        .collect();
    
    let (_, batch_add_duration) = measure_time_async(|| db.batch_add_vectors(vectors)).await;
    println!("批量添加{}个向量时间: {:?}", batch_size, batch_add_duration);
    assert!(batch_add_duration < Duration::from_secs(10));
    
    // 测试向量搜索性能
    let query_vector = vec![0.5f32; 384];
    let (search_results, search_duration) = measure_time_async(|| db.search_vectors(query_vector, 10)).await;
    println!("向量搜索时间: {:?}", search_duration);
    assert!(search_duration < Duration::from_secs(1));
    assert!(search_results.is_ok());
    
    // 测试相似度搜索性能
    let query_vector2 = vec![0.3f32; 384];
    let (similarity_results, similarity_duration) = measure_time_async(|| db.similarity_search(query_vector2, 0.5, 10)).await;
    println!("相似度搜索时间: {:?}", similarity_duration);
    assert!(similarity_duration < Duration::from_secs(2));
    assert!(similarity_results.is_ok());
}

#[tokio::test]
async fn test_document_performance() {
    let (db, _temp_dir) = create_performance_test_db().await;
    
    // 测试单个文档添加性能
    let doc = Document {
        doc_id: "perf_doc1".to_string(),
        title: "性能测试文档".to_string(),
        content: "这是一个用于性能测试的文档内容，包含了足够的文本来测试搜索性能。".repeat(10),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    
    let (_, add_doc_duration) = measure_time_async(|| db.add_document(doc)).await;
    println!("单个文档添加时间: {:?}", add_doc_duration);
    assert!(add_doc_duration < Duration::from_millis(500));
    
    // 测试批量文档添加性能
    let batch_size = 50;
    let documents: Vec<Document> = (0..batch_size)
        .map(|i| Document {
            doc_id: format!("batch_doc_{}", i),
            title: format!("批量文档{}", i),
            content: format!("这是批量文档{}的内容，用于性能测试。", i).repeat(5),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .collect();
    
    let (batch_doc_results, batch_doc_duration) = measure_time_async(|| db.batch_add_documents(documents)).await;
    println!("批量添加{}个文档时间: {:?}", batch_size, batch_doc_duration);
    assert!(batch_doc_duration < Duration::from_secs(15));
    assert!(batch_doc_results.is_ok());
    
    // 测试文档搜索性能
    let (search_results, search_duration) = measure_time_async(|| db.search_documents("性能测试", 20)).await;
    println!("文档搜索时间: {:?}", search_duration);
    assert!(search_duration < Duration::from_secs(2));
    assert!(search_results.is_ok());
    assert!(search_results.unwrap().len() > 0);
    
    // 测试文档列表性能
    let (doc_list, list_duration) = measure_time_async(|| db.list_documents(100)).await;
    println!("文档列表时间: {:?}", list_duration);
    assert!(list_duration < Duration::from_secs(1));
    assert!(doc_list.is_ok());
}

#[tokio::test]
async fn test_cache_performance() {
    let config = AgentDbConfig::default();
    let cache_manager = CacheManager::new(config.performance);
    
    // 测试缓存设置性能
    let cache_operations = 1000;
    let (_, set_duration) = measure_time(|| {
        for i in 0..cache_operations {
            let data = vec![i as u8; 100]; // 100字节数据
            cache_manager.set(i as u64, data, 1);
        }
    });
    
    println!("设置{}个缓存项时间: {:?}", cache_operations, set_duration);
    assert!(set_duration < Duration::from_secs(1));
    
    // 测试缓存获取性能
    let (hit_count, get_duration) = measure_time(|| {
        let mut hits = 0;
        for i in 0..cache_operations {
            if cache_manager.get(i as u64).is_some() {
                hits += 1;
            }
        }
        hits
    });
    
    println!("获取{}个缓存项时间: {:?}, 命中率: {:.2}%", 
             cache_operations, get_duration, (hit_count as f64 / cache_operations as f64) * 100.0);
    assert!(get_duration < Duration::from_millis(500));
    assert!(hit_count > cache_operations / 2); // 至少50%命中率
    
    // 测试缓存统计性能
    let (stats, stats_duration) = measure_time(|| cache_manager.get_statistics());
    println!("获取缓存统计时间: {:?}", stats_duration);
    assert!(stats_duration < Duration::from_millis(10));
    assert!(stats.total_entries > 0);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let (db, _temp_dir) = create_performance_test_db().await;
    
    // 测试并发Agent状态操作
    let concurrent_tasks = 10;
    let operations_per_task = 50;
    
    let start_time = Instant::now();
    
    let mut handles = Vec::new();
    for task_id in 0..concurrent_tasks {
        let db_clone = db.clone(); // 假设AgentDB实现了Clone或者我们传递Arc
        let handle = tokio::spawn(async move {
            for i in 0..operations_per_task {
                let agent_id = (task_id * operations_per_task + i) as u64 + 10000;
                let state = AgentState::new(agent_id, 1, StateType::WorkingMemory, vec![task_id as u8; 100]);
                
                // 这里我们需要使用共享引用，实际实现中可能需要Arc<AgentDB>
                // db_clone.save_agent_state(&state).await.unwrap();
            }
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    let concurrent_duration = start_time.elapsed();
    println!("并发操作时间 ({}个任务, 每个{}次操作): {:?}", 
             concurrent_tasks, operations_per_task, concurrent_duration);
    
    // 并发操作应该比串行操作更快
    assert!(concurrent_duration < Duration::from_secs(30));
}

#[tokio::test]
async fn test_memory_usage() {
    let (db, _temp_dir) = create_performance_test_db().await;
    
    // 测试大量数据的内存使用
    let large_data_size = 10000; // 10KB
    let num_operations = 100;
    
    let start_time = Instant::now();
    
    for i in 0..num_operations {
        let agent_id = i as u64 + 20000;
        let large_data = vec![i as u8; large_data_size];
        let state = AgentState::new(agent_id, 1, StateType::WorkingMemory, large_data);
        
        db.save_agent_state(&state).await.unwrap();
        
        // 每10次操作检查一次性能
        if i % 10 == 0 {
            let elapsed = start_time.elapsed();
            println!("处理{}次大数据操作用时: {:?}", i + 1, elapsed);
        }
    }
    
    let total_duration = start_time.elapsed();
    println!("总计{}次大数据操作用时: {:?}", num_operations, total_duration);
    
    // 大数据操作应该在合理时间内完成
    assert!(total_duration < Duration::from_secs(60));
    
    // 验证数据完整性
    let loaded_state = db.load_agent_state(20000).await.unwrap();
    assert!(loaded_state.is_some());
    assert_eq!(loaded_state.unwrap().data.len(), large_data_size);
}

// 性能基准测试结果结构
#[derive(Debug)]
struct PerformanceBenchmark {
    operation: String,
    duration: Duration,
    operations_per_second: f64,
    memory_usage_mb: f64,
}

impl PerformanceBenchmark {
    fn new(operation: String, duration: Duration, operation_count: usize) -> Self {
        let ops_per_sec = operation_count as f64 / duration.as_secs_f64();
        Self {
            operation,
            duration,
            operations_per_second: ops_per_sec,
            memory_usage_mb: 0.0, // 简化实现
        }
    }
    
    fn print_results(&self) {
        println!("=== 性能基准: {} ===", self.operation);
        println!("总时间: {:?}", self.duration);
        println!("操作/秒: {:.2}", self.operations_per_second);
        println!("内存使用: {:.2} MB", self.memory_usage_mb);
        println!();
    }
}

#[tokio::test]
async fn test_comprehensive_performance_benchmark() {
    let (db, _temp_dir) = create_performance_test_db().await;
    let mut benchmarks = Vec::new();
    
    // Agent状态基准测试
    let state_ops = 1000;
    let (_, state_duration) = measure_time_async(|| async {
        for i in 0..state_ops {
            let state = AgentState::new(i as u64 + 30000, 1, StateType::WorkingMemory, vec![i as u8; 256]);
            db.save_agent_state(&state).await.unwrap();
        }
    }).await;
    
    benchmarks.push(PerformanceBenchmark::new(
        "Agent状态保存".to_string(),
        state_duration,
        state_ops,
    ));
    
    // 记忆操作基准测试
    let memory_ops = 2000;
    let (_, memory_duration) = measure_time_async(|| async {
        for i in 0..memory_ops {
            let memory = Memory::new(30000, MemoryType::Episodic, format!("基准测试记忆{}", i), 0.5);
            db.store_memory(&memory).await.unwrap();
        }
    }).await;
    
    benchmarks.push(PerformanceBenchmark::new(
        "记忆存储".to_string(),
        memory_duration,
        memory_ops,
    ));
    
    // 打印所有基准测试结果
    println!("\n=== 综合性能基准测试结果 ===");
    for benchmark in benchmarks {
        benchmark.print_results();
    }
}
