//! HTTP routes for the AgentMem API

pub mod memory;
pub mod health;
pub mod metrics;
pub mod docs;

use crate::error::ServerResult;
use agent_mem_core::MemoryManager;
use axum::{
    routing::{get, post, put, delete},
    Router, Extension,
};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create the main router with all routes
pub async fn create_router(memory_manager: Arc<MemoryManager>) -> ServerResult<Router> {
    let app = Router::new()
        // Memory management routes
        .route("/api/v1/memories", post(memory::add_memory))
        .route("/api/v1/memories/:id", get(memory::get_memory))
        .route("/api/v1/memories/:id", put(memory::update_memory))
        .route("/api/v1/memories/:id", delete(memory::delete_memory))
        .route("/api/v1/memories/search", post(memory::search_memories))
        .route("/api/v1/memories/:id/history", get(memory::get_memory_history))
        
        // Batch operations
        .route("/api/v1/memories/batch", post(memory::batch_add_memories))
        .route("/api/v1/memories/batch/delete", post(memory::batch_delete_memories))
        
        // Health and monitoring
        .route("/health", get(health::health_check))
        .route("/metrics", get(metrics::get_metrics))
        
        // OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        
        // Add shared state
        .layer(Extension(memory_manager))
        
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    Ok(app)
}

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        memory::add_memory,
        memory::get_memory,
        memory::update_memory,
        memory::delete_memory,
        memory::search_memories,
        memory::get_memory_history,
        memory::batch_add_memories,
        memory::batch_delete_memories,
        health::health_check,
        metrics::get_metrics,
    ),
    components(
        schemas(
            crate::models::MemoryRequest,
            crate::models::MemoryResponse,
            crate::models::SearchRequest,
            crate::models::SearchResponse,
            crate::models::BatchRequest,
            crate::models::BatchResponse,
            crate::models::HealthResponse,
            crate::models::MetricsResponse,
        )
    ),
    tags(
        (name = "memory", description = "Memory management operations"),
        (name = "batch", description = "Batch operations"),
        (name = "health", description = "Health and monitoring"),
    ),
    info(
        title = "AgentMem API",
        version = "2.0.0",
        description = "Enterprise-grade memory management API for AI agents",
        contact(
            name = "AgentMem Team",
            url = "https://github.com/agentmem/agentmem",
        ),
        license(
            name = "MIT OR Apache-2.0",
            url = "https://opensource.org/licenses/MIT",
        ),
    ),
)]
struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_core::MemoryManager;
    use tower_test::mock;

    #[tokio::test]
    async fn test_router_creation() {
        let memory_manager = Arc::new(MemoryManager::new());
        let router = create_router(memory_manager).await;
        assert!(router.is_ok());
    }
}
