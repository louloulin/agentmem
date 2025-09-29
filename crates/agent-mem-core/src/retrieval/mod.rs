//! AgentMem 7.0 主动检索机制
//!
//! 本模块实现了基于 MIRIX 架构的主动检索系统，包括：
//! - TopicExtractor: 基于 LLM 的主题提取
//! - RetrievalRouter: 智能路由和多策略检索
//! - ContextSynthesizer: 多源记忆融合和上下文合成
//!
//! 参考 MIRIX 的设计模式，但针对 Rust 的特性进行了优化

pub mod router;
pub mod synthesizer;
pub mod topic_extractor;

#[cfg(test)]
mod tests;

// Re-export main types
pub use router::{
    RetrievalRouter, RetrievalRouterConfig, RetrievalStrategy, RouteDecision, RoutingResult,
};
pub use synthesizer::{
    ConflictResolution, ContextSynthesizer, ContextSynthesizerConfig, SynthesisResult,
};
pub use topic_extractor::{
    ExtractedTopic, TopicCategory, TopicExtractor, TopicExtractorConfig, TopicHierarchy,
};

use crate::types::MemoryType;
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 检索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRequest {
    /// 查询文本
    pub query: String,
    /// 目标记忆类型（可选）
    pub target_memory_types: Option<Vec<MemoryType>>,
    /// 最大结果数量
    pub max_results: usize,
    /// 检索策略偏好
    pub preferred_strategy: Option<RetrievalStrategy>,
    /// 上下文信息
    pub context: Option<HashMap<String, serde_json::Value>>,
    /// 是否启用主题提取
    pub enable_topic_extraction: bool,
    /// 是否启用上下文合成
    pub enable_context_synthesis: bool,
}

/// 检索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResponse {
    /// 检索到的记忆项
    pub memories: Vec<RetrievedMemory>,
    /// 提取的主题
    pub extracted_topics: Vec<ExtractedTopic>,
    /// 路由决策信息
    pub routing_info: RouteDecision,
    /// 合成结果
    pub synthesis_result: Option<SynthesisResult>,
    /// 总处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 置信度分数
    pub confidence_score: f32,
}

/// 检索到的记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedMemory {
    /// 记忆ID
    pub id: String,
    /// 记忆类型
    pub memory_type: MemoryType,
    /// 内容
    pub content: String,
    /// 相关性分数
    pub relevance_score: f32,
    /// 来源智能体
    pub source_agent: String,
    /// 检索策略
    pub retrieval_strategy: RetrievalStrategy,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 主动检索系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRetrievalConfig {
    /// 主题提取器配置
    pub topic_extractor: TopicExtractorConfig,
    /// 路由器配置
    pub router: RetrievalRouterConfig,
    /// 合成器配置
    pub synthesizer: ContextSynthesizerConfig,
    /// 默认最大结果数
    pub default_max_results: usize,
    /// 默认置信度阈值
    pub default_confidence_threshold: f32,
    /// 是否启用缓存
    pub enable_caching: bool,
    /// 缓存过期时间（秒）
    pub cache_ttl_seconds: u64,
}

impl Default for ActiveRetrievalConfig {
    fn default() -> Self {
        Self {
            topic_extractor: TopicExtractorConfig::default(),
            router: RetrievalRouterConfig::default(),
            synthesizer: ContextSynthesizerConfig::default(),
            default_max_results: 10,
            default_confidence_threshold: 0.5,
            enable_caching: true,
            cache_ttl_seconds: 300, // 5分钟
        }
    }
}

/// 主动检索系统
///
/// 整合主题提取、智能路由和上下文合成功能，
/// 提供统一的主动检索接口
pub struct ActiveRetrievalSystem {
    /// 主题提取器
    topic_extractor: Arc<TopicExtractor>,
    /// 检索路由器
    router: Arc<RetrievalRouter>,
    /// 上下文合成器
    synthesizer: Arc<ContextSynthesizer>,
    /// 系统配置
    config: ActiveRetrievalConfig,
    /// 检索缓存
    cache: Arc<RwLock<HashMap<String, (RetrievalResponse, std::time::Instant)>>>,
}

