//! Server configuration

use serde::{Deserialize, Serialize};
use std::env;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,
    /// Server host
    pub host: String,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable authentication
    pub enable_auth: bool,
    /// JWT secret key
    pub jwt_secret: String,
    /// Enable metrics
    pub enable_metrics: bool,
    /// Enable OpenAPI documentation
    pub enable_docs: bool,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Max request body size in bytes
    pub max_body_size: usize,
    /// Enable request logging
    pub enable_logging: bool,
    /// Log level
    pub log_level: String,
    /// Multi-tenant mode
    pub multi_tenant: bool,
    /// Rate limiting
    pub rate_limit_requests_per_minute: u32,
    /// Database URL
    pub database_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: env::var("AGENT_MEM_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            host: env::var("AGENT_MEM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            enable_cors: env::var("AGENT_MEM_ENABLE_CORS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_auth: env::var("AGENT_MEM_ENABLE_AUTH")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            jwt_secret: env::var("AGENT_MEM_JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-change-in-production".to_string()),
            enable_metrics: env::var("AGENT_MEM_ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_docs: env::var("AGENT_MEM_ENABLE_DOCS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            request_timeout: env::var("AGENT_MEM_REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_body_size: env::var("AGENT_MEM_MAX_BODY_SIZE")
                .unwrap_or_else(|_| "1048576".to_string()) // 1MB
                .parse()
                .unwrap_or(1048576),
            enable_logging: env::var("AGENT_MEM_ENABLE_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            log_level: env::var("AGENT_MEM_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            multi_tenant: env::var("AGENT_MEM_MULTI_TENANT")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            rate_limit_requests_per_minute: env::var("AGENT_MEM_RATE_LIMIT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://agentmem:password@localhost:5432/agentmem".to_string()),
        }
    }
}

impl ServerConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }

        if self.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters".to_string());
        }

        if self.request_timeout == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }

        if self.max_body_size == 0 {
            return Err("Max body size must be greater than 0".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.enable_cors);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ServerConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());
    }
}
