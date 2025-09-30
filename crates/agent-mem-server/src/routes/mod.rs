//! HTTP routes for the AgentMem API

pub mod docs;
pub mod health;
pub mod memory;
pub mod metrics;
pub mod organizations;
pub mod users;

use crate::error::ServerResult;
use crate::routes::memory::MemoryManager;
use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
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
        .route(
            "/api/v1/memories/:id/history",
            get(memory::get_memory_history),
        )
        // Batch operations
        .route("/api/v1/memories/batch", post(memory::batch_add_memories))
        .route(
            "/api/v1/memories/batch/delete",
            post(memory::batch_delete_memories),
        )
        // User management routes
        .route("/api/v1/users/register", post(users::register_user))
        .route("/api/v1/users/login", post(users::login_user))
        .route("/api/v1/users/me", get(users::get_current_user))
        .route("/api/v1/users/me", put(users::update_current_user))
        .route("/api/v1/users/me/password", post(users::change_password))
        .route("/api/v1/users/:user_id", get(users::get_user_by_id))
        // Organization management routes
        .route("/api/v1/organizations", post(organizations::create_organization))
        .route("/api/v1/organizations/:org_id", get(organizations::get_organization))
        .route("/api/v1/organizations/:org_id", put(organizations::update_organization))
        .route("/api/v1/organizations/:org_id", delete(organizations::delete_organization))
        .route("/api/v1/organizations/:org_id/members", get(organizations::list_organization_members))
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
        users::register_user,
        users::login_user,
        users::get_current_user,
        users::update_current_user,
        users::change_password,
        users::get_user_by_id,
        organizations::create_organization,
        organizations::get_organization,
        organizations::update_organization,
        organizations::delete_organization,
        organizations::list_organization_members,
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
            users::RegisterRequest,
            users::LoginRequest,
            users::LoginResponse,
            users::UserResponse,
            users::UpdateUserRequest,
            users::ChangePasswordRequest,
            organizations::OrganizationResponse,
            organizations::OrganizationSettings,
            organizations::CreateOrganizationRequest,
            organizations::UpdateOrganizationRequest,
            organizations::OrganizationMemberResponse,
        )
    ),
    tags(
        (name = "memory", description = "Memory management operations"),
        (name = "batch", description = "Batch operations"),
        (name = "users", description = "User management operations"),
        (name = "organizations", description = "Organization management operations"),
        (name = "health", description = "Health and monitoring"),
    ),
    info(
        title = "AgentMem API",
        version = "2.0.0",
        description = "Enterprise-grade memory management API for AI agents with authentication and multi-tenancy",
        contact(
            name = "AgentMem Team",
            url = "https://github.com/agentmem/agentmem",
        ),
        license(
            name = "MIT OR Apache-2.0",
            url = "https://opensource.org/licenses/MIT",
        ),
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

/// Security addon for OpenAPI
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::memory::MemoryManager;
    use tower_test::mock;

    #[tokio::test]
    async fn test_router_creation() {
        let memory_manager = Arc::new(MemoryManager::new());
        let router = create_router(memory_manager).await;
        assert!(router.is_ok());
    }
}
