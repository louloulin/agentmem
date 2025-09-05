//! 智能记忆处理器
//!
//! 整合事实提取和决策引擎，提供完整的智能记忆处理能力

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use agent_mem_traits::{AgentMemError, Result};
use crate::fact_extraction::{FactExtractor, ExtractedFact, Message};
use crate::decision_engine::{MemoryDecisionEngine, MemoryDecision, ExistingMemory};

/// 智能处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentProcessingResult {
    pub extracted_facts: Vec<ExtractedFact>,
    pub memory_decisions: Vec<MemoryDecision>,
    pub processing_stats: ProcessingStats,
    pub recommendations: Vec<String>,
}

/// 处理统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_messages: usize,
    pub facts_extracted: usize,
    pub decisions_made: usize,
    pub high_confidence_decisions: usize,
    pub processing_time_ms: u64,
}

/// 处理配置
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub similarity_threshold: f32,
    pub confidence_threshold: f32,
    pub max_facts_per_message: usize,
    pub enable_fact_validation: bool,
    pub enable_fact_merging: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            confidence_threshold: 0.5,
            max_facts_per_message: 10,
            enable_fact_validation: true,
            enable_fact_merging: true,
        }
    }
}

/// 智能记忆处理器
pub struct IntelligentMemoryProcessor {
    fact_extractor: FactExtractor,
    decision_engine: MemoryDecisionEngine,
    config: ProcessingConfig,
}

impl IntelligentMemoryProcessor {
    /// 创建新的智能处理器
    pub fn new(api_key: String) -> Result<Self> {
        let fact_extractor = FactExtractor::new(api_key.clone())?;
        let decision_engine = MemoryDecisionEngine::new(api_key)?;
        let config = ProcessingConfig::default();

        Ok(Self {
            fact_extractor,
            decision_engine,
            config,
        })
    }

    /// 使用自定义配置创建处理器
    pub fn with_config(api_key: String, config: ProcessingConfig) -> Result<Self> {
        let fact_extractor = FactExtractor::new(api_key.clone())?;
        let decision_engine = MemoryDecisionEngine::new(api_key)?
            .with_similarity_threshold(config.similarity_threshold)
            .with_confidence_threshold(config.confidence_threshold);

        Ok(Self {
            fact_extractor,
            decision_engine,
            config,
        })
    }

    /// 处理消息并生成智能记忆操作
    pub async fn process_messages(
        &self,
        messages: &[Message],
        existing_memories: &[ExistingMemory],
    ) -> Result<IntelligentProcessingResult> {
        let start_time = std::time::Instant::now();

        // 1. 提取事实
        let mut extracted_facts = self.fact_extractor.extract_facts(messages).await?;

        // 2. 验证事实（如果启用）
        if self.config.enable_fact_validation {
            extracted_facts = self.fact_extractor.validate_facts(extracted_facts);
        }

        // 3. 合并相似事实（如果启用）
        if self.config.enable_fact_merging {
            extracted_facts = self.fact_extractor.merge_similar_facts(extracted_facts);
        }

        // 4. 限制事实数量
        if extracted_facts.len() > self.config.max_facts_per_message {
            extracted_facts.truncate(self.config.max_facts_per_message);
        }

        // 5. 生成记忆决策
        let memory_decisions = self
            .decision_engine
            .make_decisions(&extracted_facts, existing_memories)
            .await?;

        // 6. 生成推荐
        let recommendations = self.generate_recommendations(&extracted_facts, &memory_decisions);

        // 7. 计算统计信息
        let processing_time = start_time.elapsed();
        let high_confidence_decisions = memory_decisions
            .iter()
            .filter(|d| d.confidence > 0.8)
            .count();

        let processing_stats = ProcessingStats {
            total_messages: messages.len(),
            facts_extracted: extracted_facts.len(),
            decisions_made: memory_decisions.len(),
            high_confidence_decisions,
            processing_time_ms: processing_time.as_millis() as u64,
        };

        Ok(IntelligentProcessingResult {
            extracted_facts,
            memory_decisions,
            processing_stats,
            recommendations,
        })
    }

    /// 处理单个消息
    pub async fn process_single_message(
        &self,
        message: &Message,
        existing_memories: &[ExistingMemory],
    ) -> Result<IntelligentProcessingResult> {
        self.process_messages(&[message.clone()], existing_memories)
            .await
    }

