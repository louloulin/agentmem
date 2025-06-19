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
                let schema = Schema::new(vec![
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("title", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("created_at", DataType::Int64, false),
                    Field::new("updated_at", DataType::Int64, false),
                    Field::new("embedding", DataType::Binary, true),
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

    /// 确保文档块表存在
    pub async fn ensure_chunk_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有块表
        match self.connection.open_table("chunks").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新的块表
                let schema = Schema::new(vec![
                    Field::new("chunk_id", DataType::Utf8, false),
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("chunk_index", DataType::UInt32, false),
                    Field::new("start_pos", DataType::UInt64, false),
                    Field::new("end_pos", DataType::UInt64, false),
                    Field::new("overlap_prev", DataType::UInt64, false),
                    Field::new("overlap_next", DataType::UInt64, false),
                    Field::new("embedding", DataType::Binary, true),
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
            vec![
                Arc::new(StringArray::from(vec![document.doc_id.clone()])),
                Arc::new(StringArray::from(vec![document.title.clone()])),
                Arc::new(StringArray::from(vec![document.content.clone()])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(Int64Array::from(vec![document.created_at])),
                Arc::new(Int64Array::from(vec![document.updated_at])),
                if let Some(emb_data) = embedding_data {
                    Arc::new(BinaryArray::from(vec![Some(emb_data.as_slice())]))
                } else {
                    Arc::new(BinaryArray::from(vec![None::<&[u8]>]))
                },
            ],
        )?;

        let doc_batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(doc_batch)),
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

                // 简化的相似性评分（实际应用中需要计算真正的余弦相似度）
                let score = 0.8 - (row as f32 * 0.1);

                search_results.push(SearchResult {
                    chunk_id,
                    doc_id,
                    content,
                    score,
                    metadata: HashMap::new(),
                });
            }
        }

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
            }
        }

        // 4. 收集并排序结果
        let mut final_results: Vec<SearchResult> = combined_results
            .into_iter()
            .map(|(_, (mut result, text_weight, vector_weight))| {
                result.score = result.score * text_weight + result.score * vector_weight;
                result
            })
            .collect();

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(limit);

        Ok(final_results)
    }

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
            0.0
        } else {
            intersection as f32 / union as f32
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
    }
}
