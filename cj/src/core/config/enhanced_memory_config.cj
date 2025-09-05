/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.config

import std.collection.HashMap
import std.collection.ArrayList

/**
 * 向量存储配置
 * 对应Mem0的VectorStoreConfig
 */
public struct VectorStoreConfig {
    public let provider: String              // 提供商 (memory, qdrant, chroma, pinecone等)
    public let dimension: Int64              // 向量维度
    public let metric: String                // 距离度量 (cosine, euclidean, dot_product)
    public let host: Option<String>          // 主机地址
    public let port: Option<Int64>           // 端口
    public let apiKey: Option<String>        // API密钥
    public let collection: String            // 集合名称
    public let config: HashMap<String, String>  // 额外配置
    
    public init(provider: String, dimension: Int64, metric: String, 
                collection: String) {
        this.provider = provider
        this.dimension = dimension
        this.metric = metric
        this.host = None
        this.port = None
        this.apiKey = None
        this.collection = collection
        this.config = HashMap<String, String>()
    }
    
    public init(provider: String, dimension: Int64, metric: String, 
                host: Option<String>, port: Option<Int64>, 
                apiKey: Option<String>, collection: String, 
                config: HashMap<String, String>) {
        this.provider = provider
        this.dimension = dimension
        this.metric = metric
        this.host = host
        this.port = port
        this.apiKey = apiKey
        this.collection = collection
        this.config = config
    }
}

/**
 * LLM配置
 * 对应Mem0的LlmConfig
 */
public struct LlmConfig {
    public let provider: String              // 提供商 (openai, anthropic, azure等)
    public let model: String                 // 模型名称
    public let apiKey: Option<String>        // API密钥
    public let baseUrl: Option<String>       // 基础URL
    public let temperature: Float64          // 温度参数
    public let maxTokens: Int64              // 最大token数
    public let config: HashMap<String, String>  // 额外配置
    
    public init(provider: String, model: String) {
        this.provider = provider
        this.model = model
        this.apiKey = None
        this.baseUrl = None
        this.temperature = 0.7
        this.maxTokens = 1000
        this.config = HashMap<String, String>()
    }
    
    public init(provider: String, model: String, apiKey: Option<String>, 
                baseUrl: Option<String>, temperature: Float64, 
                maxTokens: Int64, config: HashMap<String, String>) {
        this.provider = provider
        this.model = model
        this.apiKey = apiKey
        this.baseUrl = baseUrl
        this.temperature = temperature
        this.maxTokens = maxTokens
        this.config = config
    }
}

/**
 * 嵌入模型配置
 * 对应Mem0的EmbedderConfig
 */
public struct EmbedderConfig {
    public let provider: String              // 提供商 (openai, huggingface, sentence_transformers等)
    public let model: String                 // 模型名称
    public let dimension: Int64              // 嵌入维度
    public let apiKey: Option<String>        // API密钥
    public let config: HashMap<String, String>  // 额外配置
    
    public init(provider: String, model: String, dimension: Int64) {
        this.provider = provider
        this.model = model
        this.dimension = dimension
        this.apiKey = None
        this.config = HashMap<String, String>()
    }
    
    public init(provider: String, model: String, dimension: Int64, 
                apiKey: Option<String>, config: HashMap<String, String>) {
        this.provider = provider
        this.model = model
        this.dimension = dimension
        this.apiKey = apiKey
        this.config = config
    }
}

/**
 * 历史数据库配置
 * 对应Mem0的HistoryDbConfig
 */
public struct HistoryDbConfig {
    public let provider: String              // 提供商 (sqlite, postgresql, mysql等)
    public let url: Option<String>           // 数据库URL
    public let path: Option<String>          // 文件路径（SQLite）
    public let config: HashMap<String, String>  // 额外配置
    
    public init(provider: String) {
        this.provider = provider
        this.url = None
        this.path = None
        this.config = HashMap<String, String>()
    }
    
    public init(provider: String, url: Option<String>, path: Option<String>, 
                config: HashMap<String, String>) {
        this.provider = provider
        this.url = url
        this.path = path
        this.config = config
    }
}

