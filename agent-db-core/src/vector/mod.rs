// 向量处理和搜索模块 - 简化但功能完整的实现
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use serde_json;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use crate::core::AgentDbError;

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
    pub metric: String,
    pub index_type: String,
    pub ef_construction: usize,
    pub m: usize,
}

impl Default for VectorIndexConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            metric: "cosine".to_string(),
            index_type: "hnsw".to_string(),
            ef_construction: 200,
            m: 16,
        }
    }
}

// 向量数据存储
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorData {
    pub id: u64,
    pub vector: Vec<f32>,
    pub metadata: HashMap<String, String>,
}

// 高级向量引擎 - 简化实现
pub struct AdvancedVectorEngine {
    db_path: String,
    config: VectorIndexConfig,
    vectors: Arc<RwLock<HashMap<u64, VectorData>>>,
    // 添加查询缓存
    search_cache: Arc<RwLock<HashMap<u64, Vec<VectorSearchResult>>>>,
}

impl AdvancedVectorEngine {
    pub fn new(db_path: &str, config: VectorIndexConfig) -> Self {
        Self {
            db_path: db_path.to_string(),
            config,
            vectors: Arc::new(RwLock::new(HashMap::new())),
            search_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn init(&mut self) -> Result<(), AgentDbError> {
        // 确保目录存在
        if let Some(parent) = Path::new(&self.db_path).parent() {
            fs::create_dir_all(parent).map_err(|e| AgentDbError::Io(e))?;
        }
        
        // 加载现有向量
        self.load_from_disk().await?;
        
        Ok(())
    }

    async fn load_from_disk(&self) -> Result<(), AgentDbError> {
        let vectors_file = format!("{}/vectors.json", self.db_path);
        
        if Path::new(&vectors_file).exists() {
            let content = fs::read_to_string(&vectors_file).map_err(|e| AgentDbError::Io(e))?;
            if !content.trim().is_empty() {
                let loaded_vectors: HashMap<u64, VectorData> = serde_json::from_str(&content)
                    .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
                
                let mut vectors = self.vectors.write().unwrap();
                *vectors = loaded_vectors;
            }
        }
        
        Ok(())
    }
    
    async fn save_to_disk(&self) -> Result<(), AgentDbError> {
        // 确保目录存在
        fs::create_dir_all(&self.db_path).map_err(|e| AgentDbError::Io(e))?;
        
        let vectors_file = format!("{}/vectors.json", self.db_path);
        
        let vectors = self.vectors.read().unwrap();
        let content = serde_json::to_string_pretty(&*vectors)
            .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
        fs::write(&vectors_file, content).map_err(|e| AgentDbError::Io(e))?;
        
        Ok(())
    }

    pub async fn add_vector(&self, id: u64, vector: Vec<f32>, metadata: HashMap<String, String>) -> Result<(), AgentDbError> {
        // 验证向量维度
        if vector.len() != self.config.dimension {
            return Err(AgentDbError::InvalidArgument(
                format!("Vector dimension {} does not match config dimension {}", 
                        vector.len(), self.config.dimension)
            ));
        }
        
        let vector_data = VectorData {
            id,
            vector,
            metadata,
        };
        
        {
            let mut vectors = self.vectors.write().unwrap();
            vectors.insert(id, vector_data);
        }
        
        self.save_to_disk().await?;
        Ok(())
    }

    pub async fn search_vectors(&self, query: &[f32], limit: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        // 验证查询向量维度
        if query.len() != self.config.dimension {
            return Err(AgentDbError::InvalidArgument(
                format!("Query vector dimension {} does not match config dimension {}",
                        query.len(), self.config.dimension)
            ));
        }

        // 计算查询向量的哈希作为缓存键
        let cache_key = self.hash_vector(query);

        // 检查缓存
        {
            let cache = self.search_cache.read().unwrap();
            if let Some(cached_results) = cache.get(&cache_key) {
                let mut results = cached_results.clone();
                results.truncate(limit);
                return Ok(results);
            }
        }

        let vectors = self.vectors.read().unwrap();
        
        let mut results: Vec<VectorSearchResult> = vectors
            .values()
            .map(|vector_data| {
                let similarity = self.calculate_similarity(query, &vector_data.vector);
                let distance = 1.0 - similarity;
                
                VectorSearchResult {
                    id: vector_data.id,
                    vector: vector_data.vector.clone(),
                    metadata: vector_data.metadata.clone(),
                    similarity,
                    distance,
                }
            })
            .collect();
        
        // 按相似度排序
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));

        // 缓存结果（缓存更多结果以便不同limit的查询）
        let cache_results = results.clone();
        {
            let mut cache = self.search_cache.write().unwrap();
            cache.insert(cache_key, cache_results);

            // 限制缓存大小
            if cache.len() > 1000 {
                cache.clear();
            }
        }

        // 限制结果数量
        results.truncate(limit);

        Ok(results)
    }

