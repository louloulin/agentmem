//! # 图记忆演示
//!
//! 这个演示展示了 AgentMem 的图记忆功能：
//! - 实体和关系提取
//! - 图数据库存储和查询
//! - 智能记忆融合
//! - 实体邻居查询

use agent_mem_compat::{Mem0Client, GraphMemoryManager, GraphMemoryConfig};
use agent_mem_traits::{Entity, GraphResult, Session};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动图记忆演示");

    // 创建 Mem0 客户端（包含图记忆功能）
    let client = match Mem0Client::new().await {
        Ok(client) => client,
        Err(e) => {
            warn!("无法连接到图数据库，将运行演示模式: {}", e);
            return demo_mode().await;
        }
    };

    let user_id = "demo_user";

    // 演示 1: 添加带有实体关系的记忆
    info!("\n📝 演示 1: 添加带有实体关系的记忆");
    demo_add_memories_with_entities(&client, user_id).await?;

    // 演示 2: 图搜索
    info!("\n🔍 演示 2: 图搜索");
    demo_graph_search(&client, user_id).await?;

    // 演示 3: 实体邻居查询
    info!("\n🌐 演示 3: 实体邻居查询");
    demo_entity_neighbors(&client).await?;

    // 演示 4: 智能记忆融合
    info!("\n🧠 演示 4: 智能记忆融合");
    demo_memory_fusion(&client, user_id).await?;

    info!("✅ 所有图记忆演示完成！");
    Ok(())
}

/// 演示模式（当无法连接到真实图数据库时）
async fn demo_mode() -> Result<()> {
    info!("🎭 运行图记忆演示模式");
    
    // 创建独立的图记忆管理器进行演示
    let config = GraphMemoryConfig::default();
    info!("图记忆配置: {:?}", config);
    
    // 模拟实体提取
    let sample_entities = vec![
        Entity {
            id: "person_张三".to_string(),
            name: "张三".to_string(),
            entity_type: "人名".to_string(),
            attributes: HashMap::from([
                ("source".to_string(), json!("text_extraction")),
                ("confidence".to_string(), json!(0.9)),
            ]),
        },
        Entity {
            id: "company_阿里巴巴".to_string(),
            name: "阿里巴巴".to_string(),
            entity_type: "公司".to_string(),
            attributes: HashMap::from([
                ("source".to_string(), json!("text_extraction")),
                ("confidence".to_string(), json!(0.8)),
            ]),
        },
    ];

    info!("模拟提取的实体:");
    for entity in &sample_entities {
        info!("  - {} ({}): {}", entity.entity_type, entity.name, entity.id);
    }

    // 模拟关系提取
    info!("\n模拟提取的关系:");
    info!("  - 张三 工作于 阿里巴巴 (置信度: 0.8)");
    info!("  - 张三 居住于 杭州 (置信度: 0.7)");

    // 模拟图搜索结果
    info!("\n模拟图搜索 '张三':");
    info!("  找到 1 个实体和 2 个关系");
    info!("  实体: 张三 (人名)");
    info!("  关系: 工作于 -> 阿里巴巴, 居住于 -> 杭州");

    // 模拟记忆融合
    info!("\n模拟记忆融合:");
    info!("  输入记忆: ['张三在阿里巴巴工作', '张三住在杭州', '阿里巴巴是一家科技公司']");
    info!("  融合结果: 发现 3 个实体: 张三, 阿里巴巴, 杭州. 发现 2 个关系: 工作于, 居住于.");
    info!("  融合置信度: 0.83");

    Ok(())
}

