//! Phase 4: 智能处理集成增强演示
//! 
//! 本演示展示了 AgentMem Phase 4 的核心功能：
//! 1. 高级事实提取
//! 2. 智能决策引擎
//! 3. 冲突解决系统
//! 4. 重要性评估器
//! 5. 集成智能处理流水线

use agent_mem_intelligence::{
    fact_extraction::{FactExtractor, ExtractedFact, FactCategory},
    decision_engine::{MemoryDecisionEngine, ExistingMemory},
    conflict_resolution::{ConflictResolver, ConflictResolverConfig},
    importance_evaluator::{ImportanceEvaluator, ImportanceEvaluatorConfig},
};
use agent_mem_traits::{Message, MessageRole, MemoryItem, Session, MemoryType, LLMConfig};
use agent_mem_llm::providers::OllamaProvider;
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动 Phase 4: 智能处理集成增强演示");

    // 演示各个组件
    demo_fact_extraction().await?;
    demo_decision_engine().await?;
    demo_conflict_resolution().await?;
    demo_importance_evaluation().await?;
    demo_integrated_processing().await?;

    info!("✅ Phase 4 演示完成！");
    Ok(())
}

/// 创建真实的 LLM 提供商
async fn create_llm_provider() -> Arc<dyn agent_mem_traits::LLMProvider + Send + Sync> {
    let llm_config = LLMConfig {
        provider: "ollama".to_string(),
        model: "gpt-oss:20b".to_string(),
        api_key: None,
        base_url: Some("http://localhost:11434".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(4000),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: None,
    };

    match OllamaProvider::new(llm_config) {
        Ok(provider) => Arc::new(provider) as Arc<dyn agent_mem_traits::LLMProvider + Send + Sync>,
        Err(e) => {
            info!("⚠️ 无法连接到 Ollama，使用 MockLLMProvider: {}", e);
            Arc::new(MockLLMProvider) as Arc<dyn agent_mem_traits::LLMProvider + Send + Sync>
        }
    }
}

/// 演示高级事实提取功能
async fn demo_fact_extraction() -> Result<()> {
    info!("\n📊 === 高级事实提取演示 ===");

    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "我叫张三，今年30岁，是一名软件工程师，住在北京。我喜欢编程和阅读。".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "我在阿里巴巴工作，主要负责后端开发，使用Java和Python。".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    // 创建事实提取器
    let llm = create_llm_provider().await;
    let fact_extractor = FactExtractor::new(llm);
    
    // 提取事实
    let facts = fact_extractor.extract_facts(&messages).await?;
    
    info!("提取到 {} 个事实:", facts.len());
    for (i, fact) in facts.iter().enumerate() {
        info!("  {}. {} (置信度: {:.2}, 类别: {:?})", 
              i + 1, fact.content, fact.confidence, fact.category);
    }

    Ok(())
}

/// 演示智能决策引擎
async fn demo_decision_engine() -> Result<()> {
    info!("\n🧠 === 智能决策引擎演示 ===");

    // 创建决策引擎
    let llm = create_llm_provider().await;
    let decision_engine = MemoryDecisionEngine::new(llm);

    // 模拟提取的事实
    let facts = vec![
        ExtractedFact {
            content: "用户姓名：张三".to_string(),
            confidence: 0.95,
            category: FactCategory::Personal,
            entities: vec![],
            temporal_info: None,
            source_message_id: Some("0".to_string()),
            metadata: HashMap::new(),
        },
        ExtractedFact {
            content: "用户职业：软件工程师".to_string(),
            confidence: 0.90,
            category: FactCategory::Professional,
            entities: vec![],
            temporal_info: None,
            source_message_id: Some("0".to_string()),
            metadata: HashMap::new(),
        },
    ];

    // 模拟现有记忆
    let existing_memories = vec![
        ExistingMemory {
            id: Uuid::new_v4().to_string(),
            content: "用户姓名：李四".to_string(),
            importance: 0.8,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            metadata: HashMap::new(),
        },
    ];

    // 生成决策
    let decisions = decision_engine.make_decisions(&facts, &existing_memories).await?;
    
    info!("生成 {} 个记忆决策:", decisions.len());
    for (i, decision) in decisions.iter().enumerate() {
        info!("  {}. 动作: {:?}, 置信度: {:.2}, 原因: {}", 
              i + 1, decision.action, decision.confidence, decision.reasoning);
    }

    Ok(())
}

/// 演示冲突解决系统
async fn demo_conflict_resolution() -> Result<()> {
    info!("\n⚔️ === 冲突解决系统演示 ===");

    // 创建冲突解决器
    let llm = create_llm_provider().await;
    let conflict_resolver = ConflictResolver::new(
        llm,
        ConflictResolverConfig::default(),
    );

    // 创建测试记忆
    let new_memories = vec![
        create_test_memory("用户姓名：张三", 0.9),
        create_test_memory("用户年龄：30岁", 0.8),
    ];

    let existing_memories = vec![
        create_test_memory("用户姓名：李四", 0.7),
        create_test_memory("用户年龄：25岁", 0.6),
    ];

    // 检测冲突
    let conflicts = conflict_resolver.detect_conflicts(&new_memories, &existing_memories).await?;
    
    info!("检测到 {} 个潜在冲突:", conflicts.len());
    for (i, conflict) in conflicts.iter().enumerate() {
        info!("  {}. 冲突类型: {:?}, 置信度: {:.2}", 
              i + 1, conflict.conflict_type, conflict.confidence);
        info!("     描述: {}", conflict.description);
    }

    Ok(())
}

/// 演示重要性评估器
async fn demo_importance_evaluation() -> Result<()> {
    info!("\n⭐ === 重要性评估器演示 ===");

    // 创建重要性评估器
    let llm = create_llm_provider().await;
    let importance_evaluator = ImportanceEvaluator::new(
        llm,
        ImportanceEvaluatorConfig::default(),
    );

    // 创建测试记忆
    let memory = create_test_memory("用户是资深软件工程师，有10年经验", 0.8);
    
    // 评估重要性
    let evaluation = importance_evaluator.evaluate_importance(
        &memory,
        &[],
        &[],
    ).await?;
    
    info!("重要性评估结果:");
    info!("  重要性分数: {:.2}", evaluation.importance_score);
    info!("  置信度: {:.2}", evaluation.confidence);
    info!("  推理: {}", evaluation.reasoning);

    Ok(())
}

/// 演示集成智能处理流水线
async fn demo_integrated_processing() -> Result<()> {
    info!("\n🔄 === 集成智能处理流水线演示 ===");

    // 创建真实的 LLM 提供商
    let llm = create_llm_provider().await;

    let fact_extractor = FactExtractor::new(llm.clone());
    let decision_engine = MemoryDecisionEngine::new(llm.clone());
    let conflict_resolver = ConflictResolver::new(
        llm.clone(),
        ConflictResolverConfig::default(),
    );
    let _importance_evaluator = ImportanceEvaluator::new(
        llm.clone(),
        ImportanceEvaluatorConfig::default(),
    );

    // 准备测试消息
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "我是王五，今年35岁，在腾讯工作，是一名高级架构师。".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "我负责微服务架构设计，熟悉Kubernetes和Docker。".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    // 模拟现有记忆
    let existing_memories = vec![
        ExistingMemory {
            id: Uuid::new_v4().to_string(),
            content: "用户是软件工程师".to_string(),
            importance: 0.7,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            metadata: HashMap::new(),
        },
    ];

    // 手动执行智能处理流水线
    let start_time = std::time::Instant::now();

    // 1. 提取事实
    let extracted_facts = fact_extractor.extract_facts(&messages).await?;

    // 2. 生成决策
    let memory_decisions = decision_engine.make_decisions(&extracted_facts, &existing_memories).await?;

    // 3. 检测冲突
    let memories: Vec<_> = existing_memories.iter().map(|m| create_test_memory(&m.content, m.importance)).collect();
    let conflict_detections = conflict_resolver.detect_conflicts(&memories, &memories).await?;

    let processing_time = start_time.elapsed().as_millis() as f64;

    info!("智能处理结果:");
    info!("  提取事实数: {}", extracted_facts.len());
    info!("  记忆决策数: {}", memory_decisions.len());
    info!("  冲突检测数: {}", conflict_detections.len());
    info!("  处理时间: {:.2}ms", processing_time);

    // 显示详细结果
    if !extracted_facts.is_empty() {
        info!("  提取的事实:");
        for (i, fact) in extracted_facts.iter().enumerate() {
            info!("    {}. {} (置信度: {:.2})", i + 1, fact.content, fact.confidence);
        }
    }

    if !memory_decisions.is_empty() {
        info!("  记忆决策:");
        for (i, decision) in memory_decisions.iter().enumerate() {
            info!("    {}. {:?} (置信度: {:.2})", i + 1, decision.action, decision.confidence);
        }
    }

    Ok(())
}

/// 创建测试记忆项
fn create_test_memory(content: &str, importance: f32) -> MemoryItem {
    MemoryItem {
        id: Uuid::new_v4().to_string(),
        content: content.to_string(),
        hash: None,
        metadata: HashMap::new(),
        score: Some(importance),
        created_at: Utc::now(),
        updated_at: None,
        session: Session::default(),
        memory_type: MemoryType::Episodic,
        entities: vec![],
        relations: vec![],
        agent_id: "demo".to_string(),
        user_id: None,
        importance,
        embedding: None,
        last_accessed_at: Utc::now(),
        access_count: 0,
        expires_at: None,
        version: 1,
    }
}

/// 模拟 LLM 提供者
struct MockLLMProvider;

#[async_trait::async_trait]
impl agent_mem_traits::LLMProvider for MockLLMProvider {
    async fn generate(&self, messages: &[Message]) -> agent_mem_traits::Result<String> {
        // 根据请求内容返回不同的响应格式
        let prompt = &messages[0].content;

        if prompt.contains("Extract structured facts") || prompt.contains("Extract key facts") {
            // 事实提取响应
            Ok(json!({
                "facts": [
                    {
                        "content": "用户基本信息已记录",
                        "confidence": 0.9,
                        "category": "Personal",
                        "entities": [],
                        "temporal_info": null,
                        "source_message_id": "0",
                        "metadata": {}
                    }
                ],
                "confidence": 0.9,
                "reasoning": "成功提取用户基本信息"
            }).to_string())
        } else if prompt.contains("Make memory decisions") || prompt.contains("decision") {
            // 决策引擎响应
            Ok(json!({
                "decisions": [
                    {
                        "action": {
                            "Add": {
                                "content": "用户基本信息：张三，30岁，软件工程师，住在北京",
                                "importance": 0.8,
                                "metadata": {}
                            }
                        },
                        "confidence": 0.85,
                        "reasoning": "重要的用户信息需要存储",
                        "affected_memories": [],
                        "estimated_impact": 0.7
                    }
                ],
                "overall_confidence": 0.85,
                "reasoning": "基于内容重要性和用户偏好的决策"
            }).to_string())
        } else if prompt.contains("Detect conflicts") || prompt.contains("conflict") {
            // 冲突解决响应
            Ok(json!({
                "conflicts": [],
                "confidence": 0.95,
                "reasoning": "未检测到冲突"
            }).to_string())
        } else if prompt.contains("Evaluate importance") || prompt.contains("importance") {
            // 重要性评估响应
            Ok(json!({
                "importance_score": 0.8,
                "confidence": 0.9,
                "factors": [
                    {
                        "factor_type": "Content",
                        "score": 0.8,
                        "reasoning": "包含重要的用户信息"
                    }
                ],
                "evaluated_at": "2025-09-09T12:55:00Z",
                "reasoning": "重要的用户信息"
            }).to_string())
        } else {
            // 默认响应 - 假设是事实提取
            Ok(json!({
                "facts": [
                    {
                        "content": "用户基本信息已记录",
                        "confidence": 0.9,
                        "category": "Personal",
                        "entities": [],
                        "temporal_info": null,
                        "source_message_id": "0",
                        "metadata": {}
                    }
                ],
                "confidence": 0.9,
                "reasoning": "成功提取用户基本信息"
            }).to_string())
        }
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> agent_mem_traits::Result<Box<dyn futures::Stream<Item = agent_mem_traits::Result<String>> + Send + Unpin>> {
        use std::pin::Pin;
        use std::task::{Context, Poll};
        
        struct EmptyStream;
        
        impl futures::Stream for EmptyStream {
            type Item = agent_mem_traits::Result<String>;
            
            fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                Poll::Ready(None)
            }
        }
        
        Ok(Box::new(EmptyStream))
    }

    fn get_model_info(&self) -> agent_mem_traits::ModelInfo {
        agent_mem_traits::ModelInfo {
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
            max_tokens: 4096,
            supports_streaming: false,
            supports_functions: false,
        }
    }

    fn validate_config(&self) -> agent_mem_traits::Result<()> {
        Ok(())
    }
}
