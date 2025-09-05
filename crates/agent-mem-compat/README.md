# AgentMem Mem0 Compatibility Layer

This crate provides a compatibility layer that allows AgentMem to be used as a drop-in replacement for Mem0. It implements the Mem0 API surface while leveraging AgentMem's advanced memory management capabilities.

## Features

- **Drop-in Replacement**: Compatible with existing Mem0 code
- **Enhanced Performance**: Leverages AgentMem's optimized storage and retrieval
- **Advanced Memory Types**: Supports episodic, semantic, procedural, and working memory
- **Intelligent Processing**: Automatic importance scoring and memory consolidation
- **Flexible Storage**: Multiple vector database backends
- **Rich Metadata**: Support for complex metadata and filtering
- **Session Tracking**: Agent and run ID support for context management

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
agent-mem-compat = { path = "path/to/agent-mem-compat" }
tokio = { version = "1.0", features = ["full"] }
```

## Basic Usage

```rust
use agent_mem_compat::Mem0Client;
use std::collections::HashMap;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = Mem0Client::new().await?;
    
    // Add a memory
    let memory_id = client.add("user123", "I love pizza", None).await?;
    
    // Add a memory with metadata
    let memory_id = client.add(
        "user123", 
        "My favorite programming language is Rust",
        Some(HashMap::from([
            ("category".to_string(), json!("preference")),
            ("importance".to_string(), json!(0.8)),
        ]))
    ).await?;
    
    // Search memories
    let memories = client.search("food preferences", "user123", None).await?;
    
    // Get all memories for a user
    let all_memories = client.get_all("user123", None).await?;
    
    // Update a memory
    let updated = client.update(
        &memory_id,
        "user123",
        agent_mem_compat::types::UpdateMemoryRequest {
            memory: Some("My favorite language is Rust - it's fast and safe!".to_string()),
            metadata: None,
        }
    ).await?;
    
    // Delete a memory
    let result = client.delete(&memory_id, "user123").await?;
    
    Ok(())
}
```

## Configuration

The compatibility layer supports multiple configuration options:

```rust
use agent_mem_compat::{Mem0Client, Mem0Config};

// OpenAI configuration
let config = Mem0Config::openai();
let client = Mem0Client::with_config(config).await?;

// Anthropic configuration
let config = Mem0Config::anthropic();
let client = Mem0Client::with_config(config).await?;

// Local/offline configuration
let config = Mem0Config::local();
let client = Mem0Client::with_config(config).await?;

// Custom configuration
let mut config = Mem0Config::default();
config.llm.provider = "custom-provider".to_string();
config.memory.auto_consolidation = true;
let client = Mem0Client::with_config(config).await?;
```

## Advanced Features

### Filtering and Search

```rust
use agent_mem_compat::types::{MemoryFilter, SearchMemoryRequest};

// Search with filters
let request = SearchMemoryRequest {
    query: "programming".to_string(),
    user_id: "user123".to_string(),
    filters: Some(MemoryFilter {
        agent_id: Some("my-agent".to_string()),
        limit: Some(10),
        ..Default::default()
    }),
    limit: Some(5),
};

let results = client.search_with_options(request).await?;
```

### Memory Statistics

```rust
// Get user statistics
let stats = client.get_stats("user123").await?;
println!("Total memories: {}", stats["total_memories"]);
println!("Average importance: {}", stats["average_importance"]);
```

### Session and Agent Tracking

```rust
use agent_mem_compat::types::AddMemoryRequest;

// Add memory with session tracking
let request = AddMemoryRequest {
    memory: "User prefers dark mode".to_string(),
    user_id: "user123".to_string(),
    agent_id: Some("ui-agent".to_string()),
    run_id: Some("session-001".to_string()),
    metadata: HashMap::new(),
};

let memory_id = client.add_with_options(request).await?;
```

## API Reference

### Core Methods

- `new()` - Create a client with default configuration
- `with_config(config)` - Create a client with custom configuration
- `add(user_id, memory, metadata)` - Add a simple memory
- `add_with_options(request)` - Add a memory with full options
- `search(query, user_id, filters)` - Search memories
- `search_with_options(request)` - Search with advanced options
- `get(memory_id, user_id)` - Get a specific memory
- `update(memory_id, user_id, request)` - Update a memory
- `delete(memory_id, user_id)` - Delete a memory
- `get_all(user_id, filters)` - Get all memories for a user
- `delete_all(user_id)` - Delete all memories for a user
- `get_stats(user_id)` - Get memory statistics
- `reset()` - Clear all data

### Configuration Options

- `Mem0Config::openai()` - OpenAI configuration
- `Mem0Config::anthropic()` - Anthropic configuration  
- `Mem0Config::local()` - Local/offline configuration
- `Mem0Config::default()` - Default configuration

## Examples

See the `examples/mem0-compat-demo` directory for a comprehensive demonstration of all features.

Run the demo with:

```bash
cargo run --bin mem0-demo
```

## Migration from Mem0

This compatibility layer is designed to be a drop-in replacement for Mem0. Most existing Mem0 code should work with minimal changes:

1. Replace `mem0` imports with `agent_mem_compat`
2. Update client initialization if needed
3. Enjoy enhanced performance and features!

## Current Limitations

- This is a demonstration implementation using in-memory storage
- Full AgentMem integration is planned for future releases
- Some advanced Mem0 features may not be fully implemented yet

## Contributing

Contributions are welcome! Please see the main AgentMem repository for contribution guidelines.

## License

This project is licensed under the same terms as the main AgentMem project.
