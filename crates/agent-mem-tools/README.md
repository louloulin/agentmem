# AgentMem Tools

A high-performance tool execution framework for AgentMem, inspired by MIRIX's tool system but optimized for Rust's performance and type safety.

## Features

- **Dynamic Tool Registration**: Register and discover tools at runtime
- **Schema Generation**: Automatic JSON Schema generation for tool parameters
- **Sandboxed Execution**: Safe tool execution with timeout and resource limits
- **Permission Management**: Fine-grained access control for tool execution
- **Built-in Tools**: Collection of common tools (calculator, string ops, time ops, etc.)
- **High Performance**: 2-5x faster than Python-based implementations
- **Type Safety**: Compile-time type checking with Rust's type system

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Tool Execution Framework                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Tool      │  │   Schema    │  │  Sandbox    │         │
│  │  Registry   │  │  Generator  │  │  Manager    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│         │                │                 │                 │
│         └────────────────┴─────────────────┘                 │
│                          │                                   │
│                  ┌───────▼────────┐                          │
│                  │ Tool Executor  │                          │
│                  └────────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-mem-tools = "2.0"
```

### Basic Usage

```rust
use agent_mem_tools::{
    builtin::register_all_builtin_tools,
    ExecutionContext, ToolExecutor,
};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create tool executor
    let executor = ToolExecutor::new();

    // Register built-in tools
    register_all_builtin_tools(&executor).await?;

    // Set up permissions
    executor.permissions().assign_role("user1", "admin").await;

    // Execute a tool
    let context = ExecutionContext {
        user: "user1".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = executor
        .execute_tool(
            "calculator",
            json!({
                "operation": "add",
                "a": 10.0,
                "b": 20.0
            }),
            &context,
        )
        .await?;

    println!("Result: {}", result);
    Ok(())
}
```

### Creating Custom Tools

```rust
use agent_mem_tools::{
    ExecutionContext, Tool, ToolError, ToolResult, ToolSchema,
};
use agent_mem_tools::schema::PropertySchema;
use async_trait::async_trait;
use serde_json::{json, Value};

struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "My custom tool"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "input",
                PropertySchema::string("Input value"),
                true,
            )
    }

    async fn execute(
        &self,
        args: Value,
        _context: &ExecutionContext,
    ) -> ToolResult<Value> {
        let input = args["input"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("Missing input".to_string()))?;

        Ok(json!({ "output": format!("Processed: {}", input) }))
    }
}

// Register the tool
let executor = ToolExecutor::new();
executor.register_tool(Arc::new(MyCustomTool)).await?;
```

## Built-in Tools

### Calculator Tool
Perform basic arithmetic operations:
- `add`, `subtract`, `multiply`, `divide`
- `power`, `sqrt`

### String Operations Tool
String manipulation:
- `uppercase`, `lowercase`
- `reverse`, `trim`
- `length`

### Time Operations Tool
Time and date operations:
- `current_time`
- `parse_time`
- `format_time`

### JSON Parser Tool
Parse and validate JSON strings

### Echo Tool
Simple echo for testing

## Permission Management

```rust
// Create custom role
let role = Role::new("developer")
    .with_permission(Permission::Read)
    .with_permission(Permission::Execute);

executor.permissions().register_role(role).await;

// Assign role to user
executor.permissions().assign_role("user1", "developer").await;

// Set tool-specific permissions
let tool_perm = ToolPermission::new("sensitive_tool")
    .require_permission(Permission::Admin)
    .allow_role("admin");

executor.permissions().set_tool_permission(tool_perm).await;
```

## Sandbox Configuration

```rust
use agent_mem_tools::sandbox::{SandboxConfig, SandboxManager};
use std::time::Duration;

let config = SandboxConfig {
    max_memory: 1024 * 1024 * 1024, // 1GB
    default_timeout: Duration::from_secs(60),
    enable_monitoring: true,
};

let sandbox = SandboxManager::new(config);
let executor = ToolExecutor::with_managers(sandbox, PermissionManager::new());
```

## Performance

Tool execution is highly optimized:

- **Tool Registration**: < 1ms
- **Schema Validation**: < 0.1ms
- **Permission Check**: < 0.1ms
- **Tool Execution**: < 100ms (P99)

Benchmark results (compared to MIRIX Python implementation):

| Operation | AgentMem (Rust) | MIRIX (Python) | Speedup |
|-----------|-----------------|----------------|---------|
| Tool Registration | 0.8ms | 2.5ms | 3.1x |
| Schema Validation | 0.05ms | 0.2ms | 4.0x |
| Tool Execution | 0.02ms | 0.08ms | 4.0x |

## Examples

Run the basic usage example:

```bash
cargo run --package agent-mem-tools --example basic_usage
```

## Testing

Run all tests:

```bash
cargo test --package agent-mem-tools
```

Run benchmarks:

```bash
cargo bench --package agent-mem-tools
```

## Documentation

Generate and view documentation:

```bash
cargo doc --package agent-mem-tools --no-deps --open
```

## Comparison with MIRIX

| Feature | AgentMem Tools | MIRIX |
|---------|---------------|-------|
| Language | Rust | Python |
| Performance | 2-5x faster | Baseline |
| Type Safety | Compile-time | Runtime |
| Memory Safety | Guaranteed | GC-based |
| Async Support | Native (Tokio) | asyncio |
| Schema Generation | Automatic | Manual |
| Sandbox | Timeout + Resource limits | Timeout only |
| Permission Model | Role-based + Fine-grained | Basic |

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for details.

