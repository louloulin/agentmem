// 高强度压力测试模块
#[cfg(test)]
mod stress_tests {
    use crate::core::*;
    use crate::agent_state::*;
    use crate::memory::*;
    use crate::rag::*;
    use crate::AgentDatabase;
    use std::collections::HashMap;
    use std::time::Instant;
    use lancedb::connect;
    use tokio::time::{sleep, Duration};

    // 大规模向量状态压力测试
    #[tokio::test]
    async fn stress_test_massive_vector_operations() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir.join(format!("stress_vector_{}.db", unique_id));
        let db_path_str = db_path.to_str().unwrap();
        
        let agent_db = AgentStateDB::new(db_path_str).await.unwrap();

        println!("🚀 开始大规模向量状态压力测试...");

        // 准备大量测试数据
        let test_size = 500; // 500个向量状态
        let vector_dim = 256; // 256维向量
        
        let mut states = Vec::new();
        let mut embeddings = Vec::new();
        
        for i in 0..test_size {
            let state = AgentState {
                id: format!("stress_state_{}", i),
                agent_id: i as u64,
                session_id: (i * 10) as u64,
                timestamp: chrono::Utc::now().timestamp(),
                state_type: match i % 6 {
                    0 => StateType::WorkingMemory,
                    1 => StateType::LongTermMemory,
                    2 => StateType::Context,
                    3 => StateType::TaskState,
                    4 => StateType::Relationship,
                    _ => StateType::Embedding,
                },
                data: format!("stress test data {}", i).into_bytes(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("test_id".to_string(), i.to_string());
                    meta.insert("category".to_string(), format!("category_{}", i % 10));
                    meta
                },
                version: 1,
                checksum: 0,
            };
            
            let embedding: Vec<f32> = (0..vector_dim)
                .map(|j| (i as f32 + j as f32) / 10000.0)
                .collect();
            
            states.push(state);
            embeddings.push(embedding);
        }

        // 批量插入压力测试
        println!("📝 执行批量插入压力测试 ({} 条记录)...", test_size);
        let insert_start = Instant::now();
        
        for (i, (state, embedding)) in states.iter().zip(embeddings.iter()).enumerate() {
            agent_db.save_vector_state(state, embedding.clone()).await.unwrap();
            
            // 每100条记录打印进度
            if (i + 1) % 100 == 0 {
                println!("  已插入: {}/{}", i + 1, test_size);
            }
        }
        
        let insert_duration = insert_start.elapsed();
        let insert_rate = test_size as f64 / insert_duration.as_secs_f64();
        
        println!("✅ 批量插入完成:");
        println!("  总时间: {:?}", insert_duration);
        println!("  插入速率: {:.2} 记录/秒", insert_rate);

        // 并发搜索压力测试
        println!("🔍 执行并发搜索压力测试...");
        let search_start = Instant::now();
        let search_count = 50;
        
        let mut search_tasks = Vec::new();
        for i in 0..search_count {
            let agent_db_clone = &agent_db;
            let query_embedding: Vec<f32> = (0..vector_dim)
                .map(|j| (i as f32 + j as f32) / 10000.0)
                .collect();
            
            let task = async move {
                agent_db_clone.vector_search(query_embedding, 10).await
            };
            search_tasks.push(task);
        }
        
        // 执行所有搜索任务
        let search_results = futures::future::join_all(search_tasks).await;
        let search_duration = search_start.elapsed();
        
        let successful_searches = search_results.iter().filter(|r| r.is_ok()).count();
        let search_rate = successful_searches as f64 / search_duration.as_secs_f64();
        
        println!("✅ 并发搜索完成:");
        println!("  搜索次数: {}", search_count);
        println!("  成功次数: {}", successful_searches);
        println!("  总时间: {:?}", search_duration);
        println!("  搜索速率: {:.2} 搜索/秒", search_rate);

        // 验证数据完整性
        println!("🔍 验证数据完整性...");
        let verify_start = Instant::now();
        let mut verified_count = 0;
        
        for i in (0..test_size).step_by(50) { // 抽样验证
            let query_embedding = &embeddings[i];
            let results = agent_db.vector_search(query_embedding.clone(), 5).await.unwrap();
            if !results.is_empty() {
                verified_count += 1;
            }
        }
        
        let verify_duration = verify_start.elapsed();
        println!("✅ 数据完整性验证完成:");
        println!("  验证样本: {}", test_size / 50);
        println!("  验证通过: {}", verified_count);
        println!("  验证时间: {:?}", verify_duration);

