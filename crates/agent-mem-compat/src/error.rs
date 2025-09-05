//! Error types for Mem0 compatibility layer

use thiserror::Error;

/// Result type for Mem0 compatibility operations
pub type Result<T> = std::result::Result<T, Mem0Error>;

/// Errors that can occur in the Mem0 compatibility layer
#[derive(Error, Debug)]
pub enum Mem0Error {
    /// Memory not found
    #[error("Memory not found: {id}")]
    MemoryNotFound { id: String },

    /// Invalid user ID
    #[error("Invalid user ID: {user_id}")]
    InvalidUserId { user_id: String },

    /// Invalid memory content
    #[error("Invalid memory content: {reason}")]
    InvalidContent { reason: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Storage error
    #[error("Storage error: {message}")]
    StorageError { message: String },

    /// Embedding error
    #[error("Embedding error: {message}")]
    EmbeddingError { message: String },

    /// LLM error
    #[error("LLM error: {message}")]
    LlmError { message: String },

    /// Network error
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Internal error
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },

    /// Authentication error
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
}

impl From<agent_mem_traits::AgentMemError> for Mem0Error {
    fn from(error: agent_mem_traits::AgentMemError) -> Self {
        match error {
            agent_mem_traits::AgentMemError::NotFound { resource, id } => {
                Mem0Error::MemoryNotFound { id: format!("{}:{}", resource, id) }
            }
            agent_mem_traits::AgentMemError::InvalidInput { field, reason } => {
                Mem0Error::InvalidContent { reason: format!("{}: {}", field, reason) }
            }
            agent_mem_traits::AgentMemError::StorageError { message } => {
                Mem0Error::StorageError { message }
            }
            agent_mem_traits::AgentMemError::EmbeddingError { message } => {
                Mem0Error::EmbeddingError { message }
            }
            agent_mem_traits::AgentMemError::LlmError { message } => {
                Mem0Error::LlmError { message }
            }
            agent_mem_traits::AgentMemError::NetworkError { message } => {
                Mem0Error::NetworkError { message }
            }
            agent_mem_traits::AgentMemError::SerializationError { message } => {
                Mem0Error::SerializationError { message }
            }
            agent_mem_traits::AgentMemError::ConfigurationError { message } => {
                Mem0Error::ConfigError { message }
            }
            agent_mem_traits::AgentMemError::InternalError { message } => {
                Mem0Error::InternalError { message }
            }
            agent_mem_traits::AgentMemError::RateLimitExceeded { message } => {
                Mem0Error::RateLimitExceeded { message }
            }
            agent_mem_traits::AgentMemError::AuthenticationError { message } => {
                Mem0Error::AuthenticationError { message }
            }
            agent_mem_traits::AgentMemError::PermissionDenied { message } => {
                Mem0Error::PermissionDenied { message }
            }
        }
    }
}

impl From<serde_json::Error> for Mem0Error {
    fn from(error: serde_json::Error) -> Self {
        Mem0Error::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for Mem0Error {
    fn from(error: reqwest::Error) -> Self {
        Mem0Error::NetworkError {
            message: error.to_string(),
        }
    }
}
