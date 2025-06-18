// 类型定义模块
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 错误类型定义
#[derive(Debug, thiserror::Error)]
pub enum AgentDbError {
    #[error("Lance error: {0}")]
    Lance(#[from] lancedb::Error),
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Sync error: {0}")]
    Sync(String),
}

// Agent状态类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateType {
    WorkingMemory,
    LongTermMemory,
    Context,
    TaskState,
    Relationship,
    Embedding,
}

impl StateType {
    pub fn to_string(&self) -> &'static str {
        match self {
            StateType::WorkingMemory => "working_memory",
            StateType::LongTermMemory => "long_term_memory",
            StateType::Context => "context",
            StateType::TaskState => "task_state",
            StateType::Relationship => "relationship",
            StateType::Embedding => "embedding",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "working_memory" => Some(StateType::WorkingMemory),
            "long_term_memory" => Some(StateType::LongTermMemory),
            "context" => Some(StateType::Context),
            "task_state" => Some(StateType::TaskState),
            "relationship" => Some(StateType::Relationship),
            "embedding" => Some(StateType::Embedding),
            _ => None,
        }
    }
}

// Agent状态结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub agent_id: u64,
    pub session_id: u64,
    pub timestamp: i64,
    pub state_type: StateType,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub version: u32,
    pub checksum: u32,
}

impl AgentState {
    pub fn new(
        agent_id: u64,
        session_id: u64,
        state_type: StateType,
        data: Vec<u8>,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let checksum = Self::calculate_checksum(&data);

        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            session_id,
            timestamp,
            state_type,
            data,
            metadata: HashMap::new(),
            version: 1,
            checksum,
        }
    }

    pub fn calculate_checksum(data: &[u8]) -> u32 {
        data.iter().fold(0u32, |acc, &byte| acc.wrapping_add(byte as u32))
    }

    pub fn validate_checksum(&self) -> bool {
        Self::calculate_checksum(&self.data) == self.checksum
    }

    pub fn update_data(&mut self, new_data: Vec<u8>) {
        self.data = new_data;
        self.checksum = Self::calculate_checksum(&self.data);
        self.version += 1;
        self.timestamp = chrono::Utc::now().timestamp();
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

// 记忆类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Episodic,
    Semantic,
    Procedural,
    Working,
}

impl MemoryType {
    pub fn to_string(&self) -> &'static str {
        match self {
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
            MemoryType::Working => "working",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "episodic" => Some(MemoryType::Episodic),
            "semantic" => Some(MemoryType::Semantic),
            "procedural" => Some(MemoryType::Procedural),
            "working" => Some(MemoryType::Working),
            _ => None,
        }
    }
}

// 记忆结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub memory_id: String,
    pub agent_id: u64,
    pub memory_type: MemoryType,
    pub content: String,
    pub importance: f32,
    pub embedding: Option<Vec<f32>>,
    pub created_at: i64,
    pub access_count: u32,
    pub last_access: i64,
    pub expires_at: Option<i64>,
}

impl Memory {
    pub fn new(agent_id: u64, memory_type: MemoryType, content: String, importance: f32) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            memory_id: Uuid::new_v4().to_string(),
            agent_id,
            memory_type,
            content,
            importance,
            embedding: None,
            created_at: now,
            access_count: 0,
            last_access: now,
            expires_at: None,
        }
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_access = chrono::Utc::now().timestamp();
    }

    pub fn calculate_importance(&self, current_time: i64) -> f32 {
        let time_decay = (current_time - self.created_at) as f32 / (24.0 * 3600.0);
        let access_factor = (self.access_count as f32 + 1.0).ln();
        self.importance * (-time_decay * 0.01).exp() * access_factor
    }

    pub fn is_expired(&self, current_time: i64) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_time > expires_at
        } else {
            false
        }
    }
}

// 文档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_id: String,
    pub title: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub chunks: Vec<Chunk>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Document {
    pub fn new(title: String, content: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            doc_id: Uuid::new_v4().to_string(),
            title,
            content,
            metadata: HashMap::new(),
            chunks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn chunk_document(&mut self, chunk_size: usize, overlap: usize) -> Result<(), AgentDbError> {
        let mut chunks = Vec::new();
        let content_bytes = self.content.as_bytes();
        let mut pos = 0;
        let mut chunk_index = 0;

        while pos < content_bytes.len() {
            let end = std::cmp::min(pos + chunk_size, content_bytes.len());
            let chunk_content = String::from_utf8_lossy(&content_bytes[pos..end]).to_string();

            let chunk = Chunk {
                chunk_id: format!("{}_{}", self.doc_id, chunk_index),
                doc_id: self.doc_id.clone(),
                content: chunk_content,
                chunk_index,
                position: pos,
                size: end - pos,
            };

            chunks.push(chunk);
            
            if end >= content_bytes.len() {
                break;
            }
            
            pos = if chunk_size > overlap { 
                pos + chunk_size - overlap 
            } else { 
                pos + 1 
            };
            chunk_index += 1;
        }

        self.chunks = chunks;
        Ok(())
    }
}

// 文档块结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub doc_id: String,
    pub content: String,
    pub chunk_index: u32,
    pub position: usize,
    pub size: usize,
}

// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunk_id: String,
    pub doc_id: String,
    pub content: String,
    pub score: f32,
}

// 向量索引类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VectorIndexType {
    Flat,
    HNSW,
    IVF,
}

// 向量搜索结果
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub vector_id: String,
    pub distance: f32,
    pub metadata: String,
}

// 查询类型
#[derive(Debug, Clone)]
pub enum QueryType {
    VectorSearch,
    MemoryRetrieval,
    AgentState,
    RAG,
    Hybrid,
}

// 查询计划
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub query_id: String,
    pub query_type: QueryType,
    pub estimated_cost: f64,
    pub estimated_time: f64,
    pub index_recommendations: Vec<String>,
    pub optimization_hints: Vec<String>,
}

// 索引统计
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub index_id: String,
    pub index_type: VectorIndexType,
    pub dimension: usize,
    pub vector_count: usize,
    pub memory_usage: usize,
    pub avg_query_time: f64,
    pub last_updated: i64,
}