/**
 * 图存储配置
 * 对应Mem0的GraphStoreConfig
 */
public struct GraphStoreConfig {
    public let provider: String              // 提供商 (neo4j, memgraph, neptune等)
    public let url: Option<String>           // 连接URL
    public let username: Option<String>      // 用户名
    public let password: Option<String>      // 密码
    public let database: Option<String>      // 数据库名
    public let config: HashMap<String, String>  // 额外配置
    
    public init(provider: String) {
        this.provider = provider
        this.url = None
        this.username = None
        this.password = None
        this.database = None
        this.config = HashMap<String, String>()
    }
    
    public init(provider: String, url: Option<String>, username: Option<String>, 
                password: Option<String>, database: Option<String>, 
                config: HashMap<String, String>) {
        this.provider = provider
        this.url = url
        this.username = username
        this.password = password
        this.database = database
        this.config = config
    }
}

/**
 * 增强版记忆配置
 * 对应Mem0的MemoryConfig，包含所有子配置
 */
public struct EnhancedMemoryConfig {
    public let vectorStore: VectorStoreConfig    // 向量存储配置
    public let llm: LlmConfig                    // LLM配置
    public let embedder: EmbedderConfig          // 嵌入模型配置
    public let historyDb: HistoryDbConfig        // 历史数据库配置
    public let graphStore: Option<GraphStoreConfig>  // 图存储配置（可选）
    public let customPrompt: Option<String>      // 自定义提示词
    public let apiVersion: String                // API版本
    public let enableHistory: Bool               // 是否启用历史记录
    public let enableGraph: Bool                 // 是否启用图存储
    public let maxMemories: Int64                // 最大记忆数量
    public let defaultThreshold: Float64         // 默认相似度阈值
    
    /**
     * 完整构造函数
     */
    public init(vectorStore: VectorStoreConfig, llm: LlmConfig, 
                embedder: EmbedderConfig, historyDb: HistoryDbConfig, 
                graphStore: Option<GraphStoreConfig>, customPrompt: Option<String>, 
                apiVersion: String, enableHistory: Bool, enableGraph: Bool, 
                maxMemories: Int64, defaultThreshold: Float64) {
        this.vectorStore = vectorStore
        this.llm = llm
        this.embedder = embedder
        this.historyDb = historyDb
        this.graphStore = graphStore
        this.customPrompt = customPrompt
        this.apiVersion = apiVersion
        this.enableHistory = enableHistory
        this.enableGraph = enableGraph
        this.maxMemories = maxMemories
        this.defaultThreshold = defaultThreshold
    }
    
    /**
     * 默认配置构造函数
     */
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
     * 验证配置
     * @return 验证结果和错误信息
     */
    public func validate(): (Bool, Array<String>) {
        let errors = ArrayList<String>()
        
        // 验证向量存储配置
        if (vectorStore.dimension <= 0) {
            errors.add("向量维度必须大于0")
        }
        
        if (vectorStore.provider.isEmpty()) {
            errors.add("向量存储提供商不能为空")
        }
        
        // 验证LLM配置
        if (llm.provider.isEmpty()) {
            errors.add("LLM提供商不能为空")
        }
        
        if (llm.model.isEmpty()) {
            errors.add("LLM模型不能为空")
        }
        
        if (llm.temperature < 0.0 || llm.temperature > 2.0) {
            errors.add("LLM温度参数必须在0.0-2.0之间")
        }
        
        // 验证嵌入模型配置
        if (embedder.provider.isEmpty()) {
            errors.add("嵌入模型提供商不能为空")
        }
        
        if (embedder.dimension <= 0) {
            errors.add("嵌入维度必须大于0")
        }
        
        // 验证其他配置
        if (maxMemories <= 0) {
            errors.add("最大记忆数量必须大于0")
        }
        
        if (defaultThreshold < 0.0 || defaultThreshold > 1.0) {
            errors.add("默认阈值必须在0.0-1.0之间")
        }
        
        return (errors.size == 0, errors.toArray())
    }
}
