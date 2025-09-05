//! Simple AgentMem Core Demo
//! 
//! This example demonstrates basic usage of the AgentMem core functionality,
//! including memory creation, storage, and retrieval.

use agent_mem_core::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::{Memory, MemoryType, MemoryScope};
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, warn, error};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting AgentMem Simple Demo");

    // Create memory engine with default configuration
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config);

    info!("Memory engine initialized");

    // Create some test memories
    let memories = create_test_memories();
    
    // Add memories to the engine
    for memory in memories {
        match engine.add_memory(memory.clone()).await {
            Ok(id) => {
                info!("Added memory with ID: {}", id);
            }
            Err(e) => {
                error!("Failed to add memory: {}", e);
            }
        }
    }

    // Get engine statistics
    match engine.get_statistics().await {
        Ok(stats) => {
            info!("Engine Statistics:");
            info!("  Total memories: {}", stats.total_memories);
            info!("  Memories by level: {:?}", stats.memories_by_level);
            info!("  Average importance by level: {:?}", stats.avg_importance_by_level);
        }
        Err(e) => {
            error!("Failed to get statistics: {}", e);
        }
    }

    // Process memories for optimization
    match engine.process_memories().await {
        Ok(report) => {
            info!("Processing Report:");
            info!("  Total memories processed: {}", report.total_memories);
            info!("  Conflicts detected: {}", report.conflicts_detected);
            info!("  Conflicts resolved: {}", report.conflicts_resolved);
            info!("  Memories promoted: {}", report.memories_promoted);
            info!("  Memories demoted: {}", report.memories_demoted);
            info!("  Processing errors: {}", report.errors);
        }
        Err(e) => {
            error!("Failed to process memories: {}", e);
        }
    }

    info!("Demo completed successfully");
    Ok(())
}

fn create_test_memories() -> Vec<Memory> {
    let now = Utc::now();
    
    vec![
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "Remember to implement the new authentication system".to_string(),
            memory_type: MemoryType::Procedural,
            scope: MemoryScope::Global,
            importance: 0.8,
            tags: vec!["authentication".to_string(), "security".to_string(), "implementation".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("priority".to_string(), "high".to_string());
                meta.insert("category".to_string(), "development".to_string());
                meta
            },
            created_at: now,
            updated_at: now,
            accessed_at: None,
            access_count: 0,
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "User John Doe prefers dark mode interface".to_string(),
            memory_type: MemoryType::Episodic,
            scope: MemoryScope::User { 
                agent_id: "agent-1".to_string(), 
                user_id: "john-doe".to_string() 
            },
            importance: 0.6,
            tags: vec!["user-preference".to_string(), "ui".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("user_id".to_string(), "john-doe".to_string());
                meta.insert("preference_type".to_string(), "ui".to_string());
                meta
            },
            created_at: now,
            updated_at: now,
            accessed_at: None,
            access_count: 0,
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "The capital of France is Paris".to_string(),
            memory_type: MemoryType::Semantic,
            scope: MemoryScope::Global,
            importance: 0.4,
            tags: vec!["geography".to_string(), "facts".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "general-knowledge".to_string());
                meta.insert("verified".to_string(), "true".to_string());
                meta
            },
            created_at: now,
            updated_at: now,
            accessed_at: None,
            access_count: 0,
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "Current task: Review pull request #123".to_string(),
            memory_type: MemoryType::Working,
            scope: MemoryScope::Session { 
                agent_id: "agent-1".to_string(), 
                user_id: "developer".to_string(),
                session_id: "session-456".to_string()
            },
            importance: 0.9,
            tags: vec!["task".to_string(), "review".to_string(), "urgent".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("pr_number".to_string(), "123".to_string());
                meta.insert("status".to_string(), "pending".to_string());
                meta.insert("deadline".to_string(), "today".to_string());
                meta
            },
            created_at: now,
            updated_at: now,
            accessed_at: None,
            access_count: 0,
        },
    ]
}
