//! LLM provider trait definitions

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use crate::{Result, Message, LLMConfig};

/// Core trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a text response from messages
    async fn generate_response(&self, messages: &[Message], config: &LLMConfig) -> Result<String>;
    
    /// Generate a structured response (JSON) from messages
    async fn generate_structured<T>(&self, messages: &[Message]) -> Result<T> 
    where T: DeserializeOwned + Send;
    
    /// Check if the provider supports tool calling
    fn supports_tools(&self) -> bool;
    
    /// Check if the provider supports vision/image inputs
    fn supports_vision(&self) -> bool;
    
    /// Get the provider name
    fn provider_name(&self) -> &str;
    
    /// Get the model name being used
    fn model_name(&self) -> &str;
    
    /// Check if the provider is available/healthy
    async fn health_check(&self) -> Result<bool>;
}
