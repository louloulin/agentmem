//! LLM configuration extensions

use serde::{Deserialize, Serialize};
use agent_mem_traits::LLMConfig as BaseLLMConfig;

/// Extended LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    #[serde(flatten)]
    pub base: BaseLLMConfig,
    
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
    
    /// Rate limiting: requests per minute
    pub rate_limit_rpm: Option<u32>,
    
    /// Enable request/response logging
    pub enable_logging: bool,
    
    /// Custom headers for requests
    pub custom_headers: std::collections::HashMap<String, String>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            base: BaseLLMConfig::default(),
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_rpm: None,
            enable_logging: false,
            custom_headers: std::collections::HashMap::new(),
        }
    }
}

impl LLMConfig {
    pub fn openai() -> Self {
        Self {
            base: BaseLLMConfig {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    pub fn anthropic() -> Self {
        Self {
            base: BaseLLMConfig {
                provider: "anthropic".to_string(),
                model: "claude-3-sonnet-20240229".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    pub fn azure() -> Self {
        Self {
            base: BaseLLMConfig {
                provider: "azure".to_string(),
                model: "gpt-35-turbo".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    pub fn gemini() -> Self {
        Self {
            base: BaseLLMConfig {
                provider: "gemini".to_string(),
                model: "gemini-pro".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    pub fn ollama() -> Self {
        Self {
            base: BaseLLMConfig {
                provider: "ollama".to_string(),
                model: "llama2".to_string(),
                base_url: Some("http://localhost:11434".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.base.api_key = Some(api_key.to_string());
        self
    }
    
    pub fn with_model(mut self, model: &str) -> Self {
        self.base.model = model.to_string();
        self
    }
    
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.base.temperature = Some(temperature);
        self
    }
    
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.base.max_tokens = Some(max_tokens);
        self
    }
    
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
    
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}
