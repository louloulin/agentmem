//! 冲突解决系统
//!
//! 提供智能记忆冲突检测和解决功能，包括：
//! - 语义冲突检测
//! - 时间冲突检测
//! - 智能合并策略
//! - 置信度评估

use crate::fact_extraction::{Entity, Relation, StructuredFact};
use crate::similarity::SemanticSimilarity;
use agent_mem_core::Memory;
use agent_mem_llm::LLMProvider;
use agent_mem_traits::{Message, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// 语义冲突 - 内容相互矛盾
    Semantic,
    /// 时间冲突 - 时间信息不一致
    Temporal,
    /// 实体冲突 - 同一实体的不同描述
    Entity,
    /// 关系冲突 - 关系信息矛盾
    Relation,
    /// 重复冲突 - 内容重复
    Duplicate,
}

/// 冲突检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetection {
    /// 冲突ID
    pub id: String,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 涉及的记忆ID
    pub memory_ids: Vec<String>,
    /// 冲突描述
    pub description: String,
    /// 冲突严重程度 (0.0-1.0)
    pub severity: f32,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 建议的解决方案
    pub suggested_resolution: ResolutionStrategy,
    /// 检测时间
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// 解决策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// 保留最新的记忆
    KeepLatest,
    /// 保留置信度最高的记忆
    KeepHighestConfidence,
    /// 合并记忆内容
    Merge {
        strategy: MergeStrategy,
        merged_content: String,
    },
    /// 标记为冲突，需要人工处理
    MarkForManualReview,
    /// 删除重复项
    RemoveDuplicates { keep_memory_id: String },
}

/// 合并策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// 简单连接
    Concatenate,
    /// 智能合并
    Intelligent,
    /// 保留关键信息
    KeepKeyInfo,
    /// 时间序列合并
    Temporal,
}

/// 冲突解决结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// 解决的冲突ID
    pub conflict_id: String,
    /// 应用的解决策略
    pub applied_strategy: ResolutionStrategy,
    /// 解决结果
    pub resolution_result: ResolutionResult,
    /// 解决时间
    pub resolved_at: chrono::DateTime<chrono::Utc>,
    /// 解决置信度
    pub resolution_confidence: f32,
}

/// 解决结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionResult {
    /// 成功解决
    Success {
        updated_memories: Vec<String>,
        deleted_memories: Vec<String>,
    },
    /// 需要人工干预
    RequiresManualIntervention { reason: String },
    /// 解决失败
    Failed { error: String },
}

/// 冲突解决器
pub struct ConflictResolver {
    llm: Arc<dyn LLMProvider + Send + Sync>,
    similarity: SemanticSimilarity,
    config: ConflictResolverConfig,
}

/// 冲突解决器配置
#[derive(Debug, Clone)]
pub struct ConflictResolverConfig {
    /// 语义相似度阈值
    pub semantic_similarity_threshold: f32,
    /// 时间冲突检测窗口（小时）
    pub temporal_conflict_window_hours: i64,
    /// 自动解决阈值
    pub auto_resolution_threshold: f32,
    /// 最大合并记忆数量
    pub max_merge_memories: usize,
}

impl Default for ConflictResolverConfig {
    fn default() -> Self {
        Self {
            semantic_similarity_threshold: 0.85,
            temporal_conflict_window_hours: 24,
            auto_resolution_threshold: 0.8,
            max_merge_memories: 5,
        }
    }
}

impl ConflictResolver {
    /// 创建新的冲突解决器
    pub fn new(llm: Arc<dyn LLMProvider + Send + Sync>, config: ConflictResolverConfig) -> Self {
        let similarity = SemanticSimilarity::default();

        Self {
            llm,
            similarity,
            config,
        }
    }

    /// 检测记忆冲突
    pub async fn detect_conflicts(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        info!(
            "开始检测冲突，新记忆: {}, 现有记忆: {}",
            new_memories.len(),
            existing_memories.len()
        );

        let mut conflicts = Vec::new();

        // 1. 检测语义冲突
        let semantic_conflicts = self
            .detect_semantic_conflicts(new_memories, existing_memories)
            .await?;
        conflicts.extend(semantic_conflicts);

        // 2. 检测时间冲突
        let temporal_conflicts = self
            .detect_temporal_conflicts(new_memories, existing_memories)
            .await?;
        conflicts.extend(temporal_conflicts);

        // 3. 检测重复内容
        let duplicate_conflicts = self
            .detect_duplicates(new_memories, existing_memories)
            .await?;
        conflicts.extend(duplicate_conflicts);

        info!("检测到 {} 个冲突", conflicts.len());
        Ok(conflicts)
    }

