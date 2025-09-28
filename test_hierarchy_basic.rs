//! 基础层次管理器测试

use agent_mem_core::{
    hierarchy::{DefaultHierarchyManager, HierarchyConfig, HierarchyManager, MemoryScope},
    types::{Memory, MemoryType},
};
use std::collections::HashMap;

#[tokio::test]
async fn test_basic_hierarchy_operations() {
    // 创建层次管理器
    let config = HierarchyConfig::default();
    let manager = DefaultHierarchyManager::new(config);

    // 创建测试记忆
    let mut metadata = HashMap::new();
    metadata.insert("test".to_string(), serde_json::Value::String("value".to_string()));

    let memory = Memory {
        id: "test-memory-1".to_string(),
        agent_id: "test-agent".to_string(),
        content: "This is a test memory".to_string(),
        memory_type: MemoryType::Episodic,
        score: Some(0.8),
        metadata,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    // 测试添加记忆
    let hierarchical_memory = manager.add_memory(memory.clone()).await.unwrap();
    println!("Added memory: {:?}", hierarchical_memory.memory.id);

    // 测试获取记忆
    let retrieved = manager.get_memory(&memory.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_memory = retrieved.unwrap();
    assert_eq!(retrieved_memory.memory.id, memory.id);
    assert_eq!(retrieved_memory.memory.content, memory.content);

    // 测试搜索记忆
    let search_results = manager
        .search_memories("test memory", None, Some(5))
        .await
        .unwrap();
    assert!(!search_results.is_empty());
    assert_eq!(search_results[0].memory.id, memory.id);

    // 测试按级别获取记忆
    let level_memories = manager
        .get_memories_at_level(hierarchical_memory.level)
        .await
        .unwrap();
    assert!(!level_memories.is_empty());

    // 测试删除记忆
    let removed = manager.remove_memory(&memory.id).await.unwrap();
    assert!(removed);

    // 验证删除后无法获取
    let after_removal = manager.get_memory(&memory.id).await.unwrap();
    assert!(after_removal.is_none());

    println!("✅ All basic hierarchy operations passed!");
}

#[tokio::test]
async fn test_hierarchy_statistics() {
    let config = HierarchyConfig::default();
    let manager = DefaultHierarchyManager::new(config);

    // 添加多个不同重要性的记忆
    for i in 0..5 {
        let memory = Memory {
            id: format!("memory-{}", i),
            agent_id: "test-agent".to_string(),
            content: format!("Test memory content {}", i),
            memory_type: MemoryType::Semantic,
            score: Some(0.2 + (i as f32 * 0.2)), // 0.2, 0.4, 0.6, 0.8, 1.0
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        manager.add_memory(memory).await.unwrap();
    }

    // 获取统计信息
    let stats = manager.get_hierarchy_stats().await.unwrap();
    
    // 验证统计信息
    assert!(!stats.memories_by_level.is_empty());
    assert!(!stats.avg_importance_by_level.is_empty());
    
    println!("📊 Hierarchy statistics: {:?}", stats);
    println!("✅ Hierarchy statistics test passed!");
}

fn main() {
    println!("Run with: cargo test --test test_hierarchy_basic");
}
