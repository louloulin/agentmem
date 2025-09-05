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
pub mod fact_extraction;
pub mod decision_engine;
pub mod intelligent_processor;

// 重新导出常用类型
pub use agent_mem_traits::{AgentMemError, Result};

// 导出主要功能模块
pub use clustering::MemoryClusterer;
pub use importance::ImportanceEvaluator;
pub use processing::{MemoryProcessor, ProcessingConfig, ProcessingStats};
pub use reasoning::MemoryReasoner;
pub use similarity::SemanticSimilarity;

// 导出新的智能推理模块
pub use fact_extraction::{FactExtractor, ExtractedFact, Message, FactCategory};
pub use decision_engine::{MemoryDecisionEngine, MemoryDecision, MemoryAction, ExistingMemory};
pub use intelligent_processor::{IntelligentMemoryProcessor, IntelligentProcessingResult, MemoryHealthReport};
