/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import contextengine.models.*
import contextengine.storage.history.HistoryRecord

/**
 * 记忆服务核心接口
 * 提供记忆的增删改查和搜索功能
 */
public interface MemoryService {
    
    /**
     * 添加新记忆
     * @param content 记忆内容
     * @param metadata 元数据
     * @return 记忆记录
     */
    func add(content: String, metadata: HashMap<String, String>): MemoryRecord
    
    /**
     * 简化添加记忆方法
     * @param content 记忆内容
     * @return 记忆记录
     */
    func add(content: String): MemoryRecord
    
    /**
     * 搜索记忆
     * @param query 查询内容
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @param filters 过滤条件
     * @return 搜索结果
     */
    func search(query: String, limit: Int64, threshold: Float64, 
                filters: HashMap<String, String>): SearchResult
    
    /**
     * 简化搜索方法
     * @param query 查询内容
     * @return 搜索结果
     */
    func search(query: String): SearchResult
    
    /**
     * 根据ID获取记忆
     * @param id 记忆ID
     * @return 记忆记录（可能为空）
     */
    func get(id: String): Option<MemoryRecord>
    
    /**
     * 更新记忆内容
     * @param id 记忆ID
     * @param content 新内容
     * @return 是否更新成功
     */
    func update(id: String, content: String): Bool
    
    /**
     * 删除记忆
     * @param id 记忆ID
     * @return 是否删除成功
     */
    func delete(id: String): Bool
    
    /**
     * 删除所有记忆（根据过滤条件）
     * @param filters 过滤条件
     * @return 删除的记忆数量
     */
    func deleteAll(filters: HashMap<String, String>): Int64
    
    /**
     * 获取记忆历史记录
     * @param id 记忆ID
     * @return 历史记录列表
     */
    func getHistory(id: String): Array<HistoryRecord>
    
    /**
     * 生成摘要
     * @param memoryIds 记忆ID列表
     * @return 摘要记录
     */
    func createSummary(memoryIds: Array<String>): SummaryRecord
    
    /**
     * 重置所有记忆
     * @return 是否重置成功
     */
    func reset(): Bool
}

/**
 * 记忆服务异常类
 */
public class MemoryServiceException <: Exception {
    public init(message: String) {
        super(message)
    }
    
    public init(message: String, cause: Exception) {
        super(message)
    }
}

/**
 * 记忆配置结构
 */
public struct MemoryConfig {
    public let vectorDimension: Int64        // 向量维度
    public let maxMemories: Int64            // 最大记忆数量
    public let defaultThreshold: Float64     // 默认相似度阈值
    public let enableGraph: Bool             // 是否启用图存储
    public let storagePath: String           // 存储路径
    
    /**
     * 构造函数
     */
    public init(vectorDimension: Int64, maxMemories: Int64, 
                defaultThreshold: Float64, enableGraph: Bool, storagePath: String) {
        this.vectorDimension = vectorDimension
        this.maxMemories = maxMemories
        this.defaultThreshold = defaultThreshold
        this.enableGraph = enableGraph
        this.storagePath = storagePath
    }
    
    /**
     * 默认配置构造函数
     */
    public init() {
        this.vectorDimension = 768
        this.maxMemories = 10000
        this.defaultThreshold = 0.7
        this.enableGraph = false
        this.storagePath = "./memory_storage"
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "MemoryConfig(dim=${vectorDimension}, max=${maxMemories}, threshold=${defaultThreshold})"
    }
}