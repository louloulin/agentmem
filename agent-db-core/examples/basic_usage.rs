// AgentDB 基本使用示例
use agent_db_core::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentDB 基本使用示例");
    
    // 创建数据库
    let db = create_database("./test_db").await?
        .with_rag_engine().await?;
    println!("✅ 数据库创建成功");
    
    // 测试智能体状态管理
    test_agent_state(&db).await?;
    
    // 测试记忆管理
    test_memory_management(&db).await?;
    
    // 测试 RAG 功能
    test_rag_functionality(&db).await?;
    
    println!("🎉 所有测试完成！");
    Ok(())
}

async fn test_agent_state(db: &AgentDatabase) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 测试智能体状态管理...");
    
    // 创建一个智能体状态
    let state = AgentState::new(
        1001, // agent_id
        1,    // session_id
        StateType::WorkingMemory,
        b"Hello, this is agent 1001's working memory".to_vec(),
    );
    
    // 保存状态
    db.save_agent_state(&state).await?;
    println!("✅ 智能体状态保存成功");
    
    // 加载状态
    if let Some(loaded_state) = db.load_agent_state(1001).await? {
        println!("✅ 智能体状态加载成功: agent_id = {}", loaded_state.agent_id);
        println!("   数据: {:?}", String::from_utf8_lossy(&loaded_state.data));
    }
    
    Ok(())
}

async fn test_memory_management(db: &AgentDatabase) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 测试记忆管理...");
    
    // 创建一个记忆
    let memory = Memory {
        memory_id: "mem_001".to_string(),
        agent_id: 1001,
        memory_type: MemoryType::Episodic,
        content: "今天学习了 Rust 编程语言".to_string(),
        importance: 0.8,
        created_at: chrono::Utc::now().timestamp(),
        last_access: chrono::Utc::now().timestamp(),
        access_count: 1,
        expires_at: None,
        embedding: None,
    };
    
    // 存储记忆
    db.store_memory(&memory).await?;
    println!("✅ 记忆存储成功");
    
    // 获取智能体的记忆
    let memories = db.get_memories(1001).await?;
    println!("✅ 获取到 {} 条记忆", memories.len());
    
    for mem in &memories {
        println!("   记忆: {}", mem.content);
    }
    
    Ok(())
}

async fn test_rag_functionality(db: &AgentDatabase) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📚 测试 RAG 功能...");
    
    // 创建一个文档
    let mut document = Document::new(
        "Rust 编程指南".to_string(),
        "Rust 是一种系统编程语言，专注于安全、速度和并发。它由 Mozilla 开发，旨在解决 C++ 的内存安全问题。".to_string(),
    );
    
    // 添加元数据
    document.metadata.insert("author".to_string(), "Rust Team".to_string());
    document.metadata.insert("category".to_string(), "Programming".to_string());
    
    // 对文档进行分块
    document.chunk_document(200, 50)?;
    
    // 索引文档
    db.index_document(&document).await?;
    println!("✅ 文档索引成功");
    
    // 搜索文档
    let search_results = db.search_documents("Rust 编程语言", 5).await?;
    println!("✅ 搜索到 {} 个结果", search_results.len());
    
    for result in &search_results {
        println!("   结果 (分数: {:.2}): {}", result.score, result.content);
    }
    
    Ok(())
}
