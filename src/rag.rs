<<<<<<< HEAD
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
=======
// RAG引擎模块 - 检索增强生成
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, BinaryArray, Int64Array, StringArray, UInt32Array, UInt64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{connect, Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::AgentDbError;

// RAG相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_id: String,
    pub title: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, String>,
    pub chunks: Vec<DocumentChunk>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub chunk_id: String,
    pub doc_id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub chunk_index: u32,
    pub start_pos: usize,
    pub end_pos: usize,
    pub overlap_prev: usize,
    pub overlap_next: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub doc_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGContext {
    pub query: String,
    pub retrieved_chunks: Vec<SearchResult>,
    pub context_window: String,
    pub relevance_scores: Vec<f32>,
    pub total_tokens: usize,
}

impl Document {
    /// 创建新文档
    pub fn new(title: String, content: String) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            doc_id: Uuid::new_v4().to_string(),
            title,
            content,
            embedding: None,
            metadata: HashMap::new(),
            chunks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 将文档分块处理
    pub fn chunk_document(&mut self, chunk_size: usize, overlap: usize) -> Result<(), AgentDbError> {
        self.chunks.clear();

        if self.content.is_empty() {
            return Ok(());
        }

        let content_bytes = self.content.as_bytes();
        let total_len = content_bytes.len();
        let mut start_pos = 0;
        let mut chunk_index = 0;

        while start_pos < total_len {
            let end_pos = std::cmp::min(start_pos + chunk_size, total_len);

            // 尝试在单词边界处分割
            let actual_end_pos = if end_pos < total_len {
                self.find_word_boundary(start_pos, end_pos)
            } else {
                end_pos
            };

            let chunk_content = String::from_utf8_lossy(&content_bytes[start_pos..actual_end_pos]).to_string();

            let chunk = DocumentChunk {
                chunk_id: Uuid::new_v4().to_string(),
                doc_id: self.doc_id.clone(),
                content: chunk_content,
                embedding: None,
                chunk_index,
                start_pos,
                end_pos: actual_end_pos,
                overlap_prev: if chunk_index > 0 { overlap } else { 0 },
                overlap_next: if actual_end_pos < total_len { overlap } else { 0 },
            };

            self.chunks.push(chunk);

            // 计算下一个块的起始位置，考虑重叠
            start_pos = if actual_end_pos < total_len {
                std::cmp::max(actual_end_pos - overlap, start_pos + 1)
            } else {
                actual_end_pos
            };

            chunk_index += 1;
        }

        Ok(())
    }

    /// 查找单词边界
    fn find_word_boundary(&self, start: usize, end: usize) -> usize {
        let content_bytes = self.content.as_bytes();

        // 从end位置向前查找空格或标点符号
        for i in (start..end).rev() {
            if i < content_bytes.len() {
                let ch = content_bytes[i] as char;
                if ch.is_whitespace() || ch.is_ascii_punctuation() {
                    return i + 1;
                }
            }
        }

        // 如果找不到合适的边界，返回原始end位置
        end
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 设置文档嵌入向量
    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

impl DocumentChunk {
    /// 设置块的嵌入向量
    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
    }

    /// 获取token数量估算
    pub fn get_token_count(&self) -> usize {
        // 简单的token计数估算（实际应用中可能需要更精确的tokenizer）
        self.content.split_whitespace().count()
    }
}

// RAG引擎主结构
pub struct RAGEngine {
    connection: Connection,
}

impl RAGEngine {
    /// 创建新的RAG引擎实例
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;
        Ok(Self { connection })
    }

    /// 确保文档表存在
    pub async fn ensure_document_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有文档表
        match self.connection.open_table("documents").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新的文档表
>>>>>>> origin/feature-module
                let schema = Schema::new(vec![
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("title", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("created_at", DataType::Int64, false),
                    Field::new("updated_at", DataType::Int64, false),
<<<<<<< HEAD
=======
                    Field::new("embedding", DataType::Binary, true),
>>>>>>> origin/feature-module
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

<<<<<<< HEAD
    pub async fn ensure_chunks_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("chunks").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
=======
    /// 确保文档块表存在
    pub async fn ensure_chunk_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有块表
        match self.connection.open_table("chunks").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新的块表
>>>>>>> origin/feature-module
                let schema = Schema::new(vec![
                    Field::new("chunk_id", DataType::Utf8, false),
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("chunk_index", DataType::UInt32, false),
<<<<<<< HEAD
                    Field::new("position", DataType::UInt32, false),
                    Field::new("size", DataType::UInt32, false),
=======
                    Field::new("start_pos", DataType::UInt64, false),
                    Field::new("end_pos", DataType::UInt64, false),
                    Field::new("overlap_prev", DataType::UInt64, false),
                    Field::new("overlap_next", DataType::UInt64, false),
                    Field::new("embedding", DataType::Binary, true),
>>>>>>> origin/feature-module
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

<<<<<<< HEAD
    pub async fn add_document(&self, mut document: Document) -> Result<(), AgentDbError> {
        // 首先对文档进行分块
        document.chunk_document(1000, 100)?;

        // 存储文档
        let docs_table = self.ensure_documents_table().await?;
        let docs_schema = docs_table.schema().await?;

        let metadata_json = serde_json::to_string(&document.metadata)?;

        let doc_batch = RecordBatch::try_new(
            docs_schema.clone(),
=======
    /// 索引文档到数据库
    pub async fn index_document(&self, document: &Document) -> Result<String, AgentDbError> {
        // 1. 存储文档元数据
        let doc_table = self.ensure_document_table().await?;
        let doc_schema = doc_table.schema().await?;

        let metadata_json = serde_json::to_string(&document.metadata)?;
        let embedding_data = if let Some(ref emb) = document.embedding {
            Some(serde_json::to_vec(emb).unwrap())
        } else {
            None
        };

        let doc_batch = RecordBatch::try_new(
            doc_schema.clone(),
>>>>>>> origin/feature-module
            vec![
                Arc::new(StringArray::from(vec![document.doc_id.clone()])),
                Arc::new(StringArray::from(vec![document.title.clone()])),
                Arc::new(StringArray::from(vec![document.content.clone()])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(Int64Array::from(vec![document.created_at])),
                Arc::new(Int64Array::from(vec![document.updated_at])),
<<<<<<< HEAD
=======
                if let Some(emb_data) = embedding_data {
                    Arc::new(BinaryArray::from(vec![Some(emb_data.as_slice())]))
                } else {
                    Arc::new(BinaryArray::from(vec![None::<&[u8]>]))
                },
>>>>>>> origin/feature-module
            ],
        )?;

        let doc_batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(doc_batch)),
<<<<<<< HEAD
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
=======
            doc_schema,
        );
        doc_table.add(Box::new(doc_batch_iter)).execute().await?;

        // 2. 存储文档块
        if !document.chunks.is_empty() {
            let chunk_table = self.ensure_chunk_table().await?;
            let chunk_schema = chunk_table.schema().await?;

            for chunk in &document.chunks {
                let chunk_embedding_data = if let Some(ref emb) = chunk.embedding {
                    Some(serde_json::to_vec(emb).unwrap())
                } else {
                    None
                };

                let chunk_batch = RecordBatch::try_new(
                    chunk_schema.clone(),
                    vec![
                        Arc::new(StringArray::from(vec![chunk.chunk_id.clone()])),
                        Arc::new(StringArray::from(vec![chunk.doc_id.clone()])),
                        Arc::new(StringArray::from(vec![chunk.content.clone()])),
                        Arc::new(UInt32Array::from(vec![chunk.chunk_index])),
                        Arc::new(UInt64Array::from(vec![chunk.start_pos as u64])),
                        Arc::new(UInt64Array::from(vec![chunk.end_pos as u64])),
                        Arc::new(UInt64Array::from(vec![chunk.overlap_prev as u64])),
                        Arc::new(UInt64Array::from(vec![chunk.overlap_next as u64])),
                        if let Some(emb_data) = chunk_embedding_data {
                            Arc::new(BinaryArray::from(vec![Some(emb_data.as_slice())]))
                        } else {
                            Arc::new(BinaryArray::from(vec![None::<&[u8]>]))
                        },
                    ],
                )?;

                let chunk_batch_iter = RecordBatchIterator::new(
                    std::iter::once(Ok(chunk_batch)),
                    chunk_schema.clone(),
                );
                chunk_table.add(Box::new(chunk_batch_iter)).execute().await?;
            }
        }

        Ok(document.doc_id.clone())
    }

    /// 语义搜索 - 基于向量相似性
    pub async fn semantic_search(&self, _query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        let chunk_table = self.ensure_chunk_table().await?;

        // 简化的搜索实现（实际应用中需要真正的向量相似性搜索）
        let mut results = chunk_table
            .query()
>>>>>>> origin/feature-module
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

<<<<<<< HEAD
                // 计算简单的相关性分数（基于查询词出现次数）
                let score = self.calculate_relevance_score(&content, query);
=======
                // 简化的相似性评分（实际应用中需要计算真正的余弦相似度）
                let score = 0.8 - (row as f32 * 0.1);
>>>>>>> origin/feature-module

                search_results.push(SearchResult {
                    chunk_id,
                    doc_id,
                    content,
                    score,
<<<<<<< HEAD
=======
                    metadata: HashMap::new(),
>>>>>>> origin/feature-module
                });
            }
        }

<<<<<<< HEAD
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
=======
        Ok(search_results)
    }

    /// 文本搜索 - 基于关键词匹配
    pub async fn search_by_text(&self, text_query: &str, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        let chunk_table = self.ensure_chunk_table().await?;

        // 简化的文本搜索（实际应用中需要全文搜索引擎）
        let mut results = chunk_table
            .query()
            .limit(limit * 2) // 获取更多结果用于过滤
            .execute()
            .await?;

        let mut search_results = Vec::new();
        let query_lower = text_query.to_lowercase();

        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let chunk_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let doc_id_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();

                let chunk_id = chunk_id_array.value(row).to_string();
                let doc_id = doc_id_array.value(row).to_string();
                let content = content_array.value(row).to_string();
                let content_lower = content.to_lowercase();

                // 简单的文本匹配评分
                if content_lower.contains(&query_lower) {
                    let score = self.calculate_text_similarity(&query_lower, &content_lower);

                    search_results.push(SearchResult {
                        chunk_id,
                        doc_id,
                        content,
                        score,
                        metadata: HashMap::new(),
                    });

                    if search_results.len() >= limit {
                        break;
                    }
                }
            }

            if search_results.len() >= limit {
                break;
            }
        }

        // 按分数排序
        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        search_results.truncate(limit);

        Ok(search_results)
    }

    /// 混合搜索 - 结合文本和向量搜索
    pub async fn hybrid_search(&self, text_query: &str, query_embedding: Vec<f32>, alpha: f32, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        // 1. 获取文本搜索结果
        let text_results = self.search_by_text(text_query, limit * 2).await?;

        // 2. 获取向量搜索结果
        let vector_results = self.semantic_search(query_embedding, limit * 2).await?;
>>>>>>> origin/feature-module

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
<<<<<<< HEAD
                existing.score = *text_score * alpha + result.score * (1.0 - alpha);
            } else {
                // 如果不存在，添加新结果
                let mut new_result = result;
                new_result.score = new_result.score * (1.0 - alpha);
                combined_results.insert(key, (new_result, 0.0, 1.0 - alpha));
=======
                *existing = SearchResult {
                    chunk_id: existing.chunk_id.clone(),
                    doc_id: existing.doc_id.clone(),
                    content: existing.content.clone(),
                    score: *text_score * alpha + result.score * (1.0 - alpha),
                    metadata: existing.metadata.clone(),
                };
            } else {
                // 如果不存在，添加新结果
                combined_results.insert(key, (result, 0.0, 1.0 - alpha));
>>>>>>> origin/feature-module
            }
        }

        // 4. 收集并排序结果
        let mut final_results: Vec<SearchResult> = combined_results
            .into_iter()
<<<<<<< HEAD
            .map(|(_, (result, _, _))| result)
=======
            .map(|(_, (mut result, text_weight, vector_weight))| {
                result.score = result.score * text_weight + result.score * vector_weight;
                result
            })
>>>>>>> origin/feature-module
            .collect();

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(limit);

        Ok(final_results)
    }

<<<<<<< HEAD
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
=======
    /// 构建RAG上下文
    pub async fn build_context(&self, query: &str, search_results: Vec<SearchResult>, max_tokens: usize) -> Result<RAGContext, AgentDbError> {
        let mut context_chunks = Vec::new();
        let mut total_tokens = 0;
        let mut relevance_scores = Vec::new();

        for result in search_results {
            let chunk_tokens = result.content.split_whitespace().count();

            if total_tokens + chunk_tokens <= max_tokens {
                total_tokens += chunk_tokens;
                relevance_scores.push(result.score);
                context_chunks.push(result);
            } else {
                break;
            }
        }

        // 构建上下文窗口
        let context_window = context_chunks
            .iter()
            .map(|chunk| format!("Document: {}\nContent: {}\n", chunk.doc_id, chunk.content))
            .collect::<Vec<_>>()
            .join("\n---\n");

        Ok(RAGContext {
            query: query.to_string(),
            retrieved_chunks: context_chunks,
            context_window,
            relevance_scores,
            total_tokens,
        })
    }

    /// 计算文本相似性
    fn calculate_text_similarity(&self, query: &str, content: &str) -> f32 {
        let query_words: std::collections::HashSet<&str> = query.split_whitespace().collect();
        let content_words: std::collections::HashSet<&str> = content.split_whitespace().collect();

        let intersection = query_words.intersection(&content_words).count();
        let union = query_words.union(&content_words).count();

        if union == 0 {
            0.1 // 基础分数，避免0分
        } else {
            let jaccard_similarity = intersection as f32 / union as f32;
            // 确保至少有基础分数，如果包含查询词则给予额外加分
            let base_score = 0.1;
            let contains_bonus = if content.to_lowercase().contains(&query.to_lowercase()) { 0.5 } else { 0.0 };
            (base_score + jaccard_similarity + contains_bonus).min(1.0)
        }
    }

    /// 获取文档
    pub async fn get_document(&self, doc_id: &str) -> Result<Option<Document>, AgentDbError> {
        let doc_table = self.ensure_document_table().await?;

        let mut results = doc_table
            .query()
            .only_if(&format!("doc_id = '{}'", doc_id))
            .limit(1)
            .execute()
            .await?;

        if let Some(batch) = results.try_next().await? {
            if batch.num_rows() > 0 {
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

                return Ok(Some(Document {
                    doc_id,
                    title,
                    content,
                    embedding: None, // 简化处理
                    metadata,
                    chunks,
                    created_at,
                    updated_at,
                }));
            }
        }

        Ok(None)
    }

    /// 获取文档的所有块
    pub async fn get_document_chunks(&self, doc_id: &str) -> Result<Vec<DocumentChunk>, AgentDbError> {
        let chunk_table = self.ensure_chunk_table().await?;

        let mut results = chunk_table
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
                let start_pos_array = batch.column(4).as_any().downcast_ref::<UInt64Array>().unwrap();
                let end_pos_array = batch.column(5).as_any().downcast_ref::<UInt64Array>().unwrap();
                let overlap_prev_array = batch.column(6).as_any().downcast_ref::<UInt64Array>().unwrap();
                let overlap_next_array = batch.column(7).as_any().downcast_ref::<UInt64Array>().unwrap();

                let chunk = DocumentChunk {
                    chunk_id: chunk_id_array.value(row).to_string(),
                    doc_id: doc_id_array.value(row).to_string(),
                    content: content_array.value(row).to_string(),
                    embedding: None, // 简化处理
                    chunk_index: chunk_index_array.value(row),
                    start_pos: start_pos_array.value(row) as usize,
                    end_pos: end_pos_array.value(row) as usize,
                    overlap_prev: overlap_prev_array.value(row) as usize,
                    overlap_next: overlap_next_array.value(row) as usize,
                };

                chunks.push(chunk);
            }
        }

        // 按chunk_index排序
        chunks.sort_by_key(|chunk| chunk.chunk_index);

        Ok(chunks)
>>>>>>> origin/feature-module
    }
}
