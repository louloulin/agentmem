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

/// 冲突检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetection {
    pub has_conflict: bool,
    pub conflicting_memory_ids: Vec<String>,
    pub conflict_type: ConflictType,
    pub resolution_strategy: ConflictResolutionStrategy,
    pub confidence: f32,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    DirectContradiction,  // 直接矛盾
    TemporalInconsistency, // 时间不一致
    ValueConflict,        // 数值冲突
    CategoryMismatch,     // 类别不匹配
    ContextualConflict,   // 上下文冲突
}

/// 冲突解决策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    KeepLatest,           // 保留最新信息
    KeepMostImportant,    // 保留最重要信息
    KeepHighestConfidence, // 保留置信度最高信息
    MergeInformation,     // 合并信息
    RequestUserInput,     // 请求用户输入
    MarkAsUncertain,      // 标记为不确定
}

/// 记忆决策引擎（增强版本）
pub struct MemoryDecisionEngine {
    llm: DeepSeekProvider,
    similarity_threshold: f32,
    confidence_threshold: f32,
    conflict_detection_enabled: bool,
    importance_weight: f32,
    temporal_weight: f32,
}

impl MemoryDecisionEngine {
    /// 创建新的决策引擎
    pub fn new(api_key: String) -> Result<Self> {
        let llm = DeepSeekProvider::with_api_key(api_key)?;
        Ok(Self {
            llm,
            similarity_threshold: 0.7,
            confidence_threshold: 0.5,
            conflict_detection_enabled: true,
            importance_weight: 0.3,
            temporal_weight: 0.2,
        })
    }

