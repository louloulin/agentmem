//! Core memory types and data structures

use agent_mem_traits::{AgentMemError, MemoryItem, Result, Vector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Memory type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Episodic memories - specific events and experiences
    Episodic,
    /// Semantic memories - facts and general knowledge
    Semantic,
    /// Procedural memories - skills and procedures
    Procedural,
    /// Working memories - temporary information
    Working,
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
            MemoryType::Working => "working",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "episodic" => Some(MemoryType::Episodic),
            "semantic" => Some(MemoryType::Semantic),
            "procedural" => Some(MemoryType::Procedural),
            "working" => Some(MemoryType::Working),
            _ => None,
        }
    }
}

/// Memory importance level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum ImportanceLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl ImportanceLevel {
    pub fn from_score(score: f32) -> Self {
        if score >= 0.8 {
            ImportanceLevel::Critical
        } else if score >= 0.6 {
            ImportanceLevel::High
        } else if score >= 0.4 {
            ImportanceLevel::Medium
        } else {
            ImportanceLevel::Low
        }
    }

    pub fn to_score(&self) -> f32 {
        match self {
            ImportanceLevel::Low => 0.25,
            ImportanceLevel::Medium => 0.5,
            ImportanceLevel::High => 0.75,
            ImportanceLevel::Critical => 1.0,
        }
    }
}

/// Core memory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique memory identifier
    pub id: String,
    /// Agent identifier
    pub agent_id: String,
    /// User identifier (optional)
    pub user_id: Option<String>,
    /// Memory type
    pub memory_type: MemoryType,
    /// Memory content
    pub content: String,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
    /// Vector embedding (optional)
    pub embedding: Option<Vector>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last access timestamp
    pub last_accessed_at: i64,
    /// Access count
    pub access_count: u32,
    /// Expiration timestamp (optional)
    pub expires_at: Option<i64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Memory version for conflict resolution
    pub version: u32,
}

impl Memory {
    /// Create a new memory
    pub fn new(
        agent_id: String,
        user_id: Option<String>,
        memory_type: MemoryType,
        content: String,
        importance: f32,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            user_id,
            memory_type,
            content,
            importance: importance.clamp(0.0, 1.0),
            embedding: None,
            created_at: now,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            metadata: HashMap::new(),
            version: 1,
        }
    }

    /// Record access to this memory
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed_at = chrono::Utc::now().timestamp();
    }

    /// Calculate current importance with decay
    pub fn calculate_current_importance(&self) -> f32 {
        let current_time = chrono::Utc::now().timestamp();
        let time_decay = (current_time - self.created_at) as f32 / (24.0 * 3600.0); // days
        let access_factor = (self.access_count as f32 + 1.0).ln();

        // Apply time decay and access boost
        self.importance * (-time_decay * 0.01).exp() * (1.0 + access_factor * 0.1)
    }

    /// Check if memory is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now().timestamp() > expires_at
        } else {
            false
        }
    }

    /// Set expiration time
    pub fn set_expiration(&mut self, expires_at: i64) {
        self.expires_at = Some(expires_at);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Update content and increment version
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.version += 1;
        self.last_accessed_at = chrono::Utc::now().timestamp();
    }
}

impl From<Memory> for MemoryItem {
    fn from(memory: Memory) -> Self {
        use agent_mem_traits::{MemoryType as TraitMemoryType, Session};
        use chrono::{DateTime, Utc};

        // Convert metadata from String to serde_json::Value
        let metadata: std::collections::HashMap<String, serde_json::Value> = memory
            .metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // Create a session from memory data
        let mut session = Session::new();
        if let Some(user_id) = memory.user_id {
            session = session.with_user_id(Some(user_id));
        }
        session = session.with_agent_id(Some(memory.agent_id));

        MemoryItem {
            id: memory.id,
            content: memory.content,
            hash: None, // TODO: Calculate hash if needed
            metadata,
            score: Some(memory.importance),
            created_at: DateTime::from_timestamp(memory.created_at, 0)
                .unwrap_or_else(|| Utc::now()),
            updated_at: Some(
                DateTime::from_timestamp(memory.last_accessed_at, 0).unwrap_or_else(|| Utc::now()),
            ),
            session,
            memory_type: match memory.memory_type {
                MemoryType::Episodic => TraitMemoryType::Episodic,
                MemoryType::Semantic => TraitMemoryType::Semantic,
                MemoryType::Procedural => TraitMemoryType::Procedural,
                MemoryType::Working => TraitMemoryType::Working,
            },
            entities: Vec::new(),  // TODO: Extract entities if needed
            relations: Vec::new(), // TODO: Extract relations if needed
            // Additional fields for compatibility
            agent_id: memory.agent_id,
            user_id: memory.user_id,
            importance: memory.importance,
            embedding: memory.embedding.map(|v| v.values),
            last_accessed_at: DateTime::from_timestamp(memory.last_accessed_at, 0)
                .unwrap_or_else(|| Utc::now()),
            access_count: memory.access_count,
            expires_at: memory.expires_at.map(|ts|
                DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now())
            ),
            version: memory.version,
        }
    }
}

