// RAG (检索增强生成) 模块
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, StringArray, UInt32Array, Int64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::types::{AgentDbError, Document, Chunk, SearchResult};

pub struct RAGEngine {
    connection: Arc<Connection>,
}

impl RAGEngine {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self { connection }
    }

    pub async fn ensure_documents_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("documents").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                let schema = Schema::new(vec![
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("title", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("created_at", DataType::Int64, false),
                    Field::new("updated_at", DataType::Int64, false),
                ]);

                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("documents", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    pub async fn ensure_chunks_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("chunks").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                let schema = Schema::new(vec![
                    Field::new("chunk_id", DataType::Utf8, false),
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("chunk_index", DataType::UInt32, false),
                    Field::new("position", DataType::UInt32, false),
                    Field::new("size", DataType::UInt32, false),
                ]);

                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("chunks", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    pub async fn add_document(&self, mut document: Document) -> Result<(), AgentDbError> {
        // 首先对文档进行分块
        document.chunk_document(1000, 100)?;

        // 存储文档
        let docs_table = self.ensure_documents_table().await?;
        let docs_schema = docs_table.schema().await?;

        let metadata_json = serde_json::to_string(&document.metadata)?;

        let doc_batch = RecordBatch::try_new(
            docs_schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![document.doc_id.clone()])),
                Arc::new(StringArray::from(vec![document.title.clone()])),
                Arc::new(StringArray::from(vec![document.content.clone()])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(Int64Array::from(vec![document.created_at])),
                Arc::new(Int64Array::from(vec![document.updated_at])),
            ],
        )?;

        let doc_batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(doc_batch)),
            docs_schema,
        );
        docs_table.add(Box::new(doc_batch_iter)).execute().await?;

        // 存储文档块
        if !document.chunks.is_empty() {
            self.add_chunks(&document.chunks).await?;
        }

        Ok(())
    }

    pub async fn add_chunks(&self, chunks: &[Chunk]) -> Result<(), AgentDbError> {
        let chunks_table = self.ensure_chunks_table().await?;
        let chunks_schema = chunks_table.schema().await?;

        let chunk_ids: Vec<String> = chunks.iter().map(|c| c.chunk_id.clone()).collect();
        let doc_ids: Vec<String> = chunks.iter().map(|c| c.doc_id.clone()).collect();
        let contents: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let chunk_indices: Vec<u32> = chunks.iter().map(|c| c.chunk_index).collect();
        let positions: Vec<u32> = chunks.iter().map(|c| c.position as u32).collect();
        let sizes: Vec<u32> = chunks.iter().map(|c| c.size as u32).collect();

        let chunks_batch = RecordBatch::try_new(
            chunks_schema.clone(),
            vec![
                Arc::new(StringArray::from(chunk_ids)),
                Arc::new(StringArray::from(doc_ids)),
                Arc::new(StringArray::from(contents)),
                Arc::new(UInt32Array::from(chunk_indices)),
                Arc::new(UInt32Array::from(positions)),
                Arc::new(UInt32Array::from(sizes)),
            ],
        )?;

        let chunks_batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(chunks_batch)),
            chunks_schema,
        );
        chunks_table.add(Box::new(chunks_batch_iter)).execute().await?;

        Ok(())
    }

    pub async fn get_document(&self, doc_id: &str) -> Result<Option<Document>, AgentDbError> {
        let table = self.ensure_documents_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("doc_id = '{}'", doc_id))
            .limit(1)
            .execute()
            .await?;

        let batch = match results.try_next().await? {
            Some(batch) => batch,
            None => return Ok(None),
        };

        if batch.num_rows() == 0 {
            return Ok(None);
        }

        let doc_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
        let title_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
        let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
        let metadata_array = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
        let created_at_array = batch.column(4).as_any().downcast_ref::<Int64Array>().unwrap();
        let updated_at_array = batch.column(5).as_any().downcast_ref::<Int64Array>().unwrap();

        let doc_id = doc_id_array.value(0).to_string();
        let title = title_array.value(0).to_string();
        let content = content_array.value(0).to_string();
        let metadata_json = metadata_array.value(0);
        let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;
        let created_at = created_at_array.value(0);
        let updated_at = updated_at_array.value(0);

        // 获取文档的块
        let chunks = self.get_document_chunks(&doc_id).await?;

        Ok(Some(Document {
            doc_id,
            title,
            content,
            metadata,
            chunks,
            created_at,
            updated_at,
        }))
    }

    pub async fn get_document_chunks(&self, doc_id: &str) -> Result<Vec<Chunk>, AgentDbError> {
        let table = self.ensure_chunks_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("doc_id = '{}'", doc_id))
            .execute()
            .await?;

        let mut chunks = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let chunk_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let doc_id_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
                let chunk_index_array = batch.column(3).as_any().downcast_ref::<UInt32Array>().unwrap();
                let position_array = batch.column(4).as_any().downcast_ref::<UInt32Array>().unwrap();
                let size_array = batch.column(5).as_any().downcast_ref::<UInt32Array>().unwrap();

                let chunk = Chunk {
                    chunk_id: chunk_id_array.value(row).to_string(),
                    doc_id: doc_id_array.value(row).to_string(),
                    content: content_array.value(row).to_string(),
                    chunk_index: chunk_index_array.value(row),
                    position: position_array.value(row) as usize,
                    size: size_array.value(row) as usize,
                };

                chunks.push(chunk);
            }
        }

        // 按chunk_index排序
        chunks.sort_by_key(|c| c.chunk_index);
        Ok(chunks)
    }

    pub async fn search_documents(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        let table = self.ensure_chunks_table().await?;

        // 简单的文本搜索
        let filter = format!(
            "content LIKE '%{}%'",
            query.replace("'", "''") // 转义单引号
        );

        let mut results = table
            .query()
            .only_if(&filter)
            .limit(limit)
            .execute()
            .await?;

        let mut search_results = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let chunk_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let doc_id_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();

                let chunk_id = chunk_id_array.value(row).to_string();
                let doc_id = doc_id_array.value(row).to_string();
                let content = content_array.value(row).to_string();

                // 计算简单的相关性分数（基于查询词出现次数）
                let score = self.calculate_relevance_score(&content, query);

                search_results.push(SearchResult {
                    chunk_id,
                    doc_id,
                    content,
                    score,
                });
            }
        }

        // 按分数排序
        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(search_results)
    }

    pub async fn delete_document(&self, doc_id: &str) -> Result<(), AgentDbError> {
        // 删除文档块
        let chunks_table = self.ensure_chunks_table().await?;
        chunks_table.delete(&format!("doc_id = '{}'", doc_id)).await?;

        // 删除文档
        let docs_table = self.ensure_documents_table().await?;
        docs_table.delete(&format!("doc_id = '{}'", doc_id)).await?;

        Ok(())
    }

    pub async fn list_documents(&self, limit: usize) -> Result<Vec<Document>, AgentDbError> {
        let table = self.ensure_documents_table().await?;

        let mut results = table
            .query()
            .limit(limit)
            .execute()
            .await?;

        let mut documents = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let doc_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let title_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
                let metadata_array = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
                let created_at_array = batch.column(4).as_any().downcast_ref::<Int64Array>().unwrap();
                let updated_at_array = batch.column(5).as_any().downcast_ref::<Int64Array>().unwrap();

                let doc_id = doc_id_array.value(row).to_string();
                let title = title_array.value(row).to_string();
                let content = content_array.value(row).to_string();
                let metadata_json = metadata_array.value(row);
                let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;
                let created_at = created_at_array.value(row);
                let updated_at = updated_at_array.value(row);

                // 获取文档的块（可选，根据需要）
                let chunks = Vec::new(); // 为了性能，这里不加载块

                documents.push(Document {
                    doc_id,
                    title,
                    content,
                    metadata,
                    chunks,
                    created_at,
                    updated_at,
                });
            }
        }

        Ok(documents)
    }

    fn calculate_relevance_score(&self, content: &str, query: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut score = 0.0;
        for word in query_words {
            let count = content_lower.matches(word).count() as f32;
            score += count;
        }

        // 归一化分数
        score / content.len() as f32 * 1000.0
    }

    // 语义搜索（需要向量引擎支持）
    pub async fn semantic_search(&self, _query_embedding: Vec<f32>, _limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        // 这里需要与向量引擎集成
        // 暂时返回空结果，实际实现需要向量搜索
        Ok(Vec::new())
    }

    // 混合搜索：结合文本搜索和语义搜索
    pub async fn hybrid_search(
        &self,
        text_query: &str,
        query_embedding: Option<Vec<f32>>,
        alpha: f32, // 文本搜索权重
        limit: usize,
    ) -> Result<Vec<SearchResult>, AgentDbError> {
        // 1. 获取文本搜索结果
        let text_results = self.search_documents(text_query, limit * 2).await?;

        // 2. 如果有向量查询，获取语义搜索结果
        let vector_results = if let Some(embedding) = query_embedding {
            self.semantic_search(embedding, limit * 2).await?
        } else {
            Vec::new()
        };

        // 3. 合并和重新评分
        let mut combined_results = HashMap::new();

        // 添加文本搜索结果
        for result in text_results {
            let key = result.chunk_id.clone();
            combined_results.insert(key, (result, alpha, 0.0));
        }

        // 添加向量搜索结果
        for result in vector_results {
            let key = result.chunk_id.clone();
            if let Some((existing, text_score, _)) = combined_results.get_mut(&key) {
                // 如果已存在，更新向量分数
                existing.score = *text_score * alpha + result.score * (1.0 - alpha);
            } else {
                // 如果不存在，添加新结果
                let mut new_result = result;
                new_result.score = new_result.score * (1.0 - alpha);
                combined_results.insert(key, (new_result, 0.0, 1.0 - alpha));
            }
        }

        // 4. 收集并排序结果
        let mut final_results: Vec<SearchResult> = combined_results
            .into_iter()
            .map(|(_, (result, _, _))| result)
            .collect();

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(limit);

        Ok(final_results)
    }

    // 上下文窗口管理
    pub async fn build_context_window(
        &self,
        search_results: &[SearchResult],
        max_tokens: usize,
    ) -> Result<String, AgentDbError> {
        let mut context = String::new();
        let mut token_count = 0;

        for result in search_results {
            // 简单的token计数（实际应用中需要更精确的tokenizer）
            let chunk_tokens = result.content.split_whitespace().count();

            if token_count + chunk_tokens > max_tokens {
                break;
            }

            if !context.is_empty() {
                context.push_str("\n\n");
            }
            context.push_str(&result.content);
            token_count += chunk_tokens;
        }

        Ok(context)
    }

    // 相关性重排序
    pub fn rerank_results(&self, results: &mut [SearchResult], query: &str) {
        // 基于查询的相关性重新计算分数
        for result in results.iter_mut() {
            let relevance = self.calculate_advanced_relevance(&result.content, query);
            result.score = result.score * 0.7 + relevance * 0.3; // 混合原始分数和相关性分数
        }

        // 重新排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    }

    // 高级相关性计算
    fn calculate_advanced_relevance(&self, content: &str, query: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut score = 0.0;
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        // 1. 精确匹配分数
        for word in &query_words {
            let exact_matches = content_words.iter().filter(|&&w| w == *word).count() as f32;
            score += exact_matches * 2.0;
        }

        // 2. 部分匹配分数
        for word in &query_words {
            let partial_matches = content_words.iter()
                .filter(|&&w| w.contains(word) && w != *word)
                .count() as f32;
            score += partial_matches * 1.0;
        }

        // 3. 位置权重（查询词在开头的权重更高）
        let content_start = content_lower.chars().take(100).collect::<String>();
        for word in &query_words {
            if content_start.contains(word) {
                score += 1.5;
            }
        }

        // 4. 查询词密度
        let query_word_count = query_words.len() as f32;
        let content_word_count = content_words.len() as f32;
        let density = query_word_count / content_word_count.max(1.0);
        score += density * 10.0;

        score
    }

    // 获取文档统计信息
    pub async fn get_document_stats(&self) -> Result<HashMap<String, usize>, AgentDbError> {
        let docs_table = self.ensure_documents_table().await?;
        let chunks_table = self.ensure_chunks_table().await?;

        // 计算文档数量
        let mut doc_results = docs_table.query().execute().await?;
        let mut doc_count = 0;
        while let Some(batch) = doc_results.try_next().await? {
            doc_count += batch.num_rows();
        }

        // 计算块数量
        let mut chunk_results = chunks_table.query().execute().await?;
        let mut chunk_count = 0;
        while let Some(batch) = chunk_results.try_next().await? {
            chunk_count += batch.num_rows();
        }

        let mut stats = HashMap::new();
        stats.insert("documents".to_string(), doc_count);
        stats.insert("chunks".to_string(), chunk_count);

        Ok(stats)
    }
}