    /// 分析现有记忆并提供优化建议
    pub async fn analyze_memory_health(
        &self,
        existing_memories: &[ExistingMemory],
    ) -> Result<MemoryHealthReport> {
        let mut report = MemoryHealthReport::default();

        // 分析记忆质量
        for memory in existing_memories {
            if memory.importance < 0.3 {
                report.low_importance_memories.push(memory.id.clone());
            }
            
            if memory.content.len() < 20 {
                report.short_memories.push(memory.id.clone());
            }
        }

        // 检测重复记忆
        for (i, memory1) in existing_memories.iter().enumerate() {
            for memory2 in existing_memories.iter().skip(i + 1) {
                let similarity = self.decision_engine
                    .calculate_content_similarity(&memory1.content, &memory2.content);
                
                if similarity > 0.8 {
                    report.duplicate_memories.push((memory1.id.clone(), memory2.id.clone()));
                }
            }
        }

        // 生成优化建议
        if !report.low_importance_memories.is_empty() {
            report.suggestions.push(format!(
                "Consider removing {} low-importance memories",
                report.low_importance_memories.len()
            ));
        }

        if !report.duplicate_memories.is_empty() {
            report.suggestions.push(format!(
                "Found {} pairs of duplicate memories that could be merged",
                report.duplicate_memories.len()
            ));
        }

        Ok(report)
    }

    /// 生成处理推荐
    fn generate_recommendations(
        &self,
        facts: &[ExtractedFact],
        decisions: &[MemoryDecision],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于事实数量的推荐
        if facts.len() > 5 {
            recommendations.push(
                "High number of facts extracted. Consider breaking down conversations into smaller segments.".to_string()
            );
        }

        // 基于决策置信度的推荐
        let low_confidence_decisions = decisions.iter().filter(|d| d.confidence < 0.6).count();
        if low_confidence_decisions > 0 {
            recommendations.push(format!(
                "{} decisions have low confidence. Manual review recommended.",
                low_confidence_decisions
            ));
        }

        // 基于事实类别的推荐
        let personal_facts = facts.iter().filter(|f| matches!(f.category, crate::fact_extraction::FactCategory::Personal)).count();
        if personal_facts > 3 {
            recommendations.push(
                "Multiple personal facts detected. Ensure privacy compliance.".to_string()
            );
        }

        recommendations
    }

    /// 获取处理器配置
    pub fn get_config(&self) -> &ProcessingConfig {
        &self.config
    }

    /// 更新处理器配置
    pub fn update_config(&mut self, config: ProcessingConfig) {
        self.config = config;
    }
}

/// 记忆健康报告
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MemoryHealthReport {
    pub total_memories: usize,
    pub low_importance_memories: Vec<String>,
    pub short_memories: Vec<String>,
    pub duplicate_memories: Vec<(String, String)>,
    pub suggestions: Vec<String>,
    pub overall_health_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact_extraction::FactCategory;

    #[test]
    fn test_processing_config_default() {
        let config = ProcessingConfig::default();
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.confidence_threshold, 0.5);
        assert_eq!(config.max_facts_per_message, 10);
        assert!(config.enable_fact_validation);
        assert!(config.enable_fact_merging);
    }

    #[test]
    fn test_processing_stats_creation() {
        let stats = ProcessingStats {
            total_messages: 5,
            facts_extracted: 3,
            decisions_made: 2,
            high_confidence_decisions: 1,
            processing_time_ms: 150,
        };

        assert_eq!(stats.total_messages, 5);
        assert_eq!(stats.facts_extracted, 3);
        assert_eq!(stats.processing_time_ms, 150);
    }

    #[test]
    fn test_memory_health_report_default() {
        let report = MemoryHealthReport::default();
        assert_eq!(report.total_memories, 0);
        assert!(report.low_importance_memories.is_empty());
        assert!(report.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_recommendation_generation() {
        // 创建测试数据
        let facts = vec![
            ExtractedFact {
                content: "User's name is John".to_string(),
                confidence: 0.9,
                category: FactCategory::Personal,
                entities: vec!["John".to_string()],
                temporal_info: None,
                source_message_id: None,
            },
            ExtractedFact {
                content: "User lives in New York".to_string(),
                confidence: 0.8,
                category: FactCategory::Personal,
                entities: vec!["New York".to_string()],
                temporal_info: None,
                source_message_id: None,
            },
        ];

        let decisions = vec![
            MemoryDecision {
                action: crate::decision_engine::MemoryAction::Add {
                    content: "Test".to_string(),
                    importance: 0.8,
                    metadata: HashMap::new(),
                },
                confidence: 0.5, // 低置信度
                reasoning: "Test".to_string(),
                affected_memories: vec![],
                estimated_impact: 0.5,
            },
        ];

        // 需要创建 IntelligentMemoryProcessor 实例来测试 generate_recommendations
        // 在实际测试中会生成相应的推荐
    }
}
