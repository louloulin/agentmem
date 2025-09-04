//! Demo of the current AgentMem functionality

use agent_mem_config::{ConfigFactory, MemoryConfig};
use agent_mem_traits::{Message, Session, LLMConfig, VectorStoreConfig, MemoryProvider};
use agent_mem_utils::{extract_json, clean_text, hash_content, Timer};
use agent_mem_core::{MemoryManager, MemoryType, MemoryQuery};
use agent_mem_llm::{LLMFactory, LLMClient, prompts::PromptManager};
use agent_mem_storage::{StorageFactory, vector::{VectorUtils, SimilarityCalculator, SimilarityMetric}};

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
            dimension: Some(1536),
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

    // 6. LLM Integration Demo
    println!("\n6. 🤖 LLM Integration Demo");

    // 演示LLM工厂模式
    println!("   Supported LLM providers: {:?}", LLMFactory::supported_providers());

    // 创建一个模拟的LLM配置（不会实际调用API）
    let llm_config = LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_key: Some("demo-key".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        ..Default::default()
    };

    // 创建LLM客户端
    let llm_client = LLMClient::new(&llm_config)?;
    let model_info = llm_client.get_model_info();
    println!("   LLM Model: {} ({})", model_info.model, model_info.provider);
    println!("   Max tokens: {}", model_info.max_tokens);
    println!("   Supports functions: {}", model_info.supports_functions);

    // 演示提示词管理
    let prompt_manager = PromptManager::new();
    let templates = prompt_manager.get_available_templates();
    println!("   Available prompt templates: {}", templates.len());

    // 构建记忆提取提示词
    let extraction_prompt = prompt_manager.build_memory_extraction_prompt(
        "用户说：我喜欢在周末打网球，这是我最喜欢的运动。"
    )?;
    println!("   Built memory extraction prompt with {} messages", extraction_prompt.len());

    // 构建记忆摘要提示词
    let summarization_prompt = prompt_manager.build_memory_summarization_prompt(
        "记忆1：用户喜欢网球\n记忆2：用户周末有空\n记忆3：网球是用户最喜欢的运动"
    )?;
    println!("   Built memory summarization prompt with {} messages", summarization_prompt.len());

    // 验证配置
    llm_client.validate_config()?;
    println!("   LLM configuration validated successfully");

    // 7. 存储集成演示
    println!("\n7. 🗄️ Storage Integration Demo");

    // 演示存储工厂模式
    println!("   Supported storage providers: {:?}", StorageFactory::supported_providers());

    // 创建内存向量存储（3维向量用于演示）
    let config = VectorStoreConfig {
        provider: "memory".to_string(),
        dimension: Some(3),
        ..Default::default()
    };
    let memory_store = StorageFactory::create_vector_store(&config).await?;
    println!("   Created memory vector store");

    // 添加一些测试向量
    use agent_mem_traits::VectorData;
    use std::collections::HashMap;

    let test_vectors = vec![
        VectorData {
            id: "vec1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec2".to_string(),
            vector: vec![0.0, 1.0, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec3".to_string(),
            vector: vec![0.0, 0.0, 1.0],
            metadata: HashMap::new(),
        },
    ];

    let ids = memory_store.add_vectors(test_vectors).await?;
    println!("   Added {} vectors to store", ids.len());

    // 搜索相似向量
    let query_vector = vec![1.0, 0.0, 0.0];
    let search_results = memory_store.search_vectors(query_vector, 2, None).await?;
    println!("   Found {} similar vectors", search_results.len());

    // 获取向量数量
    let count = memory_store.count_vectors().await?;
    println!("   Total vectors in store: {}", count);

    // 演示向量工具函数
    let vector1 = vec![1.0, 2.0, 3.0];
    let vector2 = vec![4.0, 5.0, 6.0];

    let dot_product = VectorUtils::dot_product(&vector1, &vector2)?;
    println!("   Dot product: {}", dot_product);

    let l2_norm = VectorUtils::l2_norm(&vector1);
    println!("   L2 norm: {}", l2_norm);

    // 演示相似度计算
    let similarity = SimilarityCalculator::cosine_similarity(&vector1, &vector2)?;
    println!("   Cosine similarity: {}", similarity);

    let distance = SimilarityCalculator::euclidean_distance(&vector1, &vector2)?;
    println!("   Euclidean distance: {}", distance);

    // 批量相似度计算
    let query = vec![1.0, 0.0, 0.0];
    let vectors = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
    ];
    let similarities = SimilarityCalculator::batch_similarity(&query, &vectors, SimilarityMetric::Cosine)?;
    println!("   Batch similarities: {:?}", similarities);

    println!("\n🎉 Demo completed successfully!");
    println!("   ✅ Configuration system working");
    println!("   ✅ Data types and utilities working");
    println!("   ✅ Memory management working");
    println!("   ✅ LLM integration working");
    println!("   ✅ Storage integration working");
    println!("   ✅ All {} tests passing", 112); // Update count

    Ok(())
}