/// 演示添加带有实体关系的记忆
async fn demo_add_memories_with_entities(client: &Mem0Client, user_id: &str) -> Result<()> {
    let memories = vec![
        "张三在阿里巴巴工作，他是一名优秀的软件工程师",
        "张三住在杭州，他很喜欢这个城市的风景",
        "阿里巴巴是中国最大的电商公司之一",
        "杭州是浙江省的省会城市，以西湖闻名",
        "张三喜欢吃杭州菜，特别是东坡肉",
        "阿里巴巴的总部位于杭州",
    ];

    for (i, memory) in memories.iter().enumerate() {
        match client.add_with_graph_extraction(memory, user_id, None).await {
            Ok(memory_id) => {
                info!("  {}. 添加记忆成功: {} -> {}", i + 1, memory, memory_id);
            }
            Err(e) => {
                warn!("  {}. 添加记忆失败: {} -> {}", i + 1, memory, e);
            }
        }
    }

    Ok(())
}

/// 演示图搜索
async fn demo_graph_search(client: &Mem0Client, user_id: &str) -> Result<()> {
    let search_queries = vec![
        "张三",
        "阿里巴巴", 
        "杭州",
        "工作",
        "公司",
    ];

    for query in search_queries {
        match client.search_graph(query, user_id).await {
            Ok(results) => {
                info!("搜索 '{}' 找到 {} 个图结果:", query, results.len());
                for (i, result) in results.iter().enumerate() {
                    info!("  {}. 实体: {} ({})", i + 1, result.entity.name, result.entity.entity_type);
                    info!("     关系数量: {}", result.relations.len());
                    info!("     相关性分数: {:.3}", result.score);
                }
            }
            Err(e) => {
                warn!("图搜索 '{}' 失败: {}", query, e);
            }
        }
    }

    Ok(())
}

/// 演示实体邻居查询
async fn demo_entity_neighbors(client: &Mem0Client) -> Result<()> {
    let entity_ids = vec![
        "person_张三",
        "company_阿里巴巴",
        "地点_杭州",
    ];

    for entity_id in entity_ids {
        match client.get_entity_neighbors(entity_id, Some(2)).await {
            Ok(neighbors) => {
                info!("实体 '{}' 的邻居 ({} 个):", entity_id, neighbors.len());
                for (i, neighbor) in neighbors.iter().enumerate() {
                    info!("  {}. {} ({})", i + 1, neighbor.name, neighbor.entity_type);
                }
            }
            Err(e) => {
                warn!("获取实体 '{}' 邻居失败: {}", entity_id, e);
            }
        }
    }

    Ok(())
}

/// 演示智能记忆融合
async fn demo_memory_fusion(client: &Mem0Client, user_id: &str) -> Result<()> {
    // 首先获取一些记忆 ID
    let all_memories = client.get_all(user_id, None).await?;
    
    if all_memories.len() < 3 {
        warn!("记忆数量不足，无法进行融合演示");
        return Ok(());
    }

    // 选择前3个记忆进行融合
    let memory_ids: Vec<String> = all_memories.iter()
        .take(3)
        .map(|m| m.id.clone())
        .collect();

    info!("融合记忆 IDs: {:?}", memory_ids);

    match client.fuse_memories(&memory_ids, user_id).await {
        Ok(fused_memory) => {
            info!("记忆融合成功:");
            info!("  摘要: {}", fused_memory.summary);
            info!("  实体数量: {}", fused_memory.entities.len());
            info!("  关系数量: {}", fused_memory.relations.len());
            info!("  融合置信度: {:.3}", fused_memory.confidence);
            
            info!("  提取的实体:");
            for (i, entity) in fused_memory.entities.iter().enumerate() {
                info!("    {}. {} ({})", i + 1, entity.name, entity.entity_type);
            }
            
            info!("  提取的关系:");
            for (i, relation) in fused_memory.relations.iter().enumerate() {
                info!("    {}. {} -> {} -> {} (置信度: {:.3})", 
                    i + 1, 
                    relation.source, 
                    relation.relation, 
                    relation.target, 
                    relation.confidence
                );
            }
        }
        Err(e) => {
            warn!("记忆融合失败: {}", e);
        }
    }

    Ok(())
}
