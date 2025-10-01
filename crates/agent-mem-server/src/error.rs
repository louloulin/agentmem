//! Error handling for the server

use crate::models::ErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use thiserror::Error;

/// Server error types
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Memory operation failed: {0}")]
    MemoryError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Access forbidden: {0}")]
    Forbidden(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Server binding failed: {0}")]
    BindError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Telemetry setup failed: {0}")]
    TelemetryError(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Server result type
pub type ServerResult<T> = Result<T, ServerError>;

impl ServerError {
    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        ServerError::NotFound(msg.into())
    }

    /// Create a bad request error
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ServerError::BadRequest(msg.into())
    }

    /// Create an unauthorized error
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        ServerError::Unauthorized(msg.into())
    }

    /// Create a forbidden error
    pub fn forbidden(msg: impl Into<String>) -> Self {
        ServerError::Forbidden(msg.into())
    }

    /// Create an internal error
    pub fn internal_error(msg: impl Into<String>) -> Self {
        ServerError::Internal(msg.into())
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ServerError::MemoryError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "MEMORY_ERROR", msg)
            }
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            ServerError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            ServerError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
            ServerError::QuotaExceeded(msg) => {
                (StatusCode::TOO_MANY_REQUESTS, "QUOTA_EXCEEDED", msg)
            }
            ServerError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            ServerError::BindError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "BIND_ERROR", msg),
            ServerError::ServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "SERVER_ERROR", msg)
            }
            ServerError::ConfigError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", msg)
            }
            ServerError::TelemetryError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "TELEMETRY_ERROR", msg)
            }
            ServerError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
        };

        let error_response = ErrorResponse {
            code: code.to_string(),
            message,
            details: None,
            timestamp: Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<agent_mem_traits::AgentMemError> for ServerError {
    fn from(err: agent_mem_traits::AgentMemError) -> Self {
        ServerError::MemoryError(err.to_string())
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::BadRequest(format!("JSON parsing error: {err}"))
    }
}

impl From<validator::ValidationErrors> for ServerError {
    fn from(err: validator::ValidationErrors) -> Self {
        ServerError::ValidationError(format!("Validation failed: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_error_conversion() {
        let error = ServerError::NotFound("Test not found".to_string());
        let response = error.into_response();

        // We can't easily test the exact response content here,
        // but we can verify the error type conversion works
        assert!(matches!(response.status(), StatusCode::NOT_FOUND));
    }

    #[test]
    fn test_memory_error_conversion() {
        let memory_error = agent_mem_traits::AgentMemError::memory_error("test");
        let server_error: ServerError = memory_error.into();

        assert!(matches!(server_error, ServerError::MemoryError(_)));
    }
}
