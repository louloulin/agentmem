// 性能基准测试模块
#[cfg(test)]
mod benchmarks {
    use crate::core::*;
    use crate::agent_state::*;
    use crate::memory::*;
    use crate::rag::*;
    use crate::AgentDatabase;
    use std::collections::HashMap;
    use std::time::Instant;
    use lancedb::connect;

    // 向量搜索性能测试
    #[tokio::test]
    async fn benchmark_vector_search() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir.join(format!("benchmark_vector_{}.db", unique_id));
        let db_path_str = db_path.to_str().unwrap();
        
        let agent_db = AgentStateDB::new(db_path_str).await.unwrap();

        // 准备测试数据
        let mut states = Vec::new();
        let mut embeddings = Vec::new();
        
        for i in 0..100 {
            let state = AgentState {
                id: format!("benchmark_state_{}", i),
                agent_id: i,
                session_id: i * 10,
                timestamp: chrono::Utc::now().timestamp(),
                state_type: StateType::Embedding,
                data: format!("benchmark data {}", i).into_bytes(),
                metadata: HashMap::new(),
                version: 1,
                checksum: 0,
            };
            
            let embedding: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 1000.0).collect();
            
            states.push(state);
            embeddings.push(embedding);
        }

        // 批量插入数据
        let insert_start = Instant::now();
        for (state, embedding) in states.iter().zip(embeddings.iter()) {
            agent_db.save_vector_state(state, embedding.clone()).await.unwrap();
        }
        let insert_duration = insert_start.elapsed();
        println!("向量状态批量插入 (100条): {:?}", insert_duration);

        // 向量搜索性能测试
        let query_embedding: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        
        let search_start = Instant::now();
        let search_results = agent_db.vector_search(query_embedding, 10).await.unwrap();
        let search_duration = search_start.elapsed();
        
        println!("向量搜索 (返回10条): {:?}", search_duration);
        println!("搜索结果数量: {}", search_results.len());
        
        assert!(!search_results.is_empty(), "应该找到搜索结果");
        assert!(search_duration.as_millis() < 1000, "搜索应该在1秒内完成");
    }

    // RAG引擎性能测试
    #[tokio::test]
    async fn benchmark_rag_engine() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir.join(format!("benchmark_rag_{}.db", unique_id));
        let db_path_str = db_path.to_str().unwrap();
        
        let rag_engine = RAGEngine::new(db_path_str).await.unwrap();

        // 准备测试文档
        let documents = vec![
            "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。",
            "机器学习是人工智能的一个子集，它使计算机能够在没有明确编程的情况下学习和改进。",
            "深度学习是机器学习的一个分支，使用神经网络来模拟人脑的工作方式。",
            "自然语言处理是人工智能的一个领域，专注于计算机与人类语言之间的交互。",
            "计算机视觉是人工智能的一个分支，使计算机能够理解和解释视觉信息。",
        ];

        // 文档索引性能测试
        let index_start = Instant::now();
        for (i, content) in documents.iter().enumerate() {
            let mut document = Document::new(
                format!("测试文档 {}", i + 1),
                content.to_string(),
            );
            document.chunk_document(100, 20).unwrap();
            rag_engine.index_document(&document).await.unwrap();
        }
        let index_duration = index_start.elapsed();
        println!("文档索引 (5个文档): {:?}", index_duration);

        // 文本搜索性能测试
        let search_start = Instant::now();
        let search_results = rag_engine.search_by_text("人工智能", 10).await.unwrap();
        let search_duration = search_start.elapsed();
        
        println!("文本搜索: {:?}", search_duration);
        println!("搜索结果数量: {}", search_results.len());

        // 语义搜索性能测试
        let embedding: Vec<f32> = (0..128).map(|i| i as f32 / 100.0).collect();
        let semantic_start = Instant::now();
        let semantic_results = rag_engine.semantic_search(embedding, 10).await.unwrap();
        let semantic_duration = semantic_start.elapsed();
        
        println!("语义搜索: {:?}", semantic_duration);
        println!("语义搜索结果数量: {}", semantic_results.len());

        // 上下文构建性能测试
        let context_start = Instant::now();
        let context = rag_engine.build_context("人工智能查询", search_results, 1000).await.unwrap();
        let context_duration = context_start.elapsed();
        
        println!("上下文构建: {:?}", context_duration);
        println!("上下文长度: {} 字符", context.context_window.len());

        assert!(!semantic_results.is_empty(), "应该找到语义搜索结果");
        assert!(!context.context_window.is_empty(), "应该构建上下文");
        assert!(search_duration.as_millis() < 500, "文本搜索应该在500ms内完成");
        assert!(semantic_duration.as_millis() < 500, "语义搜索应该在500ms内完成");
    }

    // 记忆管理性能测试
    #[tokio::test]
    async fn benchmark_memory_management() {
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir.join(format!("benchmark_memory_{}.db", unique_id));
        let db_path_str = db_path.to_str().unwrap();
        
        let connection = connect(db_path_str).execute().await.unwrap();
        let memory_manager = MemoryManager::new(connection);

        // 批量创建记忆
        let mut memories = Vec::new();
        for i in 0..50 {
            let memory = Memory::new(
                12345,
                if i % 3 == 0 { MemoryType::Episodic } 
                else if i % 3 == 1 { MemoryType::Semantic } 
                else { MemoryType::Working },
                format!("测试记忆内容 {}", i),
                0.1 + (i as f64 * 0.01),
            );
            memories.push(memory);
        }

        // 批量存储性能测试
        let store_start = Instant::now();
        for memory in &memories {
            memory_manager.store_memory(memory).await.unwrap();
        }
        let store_duration = store_start.elapsed();
        println!("记忆批量存储 (50条): {:?}", store_duration);

        // 重要性检索性能测试
        let importance_start = Instant::now();
        let important_memories = memory_manager.get_memories_by_importance(12345, 0.3, 20).await.unwrap();
        let importance_duration = importance_start.elapsed();
        
        println!("重要性检索: {:?}", importance_duration);
        println!("重要记忆数量: {}", important_memories.len());

        // 相似性搜索性能测试
        let embedding: Vec<f32> = (0..64).map(|i| i as f32 / 100.0).collect();
        let similarity_start = Instant::now();
        let similar_memories = memory_manager.search_similar_memories(12345, embedding, 10).await.unwrap();
        let similarity_duration = similarity_start.elapsed();
        
        println!("相似性搜索: {:?}", similarity_duration);
        println!("相似记忆数量: {}", similar_memories.len());

        // 统计分析性能测试
        let stats_start = Instant::now();
        let stats = memory_manager.get_memory_statistics(12345).await.unwrap();
        let stats_duration = stats_start.elapsed();
        
        println!("统计分析: {:?}", stats_duration);
        println!("总记忆数: {}", stats.total_count);

        assert!(store_duration.as_millis() < 6000, "批量存储应该在6秒内完成");
        assert!(importance_duration.as_millis() < 500, "重要性检索应该在500ms内完成");
        assert!(similarity_duration.as_millis() < 500, "相似性搜索应该在500ms内完成");
        assert!(stats_duration.as_millis() < 300, "统计分析应该在300ms内完成");
    }

    // 集成性能测试
    #[tokio::test]
    async fn benchmark_integrated_workflow() {
        let config = DatabaseConfig::default();
        let mut db = AgentDatabase::new(config).await.unwrap();
        db = db.with_rag_engine().await.unwrap();

        let workflow_start = Instant::now();

        // 1. 保存Agent状态
        let state = AgentState {
            id: "workflow_test".to_string(),
            agent_id: 99999,
            session_id: 88888,
            timestamp: chrono::Utc::now().timestamp(),
            state_type: StateType::Embedding,
            data: b"workflow test data".to_vec(),
            metadata: HashMap::new(),
            version: 1,
            checksum: 0,
        };
        let embedding = vec![0.5, 0.4, 0.3, 0.2, 0.1];
        db.save_vector_state(&state, embedding.clone()).await.unwrap();

        // 2. 索引文档
        let mut document = Document::new(
            "工作流测试文档".to_string(),
            "这是一个完整的工作流测试文档，用于验证系统的整体性能。".to_string(),
        );
        document.chunk_document(50, 10).unwrap();
        db.index_document(&document).await.unwrap();

        // 3. 执行搜索
        let _vector_results = db.vector_search_states(embedding, 5).await.unwrap();
        let _doc_results = db.search_documents("工作流", 5).await.unwrap();

        let workflow_duration = workflow_start.elapsed();
        println!("完整工作流: {:?}", workflow_duration);

        assert!(workflow_duration.as_millis() < 3000, "完整工作流应该在3秒内完成");
    }
}
