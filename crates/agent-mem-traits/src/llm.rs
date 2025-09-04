//! LLM provider trait definitions

use async_trait::async_trait;
use crate::{Result, Message};

/// Model information structure
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub provider: String,
    pub model: String,
    pub max_tokens: u32,
    pub supports_streaming: bool,
    pub supports_functions: bool,
}

/// Core trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a text response from messages
    async fn generate(&self, messages: &[Message]) -> Result<String>;

    /// Generate a streaming response (optional)
    async fn generate_stream(&self, messages: &[Message]) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>>;

    /// Get model information
    fn get_model_info(&self) -> ModelInfo;

    /// Validate the provider configuration
    fn validate_config(&self) -> Result<()>;
}