impl ActiveRetrievalSystem {
    /// 创建新的主动检索系统
    pub async fn new(config: ActiveRetrievalConfig) -> Result<Self> {
        let topic_extractor = Arc::new(TopicExtractor::new(config.topic_extractor.clone()).await?);
        let router = Arc::new(RetrievalRouter::new(config.router.clone()).await?);
        let synthesizer = Arc::new(ContextSynthesizer::new(config.synthesizer.clone()).await?);

        Ok(Self {
            topic_extractor,
            router,
            synthesizer,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 执行主动检索
    pub async fn retrieve(&self, request: RetrievalRequest) -> Result<RetrievalResponse> {
        let start_time = std::time::Instant::now();

        // 检查缓存
        if self.config.enable_caching {
            if let Some(cached_response) = self.check_cache(&request).await? {
                return Ok(cached_response);
            }
        }

        // 1. 主题提取
        let extracted_topics = if request.enable_topic_extraction {
            self.topic_extractor
                .extract_topics(&request.query, request.context.as_ref())
                .await?
        } else {
            Vec::new()
        };

        // 2. 智能路由
        let routing_result = self
            .router
            .route_retrieval(&request, &extracted_topics)
            .await?;

        // 3. 执行检索
        let memories = self.execute_retrieval(&request, &routing_result).await?;

        // 4. 上下文合成
        let synthesis_result = if request.enable_context_synthesis && !memories.is_empty() {
            Some(
                self.synthesizer
                    .synthesize_context(&memories, &request)
                    .await?,
            )
        } else {
            None
        };

        let processing_time_ms = start_time.elapsed().as_millis().max(1) as u64;
        let confidence_score = self.calculate_confidence_score(&memories, &synthesis_result);

        let response = RetrievalResponse {
            memories,
            extracted_topics,
            routing_info: routing_result.decision,
            synthesis_result,
            processing_time_ms,
            confidence_score,
        };

        // 缓存结果
        if self.config.enable_caching {
            self.cache_response(&request, &response).await?;
        }

        Ok(response)
    }

    /// 检查缓存
    async fn check_cache(&self, request: &RetrievalRequest) -> Result<Option<RetrievalResponse>> {
        let cache_key = self.generate_cache_key(request);
        let cache = self.cache.read().await;

        if let Some((response, timestamp)) = cache.get(&cache_key) {
            if timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds {
                return Ok(Some(response.clone()));
            }
        }

        Ok(None)
    }

    /// 缓存响应
    async fn cache_response(
        &self,
        request: &RetrievalRequest,
        response: &RetrievalResponse,
    ) -> Result<()> {
        let cache_key = self.generate_cache_key(request);
        let mut cache = self.cache.write().await;
        cache.insert(cache_key, (response.clone(), std::time::Instant::now()));
        Ok(())
    }

    /// 生成缓存键
    fn generate_cache_key(&self, request: &RetrievalRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.query.hash(&mut hasher);
        request.target_memory_types.hash(&mut hasher);
        request.max_results.hash(&mut hasher);
        request.preferred_strategy.hash(&mut hasher);

        format!("retrieval_{}", hasher.finish())
    }

    /// 执行实际的检索操作
    async fn execute_retrieval(
        &self,
        _request: &RetrievalRequest,
        _routing_result: &RoutingResult,
    ) -> Result<Vec<RetrievedMemory>> {
        // TODO: 实现实际的检索逻辑
        // 这里需要与各个记忆智能体进行通信
        Ok(Vec::new())
    }

    /// 计算置信度分数
    fn calculate_confidence_score(
        &self,
        memories: &[RetrievedMemory],
        synthesis_result: &Option<SynthesisResult>,
    ) -> f32 {
        if memories.is_empty() {
            return 0.0;
        }

        let avg_relevance: f32 =
            memories.iter().map(|m| m.relevance_score).sum::<f32>() / memories.len() as f32;
        let synthesis_boost = synthesis_result
            .as_ref()
            .map(|s| s.confidence_score * 0.2)
            .unwrap_or(0.0);

        (avg_relevance + synthesis_boost).min(1.0)
    }

    /// 清理过期缓存
    pub async fn cleanup_cache(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let now = std::time::Instant::now();

        cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < self.config.cache_ttl_seconds
        });

        Ok(())
    }

    /// 获取系统统计信息
    pub async fn get_stats(&self) -> Result<RetrievalStats> {
        let cache = self.cache.read().await;

        Ok(RetrievalStats {
            cache_size: cache.len(),
            topic_extractor_stats: self.topic_extractor.get_stats().await?,
            router_stats: self.router.get_stats().await?,
            synthesizer_stats: self.synthesizer.get_stats().await?,
        })
    }
}

/// 检索系统统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalStats {
    /// 缓存大小
    pub cache_size: usize,
    /// 主题提取器统计
    pub topic_extractor_stats: serde_json::Value,
    /// 路由器统计
    pub router_stats: serde_json::Value,
    /// 合成器统计
    pub synthesizer_stats: serde_json::Value,
}
