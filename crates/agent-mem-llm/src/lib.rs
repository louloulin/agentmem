//! # Agent Memory LLM Integration
//!
//! LLM集成模块，为AgentMem记忆平台提供多种LLM提供商支持。
//!
//! 本模块提供：
//! - LLM工厂模式，支持多种提供商
//! - 统一的LLM接口抽象
//! - 提示词管理系统
//! - 错误处理和重试机制
//! - 特性门控支持

pub mod client;
pub mod factory;
pub mod prompts;
pub mod providers;

pub use client::LLMClient;
pub use factory::LLMFactory;

// 重新导出常用类型
pub use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};
