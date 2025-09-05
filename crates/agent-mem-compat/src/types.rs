//! Type definitions for Mem0 compatibility

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A memory item in Mem0 format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier for the memory
    pub id: String,
    
    /// The memory content/text
    pub memory: String,
    
    /// User ID associated with this memory
    pub user_id: String,
    
    /// Optional agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    
    /// Optional run ID for session tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    
    /// Memory metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Memory score/relevance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Search result containing memories and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// List of matching memories
    pub memories: Vec<Memory>,
    
    /// Total number of results (may be more than returned)
    pub total: usize,
    
    /// Search metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Filter options for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryFilter {
    /// Filter by agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    
    /// Filter by run ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    
    /// Filter by memory type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<String>,
    
    /// Filter by date range (start)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_after: Option<DateTime<Utc>>,
    
    /// Filter by date range (end)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<DateTime<Utc>>,
    
    /// Additional metadata filters
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    
    /// Offset for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

/// Memory addition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemoryRequest {
    /// The memory content
    pub memory: String,
    
    /// User ID
    pub user_id: String,
    
    /// Optional agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    
    /// Optional run ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Memory update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryRequest {
    /// New memory content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    
    /// Updated metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Memory search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryRequest {
    /// Search query
    pub query: String,
    
    /// User ID to search within
    pub user_id: String,
    
    /// Optional filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<MemoryFilter>,
    
    /// Maximum number of results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Memory deletion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMemoryResponse {
    /// Whether the deletion was successful
    pub success: bool,
    
    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Memory history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHistory {
    /// Memory ID
    pub memory_id: String,
    
    /// User ID
    pub user_id: String,
    
    /// Previous content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_memory: Option<String>,
    
    /// New content
    pub new_memory: String,
    
    /// Timestamp of change
    pub timestamp: DateTime<Utc>,
    
    /// Change metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Note: Conversion implementations would be added here when integrating with full AgentMem
// For now, the compatibility layer uses its own Memory type
