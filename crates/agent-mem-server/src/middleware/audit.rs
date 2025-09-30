//! Audit logging middleware
//!
//! This module provides middleware for audit logging:
//! - Request logging
//! - User action tracking
//! - Security event logging

use crate::middleware::AuthUser;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{info, warn};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub timestamp: i64,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub error: Option<String>,
}

/// Audit logging middleware
pub async fn audit_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let timestamp = Utc::now().timestamp();

    // Extract request information
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Extract user information if authenticated
    let auth_user = request.extensions().get::<AuthUser>().cloned();
    let user_id = auth_user.as_ref().map(|u| u.user_id.clone());
    let organization_id = auth_user.as_ref().map(|u| u.org_id.clone());

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration_ms = start.elapsed().as_millis() as u64;
    let status_code = response.status().as_u16();

    // Determine action and resource type from path
    let (action, resource_type, resource_id) = parse_path(&path, &method);

    // Create audit log entry
    let audit_log = AuditLog {
        timestamp,
        user_id,
        organization_id,
        action,
        resource_type,
        resource_id,
        method,
        path: path.clone(),
        status_code,
        duration_ms,
        ip_address: None, // TODO: Extract from request
        user_agent,
        error: if status_code >= 400 {
            Some(format!("HTTP {}", status_code))
        } else {
            None
        },
    };

    // Log audit entry
    log_audit_entry(&audit_log);

    response
}

/// Parse path to extract action, resource type, and resource ID
fn parse_path(path: &str, method: &str) -> (String, String, Option<String>) {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() < 3 {
        return (
            method.to_lowercase(),
            "unknown".to_string(),
            None,
        );
    }

    // Expected format: /api/v1/{resource_type}/{resource_id}
    let resource_type = if parts.len() >= 3 {
        parts[2].to_string()
    } else {
        "unknown".to_string()
    };

    let resource_id = if parts.len() >= 4 && !parts[3].starts_with('?') {
        Some(parts[3].to_string())
    } else {
        None
    };

    let action = match method {
        "GET" => if resource_id.is_some() { "read" } else { "list" },
        "POST" => "create",
        "PUT" | "PATCH" => "update",
        "DELETE" => "delete",
        _ => "unknown",
    };

    (action.to_string(), resource_type, resource_id)
}

/// Log audit entry
fn log_audit_entry(audit_log: &AuditLog) {
    let user_info = if let Some(user_id) = &audit_log.user_id {
        format!("user={}", user_id)
    } else {
        "user=anonymous".to_string()
    };

    let org_info = if let Some(org_id) = &audit_log.organization_id {
        format!("org={}", org_id)
    } else {
        String::new()
    };

    let resource_info = if let Some(resource_id) = &audit_log.resource_id {
        format!("{}:{}", audit_log.resource_type, resource_id)
    } else {
        audit_log.resource_type.clone()
    };

    if audit_log.status_code >= 400 {
        warn!(
            "AUDIT: {} {} {} {} {} status={} duration={}ms error={:?}",
            user_info,
            org_info,
            audit_log.action,
            resource_info,
            audit_log.method,
            audit_log.status_code,
            audit_log.duration_ms,
            audit_log.error
        );
    } else {
        info!(
            "AUDIT: {} {} {} {} {} status={} duration={}ms",
            user_info,
            org_info,
            audit_log.action,
            resource_info,
            audit_log.method,
            audit_log.status_code,
            audit_log.duration_ms
        );
    }

    // TODO: Store audit log in database for long-term retention
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    LoginSuccess {
        user_id: String,
        ip_address: Option<String>,
    },
    LoginFailure {
        email: String,
        ip_address: Option<String>,
        reason: String,
    },
    PasswordChanged {
        user_id: String,
    },
    ApiKeyCreated {
        user_id: String,
        key_id: String,
    },
    ApiKeyRevoked {
        user_id: String,
        key_id: String,
    },
    UnauthorizedAccess {
        path: String,
        ip_address: Option<String>,
    },
    PermissionDenied {
        user_id: String,
        resource: String,
        action: String,
    },
}

/// Log security event
pub fn log_security_event(event: SecurityEvent) {
    match event {
        SecurityEvent::LoginSuccess { user_id, ip_address } => {
            info!(
                "SECURITY: Login successful - user={} ip={:?}",
                user_id, ip_address
            );
        }
        SecurityEvent::LoginFailure { email, ip_address, reason } => {
            warn!(
                "SECURITY: Login failed - email={} ip={:?} reason={}",
                email, ip_address, reason
            );
        }
        SecurityEvent::PasswordChanged { user_id } => {
            info!("SECURITY: Password changed - user={}", user_id);
        }
        SecurityEvent::ApiKeyCreated { user_id, key_id } => {
            info!(
                "SECURITY: API key created - user={} key_id={}",
                user_id, key_id
            );
        }
        SecurityEvent::ApiKeyRevoked { user_id, key_id } => {
            info!(
                "SECURITY: API key revoked - user={} key_id={}",
                user_id, key_id
            );
        }
        SecurityEvent::UnauthorizedAccess { path, ip_address } => {
            warn!(
                "SECURITY: Unauthorized access attempt - path={} ip={:?}",
                path, ip_address
            );
        }
        SecurityEvent::PermissionDenied { user_id, resource, action } => {
            warn!(
                "SECURITY: Permission denied - user={} resource={} action={}",
                user_id, resource, action
            );
        }
    }

    // TODO: Store security events in database for analysis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let (action, resource_type, resource_id) = parse_path("/api/v1/memories/123", "GET");
        assert_eq!(action, "read");
        assert_eq!(resource_type, "memories");
        assert_eq!(resource_id, Some("123".to_string()));

        let (action, resource_type, resource_id) = parse_path("/api/v1/users", "GET");
        assert_eq!(action, "list");
        assert_eq!(resource_type, "users");
        assert_eq!(resource_id, None);

        let (action, resource_type, resource_id) = parse_path("/api/v1/agents", "POST");
        assert_eq!(action, "create");
        assert_eq!(resource_type, "agents");
        assert_eq!(resource_id, None);
    }
}

