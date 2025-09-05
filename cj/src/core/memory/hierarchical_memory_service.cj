/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.{MemoryRecord, SearchResult, HierarchicalMemoryRecord, MemoryScope, MemoryLevel}

/**
 * 分层记忆服务接口
 * 扩展基础MemoryService，添加多层记忆架构支持
 */
public interface HierarchicalMemoryService <: MemoryService {
    
    /**
     * 添加分层记忆
     * @param content 记忆内容
     * @param scope 记忆作用域
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    func addHierarchicalMemory(content: String, scope: MemoryScope, metadata: HashMap<String, String>): HierarchicalMemoryRecord
    
    /**
     * 简化添加分层记忆方法
     * @param content 记忆内容
     * @param scope 记忆作用域
     * @return 分层记忆记录
     */
    func addHierarchicalMemory(content: String, scope: MemoryScope): HierarchicalMemoryRecord
    
    /**
     * 添加用户级记忆
     * @param content 记忆内容
     * @param userId 用户ID
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    func addUserMemory(content: String, userId: String, metadata: HashMap<String, String>): HierarchicalMemoryRecord
    
    /**
     * 添加会话级记忆
     * @param content 记忆内容
     * @param userId 用户ID
     * @param sessionId 会话ID
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    func addSessionMemory(content: String, userId: String, sessionId: String, metadata: HashMap<String, String>): HierarchicalMemoryRecord
    
    /**
     * 添加代理级记忆
     * @param content 记忆内容
     * @param agentId 代理ID
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    func addAgentMemory(content: String, agentId: String, metadata: HashMap<String, String>): HierarchicalMemoryRecord
    
    /**
     * 获取分层记忆
     * @param memoryId 记忆ID
     * @param requestScope 请求作用域
     * @return 分层记忆记录（可选）
     */
    func getHierarchicalMemory(memoryId: String, requestScope: MemoryScope): Option<HierarchicalMemoryRecord>
    
    /**
     * 搜索分层记忆
     * @param query 查询内容
     * @param requestScope 请求作用域
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @param filters 过滤条件
     * @return 分层记忆搜索结果
     */
    func searchHierarchicalMemories(query: String, requestScope: MemoryScope, limit: Int64, 
                                   threshold: Float64, filters: HashMap<String, String>): HierarchicalSearchResult
    
    /**
     * 简化搜索分层记忆方法
     * @param query 查询内容
     * @param requestScope 请求作用域
     * @return 分层记忆搜索结果
     */
    func searchHierarchicalMemories(query: String, requestScope: MemoryScope): HierarchicalSearchResult
    
    /**
     * 获取指定作用域的所有记忆
     * @param requestScope 请求作用域
     * @return 分层记忆列表
     */
    func getMemoriesInScope(requestScope: MemoryScope): Array<HierarchicalMemoryRecord>
    
    /**
     * 获取用户的所有记忆
     * @param userId 用户ID
     * @param includeShared 是否包含共享记忆
     * @return 分层记忆列表
     */
    func getUserMemories(userId: String, includeShared: Bool): Array<HierarchicalMemoryRecord>
    
    /**
     * 获取会话的所有记忆
     * @param userId 用户ID
     * @param sessionId 会话ID
     * @return 分层记忆列表
     */
    func getSessionMemories(userId: String, sessionId: String): Array<HierarchicalMemoryRecord>
    
    /**
     * 获取代理的所有记忆
     * @param agentId 代理ID
     * @param includeShared 是否包含共享记忆
     * @return 分层记忆列表
     */
    func getAgentMemories(agentId: String, includeShared: Bool): Array<HierarchicalMemoryRecord>
    
    /**
     * 更新记忆重要性
     * @param memoryId 记忆ID
     * @param scope 记忆作用域
     * @param newImportance 新的重要性评分
     * @return 是否更新成功
     */
    func updateMemoryImportance(memoryId: String, scope: MemoryScope, newImportance: Float64): Bool
    
    /**
     * 批量更新记忆重要性
     * @param requestScope 请求作用域
     * @return 更新的记忆数量
     */
    func batchUpdateImportance(requestScope: MemoryScope): Int64
    
    /**
     * 删除分层记忆
     * @param memoryId 记忆ID
     * @param scope 记忆作用域
     * @return 是否删除成功
     */
    func deleteHierarchicalMemory(memoryId: String, scope: MemoryScope): Bool
    
    /**
     * 清理过期记忆
     * @param currentTime 当前时间
     * @return 清理的记忆数量
     */
    func cleanupExpiredMemories(currentTime: Int64): Int64
    
