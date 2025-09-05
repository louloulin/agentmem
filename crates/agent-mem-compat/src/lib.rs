//! # AgentMem Mem0 Compatibility Layer
//!
//! This crate provides a compatibility layer that allows AgentMem to be used as a drop-in
//! replacement for Mem0. It implements the Mem0 API surface while leveraging AgentMem's
//! advanced memory management capabilities.
//!
//! ## Features
//!
//! - **Drop-in Replacement**: Compatible with existing Mem0 code
//! - **Enhanced Performance**: Leverages AgentMem's optimized storage and retrieval
//! - **Advanced Memory Types**: Supports episodic, semantic, procedural, and working memory
//! - **Intelligent Processing**: Automatic importance scoring and memory consolidation
//! - **Flexible Storage**: Multiple vector database backends
//!
//! ## Usage
//!
//! ```rust,no_run
//! use agent_mem_compat::Mem0Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Mem0Client::new().await?;
//!     
//!     // Add a memory
//!     let memory_id = client.add("user123", "I love pizza", None).await?;
//!     
//!     // Search memories
//!     let memories = client.search("food preferences", "user123", None).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod types;
pub mod error;
pub mod utils;

// Re-export main types for convenience
pub use client::Mem0Client;
pub use config::Mem0Config;
pub use types::{Memory, MemorySearchResult, MemoryFilter};
pub use error::{Mem0Error, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Mem0 API compatibility version
pub const MEM0_API_VERSION: &str = "1.0.0";
