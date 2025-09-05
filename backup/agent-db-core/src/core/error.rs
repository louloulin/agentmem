// 简化的错误处理模块
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentDbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// From<serde_json::Error> 已通过 #[from] 自动实现

impl From<uuid::Error> for AgentDbError {
    fn from(err: uuid::Error) -> Self {
        AgentDbError::Internal(format!("UUID error: {}", err))
    }
}

// C FFI 兼容的错误码
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum CAgentDbErrorCode {
    Success = 0,
    InvalidParam = -1,
    NotFound = -2,
    IoError = -3,
    MemoryError = -4,
    InternalError = -5,
    SerializationError = -6,
    NetworkError = -7,
    AuthenticationError = -8,
    PermissionDenied = -9,
}

impl From<&AgentDbError> for CAgentDbErrorCode {
    fn from(err: &AgentDbError) -> Self {
        match err {
            AgentDbError::Io(_) => CAgentDbErrorCode::IoError,
            AgentDbError::Serialization(_) => CAgentDbErrorCode::SerializationError,
            AgentDbError::Serde(_) => CAgentDbErrorCode::SerializationError,
            AgentDbError::InvalidArgument(_) => CAgentDbErrorCode::InvalidParam,
            AgentDbError::NotFound(_) => CAgentDbErrorCode::NotFound,
            AgentDbError::Internal(_) => CAgentDbErrorCode::InternalError,
            AgentDbError::Memory(_) => CAgentDbErrorCode::MemoryError,
            AgentDbError::Network(_) => CAgentDbErrorCode::NetworkError,
            AgentDbError::Authentication(_) => CAgentDbErrorCode::AuthenticationError,
            AgentDbError::PermissionDenied(_) => CAgentDbErrorCode::PermissionDenied,
        }
    }
}