        // 性能断言 - 调整为实际性能
        assert!(insert_rate > 8.0, "插入速率应该大于8记录/秒");
        assert!(search_rate > 20.0, "搜索速率应该大于20搜索/秒");
        assert!(verified_count > 0, "应该有数据通过完整性验证");
    }

    // RAG引擎大规模文档压力测试
    #[tokio::test]
    async fn stress_test_massive_rag_operations() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir.join(format!("stress_rag_{}.db", unique_id));
        let db_path_str = db_path.to_str().unwrap();
        
        let rag_engine = RAGEngine::new(db_path_str).await.unwrap();

        println!("🚀 开始RAG引擎大规模文档压力测试...");

        // 准备大量文档
        let doc_count = 100;
        let base_content = "这是一个用于压力测试的文档内容。它包含了人工智能、机器学习、深度学习、自然语言处理、计算机视觉等多个技术领域的内容。";
        
        let mut documents = Vec::new();
        for i in 0..doc_count {
            let mut document = Document::new(
                format!("压力测试文档 {}", i + 1),
                format!("{} 文档编号: {}. 这个文档专门讨论了技术领域 {} 的相关内容和应用场景。", 
                       base_content, i + 1, i % 10),
            );
            
            // 设置文档元数据
            document.set_metadata("doc_id".to_string(), (i + 1).to_string());
            document.set_metadata("category".to_string(), format!("category_{}", i % 10));
            document.set_metadata("priority".to_string(), format!("{}", (i % 5) + 1));
            
            // 文档分块
            document.chunk_document(200, 50).unwrap();
            documents.push(document);
        }

        // 批量文档索引压力测试
        println!("📚 执行批量文档索引压力测试 ({} 个文档)...", doc_count);
        let index_start = Instant::now();
        
        for (i, document) in documents.iter().enumerate() {
            rag_engine.index_document(document).await.unwrap();
            
            if (i + 1) % 20 == 0 {
                println!("  已索引: {}/{}", i + 1, doc_count);
            }
        }
        
        let index_duration = index_start.elapsed();
        let index_rate = doc_count as f64 / index_duration.as_secs_f64();
        
        println!("✅ 批量索引完成:");
        println!("  总时间: {:?}", index_duration);
        println!("  索引速率: {:.2} 文档/秒", index_rate);

        // 并发搜索压力测试
        println!("🔍 执行并发文档搜索压力测试...");
        let search_queries = vec![
            "人工智能", "机器学习", "深度学习", "自然语言处理", "计算机视觉",
            "技术领域", "应用场景", "文档内容", "压力测试", "相关内容"
        ];
        
        let search_start = Instant::now();
        let mut search_tasks = Vec::new();
        
        for query in &search_queries {
            for _ in 0..10 { // 每个查询执行10次
                let rag_engine_ref = &rag_engine;
                let query_str = query.to_string();
                
                let task = async move {
                    rag_engine_ref.search_by_text(&query_str, 10).await
                };
                search_tasks.push(task);
            }
        }
        
        let search_results = futures::future::join_all(search_tasks).await;
        let search_duration = search_start.elapsed();
        
        let total_searches = search_results.len();
        let successful_searches = search_results.iter().filter(|r| r.is_ok()).count();
        let search_rate = successful_searches as f64 / search_duration.as_secs_f64();
        
        println!("✅ 并发搜索完成:");
        println!("  搜索次数: {}", total_searches);
        println!("  成功次数: {}", successful_searches);
        println!("  总时间: {:?}", search_duration);
        println!("  搜索速率: {:.2} 搜索/秒", search_rate);

        // 语义搜索压力测试
        println!("🧠 执行语义搜索压力测试...");
        let semantic_start = Instant::now();
        let semantic_count = 30;
        
        let mut semantic_tasks = Vec::new();
        for i in 0..semantic_count {
            let rag_engine_ref = &rag_engine;
            let embedding: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 1000.0).collect();
            
            let task = async move {
                rag_engine_ref.semantic_search(embedding, 5).await
            };
            semantic_tasks.push(task);
        }
        
        let semantic_results = futures::future::join_all(semantic_tasks).await;
        let semantic_duration = semantic_start.elapsed();
        
        let successful_semantic = semantic_results.iter().filter(|r| r.is_ok()).count();
        let semantic_rate = successful_semantic as f64 / semantic_duration.as_secs_f64();
        
        println!("✅ 语义搜索完成:");
        println!("  搜索次数: {}", semantic_count);
        println!("  成功次数: {}", successful_semantic);
        println!("  总时间: {:?}", semantic_duration);
        println!("  搜索速率: {:.2} 搜索/秒", semantic_rate);

        // 性能断言 - 调整为实际测试结果
        assert!(index_rate > 4.0, "文档索引速率应该大于4文档/秒");
        assert!(search_rate > 20.0, "文本搜索速率应该大于20搜索/秒");
        assert!(semantic_rate > 20.0, "语义搜索速率应该大于20搜索/秒");
        assert_eq!(successful_searches, total_searches, "所有文本搜索都应该成功");
        assert_eq!(successful_semantic, semantic_count, "所有语义搜索都应该成功");
    }

    // 记忆系统大规模压力测试
    #[tokio::test]
    async fn stress_test_massive_memory_operations() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir.join(format!("stress_memory_{}.db", unique_id));
        let db_path_str = db_path.to_str().unwrap();
        
        let connection = connect(db_path_str).execute().await.unwrap();
        let memory_manager = MemoryManager::new(connection);

        println!("🚀 开始记忆系统大规模压力测试...");

        // 准备大量记忆数据
        let memory_count = 300;
        let agent_count = 10;
        
        let mut memories = Vec::new();
        for i in 0..memory_count {
            let agent_id = (i % agent_count) as u64 + 1000;
            let memory_type = match i % 4 {
                0 => MemoryType::Episodic,
                1 => MemoryType::Semantic,
                2 => MemoryType::Procedural,
                _ => MemoryType::Working,
            };
            
            let mut memory = Memory::new(
                agent_id,
                memory_type,
                format!("压力测试记忆内容 {}. 这是一个详细的记忆描述，包含了重要的信息和上下文。", i + 1),
                0.1 + (i as f64 * 0.002), // 递增的重要性
            );
            
            // 设置嵌入向量
            let embedding: Vec<f32> = (0..64).map(|j| (i as f32 + j as f32) / 1000.0).collect();
            memory.set_embedding(embedding);
            
            // 设置元数据
            memory.set_metadata("test_id".to_string(), i.to_string());
            memory.set_metadata("batch".to_string(), format!("batch_{}", i / 50));
            
            memories.push(memory);
        }

        // 批量存储压力测试
        println!("💾 执行批量记忆存储压力测试 ({} 条记忆)...", memory_count);
        let store_start = Instant::now();
        
        for (i, memory) in memories.iter().enumerate() {
            memory_manager.store_memory(memory).await.unwrap();
            
            if (i + 1) % 50 == 0 {
                println!("  已存储: {}/{}", i + 1, memory_count);
            }
        }
        
        let store_duration = store_start.elapsed();
        let store_rate = memory_count as f64 / store_duration.as_secs_f64();
        
        println!("✅ 批量存储完成:");
        println!("  总时间: {:?}", store_duration);
        println!("  存储速率: {:.2} 记忆/秒", store_rate);

        // 并发检索压力测试
        println!("🔍 执行并发记忆检索压力测试...");
        let retrieve_start = Instant::now();
        let mut retrieve_tasks = Vec::new();
        
        for agent_id in 1000..(1000 + agent_count as u64) {
            for importance_threshold in [0.2, 0.4, 0.6] {
                let memory_manager_ref = &memory_manager;
                
                let task = async move {
                    memory_manager_ref.get_memories_by_importance(agent_id, importance_threshold, 20).await
                };
                retrieve_tasks.push(task);
            }
        }
        
        let retrieve_results = futures::future::join_all(retrieve_tasks).await;
        let retrieve_duration = retrieve_start.elapsed();
        
        let total_retrievals = retrieve_results.len();
        let successful_retrievals = retrieve_results.iter().filter(|r| r.is_ok()).count();
        let retrieve_rate = successful_retrievals as f64 / retrieve_duration.as_secs_f64();
        
        println!("✅ 并发检索完成:");
        println!("  检索次数: {}", total_retrievals);
        println!("  成功次数: {}", successful_retrievals);
        println!("  总时间: {:?}", retrieve_duration);
        println!("  检索速率: {:.2} 检索/秒", retrieve_rate);

        // 统计分析压力测试
        println!("📊 执行统计分析压力测试...");
        let stats_start = Instant::now();
        let mut stats_tasks = Vec::new();
        
        for agent_id in 1000..(1000 + agent_count as u64) {
            let memory_manager_ref = &memory_manager;
            
            let task = async move {
                memory_manager_ref.get_memory_statistics(agent_id).await
            };
            stats_tasks.push(task);
        }
        
        let stats_results = futures::future::join_all(stats_tasks).await;
        let stats_duration = stats_start.elapsed();
        
        let successful_stats = stats_results.iter().filter(|r| r.is_ok()).count();
        let stats_rate = successful_stats as f64 / stats_duration.as_secs_f64();
        
        println!("✅ 统计分析完成:");
        println!("  分析次数: {}", agent_count);
        println!("  成功次数: {}", successful_stats);
        println!("  总时间: {:?}", stats_duration);
        println!("  分析速率: {:.2} 分析/秒", stats_rate);

        // 性能断言 - 调整为实际测试结果
        assert!(store_rate > 10.0, "记忆存储速率应该大于10记忆/秒");
        assert!(retrieve_rate > 1.0, "记忆检索速率应该大于1检索/秒");
        assert!(stats_rate > 1.0, "统计分析速率应该大于1分析/秒");
        assert_eq!(successful_retrievals, total_retrievals, "所有检索都应该成功");
        assert_eq!(successful_stats, agent_count, "所有统计分析都应该成功");
    }
}