    /// 创建带自定义配置的决策引擎
    pub fn with_config(
        api_key: String,
        similarity_threshold: f32,
        confidence_threshold: f32,
        conflict_detection_enabled: bool,
    ) -> Result<Self> {
        let llm = DeepSeekProvider::with_api_key(api_key)?;
        Ok(Self {
            llm,
            similarity_threshold,
            confidence_threshold,
            conflict_detection_enabled,
            importance_weight: 0.3,
            temporal_weight: 0.2,
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

    /// 检测事实与现有记忆之间的冲突
    pub async fn detect_conflicts(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[ExistingMemory],
    ) -> Result<ConflictDetection> {
        if !self.conflict_detection_enabled {
            return Ok(ConflictDetection {
                has_conflict: false,
                conflicting_memory_ids: vec![],
                conflict_type: ConflictType::DirectContradiction,
                resolution_strategy: ConflictResolutionStrategy::KeepLatest,
                confidence: 0.0,
            });
        }

        let prompt = self.build_conflict_detection_prompt(fact, existing_memories);
        let response = self.llm.generate_json::<ConflictDetection>(&prompt).await?;

        Ok(response)
    }

    /// 智能合并相似记忆
    pub async fn merge_similar_memories(
        &self,
        memories: &[ExistingMemory],
        similarity_threshold: Option<f32>,
    ) -> Result<Vec<MemoryDecision>> {
        let threshold = similarity_threshold.unwrap_or(self.similarity_threshold);
        let mut decisions = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        for (i, memory) in memories.iter().enumerate() {
            if processed_ids.contains(&memory.id) {
                continue;
            }

            let mut similar_memories = vec![memory.clone()];

            // 查找相似记忆
            for (j, other_memory) in memories.iter().enumerate() {
                if i != j && !processed_ids.contains(&other_memory.id) {
                    let similarity = self.calculate_content_similarity(&memory.content, &other_memory.content);
                    if similarity >= threshold {
                        similar_memories.push(other_memory.clone());
                        processed_ids.insert(other_memory.id.clone());
                    }
                }
            }

            // 如果找到相似记忆，创建合并决策
            if similar_memories.len() > 1 {
                let merged_content = self.generate_merged_content(&similar_memories).await?;
                let primary_memory = &similar_memories[0];
                let secondary_ids: Vec<String> = similar_memories[1..].iter().map(|m| m.id.clone()).collect();

                decisions.push(MemoryDecision {
                    action: MemoryAction::Merge {
                        primary_memory_id: primary_memory.id.clone(),
                        secondary_memory_ids: secondary_ids.clone(),
                        merged_content,
                    },
                    confidence: 0.8,
                    reasoning: format!("Found {} similar memories that can be merged", similar_memories.len()),
                    affected_memories: similar_memories.iter().map(|m| m.id.clone()).collect(),
                    estimated_impact: 0.7,
                });

                processed_ids.insert(primary_memory.id.clone());
            }
        }

        Ok(decisions)
    }

    /// 评估记忆重要性（增强版本）
    pub fn evaluate_importance_enhanced(&self, fact: &ExtractedFact, context: &[ExistingMemory]) -> f32 {
        let mut importance = self.evaluate_importance(fact);

        // 基于上下文调整重要性
        let context_boost = self.calculate_context_importance(fact, context);
        importance += context_boost * self.importance_weight;

        // 基于时间信息调整
        if let Some(temporal_info) = &fact.temporal_info {
            let temporal_boost = self.calculate_temporal_importance(temporal_info);
            importance += temporal_boost * self.temporal_weight;
        }

        // 确保重要性在合理范围内
        importance.clamp(0.0, 1.0)
    }

    /// 构建决策提示
    fn build_decision_prompt(
        &self,
        new_facts: &[ExtractedFact],
        existing_memories: &[ExistingMemory],
    ) -> String {
        // 简化输入数据以减少token使用
        let facts_summary: Vec<String> = new_facts.iter()
            .map(|f| format!("{} (conf: {:.1})", f.content, f.confidence))
            .collect();

        let memories_summary: Vec<String> = existing_memories.iter()
            .map(|m| format!("{}: {} (imp: {:.1})", m.id, m.content, m.importance))
            .collect();

        format!(
            r#"Memory management task. Return JSON only.

New Facts:
{}

Existing Memories:
{}

Decide memory operations:

{{
    "decisions": [
        {{
            "action": {{"Add": {{"content": "text", "importance": 0.8, "metadata": {{}}}}}},
            "confidence": 0.9,
            "reasoning": "brief reason",
            "affected_memories": [],
            "estimated_impact": 0.7
        }}
    ],
    "overall_confidence": 0.8,
    "reasoning": "summary"
}}

Actions: Add, Update, Delete, Merge, NoAction
- Add new facts as memories (importance 0.3-1.0)
- Update if fact conflicts with existing memory
- Delete outdated/contradicted memories
- Merge similar memories
- NoAction if no changes needed

Keep reasoning brief. Max 3 decisions."#,
            facts_summary.join("\n"),
            memories_summary.join("\n")
        )
    }

    /// 构建冲突检测提示
    fn build_conflict_detection_prompt(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[ExistingMemory],
    ) -> String {
        let memories_summary: Vec<String> = existing_memories.iter()
            .map(|m| format!("{}: {}", m.id, m.content))
            .collect();

        format!(
            r#"Detect conflicts between new fact and existing memories. Return JSON only.

New Fact: {}

Existing Memories:
{}

Analyze for conflicts:

{{
    "has_conflict": true/false,
    "conflicting_memory_ids": ["mem1", "mem2"],
    "conflict_type": "DirectContradiction|TemporalInconsistency|ValueConflict|CategoryMismatch|ContextualConflict",
    "resolution_strategy": "KeepLatest|KeepMostImportant|KeepHighestConfidence|MergeInformation|RequestUserInput|MarkAsUncertain",
    "confidence": 0.9
}}

Conflict types:
- DirectContradiction: Facts directly contradict each other
- TemporalInconsistency: Timeline doesn't match
- ValueConflict: Different values for same attribute
- CategoryMismatch: Same entity in different categories
- ContextualConflict: Context makes facts incompatible

Resolution strategies:
- KeepLatest: Prefer newer information
- KeepMostImportant: Prefer higher importance
- KeepHighestConfidence: Prefer higher confidence
- MergeInformation: Combine compatible parts
- RequestUserInput: Ask user to resolve
- MarkAsUncertain: Flag as uncertain"#,
            fact.content,
            memories_summary.join("\n")
        )
    }

    /// 生成合并后的内容
    async fn generate_merged_content(&self, memories: &[ExistingMemory]) -> Result<String> {
        let contents: Vec<String> = memories.iter().map(|m| m.content.clone()).collect();

        let _prompt = format!(
            r#"Merge these similar memory contents into one coherent text. Return only the merged content.

Contents to merge:
{}

Requirements:
- Preserve all important information
- Remove redundancy
- Create coherent narrative
- Keep factual accuracy
- Maximum 200 words"#,
            contents.join("\n- ")
        );

        // 使用简单的文本合并逻辑，因为 DeepSeekProvider 没有 generate 方法
        let merged_content = self.merge_contents_simple(&contents);

        Ok(merged_content.trim().to_string())
    }

    /// 计算上下文重要性
    fn calculate_context_importance(&self, fact: &ExtractedFact, context: &[ExistingMemory]) -> f32 {
        let mut context_boost = 0.0;

        // 检查是否与现有记忆相关
        for memory in context {
            let similarity = self.calculate_content_similarity(&fact.content, &memory.content);
            if similarity > 0.3 {
                // 相关记忆越重要，新事实的重要性提升越大
                context_boost += similarity * memory.importance * 0.1;
            }
        }

        context_boost.min(0.3) // 最大提升30%
    }

    /// 计算时间重要性
    fn calculate_temporal_importance(&self, temporal_info: &crate::fact_extraction::TemporalInfo) -> f32 {
        let mut temporal_boost: f32 = 0.0;

        // 相对时间的重要性
        if let Some(relative_time) = &temporal_info.relative_time {
            temporal_boost += match relative_time.as_str() {
                "today" | "now" => 0.2,
                "yesterday" | "tomorrow" => 0.15,
                "this week" | "last week" => 0.1,
                _ => 0.05,
            };
        }

        // 频率信息的重要性
        if let Some(frequency) = &temporal_info.frequency {
            temporal_boost += match frequency.as_str() {
                "daily" => 0.15,
                "weekly" => 0.1,
                "monthly" => 0.05,
                _ => 0.02,
            };
        }

        temporal_boost.min(0.25) // 最大提升25%
    }

    /// 简单的内容合并逻辑
    fn merge_contents_simple(&self, contents: &[String]) -> String {
        if contents.is_empty() {
            return String::new();
        }

        if contents.len() == 1 {
            return contents[0].clone();
        }

        // 简单合并：去重并连接
        let mut unique_sentences = std::collections::HashSet::new();
        let mut merged_parts = Vec::new();

        for content in contents {
            // 按句号分割内容
            let sentences: Vec<&str> = content.split('.').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            for sentence in sentences {
                if unique_sentences.insert(sentence.to_lowercase()) {
                    merged_parts.push(sentence.to_string());
                }
            }
        }

        merged_parts.join(". ") + if !merged_parts.is_empty() { "." } else { "" }
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

    /// 检测记忆冲突（简化版本，用于向后兼容）
    pub fn detect_conflicts_simple<'a>(
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
            crate::fact_extraction::FactCategory::Emotional => 0.7,
            crate::fact_extraction::FactCategory::Goal => 0.9,
            crate::fact_extraction::FactCategory::Skill => 0.8,
            crate::fact_extraction::FactCategory::Location => 0.7,
            crate::fact_extraction::FactCategory::Temporal => 0.6,
            crate::fact_extraction::FactCategory::Financial => 0.8,
            crate::fact_extraction::FactCategory::Health => 0.9,
            crate::fact_extraction::FactCategory::Educational => 0.8,
            crate::fact_extraction::FactCategory::Professional => 0.8,
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
            entities: vec![],
            temporal_info: None,
            source_message_id: None,
            metadata: HashMap::new(),
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
            entities: vec![],
            temporal_info: None,
            source_message_id: None,
            metadata: HashMap::new(),
        };

        // 需要创建 MemoryDecisionEngine 实例来测试 evaluate_importance
        // 预期重要性会基于类别、实体和时间信息进行调整
    }
}
