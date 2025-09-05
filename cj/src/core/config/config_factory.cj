/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.config

import std.collection.HashMap

/**
 * 配置工厂类
 * 提供预定义的配置模板和配置构建器
 */
public class ConfigFactory {
    
    /**
     * 创建默认配置
     */
    public static func createDefault(): EnhancedMemoryConfig {
        return EnhancedMemoryConfig()
    }
    
    /**
     * 创建开发环境配置
     */
    public static func createDevelopment(): EnhancedMemoryConfig {
        let vectorStore = VectorStoreConfig("memory", 384, "cosine", "dev_collection")
        let llm = LlmConfig("openai", "gpt-3.5-turbo")
        let embedder = EmbedderConfig("openai", "text-embedding-ada-002", 1536)
        let historyDb = HistoryDbConfig("sqlite")
        
        return EnhancedMemoryConfig(
            vectorStore,
            llm,
            embedder,
            historyDb,
            None,
            None,
            "v1",
            true,
            false,
            1000,
            0.7
        )
    }
    
    /**
     * 创建生产环境配置
     */
    public static func createProduction(): EnhancedMemoryConfig {
        let vectorStore = VectorStoreConfig("qdrant", 1536, "cosine", "prod_collection")
        let llm = LlmConfig("openai", "gpt-4")
        let embedder = EmbedderConfig("openai", "text-embedding-ada-002", 1536)
        let historyDb = HistoryDbConfig("postgresql")
        let graphStore = Some(GraphStoreConfig("neo4j"))
        
        return EnhancedMemoryConfig(
            vectorStore,
            llm,
            embedder,
            historyDb,
            graphStore,
            None,
            "v1",
            true,
            true,
            100000,
            0.8
        )
    }
    
    /**
     * 创建测试环境配置
     */
    public static func createTest(): EnhancedMemoryConfig {
        let vectorStore = VectorStoreConfig("memory", 128, "cosine", "test_collection")
        let llm = LlmConfig("mock", "mock-model")
        let embedder = EmbedderConfig("mock", "mock-embedder", 128)
        let historyDb = HistoryDbConfig("memory")
        
        return EnhancedMemoryConfig(
            vectorStore,
            llm,
            embedder,
            historyDb,
            None,
            None,
            "v1",
            true,
            false,
            100,
            0.5
        )
    }
    
    /**
     * 创建高性能配置
     */
    public static func createHighPerformance(): EnhancedMemoryConfig {
        let vectorStore = VectorStoreConfig("pinecone", 1536, "cosine", "hp_collection")
        let llm = LlmConfig("anthropic", "claude-3-sonnet")
        let embedder = EmbedderConfig("openai", "text-embedding-3-large", 3072)
        let historyDb = HistoryDbConfig("postgresql")
        let graphStore = Some(GraphStoreConfig("memgraph"))
        
        return EnhancedMemoryConfig(
            vectorStore,
            llm,
            embedder,
            historyDb,
            graphStore,
            Some("You are an intelligent memory system. Extract and store key information efficiently."),
            "v1",
            true,
            true,
            1000000,
            0.85
        )
    }
    
    /**
     * 从环境变量创建配置
     */
    public static func createFromEnvironment(): EnhancedMemoryConfig {
        // 这里应该从环境变量读取配置
        // 简化实现，返回默认配置
        return createDefault()
    }
}

/**
 * 配置构建器
 * 提供流式API来构建配置
 */
public class ConfigBuilder {
    private var vectorStore: VectorStoreConfig
    private var llm: LlmConfig
    private var embedder: EmbedderConfig
    private var historyDb: HistoryDbConfig
    private var graphStore: Option<GraphStoreConfig>
    private var customPrompt: Option<String>
    private var apiVersion: String
    private var enableHistory: Bool
    private var enableGraph: Bool
    private var maxMemories: Int64
    private var defaultThreshold: Float64
    
    public init() {
        this.vectorStore = VectorStoreConfig("memory", 384, "cosine", "default")
        this.llm = LlmConfig("openai", "gpt-3.5-turbo")
        this.embedder = EmbedderConfig("openai", "text-embedding-ada-002", 1536)
        this.historyDb = HistoryDbConfig("sqlite")
        this.graphStore = None
        this.customPrompt = None
        this.apiVersion = "v1"
        this.enableHistory = true
        this.enableGraph = false
        this.maxMemories = 10000
        this.defaultThreshold = 0.7
    }
    
    /**
     * 设置向量存储配置
     */
    public func withVectorStore(provider: String, dimension: Int64, 
                               metric: String, collection: String): ConfigBuilder {
        this.vectorStore = VectorStoreConfig(provider, dimension, metric, collection)
        return this
    }
    
    /**
     * 设置LLM配置
     */
    public func withLlm(provider: String, model: String): ConfigBuilder {
        this.llm = LlmConfig(provider, model)
        return this
    }
    
    /**
     * 设置嵌入模型配置
     */
    public func withEmbedder(provider: String, model: String, dimension: Int64): ConfigBuilder {
        this.embedder = EmbedderConfig(provider, model, dimension)
        return this
    }
    
    /**
     * 设置历史数据库配置
     */
    public func withHistoryDb(provider: String): ConfigBuilder {
        this.historyDb = HistoryDbConfig(provider)
        return this
    }
    
    /**
     * 设置图存储配置
     */
    public func withGraphStore(provider: String): ConfigBuilder {
        this.graphStore = Some(GraphStoreConfig(provider))
        this.enableGraph = true
        return this
    }
    
    /**
     * 设置自定义提示词
     */
    public func withCustomPrompt(prompt: String): ConfigBuilder {
        this.customPrompt = Some(prompt)
        return this
    }
    
    /**
     * 设置最大记忆数量
     */
    public func withMaxMemories(maxMemories: Int64): ConfigBuilder {
        this.maxMemories = maxMemories
        return this
    }
    
    /**
     * 设置默认阈值
     */
    public func withDefaultThreshold(threshold: Float64): ConfigBuilder {
        this.defaultThreshold = threshold
        return this
    }
    
    /**
     * 启用或禁用历史记录
     */
    public func withHistory(enabled: Bool): ConfigBuilder {
        this.enableHistory = enabled
        return this
    }
    
    /**
     * 构建配置
     */
    public func build(): EnhancedMemoryConfig {
        return EnhancedMemoryConfig(
            vectorStore,
            llm,
            embedder,
            historyDb,
            graphStore,
            customPrompt,
            apiVersion,
            enableHistory,
            enableGraph,
            maxMemories,
            defaultThreshold
        )
    }
}
