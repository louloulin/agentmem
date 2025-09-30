//! Organization management API routes
//!
//! This module provides REST API endpoints for organization management:
//! - Create organization
//! - Get organization details
//! - Update organization
//! - List organization members
//! - Manage organization settings

use crate::error::{ServerError, ServerResult};
use crate::middleware::AuthUser;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// Organization response
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub settings: OrganizationSettings,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Organization settings
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationSettings {
    pub max_users: Option<i32>,
    pub max_agents: Option<i32>,
    pub max_memories: Option<i32>,
    pub retention_days: Option<i32>,
}

/// Create organization request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 2))]
    pub name: String,
    pub description: Option<String>,
}

/// Update organization request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOrganizationRequest {
    #[validate(length(min = 2))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub settings: Option<OrganizationSettings>,
}

/// Organization member response
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationMemberResponse {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
    pub joined_at: i64,
}

/// List organizations query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListOrganizationsQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// Create a new organization
#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created successfully", body = OrganizationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_organization(
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateOrganizationRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {}", e)))?;

    // TODO: Save organization to database
    // For now, return mock data
    let org_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let organization = OrganizationResponse {
        id: org_id,
        name: request.name,
        description: request.description,
        settings: OrganizationSettings {
            max_users: Some(100),
            max_agents: Some(50),
            max_memories: Some(10000),
            retention_days: Some(365),
        },
        created_at: now,
        updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(organization)))
}

/// Get organization by ID
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Organization details", body = OrganizationResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_organization(
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    // Check if user belongs to the organization
    if auth_user.org_id != org_id && !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden(
            "Access to this organization is forbidden".to_string(),
        ));
    }

    // TODO: Fetch organization from database
    // For now, return mock data
    let organization = OrganizationResponse {
        id: org_id,
        name: "Test Organization".to_string(),
        description: Some("A test organization".to_string()),
        settings: OrganizationSettings {
            max_users: Some(100),
            max_agents: Some(50),
            max_memories: Some(10000),
            retention_days: Some(365),
        },
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    Ok(Json(organization))
}

/// Update organization
#[utoipa::path(
    put,
    path = "/api/v1/organizations/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID")
    ),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated successfully", body = OrganizationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_organization(
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
    Json(request): Json<UpdateOrganizationRequest>,
) -> ServerResult<impl IntoResponse> {
    // Check if user belongs to the organization and has admin role
    if auth_user.org_id != org_id || !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden(
            "Admin role required for this organization".to_string(),
        ));
    }

    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {}", e)))?;

    // TODO: Update organization in database
    // For now, return mock data
    let organization = OrganizationResponse {
        id: org_id,
        name: request.name.unwrap_or_else(|| "Test Organization".to_string()),
        description: request.description,
        settings: request.settings.unwrap_or(OrganizationSettings {
            max_users: Some(100),
            max_agents: Some(50),
            max_memories: Some(10000),
            retention_days: Some(365),
        }),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    Ok(Json(organization))
}

/// List organization members
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/members",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
        ListOrganizationsQuery
    ),
    responses(
        (status = 200, description = "List of organization members", body = Vec<OrganizationMemberResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_organization_members(
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
    Query(query): Query<ListOrganizationsQuery>,
) -> ServerResult<impl IntoResponse> {
    // Check if user belongs to the organization
    if auth_user.org_id != org_id && !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden(
            "Access to this organization is forbidden".to_string(),
        ));
    }

    // TODO: Fetch members from database
    // For now, return mock data
    let members = vec![
        OrganizationMemberResponse {
            user_id: auth_user.user_id.clone(),
            email: "user@example.com".to_string(),
            name: "Test User".to_string(),
            roles: auth_user.roles.clone(),
            joined_at: chrono::Utc::now().timestamp(),
        },
    ];

    Ok(Json(members))
}

/// Delete organization (admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID")
    ),
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_organization(
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    // Check if user is admin
    if !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden("Admin role required".to_string()));
    }

    // TODO: Delete organization from database (soft delete)
    // For now, just return success

    Ok(StatusCode::NO_CONTENT)
}