impl TryFrom<MemoryItem> for Memory {
    type Error = AgentMemError;

    fn try_from(item: MemoryItem) -> Result<Self> {
        // Convert metadata from serde_json::Value to String
        let metadata: std::collections::HashMap<String, String> = item
            .metadata
            .into_iter()
            .filter_map(|(k, v)| match v {
                serde_json::Value::String(s) => Some((k, s)),
                _ => Some((k, v.to_string())),
            })
            .collect();

        let agent_id = item
            .session
            .agent_id
            .ok_or_else(|| AgentMemError::memory_error("Missing agent_id in session"))?;

        let memory_type = match item.memory_type {
            agent_mem_traits::MemoryType::Episodic => MemoryType::Episodic,
            agent_mem_traits::MemoryType::Semantic => MemoryType::Semantic,
            agent_mem_traits::MemoryType::Procedural => MemoryType::Procedural,
            agent_mem_traits::MemoryType::Working => MemoryType::Working,
            agent_mem_traits::MemoryType::Factual => MemoryType::Semantic, // Map Factual to Semantic
        };

        Ok(Memory {
            id: item.id,
            agent_id,
            user_id: item.session.user_id,
            memory_type,
            content: item.content,
            importance: item.score.unwrap_or(0.5),
            embedding: None,
            created_at: item.created_at.timestamp(),
            last_accessed_at: item
                .updated_at
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|| chrono::Utc::now().timestamp()),
            access_count: 0,  // Default value
            expires_at: None, // Default value
            metadata,
            version: 1,
        })
    }
}

/// Memory search query
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    /// Agent ID to search within
    pub agent_id: String,
    /// User ID filter (optional)
    pub user_id: Option<String>,
    /// Memory type filter (optional)
    pub memory_type: Option<MemoryType>,
    /// Text query for content search
    pub text_query: Option<String>,
    /// Vector query for semantic search
    pub vector_query: Option<Vector>,
    /// Minimum importance threshold
    pub min_importance: Option<f32>,
    /// Maximum age in seconds
    pub max_age_seconds: Option<i64>,
    /// Maximum number of results
    pub limit: usize,
}

impl MemoryQuery {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            user_id: None,
            memory_type: None,
            text_query: None,
            vector_query: None,
            min_importance: None,
            max_age_seconds: None,
            limit: 10,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_memory_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    pub fn with_text_query(mut self, query: String) -> Self {
        self.text_query = Some(query);
        self
    }

    pub fn with_vector_query(mut self, vector: Vector) -> Self {
        self.vector_query = Some(vector);
        self
    }

    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = Some(importance);
        self
    }

    pub fn with_max_age_seconds(mut self, seconds: i64) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Memory search result
#[derive(Debug, Clone)]
pub struct MemorySearchResult {
    pub memory: Memory,
    pub score: f32,
    pub match_type: MatchType,
}

/// Type of match found
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    ExactText,
    PartialText,
    Semantic,
    Metadata,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub memories_by_type: HashMap<MemoryType, usize>,
    pub memories_by_agent: HashMap<String, usize>,
    pub average_importance: f32,
    pub oldest_memory_age_days: f32,
    pub most_accessed_memory_id: Option<String>,
    pub total_access_count: u64,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_memories: 0,
            memories_by_type: HashMap::new(),
            memories_by_agent: HashMap::new(),
            average_importance: 0.0,
            oldest_memory_age_days: 0.0,
            most_accessed_memory_id: None,
            total_access_count: 0,
        }
    }
}
