//! AgentMem REST API Server
//! 
//! Enterprise-grade REST API server for AgentMem memory management platform.
//! Provides HTTP endpoints for all memory operations with authentication,
//! multi-tenancy, and comprehensive monitoring.

pub mod server;
pub mod routes;
pub mod middleware;
pub mod auth;
pub mod models;
pub mod error;
pub mod config;
pub mod telemetry;

pub use server::MemoryServer;
pub use config::ServerConfig;
pub use error::{ServerError, ServerResult};

/// Re-export commonly used types
pub use models::{
    MemoryRequest, MemoryResponse, SearchRequest, SearchResponse,
    BatchRequest, BatchResponse, HealthResponse, MetricsResponse
};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::default();
        let server = MemoryServer::new(config).await;
        assert!(server.is_ok());
    }
}
