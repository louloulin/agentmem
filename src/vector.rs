// 向量搜索模块
use std::sync::Arc;
use arrow::array::{Array, FixedSizeListArray, Float32Array, StringArray, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::types::{AgentDbError, VectorIndexType, VectorSearchResult, IndexStats};

pub struct VectorSearchEngine {
    connection: Arc<Connection>,
    dimension: usize,
}

impl VectorSearchEngine {
    pub fn new(connection: Arc<Connection>, dimension: usize) -> Self {
        Self { connection, dimension }
    }

    pub async fn ensure_vectors_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("vectors").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                let schema = Schema::new(vec![
                    Field::new("vector_id", DataType::Utf8, false),
                    Field::new(
                        "vector",
                        DataType::FixedSizeList(
                            Arc::new(Field::new("item", DataType::Float32, true)),
                            self.dimension as i32,
                        ),
                        false,
                    ),
                    Field::new("metadata", DataType::Utf8, false),
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

    pub async fn add_vector(
        &self,
        vector_id: String,
        vector: Vec<f32>,
        metadata: String,
    ) -> Result<(), AgentDbError> {
        if vector.len() != self.dimension {
            return Err(AgentDbError::InvalidArgument(format!(
                "Vector dimension {} does not match expected dimension {}",
                vector.len(),
                self.dimension
            )));
        }

        let table = self.ensure_vectors_table().await?;
        let schema = table.schema().await?;

        // 创建固定大小列表数组
        let vector_values = Float32Array::from(vector);
        let vector_list = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, false)),
            self.dimension as i32,
            Arc::new(vector_values),
            None,
        )?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![vector_id])),
                Arc::new(vector_list),
                Arc::new(StringArray::from(vec![metadata])),
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    pub async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        if query_vector.len() != self.dimension {
            return Err(AgentDbError::InvalidArgument(format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.dimension
            )));
        }

        let table = self.ensure_vectors_table().await?;

        let mut results = table
            .vector_search(query_vector)?
            .limit(limit)
            .execute()
            .await?;

        let mut search_results = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let vector_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let metadata_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();

                // 获取距离（通常在最后一列）
                let distance = if batch.num_columns() > 3 {
                    let distance_array = batch.column(batch.num_columns() - 1)
                        .as_any()
                        .downcast_ref::<Float32Array>()
                        .unwrap();
                    distance_array.value(row)
                } else {
                    0.0 // 默认距离
                };

                search_results.push(VectorSearchResult {
                    vector_id: vector_id_array.value(row).to_string(),
                    distance,
                    metadata: metadata_array.value(row).to_string(),
                });
            }
        }

        Ok(search_results)
    }

    pub async fn get_vector(&self, vector_id: &str) -> Result<Option<(Vec<f32>, String)>, AgentDbError> {
        let table = self.ensure_vectors_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("vector_id = '{}'", vector_id))
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

        let vector_array = batch.column(1).as_any().downcast_ref::<FixedSizeListArray>().unwrap();
        let metadata_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();

        // 提取向量数据
        let vector_values = vector_array.value(0);
        let float_array = vector_values.as_any().downcast_ref::<Float32Array>().unwrap();
        let vector: Vec<f32> = (0..float_array.len()).map(|i| float_array.value(i)).collect();

        let metadata = metadata_array.value(0).to_string();

        Ok(Some((vector, metadata)))
    }

    pub async fn delete_vector(&self, vector_id: &str) -> Result<(), AgentDbError> {
        let table = self.ensure_vectors_table().await?;
        table.delete(&format!("vector_id = '{}'", vector_id)).await?;
        Ok(())
    }

    pub async fn update_vector(
        &self,
        vector_id: &str,
        new_vector: Vec<f32>,
        new_metadata: String,
    ) -> Result<(), AgentDbError> {
        // 先删除旧向量
        self.delete_vector(vector_id).await?;
        
        // 添加新向量
        self.add_vector(vector_id.to_string(), new_vector, new_metadata).await?;
        
        Ok(())
    }

    pub async fn batch_add_vectors(
        &self,
        vectors: Vec<(String, Vec<f32>, String)>,
    ) -> Result<(), AgentDbError> {
        if vectors.is_empty() {
            return Ok(());
        }

        // 验证所有向量的维度
        for (_, vector, _) in &vectors {
            if vector.len() != self.dimension {
                return Err(AgentDbError::InvalidArgument(format!(
                    "Vector dimension {} does not match expected dimension {}",
                    vector.len(),
                    self.dimension
                )));
            }
        }

        let table = self.ensure_vectors_table().await?;
        let schema = table.schema().await?;

        let vector_ids: Vec<String> = vectors.iter().map(|(id, _, _)| id.clone()).collect();
        let metadata_list: Vec<String> = vectors.iter().map(|(_, _, meta)| meta.clone()).collect();

        // 创建向量数组
        let mut vector_data = Vec::new();
        for (_, vector, _) in &vectors {
            vector_data.extend_from_slice(vector);
        }

        // 创建批量向量数组
        let all_vector_values = Float32Array::from(vector_data);
        let vector_list = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, false)),
            self.dimension as i32,
            Arc::new(all_vector_values),
            None,
        )?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vector_ids)),
                Arc::new(vector_list),
                Arc::new(StringArray::from(metadata_list)),
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    pub async fn similarity_search(
        &self,
        query_vector: Vec<f32>,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        let results = self.search_vectors(query_vector, limit * 2).await?;
        
        // 过滤相似度阈值
        let filtered_results: Vec<VectorSearchResult> = results
            .into_iter()
            .filter(|result| result.distance <= threshold)
            .take(limit)
            .collect();

        Ok(filtered_results)
    }

    pub async fn get_vector_count(&self) -> Result<usize, AgentDbError> {
        let table = self.ensure_vectors_table().await?;
        
        let mut results = table
            .query()
            .execute()
            .await?;

        let mut count = 0;
        while let Some(batch) = results.try_next().await? {
            count += batch.num_rows();
        }

        Ok(count)
    }

    pub async fn get_index_stats(&self) -> Result<IndexStats, AgentDbError> {
        let vector_count = self.get_vector_count().await?;
        let now = chrono::Utc::now().timestamp();

        Ok(IndexStats {
            index_id: "default_vector_index".to_string(),
            index_type: VectorIndexType::Flat, // 默认类型
            dimension: self.dimension,
            vector_count,
            memory_usage: vector_count * self.dimension * 4, // 假设每个float32占4字节
            avg_query_time: 0.0, // 需要实际测量
            last_updated: now,
        })
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    // 计算两个向量之间的余弦相似度
    pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> Result<f32, AgentDbError> {
        if vec1.len() != vec2.len() {
            return Err(AgentDbError::InvalidArgument(
                "Vectors must have the same dimension".to_string(),
            ));
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm1 * norm2))
    }

    // 计算欧几里得距离
    pub fn euclidean_distance(vec1: &[f32], vec2: &[f32]) -> Result<f32, AgentDbError> {
        if vec1.len() != vec2.len() {
            return Err(AgentDbError::InvalidArgument(
                "Vectors must have the same dimension".to_string(),
            ));
        }

        let distance: f32 = vec1
            .iter()
            .zip(vec2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt();

        Ok(distance)
    }
}
