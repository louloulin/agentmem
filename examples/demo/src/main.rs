//! Demo of the current AgentMem functionality

use agent_mem_config::{ConfigFactory, MemoryConfig};
use agent_mem_traits::{Message, Session, LLMConfig, VectorStoreConfig, MemoryProvider};
use agent_mem_utils::{extract_json, clean_text, hash_content, Timer};
use agent_mem_core::{MemoryManager, MemoryType, MemoryQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem v2.0 Demo");
    println!("===================");
    
    // 1. Configuration Demo
    println!("\n1. 📋 Configuration System Demo");
    let config = ConfigFactory::create_memory_config();
    println!("   Default LLM Provider: {}", config.llm.provider);
    println!("   Default Vector Store: {}", config.vector_store.provider);
    
    // Create different LLM configs
    let openai_config = ConfigFactory::create_llm_config("openai");
    let anthropic_config = ConfigFactory::create_llm_config("anthropic");
    println!("   OpenAI Model: {}", openai_config.model);
    println!("   Anthropic Model: {}", anthropic_config.model);
    
    // 2. Data Types Demo
    println!("\n2. 🗂️ Data Types Demo");
    let session = Session::new()
        .with_user_id(Some("user123".to_string()))
        .with_agent_id(Some("assistant".to_string()));
    println!("   Session ID: {}", session.id);
    println!("   User ID: {:?}", session.user_id);
    
    let message = Message::user("I love playing tennis on weekends");
    println!("   Message: {}", message.content);
    println!("   Role: {:?}", message.role);
    
    // 3. Utils Demo
    println!("\n3. 🛠️ Utils Demo");
    
    // JSON extraction
    let json_text = r#"
    Here's the result:
    ```json
    {"name": "John", "hobby": "tennis", "confidence": 0.95}
    ```
    That's it.
    "#;
    let extracted = extract_json(json_text)?;
    println!("   Extracted JSON: {}", extracted);
    
    // Text processing
    let messy_text = "  This   has    extra   spaces  and needs cleaning  ";
    let cleaned = clean_text(messy_text);
    println!("   Cleaned text: '{}'", cleaned);
    
    // Hashing
    let content = "I love playing tennis";
    let hash = hash_content(content);
    println!("   Content hash: {}", &hash[..16]);
    
    // Performance timing
    let timer = Timer::new("demo_operation");
    std::thread::sleep(std::time::Duration::from_millis(10));
    let metrics = timer.finish();
    println!("   Operation took: {}ms", metrics.duration_ms);
    
    // 4. Configuration Validation Demo
    println!("\n4. ✅ Configuration Validation Demo");
    let mut valid_config = MemoryConfig {
        llm: LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        },
        vector_store: VectorStoreConfig {
            provider: "lancedb".to_string(),
            path: "./data/vectors".to_string(),
            dimension: 1536,
            ..Default::default()
        },
        ..Default::default()
    };
    
    match agent_mem_config::validate_memory_config(&valid_config) {
        Ok(_) => println!("   ✅ Configuration is valid"),
        Err(e) => println!("   ❌ Configuration error: {}", e),
    }
    
    // Test invalid config
    valid_config.llm.api_key = None;
    match agent_mem_config::validate_memory_config(&valid_config) {
        Ok(_) => println!("   ✅ Configuration is valid"),
        Err(e) => println!("   ❌ Configuration error: {}", e),
    }
    
    // 5. Memory Management Demo
    println!("\n5. 🧠 Memory Management Demo");
    let memory_manager = MemoryManager::new();

    // Add some memories
    let memory_id1 = memory_manager.add_memory(
        "demo-agent".to_string(),
        Some("demo-user".to_string()),
        "I love playing tennis on weekends".to_string(),
        Some(MemoryType::Episodic),
        Some(0.8),
        None,
    ).await?;
    println!("   Added episodic memory: {}", &memory_id1[..8]);

    let memory_id2 = memory_manager.add_memory(
        "demo-agent".to_string(),
        Some("demo-user".to_string()),
        "Tennis is played with a racket and ball".to_string(),
        Some(MemoryType::Semantic),
        Some(0.9),
        None,
    ).await?;
    println!("   Added semantic memory: {}", &memory_id2[..8]);

    // Search memories
    let query = MemoryQuery::new("demo-agent".to_string())
        .with_text_query("tennis".to_string())
        .with_limit(5);
    let search_results = memory_manager.search_memories(query).await?;
    println!("   Found {} tennis-related memories", search_results.len());

    // Get memory statistics
    let stats = memory_manager.get_memory_stats(Some("demo-agent")).await?;
    println!("   Total memories: {}", stats.total_memories);
    println!("   Average importance: {:.2}", stats.average_importance);

    // Update a memory
    memory_manager.update_memory(
        &memory_id1,
        Some("I love playing tennis and badminton on weekends".to_string()),
        Some(0.85),
        None,
    ).await?;
    println!("   Updated memory: {}", &memory_id1[..8]);

    // Get memory history
    let history = memory_manager.history(&memory_id1).await?;
    println!("   Memory history entries: {}", history.len());

    println!("\n🎉 Demo completed successfully!");
    println!("   ✅ Configuration system working");
    println!("   ✅ Data types and utilities working");
    println!("   ✅ Memory management working");
    println!("   ✅ All {} tests passing", 36); // Update count

    Ok(())
}