    /// 解决记忆冲突
    pub async fn resolve_memory_conflicts(
        &self,
        conflicts: &[ConflictDetection],
        memories: &[Memory],
    ) -> Result<Vec<ConflictResolution>> {
        info!("开始解决 {} 个冲突", conflicts.len());

        let mut resolutions = Vec::new();

        for conflict in conflicts {
            let resolution = self.resolve_single_conflict(conflict, memories).await?;
            resolutions.push(resolution);
        }

        info!("完成冲突解决，生成 {} 个解决方案", resolutions.len());
        Ok(resolutions)
    }

    /// 检测语义冲突
    async fn detect_semantic_conflicts(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        let mut conflicts = Vec::new();

        for new_memory in new_memories {
            for existing_memory in existing_memories {
                // 计算语义相似度
                let similarity = self
                    .similarity
                    .calculate_similarity(&new_memory.content, &existing_memory.content)
                    .await?;

                // 如果相似度高但内容不同，可能存在冲突
                if similarity > self.config.semantic_similarity_threshold {
                    let conflict_severity = self
                        .analyze_semantic_conflict(new_memory, existing_memory)
                        .await?;

                    if conflict_severity > 0.5 {
                        let conflict = ConflictDetection {
                            id: format!("semantic_conflict_{}", conflicts.len()),
                            conflict_type: ConflictType::Semantic,
                            memory_ids: vec![new_memory.id.clone(), existing_memory.id.clone()],
                            description: format!(
                                "语义冲突：新记忆 '{}' 与现有记忆 '{}' 存在矛盾",
                                new_memory.content.chars().take(50).collect::<String>(),
                                existing_memory.content.chars().take(50).collect::<String>()
                            ),
                            severity: conflict_severity,
                            confidence: similarity,
                            suggested_resolution: self
                                .suggest_resolution_strategy(
                                    &ConflictType::Semantic,
                                    &[new_memory.clone(), existing_memory.clone()],
                                )
                                .await?,
                            detected_at: chrono::Utc::now(),
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// 检测时间冲突
    async fn detect_temporal_conflicts(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        let mut conflicts = Vec::new();

        // 简化的时间冲突检测逻辑
        for new_memory in new_memories {
            for existing_memory in existing_memories {
                let time_diff = (new_memory.created_at - existing_memory.created_at)
                    .num_hours()
                    .abs();

                if time_diff <= self.config.temporal_conflict_window_hours {
                    // 检查是否存在时间相关的冲突
                    if self.has_temporal_conflict(&new_memory.content, &existing_memory.content) {
                        let conflict = ConflictDetection {
                            id: format!("temporal_conflict_{}", conflicts.len()),
                            conflict_type: ConflictType::Temporal,
                            memory_ids: vec![new_memory.id.clone(), existing_memory.id.clone()],
                            description: "时间冲突：记忆中的时间信息不一致".to_string(),
                            severity: 0.7,
                            confidence: 0.8,
                            suggested_resolution: ResolutionStrategy::KeepLatest,
                            detected_at: chrono::Utc::now(),
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// 检测重复内容
    async fn detect_duplicates(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        let mut conflicts = Vec::new();

        for new_memory in new_memories {
            for existing_memory in existing_memories {
                let similarity = self
                    .similarity
                    .calculate_similarity(&new_memory.content, &existing_memory.content)
                    .await?;

                // 高相似度且内容长度相近，可能是重复
                if similarity > 0.95 {
                    let length_ratio =
                        (new_memory.content.len() as f32) / (existing_memory.content.len() as f32);
                    if length_ratio > 0.8 && length_ratio < 1.2 {
                        let conflict = ConflictDetection {
                            id: format!("duplicate_conflict_{}", conflicts.len()),
                            conflict_type: ConflictType::Duplicate,
                            memory_ids: vec![new_memory.id.clone(), existing_memory.id.clone()],
                            description: "重复内容：发现相似的记忆内容".to_string(),
                            severity: 0.6,
                            confidence: similarity,
                            suggested_resolution: ResolutionStrategy::RemoveDuplicates {
                                keep_memory_id: if new_memory.created_at
                                    > existing_memory.created_at
                                {
                                    new_memory.id.clone()
                                } else {
                                    existing_memory.id.clone()
                                },
                            },
                            detected_at: chrono::Utc::now(),
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// 分析语义冲突严重程度
    async fn analyze_semantic_conflict(&self, memory1: &Memory, memory2: &Memory) -> Result<f32> {
        let prompt = format!(
            r#"请分析以下两段记忆内容是否存在语义冲突，并评估冲突严重程度（0.0-1.0）：

记忆1: {}
记忆2: {}

请返回JSON格式：
{{
  "has_conflict": true/false,
  "severity": 0.0-1.0,
  "explanation": "冲突分析说明"
}}

只返回JSON，不要其他解释："#,
            memory1.content, memory2.content
        );

        let response = self.llm.generate(&[Message::user(&prompt)]).await?;

        // 解析响应
        #[derive(Deserialize)]
        struct ConflictAnalysis {
            has_conflict: bool,
            severity: f32,
            explanation: String,
        }

        match serde_json::from_str::<ConflictAnalysis>(&response) {
            Ok(analysis) => {
                if analysis.has_conflict {
                    Ok(analysis.severity.clamp(0.0, 1.0))
                } else {
                    Ok(0.0)
                }
            }
            Err(_) => {
                warn!("Failed to parse conflict analysis response");
                Ok(0.5) // 默认中等冲突
            }
        }
    }

    /// 检查是否存在时间冲突
    fn has_temporal_conflict(&self, content1: &str, content2: &str) -> bool {
        // 简化的时间冲突检测
        // 实际实现中可以使用更复杂的时间实体识别
        let time_keywords = ["昨天", "今天", "明天", "上周", "下周", "去年", "今年"];

        let has_time1 = time_keywords
            .iter()
            .any(|&keyword| content1.contains(keyword));
        let has_time2 = time_keywords
            .iter()
            .any(|&keyword| content2.contains(keyword));

        has_time1 && has_time2
    }

    /// 建议解决策略
    async fn suggest_resolution_strategy(
        &self,
        conflict_type: &ConflictType,
        memories: &[Memory],
    ) -> Result<ResolutionStrategy> {
        match conflict_type {
            ConflictType::Semantic => {
                if memories.len() == 2 {
                    let newer = &memories[0];
                    let older = &memories[1];

                    if newer.created_at > older.created_at {
                        Ok(ResolutionStrategy::KeepLatest)
                    } else {
                        Ok(ResolutionStrategy::KeepHighestConfidence)
                    }
                } else {
                    Ok(ResolutionStrategy::MarkForManualReview)
                }
            }
            ConflictType::Temporal => Ok(ResolutionStrategy::KeepLatest),
            ConflictType::Duplicate => {
                if let Some(latest) = memories.iter().max_by_key(|m| m.created_at) {
                    Ok(ResolutionStrategy::RemoveDuplicates {
                        keep_memory_id: latest.id.clone(),
                    })
                } else {
                    Ok(ResolutionStrategy::MarkForManualReview)
                }
            }
            _ => Ok(ResolutionStrategy::MarkForManualReview),
        }
    }

    /// 解决单个冲突
    async fn resolve_single_conflict(
        &self,
        conflict: &ConflictDetection,
        memories: &[Memory],
    ) -> Result<ConflictResolution> {
        let resolution_result = match &conflict.suggested_resolution {
            ResolutionStrategy::KeepLatest => {
                // 找到最新的记忆，删除其他的
                let conflict_memories: Vec<&Memory> = memories
                    .iter()
                    .filter(|m| conflict.memory_ids.contains(&m.id))
                    .collect();

                if let Some(latest) = conflict_memories.iter().max_by_key(|m| m.created_at) {
                    let to_delete: Vec<String> = conflict_memories
                        .iter()
                        .filter(|m| m.id != latest.id)
                        .map(|m| m.id.clone())
                        .collect();

                    ResolutionResult::Success {
                        updated_memories: vec![latest.id.clone()],
                        deleted_memories: to_delete,
                    }
                } else {
                    ResolutionResult::Failed {
                        error: "无法找到最新记忆".to_string(),
                    }
                }
            }
            ResolutionStrategy::RemoveDuplicates { keep_memory_id } => {
                let to_delete: Vec<String> = conflict
                    .memory_ids
                    .iter()
                    .filter(|id| *id != keep_memory_id)
                    .cloned()
                    .collect();

                ResolutionResult::Success {
                    updated_memories: vec![keep_memory_id.clone()],
                    deleted_memories: to_delete,
                }
            }
            _ => ResolutionResult::RequiresManualIntervention {
                reason: "复杂冲突需要人工处理".to_string(),
            },
        };

        Ok(ConflictResolution {
            conflict_id: conflict.id.clone(),
            applied_strategy: conflict.suggested_resolution.clone(),
            resolution_result,
            resolved_at: chrono::Utc::now(),
            resolution_confidence: conflict.confidence,
        })
    }
}
