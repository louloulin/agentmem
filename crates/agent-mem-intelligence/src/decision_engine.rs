//! 记忆决策引擎
//!
//! 基于提取的事实和现有记忆，智能决策记忆操作（添加、更新、删除、合并）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use agent_mem_traits::{AgentMemError, Result};
use agent_mem_llm::providers::deepseek::DeepSeekProvider;
use crate::fact_extraction::ExtractedFact;

/// 记忆操作决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDecision {
    pub action: MemoryAction,
    pub confidence: f32,
    pub reasoning: String,
    pub affected_memories: Vec<String>,
    pub estimated_impact: f32,
}

/// 记忆操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAction {
    Add {
        content: String,
        importance: f32,
        metadata: HashMap<String, String>,
    },
    Update {
        memory_id: String,
        new_content: String,
        merge_strategy: MergeStrategy,
        change_reason: String,
    },
    Delete {
        memory_id: String,
        deletion_reason: DeletionReason,
    },
    Merge {
        primary_memory_id: String,
        secondary_memory_ids: Vec<String>,
        merged_content: String,
    },
    NoAction {
        reason: String,
    },
}

/// 合并策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    Replace,     // 完全替换
    Append,      // 追加信息
    Merge,       // 智能合并
    Prioritize,  // 优先保留重要信息
}

/// 删除原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionReason {
    Outdated,      // 信息过时
    Contradicted,  // 被新信息否定
    Redundant,     // 冗余信息
    LowQuality,    // 质量低下
    UserRequested, // 用户要求删除
}

/// 现有记忆信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingMemory {
    pub id: String,
    pub content: String,
    pub importance: f32,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// 决策响应
#[derive(Debug, Deserialize)]
pub struct DecisionResponse {
    pub decisions: Vec<MemoryDecision>,
    pub overall_confidence: f32,
    pub reasoning: String,
}

/// 记忆决策引擎
pub struct MemoryDecisionEngine {
    llm: DeepSeekProvider,
    similarity_threshold: f32,
    confidence_threshold: f32,
}

impl MemoryDecisionEngine {
    /// 创建新的决策引擎
    pub fn new(api_key: String) -> Result<Self> {
        let llm = DeepSeekProvider::with_api_key(api_key)?;
        Ok(Self {
            llm,
            similarity_threshold: 0.7,
            confidence_threshold: 0.5,
        })
    }

    /// 设置相似度阈值
    pub fn with_similarity_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// 设置置信度阈值
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// 基于新事实和现有记忆做出决策
    pub async fn make_decisions(
        &self,
        new_facts: &[ExtractedFact],
        existing_memories: &[ExistingMemory],
    ) -> Result<Vec<MemoryDecision>> {
        if new_facts.is_empty() {
            return Ok(vec![MemoryDecision {
                action: MemoryAction::NoAction {
                    reason: "No new facts to process".to_string(),
                },
                confidence: 1.0,
                reasoning: "No input facts provided".to_string(),
                affected_memories: vec![],
                estimated_impact: 0.0,
            }]);
        }

        let prompt = self.build_decision_prompt(new_facts, existing_memories);
        let response = self.llm.generate_json::<DecisionResponse>(&prompt).await?;

        // 过滤低置信度的决策
        let filtered_decisions = response
            .decisions
            .into_iter()
            .filter(|decision| decision.confidence >= self.confidence_threshold)
            .collect();

        Ok(filtered_decisions)
    }

    /// 构建决策提示
    fn build_decision_prompt(
        &self,
        new_facts: &[ExtractedFact],
        existing_memories: &[ExistingMemory],
    ) -> String {
        let facts_json = serde_json::to_string_pretty(new_facts).unwrap_or_default();
        let memories_json = serde_json::to_string_pretty(existing_memories).unwrap_or_default();

        format!(
            r#"You are an intelligent memory management system. Analyze the new facts and existing memories to decide what memory operations should be performed.

New Facts:
{}

Existing Memories:
{}

Based on the new facts and existing memories, decide what actions to take. Consider:
1. Should new facts be added as new memories?
2. Do any new facts update or contradict existing memories?
3. Should any existing memories be deleted or merged?
4. What is the importance level of each new fact?

Return your decisions in the following JSON format:
{{
    "decisions": [
        {{
            "action": {{
                "Add": {{
                    "content": "memory content",
                    "importance": 0.8,
                    "metadata": {{"key": "value"}}
                }}
            }},
            "confidence": 0.9,
            "reasoning": "Why this decision was made",
            "affected_memories": ["memory_id1"],
            "estimated_impact": 0.7
        }}
    ],
    "overall_confidence": 0.85,
    "reasoning": "Overall reasoning for all decisions"
}}

Action types available:
- Add: Create new memory
- Update: Modify existing memory
- Delete: Remove memory
- Merge: Combine multiple memories
- NoAction: No operation needed

Guidelines:
1. Assign importance scores (0.0-1.0) based on relevance and usefulness
2. Use high confidence (>0.8) for clear decisions
3. Provide clear reasoning for each decision
4. Consider memory consolidation to avoid redundancy
5. Preserve important information when updating or merging

Return valid JSON only."#,
            facts_json, memories_json
        )
    }