    /**
     * 获取记忆层级统计信息
     * @return 统计信息
     */
    func getHierarchyStatistics(): HashMap<String, Int64>
    
    /**
     * 获取重要性统计信息
     * @param requestScope 请求作用域
     * @return 重要性统计信息
     */
    func getImportanceStatistics(requestScope: MemoryScope): HashMap<String, Float64>
    
    /**
     * 跨层级搜索记忆
     * @param query 查询内容
     * @param requestScope 请求作用域
     * @param searchLevels 要搜索的层级列表
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @return 跨层级搜索结果
     */
    func crossLevelSearch(query: String, requestScope: MemoryScope, searchLevels: Array<MemoryLevel>,
                         limit: Int64, threshold: Float64): CrossLevelSearchResult
}

/**
 * 分层记忆搜索结果
 */
public struct HierarchicalSearchResult {
    public let memories: Array<HierarchicalMemoryRecord>  // 匹配的分层记忆列表
    public let totalCount: Int64                          // 总匹配数量
    public let searchScope: MemoryScope                   // 搜索作用域
    public let searchTime: Int64                          // 搜索时间（毫秒）
    
    /**
     * 构造函数
     */
    public init(memories: Array<HierarchicalMemoryRecord>, totalCount: Int64, 
                searchScope: MemoryScope, searchTime: Int64) {
        this.memories = memories
        this.totalCount = totalCount
        this.searchScope = searchScope
        this.searchTime = searchTime
    }
    
    /**
     * 简化构造函数
     */
    public init(memories: Array<HierarchicalMemoryRecord>, searchScope: MemoryScope) {
        this.memories = memories
        this.totalCount = memories.size
        this.searchScope = searchScope
        this.searchTime = 0
    }
    
    /**
     * 转换为基础搜索结果
     */
    public func toSearchResult(): SearchResult {
        let baseMemories = ArrayList<MemoryRecord>()
        for (hierarchicalMemory in memories) {
            baseMemories.add(hierarchicalMemory.baseRecord)
        }
        return SearchResult(baseMemories.toArray())
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "HierarchicalSearchResult(memories=${memories.size}, scope=${searchScope.toString()})"
    }
}

/**
 * 跨层级搜索结果
 */
public struct CrossLevelSearchResult {
    public let userMemories: Array<HierarchicalMemoryRecord>    // 用户级记忆
    public let sessionMemories: Array<HierarchicalMemoryRecord> // 会话级记忆
    public let agentMemories: Array<HierarchicalMemoryRecord>   // 代理级记忆
    public let totalCount: Int64                                // 总匹配数量
    public let searchScope: MemoryScope                         // 搜索作用域
    public let searchTime: Int64                                // 搜索时间（毫秒）
    
    /**
     * 构造函数
     */
    public init(userMemories: Array<HierarchicalMemoryRecord>, sessionMemories: Array<HierarchicalMemoryRecord>,
                agentMemories: Array<HierarchicalMemoryRecord>, searchScope: MemoryScope, searchTime: Int64) {
        this.userMemories = userMemories
        this.sessionMemories = sessionMemories
        this.agentMemories = agentMemories
        this.totalCount = userMemories.size + sessionMemories.size + agentMemories.size
        this.searchScope = searchScope
        this.searchTime = searchTime
    }
    
    /**
     * 获取所有记忆（按重要性排序）
     */
    public func getAllMemoriesByImportance(): Array<HierarchicalMemoryRecord> {
        let allMemories = ArrayList<HierarchicalMemoryRecord>()
        
        // 添加所有记忆
        for (memory in userMemories) {
            allMemories.add(memory)
        }
        for (memory in sessionMemories) {
            allMemories.add(memory)
        }
        for (memory in agentMemories) {
            allMemories.add(memory)
        }
        
        // 按重要性排序（简化实现，实际应该使用更高效的排序算法）
        let sortedMemories = allMemories.toArray()
        // TODO: 实现按重要性排序
        
        return sortedMemories
    }
    
    /**
     * 获取指定层级的记忆
     */
    public func getMemoriesByLevel(level: MemoryLevel): Array<HierarchicalMemoryRecord> {
        match (level) {
            case MemoryLevel.USER => userMemories
            case MemoryLevel.SESSION => sessionMemories
            case MemoryLevel.AGENT => agentMemories
        }
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "CrossLevelSearchResult(user=${userMemories.size}, session=${sessionMemories.size}, agent=${agentMemories.size})"
    }
}
