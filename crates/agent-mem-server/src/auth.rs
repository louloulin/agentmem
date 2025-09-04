//! Authentication and authorization

use crate::error::{ServerError, ServerResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Organization ID
    pub org_id: Option<String>,
    /// Project ID
    pub project_id: Option<String>,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
}

/// Authentication service
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    /// Generate a JWT token
    pub fn generate_token(
        &self,
        user_id: &str,
        org_id: Option<String>,
        project_id: Option<String>,
    ) -> ServerResult<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        let claims = Claims {
            sub: user_id.to_string(),
            org_id,
            project_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ServerError::Unauthorized(format!("Token generation failed: {}", e)))
    }

    /// Validate a JWT token
    pub fn validate_token(&self, token: &str) -> ServerResult<Claims> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| ServerError::Unauthorized(format!("Token validation failed: {}", e)))
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> ServerResult<&str> {
        if auth_header.starts_with("Bearer ") {
            Ok(&auth_header[7..])
        } else {
            Err(ServerError::Unauthorized(
                "Invalid authorization header format".to_string(),
            ))
        }
    }
}

/// User context extracted from JWT
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

impl From<Claims> for UserContext {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            org_id: claims.org_id,
            project_id: claims.project_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_validation() {
        let auth_service = AuthService::new("test-secret-key-that-is-long-enough");

        let token = auth_service
            .generate_token("user123", Some("org456".to_string()), None)
            .unwrap();

        let claims = auth_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.org_id, Some("org456".to_string()));
    }

    #[test]
    fn test_extract_token_from_header() {
        let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = AuthService::extract_token_from_header(header).unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_invalid_header_format() {
        let header = "Invalid header";
        let result = AuthService::extract_token_from_header(header);
        assert!(result.is_err());
    }
}
