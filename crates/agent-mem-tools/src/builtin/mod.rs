//! Built-in tools
//!
//! This module provides a collection of built-in tools for common operations.

pub mod calculator;
pub mod echo;
pub mod json_parser;
pub mod string_ops;
pub mod time_ops;

pub use calculator::CalculatorTool;
pub use echo::EchoTool;
pub use json_parser::JsonParserTool;
pub use string_ops::StringOpsTool;
pub use time_ops::TimeOpsTool;

use crate::{ToolExecutor, ToolResult};
use std::sync::Arc;

/// Register all built-in tools
pub async fn register_all_builtin_tools(executor: &ToolExecutor) -> ToolResult<()> {
    executor.register_tool(Arc::new(CalculatorTool)).await?;
    executor.register_tool(Arc::new(EchoTool)).await?;
    executor.register_tool(Arc::new(JsonParserTool)).await?;
    executor.register_tool(Arc::new(StringOpsTool)).await?;
    executor.register_tool(Arc::new(TimeOpsTool)).await?;
    Ok(())
}
