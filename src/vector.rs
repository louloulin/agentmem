// 向量处理和搜索模块
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, BinaryArray, Float32Array, StringArray, UInt64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};
use serde::{Deserialize, Serialize};

use crate::core::AgentDbError;

// 向量相似度算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityAlgorithm {
    Cosine,
    Euclidean,
    DotProduct,
    Manhattan,
}

impl SimilarityAlgorithm {
    pub fn calculate(&self, a: &[f32], b: &[f32]) -> f32 {
        match self {
            SimilarityAlgorithm::Cosine => cosine_similarity(a, b),
            SimilarityAlgorithm::Euclidean => euclidean_distance(a, b),
            SimilarityAlgorithm::DotProduct => dot_product(a, b),
            SimilarityAlgorithm::Manhattan => manhattan_distance(a, b),
        }
    }
}

// 向量搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: u64,
    pub vector: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub similarity: f32,
    pub distance: f32,
}

// 向量索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    pub dimension: usize,
    pub algorithm: SimilarityAlgorithm,
    pub index_type: String,
    pub parameters: HashMap<String, String>,
}

impl Default for VectorIndexConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            algorithm: SimilarityAlgorithm::Cosine,
            index_type: "HNSW".to_string(),
            parameters: HashMap::new(),
        }
    }
}

// 高级向量引擎
pub struct AdvancedVectorEngine {
    connection: Connection,
    config: VectorIndexConfig,
}

impl AdvancedVectorEngine {
    pub fn new(connection: Connection, config: VectorIndexConfig) -> Self {
        Self { connection, config }
    }

    pub async fn ensure_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("vectors").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                let schema = Schema::new(vec![
                    Field::new("id", DataType::UInt64, false),
                    Field::new("vector", DataType::Binary, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("timestamp", DataType::Int64, false),
                ]);

                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("vectors", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    pub async fn add_vector(&self, id: u64, vector: Vec<f32>, metadata: HashMap<String, String>) -> Result<(), AgentDbError> {
        if vector.len() != self.config.dimension {
            return Err(AgentDbError::InvalidArgument(
                format!("Vector dimension {} doesn't match expected {}", vector.len(), self.config.dimension)
            ));
        }

        let table = self.ensure_table().await?;
        let metadata_json = serde_json::to_string(&metadata)?;
        let vector_bytes = serde_json::to_vec(&vector)?;
        let timestamp = chrono::Utc::now().timestamp();

        let schema = table.schema().await?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt64Array::from(vec![id])),
                Arc::new(BinaryArray::from(vec![vector_bytes.as_slice()])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(arrow::array::Int64Array::from(vec![timestamp])),
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    pub async fn batch_add_vectors(&self, vectors: Vec<(u64, Vec<f32>, HashMap<String, String>)>) -> Result<(), AgentDbError> {
        if vectors.is_empty() {
            return Ok(());
        }

        let table = self.ensure_table().await?;
        let schema = table.schema().await?;
        let timestamp = chrono::Utc::now().timestamp();

        let mut ids = Vec::new();
        let mut vector_bytes_vec = Vec::new();
        let mut metadata_jsons = Vec::new();
        let mut timestamps = Vec::new();

        for (id, vector, metadata) in vectors {
            if vector.len() != self.config.dimension {
                return Err(AgentDbError::InvalidArgument(
                    format!("Vector dimension {} doesn't match expected {}", vector.len(), self.config.dimension)
                ));
            }

            ids.push(id);
            vector_bytes_vec.push(serde_json::to_vec(&vector)?);
            metadata_jsons.push(serde_json::to_string(&metadata)?);
            timestamps.push(timestamp);
        }

        let vector_bytes_refs: Vec<&[u8]> = vector_bytes_vec.iter().map(|v| v.as_slice()).collect();

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt64Array::from(ids)),
                Arc::new(BinaryArray::from(vector_bytes_refs)),
                Arc::new(StringArray::from(metadata_jsons)),
                Arc::new(arrow::array::Int64Array::from(timestamps)),
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    pub async fn search_vectors(&self, query_vector: &[f32], limit: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        if query_vector.len() != self.config.dimension {
            return Err(AgentDbError::InvalidArgument(
                format!("Query vector dimension {} doesn't match expected {}", query_vector.len(), self.config.dimension)
            ));
        }

        let table = self.ensure_table().await?;
        let mut results = table.query().limit(limit * 10).execute().await?; // 获取更多结果用于排序

        let mut candidates = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let result = self.extract_vector_from_batch(&batch, row)?;
                let similarity = self.config.algorithm.calculate(query_vector, &result.vector);
                
                candidates.push(VectorSearchResult {
                    id: result.id,
                    vector: result.vector,
                    metadata: result.metadata,
                    similarity,
                    distance: 1.0 - similarity,
                });
            }
        }

        // 按相似度排序并返回前limit个结果
        candidates.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        candidates.truncate(limit);

        Ok(candidates)
    }

    pub async fn get_vector(&self, id: u64) -> Result<Option<VectorSearchResult>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("id = {}", id))
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

        let result = self.extract_vector_from_batch(&batch, 0)?;
        Ok(Some(VectorSearchResult {
            id: result.id,
            vector: result.vector,
            metadata: result.metadata,
            similarity: 1.0,
            distance: 0.0,
        }))
    }

    pub async fn update_vector(&self, id: u64, vector: Vec<f32>, metadata: HashMap<String, String>) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;
        
        // 删除旧向量
        table.delete(&format!("id = {}", id)).await?;
        
        // 添加新向量
        self.add_vector(id, vector, metadata).await?;
        
        Ok(())
    }

    pub async fn delete_vector(&self, id: u64) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;
        table.delete(&format!("id = {}", id)).await?;
        Ok(())
    }

    fn extract_vector_from_batch(&self, batch: &RecordBatch, row: usize) -> Result<VectorSearchResult, AgentDbError> {
        let id_array = batch.column(0).as_any().downcast_ref::<UInt64Array>().unwrap();
        let vector_array = batch.column(1).as_any().downcast_ref::<BinaryArray>().unwrap();
        let metadata_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();

        let id = id_array.value(row);
        let vector_bytes = vector_array.value(row);
        let vector: Vec<f32> = serde_json::from_slice(vector_bytes)?;
        let metadata_json = metadata_array.value(row);
        let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;

        Ok(VectorSearchResult {
            id,
            vector,
            metadata,
            similarity: 0.0,
            distance: 0.0,
        })
    }

    pub async fn get_vector_count(&self) -> Result<usize, AgentDbError> {
        let table = self.ensure_table().await?;
        let mut results = table.query().execute().await?;
        let mut count = 0;

        while let Some(batch) = results.try_next().await? {
            count += batch.num_rows();
        }

        Ok(count)
    }
}

// 相似度计算函数
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    let sum: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();
    sum.sqrt()
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}
