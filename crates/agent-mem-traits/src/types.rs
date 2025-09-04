//! Core data types for AgentMem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: MessageRole::System,
            content: content.to_string(),
            timestamp: Some(Utc::now()),
        }
    }
    
    pub fn user(content: &str) -> Self {
        Self {
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: Some(Utc::now()),
        }
    }
    
    pub fn assistant(content: &str) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.to_string(),
            timestamp: Some(Utc::now()),
        }
    }
}

/// A memory item stored in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub hash: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub score: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub session: Session,
    pub memory_type: MemoryType,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

/// Session information for scoping memories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub id: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            ..Default::default()
        }
    }
    
    pub fn with_user_id(mut self, user_id: Option<String>) -> Self {
        self.user_id = user_id;
        self
    }
    
    pub fn with_agent_id(mut self, agent_id: Option<String>) -> Self {
        self.agent_id = agent_id;
        self
    }
    
    pub fn with_run_id(mut self, run_id: Option<String>) -> Self {
        self.run_id = run_id;
        self
    }
}

/// Types of memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Factual,      // 事实性记忆
    Episodic,     // 情节性记忆
    Procedural,   // 程序性记忆
    Semantic,     // 语义记忆
    Working,      // 工作记忆
}

/// An extracted fact from content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    pub content: String,
    pub confidence: f32,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

/// An entity extracted from content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// A relation between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub source: String,
    pub relation: String,
    pub target: String,
    pub confidence: f32,
}

/// A vector with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Vector {
    pub fn new(values: Vec<f32>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            values,
            metadata: HashMap::new(),
        }
    }
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Filters for search operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Filters {
    pub filters: HashMap<String, serde_json::Value>,
}

impl Filters {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add(&mut self, key: &str, value: serde_json::Value) {
        self.filters.insert(key.to_string(), value);
    }
}

/// History entry for memory changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub memory_id: String,
    pub event: MemoryEvent,
    pub timestamp: DateTime<Utc>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEvent {
    Create,
    Update,
    Delete,
    Merge,
}

/// Configuration types
pub type Metadata = HashMap<String, serde_json::Value>;

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub response_format: Option<String>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            base_url: None,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            response_format: None,
        }
    }
}

/// Vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub provider: String,
    pub path: String,
    pub table_name: String,
    pub dimension: usize,
    pub api_key: Option<String>,
    pub index_name: Option<String>,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            provider: "lancedb".to_string(),
            path: "./data/vectors".to_string(),
            table_name: "memories".to_string(),
            dimension: 1536,
            api_key: None,
            index_name: None,
        }
    }
}
