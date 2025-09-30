//! Sandboxed execution environment for tools
//!
//! This module provides safe tool execution with timeout and resource limits,
//! inspired by MIRIX's tool_execution_sandbox.py.

use crate::error::{ToolError, ToolResult};
use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory usage (bytes)
    pub max_memory: usize,
    /// Default timeout duration
    pub default_timeout: Duration,
    /// Enable resource monitoring
    pub enable_monitoring: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: 512 * 1024 * 1024, // 512MB
            default_timeout: Duration::from_secs(30),
            enable_monitoring: true,
        }
    }
}

/// Sandbox manager for safe tool execution
#[derive(Default)]
pub struct SandboxManager {
    config: SandboxConfig,
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Execute a function in the sandbox with timeout control
    pub async fn execute<F, T>(&self, func: F, timeout_duration: Duration) -> ToolResult<T>
    where
        F: Future<Output = ToolResult<T>> + Send,
        T: Send,
    {
        debug!("Executing in sandbox with timeout: {:?}", timeout_duration);

        // Check resource usage before execution
        if self.config.enable_monitoring {
            self.check_resource_usage()?;
        }

        // Execute with timeout
        let result = timeout(timeout_duration, func).await;

        match result {
            Ok(Ok(value)) => {
                debug!("Sandbox execution completed successfully");
                Ok(value)
            }
            Ok(Err(e)) => {
                warn!("Sandbox execution failed: {}", e);
                Err(e)
            }
            Err(_) => {
                warn!("Sandbox execution timeout");
                Err(ToolError::Timeout)
            }
        }
    }

    /// Execute with default timeout
    pub async fn execute_default<F, T>(&self, func: F) -> ToolResult<T>
    where
        F: Future<Output = ToolResult<T>> + Send,
        T: Send,
    {
        self.execute(func, self.config.default_timeout).await
    }

    /// Check resource usage
    fn check_resource_usage(&self) -> ToolResult<()> {
        let memory_usage = self.get_memory_usage();

        if memory_usage > self.config.max_memory {
            warn!(
                "Memory limit exceeded: {} > {}",
                memory_usage, self.config.max_memory
            );
            return Err(ToolError::ResourceLimitExceeded(format!(
                "Memory usage {} exceeds limit {}",
                memory_usage, self.config.max_memory
            )));
        }

        Ok(())
    }

    /// Get current memory usage (platform-specific)
    fn get_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<usize>() {
                                return kb_val * 1024;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, we can use task_info
            // For now, return 0 as a placeholder
            // In production, use proper system calls
            0
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, use GetProcessMemoryInfo
            // For now, return 0 as a placeholder
            0
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            0
        }
    }

    /// Get sandbox statistics
    pub fn get_stats(&self) -> SandboxStats {
        SandboxStats {
            max_memory: self.config.max_memory,
            current_memory: self.get_memory_usage(),
            default_timeout: self.config.default_timeout,
        }
    }
}

/// Sandbox statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxStats {
    /// Maximum memory limit
    pub max_memory: usize,
    /// Current memory usage
    pub current_memory: usize,
    /// Default timeout
    pub default_timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_sandbox_success() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute(async { Ok::<i32, ToolError>(42) }, Duration::from_secs(1))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_sandbox_timeout() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute(
                async {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    Ok::<i32, ToolError>(42)
                },
                Duration::from_millis(100),
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::Timeout));
    }

    #[tokio::test]
    async fn test_sandbox_error() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute(
                async {
                    Err::<i32, ToolError>(ToolError::ExecutionFailed("test error".to_string()))
                },
                Duration::from_secs(1),
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::ExecutionFailed(_)));
    }

    #[test]
    fn test_sandbox_config() {
        let config = SandboxConfig {
            max_memory: 1024 * 1024 * 1024, // 1GB
            default_timeout: Duration::from_secs(60),
            enable_monitoring: true,
        };

        let sandbox = SandboxManager::new(config.clone());
        let stats = sandbox.get_stats();

        assert_eq!(stats.max_memory, config.max_memory);
        assert_eq!(stats.default_timeout, config.default_timeout);
    }

    #[tokio::test]
    async fn test_execute_default() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute_default(async { Ok::<String, ToolError>("success".to_string()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