    // 计算向量哈希用于缓存
    fn hash_vector(&self, vector: &[f32]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for &val in vector {
            val.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }

    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        match self.config.metric.as_str() {
            "cosine" => self.calculate_cosine_similarity(a, b),
            "euclidean" => self.calculate_euclidean_similarity(a, b),
            "dot" => self.calculate_dot_product(a, b),
            _ => self.calculate_cosine_similarity(a, b), // 默认使用余弦相似度
        }
    }
    
    fn calculate_cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
    
    fn calculate_euclidean_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let distance: f32 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();
        
        // 转换为相似度（距离越小，相似度越高）
        1.0 / (1.0 + distance)
    }
    
    fn calculate_dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    pub async fn update_vector(&self, id: u64, vector: Vec<f32>, metadata: HashMap<String, String>) -> Result<(), AgentDbError> {
        // 验证向量维度
        if vector.len() != self.config.dimension {
            return Err(AgentDbError::InvalidArgument(
                format!("Vector dimension {} does not match config dimension {}", 
                        vector.len(), self.config.dimension)
            ));
        }
        
        let vector_data = VectorData {
            id,
            vector,
            metadata,
        };
        
        {
            let mut vectors = self.vectors.write().unwrap();
            vectors.insert(id, vector_data);
        }
        
        self.save_to_disk().await?;
        Ok(())
    }

    pub async fn delete_vector(&self, id: u64) -> Result<bool, AgentDbError> {
        let removed = {
            let mut vectors = self.vectors.write().unwrap();
            vectors.remove(&id).is_some()
        };
        
        if removed {
            self.save_to_disk().await?;
        }
        
        Ok(removed)
    }

    pub async fn get_vector(&self, id: u64) -> Result<Option<VectorSearchResult>, AgentDbError> {
        let vectors = self.vectors.read().unwrap();
        
        if let Some(vector_data) = vectors.get(&id) {
            Ok(Some(VectorSearchResult {
                id: vector_data.id,
                vector: vector_data.vector.clone(),
                metadata: vector_data.metadata.clone(),
                similarity: 1.0, // 自身相似度为1
                distance: 0.0,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn count_vectors(&self) -> Result<usize, AgentDbError> {
        let vectors = self.vectors.read().unwrap();
        Ok(vectors.len())
    }

    pub async fn clear_all_vectors(&self) -> Result<(), AgentDbError> {
        {
            let mut vectors = self.vectors.write().unwrap();
            vectors.clear();
        }
        
        self.save_to_disk().await?;
        Ok(())
    }

    pub fn get_config(&self) -> &VectorIndexConfig {
        &self.config
    }

    pub async fn rebuild_index(&self) -> Result<(), AgentDbError> {
        // 在简化实现中，重建索引就是重新保存到磁盘
        self.save_to_disk().await?;
        Ok(())
    }
}
