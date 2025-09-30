//! User management API routes
//!
//! This module provides REST API endpoints for user management:
//! - User registration
//! - User login
//! - User profile management
//! - Password management

use crate::auth::PasswordService;
use crate::error::{ServerError, ServerResult};
use crate::middleware::AuthUser;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// User registration request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 2))]
    pub name: String,
    pub organization_id: String,
}

/// User login request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

/// User login response
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

/// User response
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub organization_id: String,
    pub roles: Vec<String>,
    pub created_at: i64,
}

/// Update user request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 2))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}

/// Change password request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/v1/users/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "User already exists")
    ),
    tag = "users"
)]
pub async fn register_user(
    Json(request): Json<RegisterRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {}", e)))?;

    // TODO: Check if user already exists in database

    // Hash password
    let password_hash = PasswordService::hash_password(&request.password)?;

    // TODO: Save user to database
    // For now, return a mock response
    let user_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let user = UserResponse {
        id: user_id,
        email: request.email,
        name: request.name,
        organization_id: request.organization_id,
        roles: vec!["user".to_string()],
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

/// Login user
#[utoipa::path(
    post,
    path = "/api/v1/users/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "users"
)]
pub async fn login_user(
    Json(request): Json<LoginRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {}", e)))?;

    // TODO: Fetch user from database and verify password
    // For now, use mock data
    let user_id = uuid::Uuid::new_v4().to_string();
    let org_id = "org123".to_string();
    let roles = vec!["user".to_string()];

    // Generate JWT token
    // TODO: Get JWT secret from config
    let auth_service = crate::auth::AuthService::new("default-secret-key-change-in-production");
    let token = auth_service.generate_token(&user_id, org_id.clone(), roles.clone(), None)?;

    let response = LoginResponse {
        token,
        user: UserResponse {
            id: user_id,
            email: request.email,
            name: "Test User".to_string(),
            organization_id: org_id,
            roles,
            created_at: chrono::Utc::now().timestamp(),
        },
    };

    Ok(Json(response))
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "User profile", body = UserResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    Extension(auth_user): Extension<AuthUser>,
) -> ServerResult<impl IntoResponse> {
    // TODO: Fetch user from database
    // For now, return mock data
    let user = UserResponse {
        id: auth_user.user_id.clone(),
        email: "user@example.com".to_string(),
        name: "Test User".to_string(),
        organization_id: auth_user.org_id.clone(),
        roles: auth_user.roles.clone(),
        created_at: chrono::Utc::now().timestamp(),
    };

    Ok(Json(user))
}

/// Update user profile
#[utoipa::path(
    put,
    path = "/api/v1/users/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 400, description = "Invalid request")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_current_user(
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateUserRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {}", e)))?;

    // TODO: Update user in database
    // For now, return mock data
    let user = UserResponse {
        id: auth_user.user_id.clone(),
        email: request.email.unwrap_or_else(|| "user@example.com".to_string()),
        name: request.name.unwrap_or_else(|| "Test User".to_string()),
        organization_id: auth_user.org_id.clone(),
        roles: auth_user.roles.clone(),
        created_at: chrono::Utc::now().timestamp(),
    };

    Ok(Json(user))
}

/// Change user password
#[utoipa::path(
    post,
    path = "/api/v1/users/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 401, description = "Not authenticated or invalid current password"),
        (status = 400, description = "Invalid request")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_password(
    Extension(_auth_user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {}", e)))?;

    // TODO: Verify current password and update in database
    // For now, just hash the new password
    let _new_password_hash = PasswordService::hash_password(&request.new_password)?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Password changed successfully"
        })),
    ))
}

/// Get user by ID (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User profile", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_by_id(
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    // Check if user is admin
    if !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden("Admin role required".to_string()));
    }

    // TODO: Fetch user from database
    // For now, return mock data
    let user = UserResponse {
        id: user_id,
        email: "user@example.com".to_string(),
        name: "Test User".to_string(),
        organization_id: auth_user.org_id.clone(),
        roles: vec!["user".to_string()],
        created_at: chrono::Utc::now().timestamp(),
    };

    Ok(Json(user))
}

