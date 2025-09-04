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

pub mod similarity;
pub mod clustering;
pub mod importance;
pub mod reasoning;
pub mod processing;

// 重新导出常用类型
pub use agent_mem_traits::{Result, AgentMemError};

// 导出主要功能模块
pub use similarity::SemanticSimilarity;
pub use clustering::MemoryClusterer;
pub use importance::ImportanceEvaluator;
pub use reasoning::MemoryReasoner;
pub use processing::{MemoryProcessor, ProcessingConfig, ProcessingStats};
