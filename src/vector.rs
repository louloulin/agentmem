// 向量搜索模块
use std::sync::Arc;
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use arrow::array::{Array, FixedSizeListArray, Float32Array, StringArray, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::types::{AgentDbError, VectorIndexType, VectorSearchResult, IndexStats};

// 高级向量索引结构
#[derive(Debug, Clone)]
pub struct VectorIndex {
    pub index_id: String,
    pub dimension: usize,
    pub index_type: VectorIndexType,
    pub vectors: Vec<Vec<f32>>,
    pub metadata: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

// HNSW节点
#[derive(Debug, Clone)]
pub struct HNSWNode {
    pub id: usize,
    pub vector: Vec<f32>,
    pub connections: Vec<Vec<usize>>, // 每层的连接
    pub level: usize,
}

// HNSW索引
#[derive(Debug, Clone)]
pub struct HNSWIndex {
    pub nodes: Vec<HNSWNode>,
    pub entry_point: Option<usize>,
    pub max_level: usize,
    pub max_connections: usize,
    pub ef_construction: usize,
    pub ml: f32,
}

// 用于排序的包装器
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct OrderedFloat(f32);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

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

// 高级向量引擎
pub struct AdvancedVectorEngine {
    connection: Arc<Connection>,
    indexes: HashMap<String, VectorIndex>,
    hnsw_indexes: HashMap<String, HNSWIndex>,
}

impl AdvancedVectorEngine {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = lancedb::connect(db_path).execute().await?;
        Ok(Self {
            connection: Arc::new(connection),
            indexes: HashMap::new(),
            hnsw_indexes: HashMap::new(),
        })
    }

    // 创建向量索引
    pub fn create_vector_index(&mut self, index_id: String, dimension: usize, index_type: VectorIndexType) -> Result<(), AgentDbError> {
        let index = VectorIndex {
            index_id: index_id.clone(),
            dimension,
            index_type,
            vectors: Vec::new(),
            metadata: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        self.indexes.insert(index_id.clone(), index);

        // 如果是HNSW索引，创建对应的HNSW结构
        if index_type == VectorIndexType::HNSW {
            let hnsw_index = HNSWIndex {
                nodes: Vec::new(),
                entry_point: None,
                max_level: 16,
                max_connections: 16,
                ef_construction: 200,
                ml: 1.0 / (2.0_f32).ln(),
            };
            self.hnsw_indexes.insert(index_id, hnsw_index);
        }

        Ok(())
    }

    // 添加向量到索引
    pub fn add_vector(&mut self, index_id: &str, vector: Vec<f32>, metadata: String) -> Result<String, AgentDbError> {
        let vector_id = uuid::Uuid::new_v4().to_string();

        let index = self.indexes.get(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument("Index not found".to_string()))?;

        let index_type = index.index_type;

        match index_type {
            VectorIndexType::Flat => {
                let index = self.indexes.get_mut(index_id).unwrap();
                index.vectors.push(vector);
                index.metadata.push(metadata);
                index.updated_at = chrono::Utc::now().timestamp();
            }
            VectorIndexType::HNSW => {
                self.add_to_hnsw(index_id, vector, metadata)?;
                let index = self.indexes.get_mut(index_id).unwrap();
                index.updated_at = chrono::Utc::now().timestamp();
            }
            VectorIndexType::IVF => {
                self.add_to_ivf(index_id, vector, metadata)?;
                let index = self.indexes.get_mut(index_id).unwrap();
                index.updated_at = chrono::Utc::now().timestamp();
            }
        }

        Ok(vector_id)
    }

    // 高性能向量搜索
    pub fn search_vectors(&self, index_id: &str, query: &[f32], k: usize, ef: Option<usize>) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        let index = self.indexes.get(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument("Index not found".to_string()))?;

        match index.index_type {
            VectorIndexType::Flat => self.search_flat(index, query, k),
            VectorIndexType::HNSW => self.search_hnsw(index_id, query, k, ef.unwrap_or(50)),
            VectorIndexType::IVF => self.search_ivf(index, query, k),
        }
    }

    // 暴力搜索
    fn search_flat(&self, index: &VectorIndex, query: &[f32], k: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        let mut results: Vec<(f32, usize)> = index.vectors.iter()
            .enumerate()
            .map(|(i, vector)| {
                let distance = euclidean_distance(query, vector);
                (distance, i)
            })
            .collect();

        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

        Ok(results.into_iter()
            .take(k)
            .map(|(distance, i)| VectorSearchResult {
                vector_id: format!("{}_{}", index.index_id, i),
                distance,
                metadata: index.metadata.get(i).cloned().unwrap_or_default(),
            })
            .collect())
    }

    // HNSW搜索
    fn search_hnsw(&self, index_id: &str, query: &[f32], k: usize, ef: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        let hnsw = self.hnsw_indexes.get(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument("HNSW index not found".to_string()))?;

        if hnsw.nodes.is_empty() {
            return Ok(Vec::new());
        }

        let entry_point = hnsw.entry_point.unwrap();
        let mut current = entry_point;

        // 从顶层向下搜索到第1层
        for lc in (1..=hnsw.max_level).rev() {
            let candidates = self.search_layer_hnsw(&hnsw.nodes, query, current, 1, lc);
            if !candidates.is_empty() {
                current = candidates[0];
            }
        }

        // 在第0层进行详细搜索
        let candidates = self.search_layer_hnsw(&hnsw.nodes, query, current, ef.max(k), 0);

        let results: Vec<VectorSearchResult> = candidates.into_iter()
            .take(k)
            .filter_map(|node_id| {
                hnsw.nodes.get(node_id).map(|node| {
                    let distance = euclidean_distance(query, &node.vector);
                    VectorSearchResult {
                        vector_id: format!("{}_{}", index_id, node_id),
                        distance,
                        metadata: format!("hnsw_node_{}", node_id),
                    }
                })
            })
            .collect();

        Ok(results)
    }

    // IVF搜索（简化实现）
    fn search_ivf(&self, index: &VectorIndex, query: &[f32], k: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        // 简化为暴力搜索
        self.search_flat(index, query, k)
    }

    // HNSW层搜索
    fn search_layer_hnsw(&self, nodes: &[HNSWNode], query: &[f32], entry_point: usize, ef: usize, level: usize) -> Vec<usize> {
        let mut visited = std::collections::HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut w = BinaryHeap::new();

        let entry_distance = euclidean_distance(query, &nodes[entry_point].vector);
        candidates.push(OrderedFloat(-entry_distance));
        w.push(OrderedFloat(entry_distance));
        visited.insert(entry_point);

        while let Some(OrderedFloat(neg_distance)) = candidates.pop() {
            let distance = -neg_distance;

            if let Some(&OrderedFloat(furthest_distance)) = w.peek() {
                if distance > furthest_distance {
                    break;
                }
            }

            // 找到当前节点
            let current_node = visited.iter()
                .find(|&&node_id| {
                    let node_distance = euclidean_distance(query, &nodes[node_id].vector);
                    (node_distance - distance).abs() < 1e-6
                })
                .copied()
                .unwrap_or(entry_point);

            if current_node < nodes.len() && level < nodes[current_node].connections.len() {
                for &neighbor in &nodes[current_node].connections[level] {
                    if !visited.contains(&neighbor) && neighbor < nodes.len() {
                        visited.insert(neighbor);
                        let neighbor_distance = euclidean_distance(query, &nodes[neighbor].vector);

                        if let Some(&OrderedFloat(furthest_distance)) = w.peek() {
                            if neighbor_distance < furthest_distance || w.len() < ef {
                                candidates.push(OrderedFloat(-neighbor_distance));
                                w.push(OrderedFloat(neighbor_distance));

                                if w.len() > ef {
                                    w.pop();
                                }
                            }
                        } else {
                            candidates.push(OrderedFloat(-neighbor_distance));
                            w.push(OrderedFloat(neighbor_distance));
                        }
                    }
                }
            }
        }

        visited.into_iter().collect()
    }

    // 添加向量到HNSW索引
    fn add_to_hnsw(&mut self, index_id: &str, vector: Vec<f32>, _metadata: String) -> Result<(), AgentDbError> {
        let ml = {
            let hnsw = self.hnsw_indexes.get(index_id)
                .ok_or_else(|| AgentDbError::InvalidArgument("HNSW index not found".to_string()))?;
            hnsw.ml
        };

        let node_id = {
            let hnsw = self.hnsw_indexes.get(index_id).unwrap();
            hnsw.nodes.len()
        };

        let level = self.get_random_level(ml);
        let connections = vec![Vec::new(); level + 1];

        let node = HNSWNode {
            id: node_id,
            vector,
            connections,
            level,
        };

        let hnsw = self.hnsw_indexes.get_mut(index_id).unwrap();
        hnsw.nodes.push(node);

        if hnsw.entry_point.is_none() {
            hnsw.entry_point = Some(node_id);
        }

        Ok(())
    }

    // 添加向量到IVF索引（简化实现）
    fn add_to_ivf(&mut self, index_id: &str, vector: Vec<f32>, metadata: String) -> Result<(), AgentDbError> {
        let index = self.indexes.get_mut(index_id).unwrap();
        index.vectors.push(vector);
        index.metadata.push(metadata);
        Ok(())
    }

    // 获取随机层级
    fn get_random_level(&self, _ml: f32) -> usize {
        let mut level = 0;
        let mut rng = rand::thread_rng();
        while rand::Rng::gen::<f32>(&mut rng) < 0.5 && level < 16 {
            level += 1;
        }
        level
    }

    // 批量向量操作
    pub fn batch_add_vectors(&mut self, index_id: &str, vectors: Vec<Vec<f32>>, metadata: Vec<String>) -> Result<Vec<String>, AgentDbError> {
        if vectors.len() != metadata.len() {
            return Err(AgentDbError::InvalidArgument("Vectors and metadata length mismatch".to_string()));
        }

        let mut vector_ids = Vec::new();
        for (vector, meta) in vectors.into_iter().zip(metadata.into_iter()) {
            let vector_id = self.add_vector(index_id, vector, meta)?;
            vector_ids.push(vector_id);
        }

        Ok(vector_ids)
    }

    // 获取索引统计信息
    pub fn get_index_stats(&self, index_id: &str) -> Result<IndexStats, AgentDbError> {
        let index = self.indexes.get(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument("Index not found".to_string()))?;

        Ok(IndexStats {
            index_id: index.index_id.clone(),
            index_type: index.index_type,
            dimension: index.dimension,
            vector_count: index.vectors.len(),
            memory_usage: index.vectors.len() * index.dimension * 4,
            avg_query_time: 0.0,
            last_updated: index.updated_at,
        })
    }
}

// 辅助函数
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }

    a.iter().zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}
