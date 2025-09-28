//! # Agent Memory Intelligence
//!
//! 智能记忆处理模块，为AgentMem记忆平台提供高级智能化功能。
//!
//! 本模块提供：
//! - 高级相似度计算和语义分析
//! - 记忆聚类和模式识别
//! - 智能重要性评估
//! - 记忆推理和关联分析
//! - 记忆生命周期管理

pub mod clustering;
pub mod importance;
pub mod processing;
pub mod reasoning;
pub mod similarity;

// 新增智能推理模块
pub mod decision_engine;
pub mod fact_extraction;
pub mod intelligent_processor;

// Mem5 增强模块
pub mod conflict_resolution;
pub mod importance_evaluator;

// 多模态内容处理模块
pub mod multimodal;

// 重新导出常用类型
pub use agent_mem_traits::{AgentMemError, Result};

// 导出主要功能模块
pub use clustering::MemoryClusterer;
pub use importance::ImportanceEvaluator;
pub use processing::{MemoryProcessor, ProcessingConfig, ProcessingStats};
pub use reasoning::MemoryReasoner;
pub use similarity::SemanticSimilarity;

// 导出新的智能推理模块
pub use conflict_resolution::{
    ConflictDetection, ConflictResolution, ConflictResolver, ConflictType, ResolutionStrategy,
};
pub use decision_engine::{
    DecisionContext, DecisionResult, EnhancedDecisionEngine, ExistingMemory, MemoryAction,
    MemoryDecision, MemoryDecisionEngine,
};
pub use fact_extraction::{
    AdvancedFactExtractor, Entity, EntityType, ExtractedFact, FactCategory, FactExtractor,
    Relation, RelationType, StructuredFact,
};
pub use importance_evaluator::{
    ImportanceEvaluation, ImportanceEvaluator as EnhancedImportanceEvaluator, ImportanceFactors,
};
pub use intelligent_processor::{
    EnhancedIntelligentProcessor, EnhancedProcessingResult, IntelligentMemoryProcessor,
    IntelligentProcessingResult, MemoryHealthReport,
};
