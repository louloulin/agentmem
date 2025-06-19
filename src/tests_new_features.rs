// 新功能测试模块
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::agent_state::*;
    use crate::memory::*;
    use crate::rag::*;
    use crate::AgentDatabase;
    use std::collections::HashMap;
    use lancedb::connect;

    // 向量状态管理测试
    #[tokio::test]
    async fn test_vector_state_management() {
        let db_path = ":memory:";
        let agent_db = AgentStateDB::new(db_path).await.unwrap();

        // 创建测试状态
        let state = AgentState {
            id: "test_vector_state".to_string(),
            agent_id: 12345,
            session_id: 67890,
            timestamp: chrono::Utc::now().timestamp(),
            state_type: StateType::Embedding,
            data: b"test vector state data".to_vec(),
            metadata: HashMap::new(),
            version: 1,
            checksum: 0,
        };

        // 创建测试向量
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        // 测试保存向量状态
        let result = agent_db.save_vector_state(&state, embedding.clone()).await;
        assert!(result.is_ok(), "向量状态保存应该成功");

        // 测试向量搜索
        let search_result = agent_db.vector_search(embedding.clone(), 10).await;
        assert!(search_result.is_ok(), "向量搜索应该成功");

        let states = search_result.unwrap();
        assert!(!states.is_empty(), "应该找到至少一个状态");

        // 测试基于Agent ID的向量搜索
        let agent_search_result = agent_db.search_by_agent_and_similarity(12345, embedding, 10).await;
        assert!(agent_search_result.is_ok(), "基于Agent ID的向量搜索应该成功");

        let agent_states = agent_search_result.unwrap();
        assert!(!agent_states.is_empty(), "应该找到至少一个Agent状态");
    }

    // RAG引擎测试
    #[tokio::test]
    async fn test_rag_engine() {
        let db_path = ":memory:";
        let rag_engine = RAGEngine::new(db_path).await.unwrap();

        // 创建测试文档
        let mut document = Document::new(
            "测试文档".to_string(),
            "这是一个用于测试RAG引擎功能的示例文档。它包含了多个句子和段落，用于验证文档分块和搜索功能。".to_string(),
        );

        // 设置文档元数据
        document.set_metadata("category".to_string(), "test".to_string());
        document.set_metadata("author".to_string(), "test_user".to_string());

        // 测试文档分块
        let chunk_result = document.chunk_document(50, 10);
        assert!(chunk_result.is_ok(), "文档分块应该成功");
        assert!(!document.chunks.is_empty(), "应该生成至少一个文档块");

        // 测试文档索引
        let index_result = rag_engine.index_document(&document).await;
        assert!(index_result.is_ok(), "文档索引应该成功");

        let doc_id = index_result.unwrap();
        assert_eq!(doc_id, document.doc_id, "返回的文档ID应该匹配");

        // 测试文本搜索
        let text_search_result = rag_engine.search_by_text("测试", 5).await;
        assert!(text_search_result.is_ok(), "文本搜索应该成功");

        let search_results = text_search_result.unwrap();
        assert!(!search_results.is_empty(), "应该找到至少一个搜索结果");

        // 验证搜索结果
        let first_result = &search_results[0];
        assert!(first_result.content.contains("测试"), "搜索结果应该包含查询词");
        assert!(first_result.score > 0.0, "搜索结果应该有正分数");

        // 测试语义搜索
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let semantic_search_result = rag_engine.semantic_search(embedding, 5).await;
        assert!(semantic_search_result.is_ok(), "语义搜索应该成功");

        // 测试上下文构建
        let context_result = rag_engine.build_context("测试查询", search_results, 1000).await;
        assert!(context_result.is_ok(), "上下文构建应该成功");

        let context = context_result.unwrap();
        assert_eq!(context.query, "测试查询", "上下文查询应该匹配");
        assert!(!context.context_window.is_empty(), "上下文窗口不应该为空");
        assert!(!context.retrieved_chunks.is_empty(), "应该有检索到的块");

        // 测试获取文档
        let get_doc_result = rag_engine.get_document(&doc_id).await;
        assert!(get_doc_result.is_ok(), "获取文档应该成功");

        let retrieved_doc = get_doc_result.unwrap();
        assert!(retrieved_doc.is_some(), "应该找到文档");

        let doc = retrieved_doc.unwrap();
        assert_eq!(doc.doc_id, document.doc_id, "文档ID应该匹配");
        assert_eq!(doc.title, document.title, "文档标题应该匹配");
    }

    // 高级记忆管理测试
    #[tokio::test]
    async fn test_advanced_memory_management() {
        let db_path = ":memory:";
        let connection = connect(db_path).execute().await.unwrap();
        let memory_manager = MemoryManager::new(connection);

        // 创建测试记忆
        let memories = vec![
            Memory::new(12345, MemoryType::Episodic, "重要的会议记录".to_string(), 0.9),
            Memory::new(12345, MemoryType::Semantic, "技术知识点".to_string(), 0.7),
            Memory::new(12345, MemoryType::Working, "临时工作笔记".to_string(), 0.3),
        ];

        // 存储记忆
        for memory in &memories {
            let result = memory_manager.store_memory(memory).await;
            assert!(result.is_ok(), "记忆存储应该成功");
        }

        // 测试基于重要性的记忆检索
        let importance_result = memory_manager.get_memories_by_importance(12345, 0.5, 10).await;
        assert!(importance_result.is_ok(), "基于重要性的记忆检索应该成功");

        let important_memories = importance_result.unwrap();
        assert_eq!(important_memories.len(), 2, "应该找到2个重要记忆");

        // 验证按重要性排序
        assert!(important_memories[0].importance >= important_memories[1].importance, "记忆应该按重要性降序排列");

        // 测试向量相似性搜索
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let similarity_result = memory_manager.search_similar_memories(12345, embedding, 5).await;
        assert!(similarity_result.is_ok(), "向量相似性搜索应该成功");

        // 测试记忆统计
        let stats_result = memory_manager.get_memory_statistics(12345).await;
        assert!(stats_result.is_ok(), "记忆统计应该成功");

        let stats = stats_result.unwrap();
        assert_eq!(stats.total_count, 3, "总记忆数应该为3");
        assert!(stats.avg_importance > 0.0, "平均重要性应该大于0");
        assert!(stats.total_size_bytes > 0, "总大小应该大于0");

        // 验证类型统计
        assert!(stats.type_counts.contains_key(&MemoryType::Episodic), "应该包含情节记忆统计");
        assert!(stats.type_counts.contains_key(&MemoryType::Semantic), "应该包含语义记忆统计");
        assert!(stats.type_counts.contains_key(&MemoryType::Working), "应该包含工作记忆统计");
    }

    // 集成数据库测试
    #[tokio::test]
    async fn test_integrated_database() {
        let config = DatabaseConfig::default();
        let mut db = AgentDatabase::new(config).await.unwrap();

        // 添加RAG引擎
        db = db.with_rag_engine().await.unwrap();

        // 测试向量状态操作
        let state = AgentState {
            id: "integrated_test".to_string(),
            agent_id: 99999,
            session_id: 88888,
            timestamp: chrono::Utc::now().timestamp(),
            state_type: StateType::Embedding,
            data: b"integrated test data".to_vec(),
            metadata: HashMap::new(),
            version: 1,
            checksum: 0,
        };

        let embedding = vec![0.5, 0.4, 0.3, 0.2, 0.1];

        // 测试保存向量状态
        let save_result = db.save_vector_state(&state, embedding.clone()).await;
        assert!(save_result.is_ok(), "集成数据库向量状态保存应该成功");

        // 测试向量搜索
        let search_result = db.vector_search_states(embedding, 5).await;
        assert!(search_result.is_ok(), "集成数据库向量搜索应该成功");

        // 测试RAG文档操作
        let mut document = Document::new(
            "集成测试文档".to_string(),
            "这是一个集成测试文档，用于验证完整的RAG功能。".to_string(),
        );
        document.chunk_document(30, 5).unwrap();

        let index_result = db.index_document(&document).await;
        assert!(index_result.is_ok(), "集成数据库文档索引应该成功");

        let search_docs_result = db.search_documents("集成测试", 5).await;
        assert!(search_docs_result.is_ok(), "集成数据库文档搜索应该成功");

        let doc_results = search_docs_result.unwrap();
        assert!(!doc_results.is_empty(), "应该找到文档搜索结果");
    }

    // Memory结构体新方法测试
    #[test]
    fn test_memory_new_methods() {
        let mut memory = Memory::new(12345, MemoryType::Episodic, "测试记忆内容".to_string(), 0.8);

        // 测试重要性计算
        let current_time = chrono::Utc::now().timestamp();
        let calculated_importance = memory.calculate_importance(current_time);
        assert!(calculated_importance > 0.0, "计算的重要性应该大于0");

        // 测试嵌入向量设置
        let embedding = vec![0.1, 0.2, 0.3];
        memory.set_embedding(embedding.clone());
        assert_eq!(memory.get_embedding(), Some(&embedding), "嵌入向量应该正确设置");

        // 测试访问计数
        let initial_access_count = memory.access_count;
        memory.access();
        assert_eq!(memory.access_count, initial_access_count + 1, "访问计数应该增加");

        // 测试过期设置
        memory.set_expiry(3600); // 1小时后过期
        assert!(memory.expiry_time.is_some(), "过期时间应该被设置");
        assert!(!memory.is_expired(), "记忆现在不应该过期");
    }

    // 文档块功能测试
    #[test]
    fn test_document_chunk_functionality() {
        let mut document = Document::new(
            "分块测试文档".to_string(),
            "这是一个很长的文档内容，用于测试文档分块功能。它包含多个句子和段落。每个块应该在合适的位置分割，保持语义的完整性。".to_string(),
        );

        // 测试文档分块
        let result = document.chunk_document(50, 10);
        assert!(result.is_ok(), "文档分块应该成功");
        assert!(!document.chunks.is_empty(), "应该生成文档块");

        // 验证块的属性
        for (i, chunk) in document.chunks.iter().enumerate() {
            assert_eq!(chunk.doc_id, document.doc_id, "块的文档ID应该匹配");
            assert_eq!(chunk.chunk_index, i as u32, "块索引应该正确");
            assert!(!chunk.content.is_empty(), "块内容不应该为空");
            assert!(chunk.get_token_count() > 0, "块应该有token计数");
        }

        // 测试元数据操作
        document.set_metadata("test_key".to_string(), "test_value".to_string());
        assert_eq!(document.get_metadata("test_key"), Some(&"test_value".to_string()), "元数据应该正确设置");

        // 测试嵌入向量设置
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        document.set_embedding(embedding.clone());
        assert_eq!(document.embedding, Some(embedding), "文档嵌入向量应该正确设置");
    }
}
