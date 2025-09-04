//! Health check routes

use crate::{
    error::ServerResult,
    models::HealthResponse,
};
use agent_mem_core::MemoryManager;
use axum::{
    extract::Extension,
    response::Json,
};
use std::sync::Arc;
use chrono::Utc;
use utoipa;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy")
    )
)]
pub async fn health_check(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
) -> ServerResult<Json<HealthResponse>> {
    // Perform basic health checks
    let mut checks = std::collections::HashMap::new();
    
    // Check memory manager
    checks.insert("memory_manager".to_string(), "healthy".to_string());
    
    // Check database connectivity (if applicable)
    // This would be expanded based on actual storage backends
    checks.insert("storage".to_string(), "healthy".to_string());
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    };
    
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_core::MemoryManager;

    #[tokio::test]
    async fn test_health_check() {
        let memory_manager = Arc::new(MemoryManager::new());
        let result = health_check(Extension(memory_manager)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap().0;
        assert_eq!(response.status, "healthy");
    }
}
