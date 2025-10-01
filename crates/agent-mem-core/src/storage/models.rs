//! Database models for AgentMem
//!
//! This module defines the database schema and models for AgentMem,
//! inspired by MIRIX's design but implemented in Rust with SQLx.
//!
//! Key design principles:
//! - Multi-tenancy through organization_id
//! - Soft deletes with is_deleted flag
//! - Audit trail with created_by_id and last_updated_by_id
//! - Timestamps for created_at and updated_at
//! - Foreign key relationships for data integrity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

/// Organization model - the highest level of the object tree
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

/// User model - represents a user within an organization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub status: String, // "active" or "inactive"
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Agent model - represents an AI agent
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: String,
    pub organization_id: String,
    pub agent_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub system: Option<String>, // System prompt
    pub topic: Option<String>,  // Current topic
    #[sqlx(json)]
    pub message_ids: Option<Vec<String>>, // In-context message IDs
    #[sqlx(json)]
    pub metadata_: Option<JsonValue>,
    #[sqlx(json)]
    pub llm_config: Option<JsonValue>,
    #[sqlx(json)]
    pub embedding_config: Option<JsonValue>,
    #[sqlx(json)]
    pub tool_rules: Option<JsonValue>,
    #[sqlx(json)]
    pub mcp_tools: Option<Vec<String>>, // MCP server names
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Message model - represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub role: String, // "user", "assistant", "system", "tool"
    pub text: Option<String>,
    #[sqlx(json)]
    pub content: Option<JsonValue>, // Message content parts
    pub model: Option<String>,
    pub name: Option<String>,
    #[sqlx(json)]
    pub tool_calls: Option<JsonValue>,
    pub tool_call_id: Option<String>,
    pub step_id: Option<String>,
    pub otid: Option<String>, // Offline threading ID
    #[sqlx(json)]
    pub tool_returns: Option<JsonValue>,
    pub group_id: Option<String>,
    pub sender_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Block model - represents a section of core memory
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Block {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub template_name: Option<String>,
    pub description: Option<String>,
    pub label: String, // "human", "persona", "system"
    pub is_template: bool,
    pub value: String,
    pub limit: i64, // Character limit
    #[sqlx(json)]
    pub metadata_: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Tool model - represents a tool that can be used by agents
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tool {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(json)]
    pub json_schema: Option<JsonValue>,
    pub source_type: Option<String>,
    pub source_code: Option<String>,
    #[sqlx(json)]
    pub tags: Option<Vec<String>>,
    #[sqlx(json)]
    pub metadata_: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Memory model - enhanced version with agent and user relationships
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Memory {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub content: String,
    pub hash: Option<String>,
    #[sqlx(json)]
    pub metadata: JsonValue,
    pub score: Option<f32>,
    pub memory_type: String,
    pub scope: String,
    pub level: String,
    pub importance: f32,
    pub access_count: i64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Junction table for blocks and agents (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlocksAgents {
    pub block_id: String,
    pub block_label: String,
    pub agent_id: String,
}

/// Junction table for tools and agents (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolsAgents {
    pub tool_id: String,
    pub agent_id: String,
}

/// Helper function to generate a new ID with prefix
pub fn generate_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

impl Organization {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("org"),
            name,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        }
    }
}

impl User {
    pub fn new(organization_id: String, name: String, timezone: String) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("user"),
            organization_id,
            name,
            status: "active".to_string(),
            timezone,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

impl Agent {
    pub fn new(organization_id: String, name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("agent"),
            organization_id,
            agent_type: None,
            name,
            description: None,
            system: None,
            topic: None,
            message_ids: None,
            metadata_: None,
            llm_config: None,
            embedding_config: None,
            tool_rules: None,
            mcp_tools: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

impl Message {
    pub fn new(
        organization_id: String,
        user_id: String,
        agent_id: String,
        role: String,
        text: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("message"),
            organization_id,
            user_id,
            agent_id,
            role,
            text,
            content: None,
            model: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            step_id: None,
            otid: None,
            tool_returns: None,
            group_id: None,
            sender_id: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

impl Block {
    pub fn new(
        organization_id: String,
        user_id: String,
        label: String,
        value: String,
        limit: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("block"),
            organization_id,
            user_id,
            template_name: None,
            description: None,
            label,
            is_template: false,
            value,
            limit,
            metadata_: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}
