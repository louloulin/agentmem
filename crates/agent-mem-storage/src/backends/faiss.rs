//! FAISS 向量存储后端实现
//! 
//! FAISS (Facebook AI Similarity Search) 是一个高效的向量相似性搜索库，
//! 特别适合大规模向量数据的本地存储和搜索。

use agent_mem_traits::{AgentMemError, Result, VectorData, VectorStore, VectorSearchResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;
use uuid::Uuid;

/// FAISS 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaissConfig {
    /// 向量维度
    pub dimension: usize,
    /// 索引类型 (Flat, IVF, HNSW 等)
    pub index_type: FaissIndexType,
    /// 数据存储路径
    pub data_path: PathBuf,
    /// 元数据存储路径
    pub metadata_path: PathBuf,
    /// 是否启用 GPU 加速
    pub use_gpu: bool,
    /// 训练样本数量 (用于 IVF 索引)
    pub train_size: Option<usize>,
    /// 聚类数量 (用于 IVF 索引)
    pub nlist: Option<usize>,
}

impl Default for FaissConfig {
    fn default() -> Self {
        Self {
            dimension: 1536, // OpenAI embedding 默认维度
            index_type: FaissIndexType::Flat,
            data_path: PathBuf::from("./data/faiss_index"),
            metadata_path: PathBuf::from("./data/faiss_metadata.json"),
            use_gpu: false,
            train_size: Some(10000),
            nlist: Some(100),
        }
    }
}

/// FAISS 索引类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaissIndexType {
    /// 暴力搜索，精确但较慢
    Flat,
    /// 倒排文件索引，快速但需要训练
    IVF,
    /// 分层导航小世界图，快速且内存高效
    HNSW,
    /// 乘积量化，内存高效
    PQ,
    /// IVF + PQ 组合
    IVFPQ,
}

impl FaissIndexType {
    /// 获取 FAISS 索引字符串
    pub fn to_faiss_string(&self, dimension: usize, nlist: Option<usize>) -> String {
        match self {
            FaissIndexType::Flat => format!("Flat"),
            FaissIndexType::IVF => format!("IVF{},Flat", nlist.unwrap_or(100)),
            FaissIndexType::HNSW => format!("HNSW32"),
            FaissIndexType::PQ => format!("PQ{}x8", dimension / 8),
            FaissIndexType::IVFPQ => format!("IVF{},PQ{}x8", nlist.unwrap_or(100), dimension / 8),
        }
    }
}

/// 向量元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    pub id: String,
    pub payload: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
}

/// FAISS 存储实现
pub struct FaissStore {
    config: FaissConfig,
    // 注意：这里我们使用一个简化的内存实现作为占位符
    // 在实际实现中，这里应该是 FAISS 索引的绑定
    vectors: Arc<Mutex<HashMap<String, VectorData>>>,
    metadata: Arc<Mutex<HashMap<String, VectorMetadata>>>,
    next_id: Arc<Mutex<usize>>,
}

impl FaissStore {
    /// 创建新的 FAISS 存储实例
    pub async fn new(config: FaissConfig) -> Result<Self> {
        // 确保数据目录存在
        if let Some(parent) = config.data_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to create data directory: {}", e)))?;
        }
        
        if let Some(parent) = config.metadata_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to create metadata directory: {}", e)))?;
        }

        let store = Self {
            config,
            vectors: Arc::new(Mutex::new(HashMap::new())),
            metadata: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        };

        // 尝试加载现有数据
        store.load_existing_data().await?;

        Ok(store)
    }

    /// 加载现有数据
    async fn load_existing_data(&self) -> Result<()> {
        // 加载元数据
        if self.config.metadata_path.exists() {
            match fs::read_to_string(&self.config.metadata_path).await {
                Ok(content) => {
                    if let Ok(metadata_map) = serde_json::from_str::<HashMap<String, VectorMetadata>>(&content) {
                        let mut metadata = self.metadata.lock().unwrap();
                        *metadata = metadata_map;
                    }
                }
                Err(_) => {
                    // 文件不存在或读取失败，忽略
                }
            }
        }

        Ok(())
    }

    /// 保存元数据到文件
    async fn save_metadata(&self) -> Result<()> {
        let metadata = self.metadata.lock().unwrap().clone();
        let content = serde_json::to_string_pretty(&metadata)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize metadata: {}", e)))?;

        // 确保父目录存在
        if let Some(parent) = self.config.metadata_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to create metadata directory: {}", e)))?;
        }

        fs::write(&self.config.metadata_path, content).await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// 生成下一个 ID
    fn next_id(&self) -> String {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        format!("faiss_{}", *next_id)
    }

    /// 计算向量相似度 (余弦相似度)
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
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

    /// 计算欧几里得距离
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

