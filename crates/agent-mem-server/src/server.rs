//! Main server implementation

use crate::{
    config::ServerConfig,
    error::{ServerError, ServerResult},
    routes::create_router,
    telemetry::setup_telemetry,
};
use agent_mem_core::MemoryManager;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

/// Main memory server
pub struct MemoryServer {
    config: ServerConfig,
    memory_manager: Arc<MemoryManager>,
    router: Router,
}

impl MemoryServer {
    /// Create a new memory server
    pub async fn new(config: ServerConfig) -> ServerResult<Self> {
        // Setup telemetry
        setup_telemetry(&config)?;
        
        // Create memory manager
        let memory_manager = Arc::new(MemoryManager::new());
        
        // Create router with all routes and middleware
        let router = create_router(memory_manager.clone()).await?;
        
        info!("Memory server initialized successfully");
        
        Ok(Self {
            config,
            memory_manager,
            router,
        })
    }
    
    /// Start the server
    pub async fn start(self) -> ServerResult<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.port));
        let listener = TcpListener::bind(addr).await
            .map_err(|e| ServerError::BindError(e.to_string()))?;
        
        info!("AgentMem server starting on {}", addr);
        info!("API documentation available at http://{}/swagger-ui/", addr);
        info!("Health check endpoint: http://{}/health", addr);
        info!("Metrics endpoint: http://{}/metrics", addr);
        
        // Start the server
        axum::serve(listener, self.router)
            .await
            .map_err(|e| ServerError::ServerError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }
    
    /// Get memory manager reference
    pub fn memory_manager(&self) -> Arc<MemoryManager> {
        self.memory_manager.clone()
    }
    
    /// Graceful shutdown
    pub async fn shutdown(&self) -> ServerResult<()> {
        info!("Shutting down AgentMem server gracefully...");
        // Perform cleanup operations here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_server_creation() {
        let mut config = ServerConfig::default();
        config.enable_logging = false; // Disable logging to avoid telemetry conflicts
        let server = MemoryServer::new(config).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_server_config() {
        let mut config = ServerConfig::default();
        config.enable_logging = false; // Disable logging to avoid telemetry conflicts
        let server = MemoryServer::new(config.clone()).await.unwrap();
        assert_eq!(server.config().port, config.port);
    }
}