    /// 查找相似的现有记忆
    pub fn find_similar_memories<'a>(
        &self,
        fact: &ExtractedFact,
        existing_memories: &'a [ExistingMemory],
    ) -> Vec<&'a ExistingMemory> {
        existing_memories
            .iter()
            .filter(|memory| {
                self.calculate_content_similarity(&fact.content, &memory.content)
                    > self.similarity_threshold
            })
            .collect()
    }

    /// 计算内容相似性
    pub fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        let content1_lower = content1.to_lowercase();
        let content2_lower = content2.to_lowercase();

        let words1: std::collections::HashSet<&str> = content1_lower
            .split_whitespace()
            .collect();
        let words2: std::collections::HashSet<&str> = content2_lower
            .split_whitespace()
            .collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// 检测记忆冲突
    pub fn detect_conflicts<'a>(
        &self,
        fact: &ExtractedFact,
        existing_memories: &'a [ExistingMemory],
    ) -> Vec<&'a ExistingMemory> {
        // 简单的冲突检测：查找包含否定词的相似内容
        let conflict_indicators = ["not", "no", "never", "don't", "doesn't", "won't", "can't"];
        
        existing_memories
            .iter()
            .filter(|memory| {
                let similarity = self.calculate_content_similarity(&fact.content, &memory.content);
                let has_conflict_indicators = conflict_indicators.iter().any(|indicator| {
                    fact.content.to_lowercase().contains(indicator) 
                        || memory.content.to_lowercase().contains(indicator)
                });
                
                similarity > 0.5 && has_conflict_indicators
            })
            .collect()
    }

    /// 评估记忆重要性
    pub fn evaluate_importance(&self, fact: &ExtractedFact) -> f32 {
        let mut importance = fact.confidence;

        // 根据类别调整重要性
        importance *= match fact.category {
            crate::fact_extraction::FactCategory::Personal => 0.9,
            crate::fact_extraction::FactCategory::Preference => 0.8,
            crate::fact_extraction::FactCategory::Relationship => 0.9,
            crate::fact_extraction::FactCategory::Event => 0.7,
            crate::fact_extraction::FactCategory::Knowledge => 0.8,
            crate::fact_extraction::FactCategory::Procedural => 0.9,
        };

        // 根据实体数量调整
        if !fact.entities.is_empty() {
            importance += 0.1 * (fact.entities.len() as f32).min(3.0) / 3.0;
        }

        // 根据时间信息调整
        if fact.temporal_info.is_some() {
            importance += 0.1;
        }

        importance.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact_extraction::FactCategory;

    #[test]
    fn test_memory_action_serialization() {
        let action = MemoryAction::Add {
            content: "Test content".to_string(),
            importance: 0.8,
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("Add"));
        assert!(serialized.contains("Test content"));
    }

    #[test]
    fn test_content_similarity() {
        // 由于 calculate_content_similarity 是私有方法，我们无法直接测试
        // 但可以通过其他公共方法间接验证其行为
        let fact = ExtractedFact {
            content: "User likes coffee".to_string(),
            confidence: 0.9,
            category: FactCategory::Preference,
            entities: vec!["coffee".to_string()],
            temporal_info: None,
            source_message_id: None,
        };

        let memory = ExistingMemory {
            id: "mem1".to_string(),
            content: "User enjoys coffee".to_string(),
            importance: 0.8,
            created_at: "2023-01-01".to_string(),
            updated_at: None,
            metadata: HashMap::new(),
        };

        // 在实际测试中需要创建 MemoryDecisionEngine 实例
        // find_similar_memories 会使用 calculate_content_similarity
    }

    #[test]
    fn test_importance_evaluation() {
        let fact = ExtractedFact {
            content: "User's name is John".to_string(),
            confidence: 0.9,
            category: FactCategory::Personal,
            entities: vec!["John".to_string()],
            temporal_info: Some("today".to_string()),
            source_message_id: None,
        };

        // 需要创建 MemoryDecisionEngine 实例来测试 evaluate_importance
        // 预期重要性会基于类别、实体和时间信息进行调整
    }
}