#[async_trait]
impl VectorStore for FaissStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        // 在作用域内处理锁，确保在 await 之前释放
        {
            let mut vector_store = self.vectors.lock().unwrap();
            let mut metadata_store = self.metadata.lock().unwrap();

            for vector_data in vectors {
                let id = if vector_data.id.is_empty() {
                    self.next_id()
                } else {
                    vector_data.id.clone()
                };

                // 验证向量维度
                if vector_data.vector.len() != self.config.dimension {
                    return Err(AgentMemError::validation_error(&format!(
                        "Vector dimension {} does not match expected dimension {}",
                        vector_data.vector.len(),
                        self.config.dimension
                    )));
                }

                // 存储向量
                vector_store.insert(id.clone(), vector_data.clone());

                // 存储元数据
                let payload: HashMap<String, serde_json::Value> = vector_data.metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect();

                let metadata = VectorMetadata {
                    id: id.clone(),
                    payload,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                metadata_store.insert(id.clone(), metadata);

                ids.push(id);
            }
        } // 锁在这里被释放

        // 异步保存元数据
        self.save_metadata().await?;

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        // 验证查询向量维度
        if query_vector.len() != self.config.dimension {
            return Err(AgentMemError::validation_error(&format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.config.dimension
            )));
        }

        let vector_store = self.vectors.lock().unwrap();
        let metadata_store = self.metadata.lock().unwrap();

        let mut results = Vec::new();

        // 计算所有向量的相似度
        for (id, vector_data) in vector_store.iter() {
            let similarity = self.cosine_similarity(&query_vector, &vector_data.vector);
            let distance = self.euclidean_distance(&query_vector, &vector_data.vector);

            // 应用阈值过滤
            if let Some(threshold) = threshold {
                if similarity < threshold {
                    continue;
                }
            }

            results.push(VectorSearchResult {
                id: id.clone(),
                vector: vector_data.vector.clone(),
                metadata: vector_data.metadata.clone(),
                similarity,
                distance,
            });
        }

        // 按相似度排序并限制结果数量
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let vector_store = self.vectors.lock().unwrap();
        Ok(vector_store.get(id).cloned())
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        let mut any_removed = false;

        // 在作用域内处理锁
        {
            let mut vector_store = self.vectors.lock().unwrap();
            let mut metadata_store = self.metadata.lock().unwrap();

            for id in ids {
                if vector_store.remove(&id).is_some() {
                    any_removed = true;
                }
                if metadata_store.remove(&id).is_some() {
                    any_removed = true;
                }
            }
        } // 锁在这里被释放

        if any_removed {
            self.save_metadata().await?;
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        // 在作用域内处理锁
        {
            let mut vector_store = self.vectors.lock().unwrap();
            let mut metadata_store = self.metadata.lock().unwrap();

            for vector_data in vectors {
                let id = if vector_data.id.is_empty() {
                    self.next_id()
                } else {
                    vector_data.id.clone()
                };

                // 验证向量维度
                if vector_data.vector.len() != self.config.dimension {
                    return Err(AgentMemError::validation_error(&format!(
                        "Vector dimension {} does not match expected dimension {}",
                        vector_data.vector.len(),
                        self.config.dimension
                    )));
                }

                // 更新向量
                vector_store.insert(id.clone(), vector_data.clone());

                // 更新元数据
                let payload: HashMap<String, serde_json::Value> = vector_data.metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect();

                let metadata = VectorMetadata {
                    id: id.clone(),
                    payload,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                metadata_store.insert(id, metadata);
            }
        } // 锁在这里被释放

        // 异步保存元数据
        self.save_metadata().await?;

        Ok(())
    }

    async fn count_vectors(&self) -> Result<usize> {
        let vector_store = self.vectors.lock().unwrap();
        Ok(vector_store.len())
    }

    async fn clear(&self) -> Result<()> {
        // 在作用域内处理锁
        {
            let mut vector_store = self.vectors.lock().unwrap();
            let mut metadata_store = self.metadata.lock().unwrap();

            vector_store.clear();
            metadata_store.clear();
        } // 锁在这里被释放

        // 保存空的元数据
        self.save_metadata().await?;

        Ok(())
    }
}
