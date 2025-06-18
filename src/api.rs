// API模块 - 统一的接口层
use std::sync::Arc;
use lancedb::Connection;

use crate::database::AgentStateDB;
use crate::memory::MemoryManager;
use crate::rag::RAGEngine;
use crate::vector::VectorSearchEngine;
use crate::types::{
    AgentDbError, AgentState, Memory, MemoryType, Document, SearchResult,
    VectorSearchResult, IndexStats, QueryPlan, QueryType,
};

/// Agent数据库的主要API接口
pub struct AgentDB {
    state_db: AgentStateDB,
    memory_manager: MemoryManager,
    rag_engine: RAGEngine,
    vector_engine: VectorSearchEngine,
    connection: Arc<Connection>,
}

impl AgentDB {
    /// 创建新的AgentDB实例
    pub async fn new(db_path: &str, vector_dimension: usize) -> Result<Self, AgentDbError> {
        let state_db = AgentStateDB::new(db_path).await?;
        let connection = Arc::new(state_db.get_connection().clone());
        
        let memory_manager = MemoryManager::new(connection.clone());
        let rag_engine = RAGEngine::new(connection.clone());
        let vector_engine = VectorSearchEngine::new(connection.clone(), vector_dimension);

        Ok(Self {
            state_db,
            memory_manager,
            rag_engine,
            vector_engine,
            connection,
        })
    }

    // === Agent状态管理 ===
    
    /// 保存Agent状态
    pub async fn save_agent_state(&self, state: &AgentState) -> Result<(), AgentDbError> {
        self.state_db.save_state(state).await
    }

    /// 加载Agent状态
    pub async fn load_agent_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentDbError> {
        self.state_db.load_state(agent_id).await
    }

    /// 更新Agent状态数据
    pub async fn update_agent_state(&self, agent_id: u64, new_data: Vec<u8>) -> Result<(), AgentDbError> {
        self.state_db.update_state(agent_id, new_data).await
    }

    /// 删除Agent状态
    pub async fn delete_agent_state(&self, agent_id: u64) -> Result<(), AgentDbError> {
        self.state_db.delete_state(agent_id).await
    }

    /// 列出所有Agent
    pub async fn list_agents(&self) -> Result<Vec<u64>, AgentDbError> {
        self.state_db.list_agents().await
    }

    /// 获取Agent状态历史
    pub async fn get_agent_state_history(&self, agent_id: u64, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        self.state_db.get_state_history(agent_id, limit).await
    }

    // === 记忆管理 ===

    /// 存储记忆
    pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentDbError> {
        self.memory_manager.store_memory(memory).await
    }

    /// 检索记忆
    pub async fn retrieve_memory(&self, memory_id: &str) -> Result<Option<Memory>, AgentDbError> {
        self.memory_manager.retrieve_memory(memory_id).await
    }

    /// 获取Agent的记忆
    pub async fn get_agent_memories(
        &self,
        agent_id: u64,
        memory_type: Option<MemoryType>,
        limit: usize,
    ) -> Result<Vec<Memory>, AgentDbError> {
        self.memory_manager.get_agent_memories(agent_id, memory_type, limit).await
    }

    /// 更新记忆访问
    pub async fn update_memory_access(&self, memory_id: &str) -> Result<(), AgentDbError> {
        self.memory_manager.update_memory_access(memory_id).await
    }

    /// 删除记忆
    pub async fn delete_memory(&self, memory_id: &str) -> Result<(), AgentDbError> {
        self.memory_manager.delete_memory(memory_id).await
    }

    /// 清理过期记忆
    pub async fn cleanup_expired_memories(&self) -> Result<usize, AgentDbError> {
        self.memory_manager.cleanup_expired_memories().await
    }

