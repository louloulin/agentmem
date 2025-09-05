//! 智能记忆处理器
//!
//! 整合事实提取和决策引擎，提供完整的智能记忆处理能力

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use agent_mem_traits::{AgentMemError, Result};
use crate::fact_extraction::{FactExtractor, ExtractedFact, Message};
use crate::decision_engine::{MemoryDecisionEngine, MemoryDecision, ExistingMemory, ConflictDetection};

/// 智能处理结果（增强版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentProcessingResult {
    pub extracted_facts: Vec<ExtractedFact>,
    pub memory_decisions: Vec<MemoryDecision>,
    pub conflict_detections: Vec<ConflictDetection>,
    pub processing_stats: ProcessingStats,
    pub recommendations: Vec<String>,
    pub quality_metrics: QualityMetrics,
    pub processing_insights: ProcessingInsights,
}

/// 质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub average_fact_confidence: f32,
    pub average_decision_confidence: f32,
    pub conflict_rate: f32,
    pub fact_diversity_score: f32,
    pub processing_efficiency: f32,
}

/// 处理洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingInsights {
    pub dominant_fact_categories: Vec<String>,
    pub memory_growth_prediction: f32,
    pub suggested_optimizations: Vec<String>,
    pub attention_areas: Vec<String>,
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

        // 5. 检测冲突
        let mut conflict_detections = Vec::new();
        for fact in &extracted_facts {
            let conflict = self
                .decision_engine
                .detect_conflicts(fact, existing_memories)
                .await?;
            if conflict.has_conflict {
                conflict_detections.push(conflict);
            }
        }

        // 6. 生成记忆决策
        let memory_decisions = self
            .decision_engine
            .make_decisions(&extracted_facts, existing_memories)
            .await?;

        // 7. 生成推荐
        let recommendations = self.generate_recommendations(&extracted_facts, &memory_decisions);

        // 8. 计算质量指标
        let quality_metrics = self.calculate_quality_metrics(&extracted_facts, &memory_decisions, &conflict_detections);

        // 9. 生成处理洞察
        let processing_insights = self.generate_processing_insights(&extracted_facts, &memory_decisions, existing_memories);

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
            conflict_detections,
            processing_stats,
            recommendations,
            quality_metrics,
            processing_insights,
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

    /// 计算质量指标
    fn calculate_quality_metrics(
        &self,
        facts: &[ExtractedFact],
        decisions: &[MemoryDecision],
        conflicts: &[ConflictDetection],
    ) -> QualityMetrics {
        let average_fact_confidence = if facts.is_empty() {
            0.0
        } else {
            facts.iter().map(|f| f.confidence).sum::<f32>() / facts.len() as f32
        };

        let average_decision_confidence = if decisions.is_empty() {
            0.0
        } else {
            decisions.iter().map(|d| d.confidence).sum::<f32>() / decisions.len() as f32
        };

        let conflict_rate = if facts.is_empty() {
            0.0
        } else {
            conflicts.len() as f32 / facts.len() as f32
        };

        // 计算事实多样性分数（基于不同类别的数量）
        let unique_categories: std::collections::HashSet<_> = facts.iter()
            .map(|f| std::mem::discriminant(&f.category))
            .collect();
        let fact_diversity_score = if facts.is_empty() {
            0.0
        } else {
            unique_categories.len() as f32 / 15.0 // 15是总类别数
        };

        // 处理效率（基于事实数量和决策数量的比率）
        let processing_efficiency = if facts.is_empty() {
            1.0
        } else {
            decisions.len() as f32 / facts.len() as f32
        };

        QualityMetrics {
            average_fact_confidence,
            average_decision_confidence,
            conflict_rate,
            fact_diversity_score,
            processing_efficiency,
        }
    }

    /// 生成处理洞察
    fn generate_processing_insights(
        &self,
        facts: &[ExtractedFact],
        decisions: &[MemoryDecision],
        existing_memories: &[ExistingMemory],
    ) -> ProcessingInsights {
        // 统计主要事实类别
        let mut category_counts = std::collections::HashMap::new();
        for fact in facts {
            let category_name = format!("{:?}", fact.category);
            *category_counts.entry(category_name).or_insert(0) += 1;
        }

        let mut dominant_fact_categories: Vec<_> = category_counts.into_iter().collect();
        dominant_fact_categories.sort_by(|a, b| b.1.cmp(&a.1));
        let dominant_fact_categories: Vec<String> = dominant_fact_categories
            .into_iter()
            .take(3)
            .map(|(category, _)| category)
            .collect();

        // 预测记忆增长
        let memory_growth_prediction = if existing_memories.is_empty() {
            facts.len() as f32
        } else {
            let growth_rate = facts.len() as f32 / existing_memories.len() as f32;
            growth_rate.min(2.0) // 限制最大增长率
        };

        // 生成优化建议
        let mut suggested_optimizations = Vec::new();
        if facts.len() > 10 {
            suggested_optimizations.push("Consider increasing batch processing size".to_string());
        }
        if decisions.iter().any(|d| d.confidence < 0.5) {
            suggested_optimizations.push("Review low-confidence decisions".to_string());
        }

        // 生成注意区域
        let mut attention_areas = Vec::new();
        if facts.iter().any(|f| matches!(f.category, crate::fact_extraction::FactCategory::Personal)) {
            attention_areas.push("Personal information detected - ensure privacy compliance".to_string());
        }
        if facts.iter().any(|f| matches!(f.category, crate::fact_extraction::FactCategory::Financial)) {
            attention_areas.push("Financial information detected - ensure security measures".to_string());
        }

        ProcessingInsights {
            dominant_fact_categories,
            memory_growth_prediction,
            suggested_optimizations,
            attention_areas,
        }
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
        if personal_facts > 0 {
            recommendations.push(
                "Personal facts detected. Ensure privacy compliance.".to_string()
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
                entities: vec![],
                temporal_info: None,
                source_message_id: None,
                metadata: std::collections::HashMap::new(),
            },
            ExtractedFact {
                content: "User lives in New York".to_string(),
                confidence: 0.8,
                category: FactCategory::Personal,
                entities: vec![],
                temporal_info: None,
                source_message_id: None,
                metadata: std::collections::HashMap::new(),
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

        // 创建处理器实例进行测试
        let processor = IntelligentMemoryProcessor::new("test-key".to_string()).unwrap();
        let recommendations = processor.generate_recommendations(&facts, &decisions);

        // 验证推荐生成
        assert!(!recommendations.is_empty());

        // 应该包含关于低置信度决策的推荐
        assert!(recommendations.iter().any(|r| r.contains("low confidence")));

        // 应该包含关于个人信息的推荐
        assert!(recommendations.iter().any(|r| r.contains("Personal facts") || r.contains("privacy")));
    }
}