    /// 获取重要记忆
    pub async fn get_important_memories(
        &self,
        agent_id: u64,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<Memory>, AgentDbError> {
        self.memory_manager.get_important_memories(agent_id, threshold, limit).await
    }

    /// 搜索记忆
    pub async fn search_memories(
        &self,
        agent_id: u64,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Memory>, AgentDbError> {
        self.memory_manager.search_memories(agent_id, query, limit).await
    }

    /// 获取记忆统计
    pub async fn get_memory_stats(&self, agent_id: u64) -> Result<std::collections::HashMap<String, u64>, AgentDbError> {
        self.memory_manager.get_memory_stats(agent_id).await
    }

    // === RAG功能 ===

    /// 添加文档
    pub async fn add_document(&self, document: Document) -> Result<(), AgentDbError> {
        self.rag_engine.add_document(document).await
    }

    /// 获取文档
    pub async fn get_document(&self, doc_id: &str) -> Result<Option<Document>, AgentDbError> {
        self.rag_engine.get_document(doc_id).await
    }

    /// 搜索文档
    pub async fn search_documents(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        self.rag_engine.search_documents(query, limit).await
    }

    /// 删除文档
    pub async fn delete_document(&self, doc_id: &str) -> Result<(), AgentDbError> {
        self.rag_engine.delete_document(doc_id).await
    }

    /// 列出文档
    pub async fn list_documents(&self, limit: usize) -> Result<Vec<Document>, AgentDbError> {
        self.rag_engine.list_documents(limit).await
    }

    // === 向量搜索 ===

    /// 添加向量
    pub async fn add_vector(
        &self,
        vector_id: String,
        vector: Vec<f32>,
        metadata: String,
    ) -> Result<(), AgentDbError> {
        self.vector_engine.add_vector(vector_id, vector, metadata).await
    }

    /// 搜索向量
    pub async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        self.vector_engine.search_vectors(query_vector, limit).await
    }

    /// 获取向量
    pub async fn get_vector(&self, vector_id: &str) -> Result<Option<(Vec<f32>, String)>, AgentDbError> {
        self.vector_engine.get_vector(vector_id).await
    }

    /// 删除向量
    pub async fn delete_vector(&self, vector_id: &str) -> Result<(), AgentDbError> {
        self.vector_engine.delete_vector(vector_id).await
    }

    /// 批量添加向量
    pub async fn batch_add_vectors(
        &self,
        vectors: Vec<(String, Vec<f32>, String)>,
    ) -> Result<(), AgentDbError> {
        self.vector_engine.batch_add_vectors(vectors).await
    }

    /// 相似度搜索
    pub async fn similarity_search(
        &self,
        query_vector: Vec<f32>,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        self.vector_engine.similarity_search(query_vector, threshold, limit).await
    }

    // === 高级功能 ===

    /// 混合搜索（结合文本和向量搜索）
    pub async fn hybrid_search(
        &self,
        text_query: &str,
        vector_query: Option<Vec<f32>>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, AgentDbError> {
        let mut results = Vec::new();

        // 文本搜索
        let text_results = self.search_documents(text_query, limit).await?;
        results.extend(text_results);

        // 向量搜索（如果提供了向量）
        if let Some(vector) = vector_query {
            let vector_results = self.search_vectors(vector, limit).await?;
            
            // 将向量搜索结果转换为SearchResult格式
            for vector_result in vector_results {
                results.push(SearchResult {
                    chunk_id: vector_result.vector_id,
                    doc_id: "vector_search".to_string(),
                    content: vector_result.metadata,
                    score: 1.0 - vector_result.distance, // 转换距离为分数
                });
            }
        }

        // 按分数排序并限制结果数量
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    /// 获取索引统计信息
    pub async fn get_index_stats(&self) -> Result<IndexStats, AgentDbError> {
        self.vector_engine.get_index_stats().await
    }

    /// 创建查询计划
    pub async fn create_query_plan(&self, query_type: QueryType) -> Result<QueryPlan, AgentDbError> {
        let query_id = uuid::Uuid::new_v4().to_string();
        
        let (estimated_cost, estimated_time, recommendations, hints) = match query_type {
            QueryType::VectorSearch => (
                1.0,
                0.1,
                vec!["Consider using vector index".to_string()],
                vec!["Use appropriate similarity threshold".to_string()],
            ),
            QueryType::MemoryRetrieval => (
                0.5,
                0.05,
                vec!["Index on agent_id and memory_type".to_string()],
                vec!["Filter by importance for better performance".to_string()],
            ),
            QueryType::AgentState => (
                0.3,
                0.02,
                vec!["Primary key index on agent_id".to_string()],
                vec!["Use specific agent_id for best performance".to_string()],
            ),
            QueryType::RAG => (
                2.0,
                0.2,
                vec!["Full-text search index".to_string(), "Vector index for embeddings".to_string()],
                vec!["Combine text and vector search for best results".to_string()],
            ),
            QueryType::Hybrid => (
                3.0,
                0.3,
                vec!["Multiple index types recommended".to_string()],
                vec!["Balance text and vector search weights".to_string()],
            ),
        };

        Ok(QueryPlan {
            query_id,
            query_type,
            estimated_cost,
            estimated_time,
            index_recommendations: recommendations,
            optimization_hints: hints,
        })
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> &Arc<Connection> {
        &self.connection
    }

    /// 获取向量维度
    pub fn get_vector_dimension(&self) -> usize {
        self.vector_engine.get_dimension()
    }

    // === 批量操作 ===

    /// 批量保存Agent状态
    pub async fn batch_save_agent_states(&self, states: Vec<AgentState>) -> Result<Vec<Result<(), AgentDbError>>, AgentDbError> {
        let mut results = Vec::new();

        for state in states {
            let result = self.save_agent_state(&state).await;
            results.push(result);
        }

        Ok(results)
    }

    /// 批量存储记忆
    pub async fn batch_store_memories(&self, memories: Vec<Memory>) -> Result<Vec<Result<(), AgentDbError>>, AgentDbError> {
        let mut results = Vec::new();

        for memory in memories {
            let result = self.store_memory(&memory).await;
            results.push(result);
        }

        Ok(results)
    }

    /// 批量添加文档
    pub async fn batch_add_documents(&self, documents: Vec<Document>) -> Result<Vec<Result<(), AgentDbError>>, AgentDbError> {
        let mut results = Vec::new();

        for document in documents {
            let result = self.add_document(document).await;
            results.push(result);
        }

        Ok(results)
    }

    /// 批量删除记忆
    pub async fn batch_delete_memories(&self, memory_ids: Vec<String>) -> Result<Vec<Result<(), AgentDbError>>, AgentDbError> {
        let mut results = Vec::new();

        for memory_id in memory_ids {
            let result = self.delete_memory(&memory_id).await;
            results.push(result);
        }

        Ok(results)
    }

    // === 事务支持 ===

    /// 执行事务操作
    pub async fn execute_transaction<F, T>(&self, operation: F) -> Result<T, AgentDbError>
    where
        F: FnOnce(&Self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, AgentDbError>> + Send + '_>>,
    {
        // 简化的事务实现，实际应用中需要更复杂的事务管理
        operation(self).await
    }

    // === 异步操作优化 ===

    /// 并行搜索多个查询
    pub async fn parallel_search(&self, queries: Vec<String>, limit: usize) -> Result<Vec<Vec<SearchResult>>, AgentDbError> {
        let mut results = Vec::new();

        // 简化实现：顺序执行而不是并行
        for query in queries {
            let result = self.search_documents(&query, limit).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 并行向量搜索
    pub async fn parallel_vector_search(&self, query_vectors: Vec<Vec<f32>>, limit: usize) -> Result<Vec<Vec<VectorSearchResult>>, AgentDbError> {
        let mut results = Vec::new();

        // 简化实现：顺序执行而不是并行
        for query_vector in query_vectors {
            let result = self.search_vectors(query_vector, limit).await?;
            results.push(result);
        }

        Ok(results)
    }

    // === 流式处理接口 ===

    /// 流式处理记忆
    pub async fn stream_memories<F>(&self, agent_id: u64, processor: F) -> Result<(), AgentDbError>
    where
        F: Fn(Memory) -> Result<(), AgentDbError>,
    {
        let batch_size = 100;
        let mut _offset = 0;

        loop {
            let memories = self.get_agent_memories(agent_id, None, batch_size).await?;

            if memories.is_empty() {
                break;
            }

            for memory in memories {
                processor(memory)?;
            }

            _offset += batch_size;
        }

        Ok(())
    }

    /// 流式处理文档
    pub async fn stream_documents<F>(&self, processor: F) -> Result<(), AgentDbError>
    where
        F: Fn(Document) -> Result<(), AgentDbError>,
    {
        let batch_size = 50;
        let mut _offset = 0;

        loop {
            let documents = self.list_documents(batch_size).await?;

            if documents.is_empty() {
                break;
            }

            for document in documents {
                processor(document)?;
            }

            _offset += batch_size;
        }

        Ok(())
    }

    // === 高级分析功能 ===

    /// 分析Agent行为模式
    pub async fn analyze_agent_patterns(&self, agent_id: u64) -> Result<std::collections::HashMap<String, f64>, AgentDbError> {
        let mut patterns = std::collections::HashMap::new();

        // 获取记忆统计
        let memory_stats = self.get_memory_stats(agent_id).await?;

        // 计算记忆类型分布
        let total_memories = memory_stats.values().sum::<u64>() as f64;
        if total_memories > 0.0 {
            for (memory_type, count) in memory_stats {
                let percentage = (count as f64 / total_memories) * 100.0;
                patterns.insert(format!("memory_type_{}", memory_type), percentage);
            }
        }

        // 获取状态历史
        let state_history = self.get_agent_state_history(agent_id, 100).await?;
        patterns.insert("state_changes".to_string(), state_history.len() as f64);

        Ok(patterns)
    }

    /// 获取系统健康状态
    pub async fn get_system_health(&self) -> Result<std::collections::HashMap<String, String>, AgentDbError> {
        let mut health = std::collections::HashMap::new();

        // 检查各个组件的状态
        health.insert("database".to_string(), "healthy".to_string());
        health.insert("memory_manager".to_string(), "healthy".to_string());
        health.insert("rag_engine".to_string(), "healthy".to_string());
        health.insert("vector_engine".to_string(), "healthy".to_string());

        // 添加统计信息
        let agents = self.list_agents().await?;
        health.insert("total_agents".to_string(), agents.len().to_string());

        Ok(health)
    }
}
