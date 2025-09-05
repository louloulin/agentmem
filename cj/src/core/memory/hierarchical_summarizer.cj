/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord
import contextengine.models.{MemoryLevel, MemoryScope, HierarchicalMemoryRecord}

/**
 * 分层摘要配置
 * 定义不同层级的摘要参数
 */
public struct HierarchicalSummaryConfig {
    public let userLevelConfig: SummaryConfig       // 用户级摘要配置
    public let sessionLevelConfig: SummaryConfig    // 会话级摘要配置
    public let agentLevelConfig: SummaryConfig      // 代理级摘要配置
    public let enableCrosslevelSummary: Bool        // 是否启用跨层级摘要
    public let summaryUpdateInterval: Int64         // 摘要更新间隔（秒）
    public let maxSummaryAge: Int64                 // 最大摘要年龄（秒）
    
    public init() {
        this.userLevelConfig = SummaryConfig(SummaryStrategy.ABSTRACTIVE, 500, 0.3)
        this.sessionLevelConfig = SummaryConfig(SummaryStrategy.HYBRID, 300, 0.4)
        this.agentLevelConfig = SummaryConfig(SummaryStrategy.EXTRACTIVE, 200, 0.5)
        this.enableCrosslevelSummary = true
        this.summaryUpdateInterval = 3600  // 1小时
        this.maxSummaryAge = 86400         // 24小时
    }
    
    public init(userConfig: SummaryConfig, sessionConfig: SummaryConfig, agentConfig: SummaryConfig,
                enableCrosslevel: Bool, updateInterval: Int64, maxAge: Int64) {
        this.userLevelConfig = userConfig
        this.sessionLevelConfig = sessionConfig
        this.agentLevelConfig = agentConfig
        this.enableCrosslevelSummary = enableCrosslevel
        this.summaryUpdateInterval = updateInterval
        this.maxSummaryAge = maxAge
    }
}

/**
 * 分层摘要结果
 * 包含不同层级的摘要信息
 */
public struct HierarchicalSummaryResult {
    public let userSummary: Option<SummaryResult>       // 用户级摘要
    public let sessionSummary: Option<SummaryResult>    // 会话级摘要
    public let agentSummary: Option<SummaryResult>      // 代理级摘要
    public let crosslevelSummary: Option<SummaryResult> // 跨层级摘要
    public let totalMemories: Int64                     // 总记忆数量
    public let processingTime: Int64                    // 处理时间
    public let timestamp: Int64                         // 生成时间戳
    
    public init(userSummary: Option<SummaryResult>, sessionSummary: Option<SummaryResult>,
                agentSummary: Option<SummaryResult>, crosslevelSummary: Option<SummaryResult>,
                totalMemories: Int64, processingTime: Int64, timestamp: Int64) {
        this.userSummary = userSummary
        this.sessionSummary = sessionSummary
        this.agentSummary = agentSummary
        this.crosslevelSummary = crosslevelSummary
        this.totalMemories = totalMemories
        this.processingTime = processingTime
        this.timestamp = timestamp
    }
}

/**
 * 分层摘要生成器
 * 负责生成不同层级的记忆摘要
 */
public class HierarchicalSummarizer {
    private let memorySummarizer: MemorySummarizer
    private let config: HierarchicalSummaryConfig
    private let summaryCache: HashMap<String, HierarchicalSummaryResult>
    private let lastUpdateTime: HashMap<String, Int64>
    
    public init(memorySummarizer: MemorySummarizer) {
        this.memorySummarizer = memorySummarizer
        this.config = HierarchicalSummaryConfig()
        this.summaryCache = HashMap<String, HierarchicalSummaryResult>()
        this.lastUpdateTime = HashMap<String, Int64>()
    }
    
    public init(memorySummarizer: MemorySummarizer, config: HierarchicalSummaryConfig) {
        this.memorySummarizer = memorySummarizer
        this.config = config
        this.summaryCache = HashMap<String, HierarchicalSummaryResult>()
        this.lastUpdateTime = HashMap<String, Int64>()
    }
    
    /**
     * 生成分层摘要
     * @param memories 分层记忆记录数组
     * @param scope 记忆作用域
     * @return 分层摘要结果
     */
    public func generateHierarchicalSummary(memories: Array<HierarchicalMemoryRecord>, 
                                           scope: MemoryScope): HierarchicalSummaryResult {
        let startTime = getCurrentTimeMillis()
        let currentTime = getCurrentTimeMillis()
        
        // 检查缓存是否需要更新
        let cacheKey = generateSummaryCacheKey(scope)
        if (shouldUseCachedSummary(cacheKey, currentTime)) {
            let cached = summaryCache.get(cacheKey)
            if (cached.isSome()) {
                return cached.getOrThrow()
            }
        }
        
        // 按层级分组记忆
        let groupedMemories = groupMemoriesByLevel(memories)
        
        // 生成各层级摘要
        let userSummary = generateLevelSummary(groupedMemories.get("USER"), config.userLevelConfig)
        let sessionSummary = generateLevelSummary(groupedMemories.get("SESSION"), config.sessionLevelConfig)
        let agentSummary = generateLevelSummary(groupedMemories.get("AGENT"), config.agentLevelConfig)
        
        // 生成跨层级摘要
        let crosslevelSummary = if (config.enableCrosslevelSummary) {
            generateCrosslevelSummary(userSummary, sessionSummary, agentSummary)
        } else {
            None<SummaryResult>
        }
        
        let endTime = getCurrentTimeMillis()
        let processingTime = endTime - startTime
        
        let result = HierarchicalSummaryResult(
            userSummary, sessionSummary, agentSummary, crosslevelSummary,
            Int64(memories.size), processingTime, currentTime
        )
        
        // 缓存结果
        summaryCache[cacheKey] = result
        lastUpdateTime[cacheKey] = currentTime
        
        return result
    }
    
    /**
     * 更新分层摘要
     * @param scope 记忆作用域
     * @param newMemories 新增记忆
     * @return 更新后的摘要结果
     */
    public func updateHierarchicalSummary(scope: MemoryScope, 
                                         newMemories: Array<HierarchicalMemoryRecord>): HierarchicalSummaryResult {
        let cacheKey = generateSummaryCacheKey(scope)
        
        // 移除旧缓存，强制重新生成
        summaryCache.remove(cacheKey)
        lastUpdateTime.remove(cacheKey)
        
        return generateHierarchicalSummary(newMemories, scope)
    }
    
    /**
     * 获取层级摘要统计
     * @return 统计信息
     */
    public func getHierarchicalSummaryStatistics(): HashMap<String, String> {
        let stats = HashMap<String, String>()
        
        stats["cached_summaries"] = summaryCache.size.toString()
        stats["user_summaries"] = countSummariesByLevel("USER").toString()
        stats["session_summaries"] = countSummariesByLevel("SESSION").toString()
        stats["agent_summaries"] = countSummariesByLevel("AGENT").toString()
        
        // 计算平均处理时间
        var totalProcessingTime: Int64 = 0
        var summaryCount: Int64 = 0
        
        for ((_, summary) in summaryCache) {
            totalProcessingTime = totalProcessingTime + summary.processingTime
            summaryCount = summaryCount + 1
        }
        
        if (summaryCount > 0) {
            let avgProcessingTime = totalProcessingTime / summaryCount
            stats["avg_processing_time_ms"] = avgProcessingTime.toString()
        }
        
        return stats
    }
    
    /**
     * 清理过期摘要
     * @return 清理的摘要数量
     */
    public func cleanupExpiredSummaries(): Int64 {
        let currentTime = getCurrentTimeMillis()
        let expiredKeys = ArrayList<String>()
        
        for ((key, summary) in summaryCache) {
            let age = currentTime - summary.timestamp
            if (age > config.maxSummaryAge) {
                expiredKeys.add(key)
            }
        }
        
        for (key in expiredKeys.toArray()) {
            summaryCache.remove(key)
            lastUpdateTime.remove(key)
        }
        
        return Int64(expiredKeys.size)
    }
    
    /**
     * 清空摘要缓存
     */
    public func clearSummaryCache(): Unit {
        summaryCache.clear()
        lastUpdateTime.clear()
    }
    
    // ===== 私有方法 =====
    
    /**
     * 按层级分组记忆
     */
    private func groupMemoriesByLevel(memories: Array<HierarchicalMemoryRecord>): HashMap<String, Array<MemoryRecord>> {
        let grouped = HashMap<String, Array<MemoryRecord>>()
        let userMemories = ArrayList<MemoryRecord>()
        let sessionMemories = ArrayList<MemoryRecord>()
        let agentMemories = ArrayList<MemoryRecord>()
        
        for (memory in memories) {
            if (memory.scope.level == MemoryLevel.USER) {
                userMemories.add(memory.baseRecord)
            } else if (memory.scope.level == MemoryLevel.SESSION) {
                sessionMemories.add(memory.baseRecord)
            } else if (memory.scope.level == MemoryLevel.AGENT) {
                agentMemories.add(memory.baseRecord)
            }
        }
        
        grouped["USER"] = userMemories.toArray()
        grouped["SESSION"] = sessionMemories.toArray()
        grouped["AGENT"] = agentMemories.toArray()
        
        return grouped
    }
    
    /**
     * 生成层级摘要
     */
    private func generateLevelSummary(memories: Option<Array<MemoryRecord>>, config: SummaryConfig): Option<SummaryResult> {
        if (memories.isNone()) {
            return None
        }
        
        let memoryArray = memories.getOrThrow()
        if (memoryArray.size == 0) {
            return None
        }
        
        return Some(memorySummarizer.summarizeMemoryCollection(memoryArray, Some(config)))
    }
    
    /**
     * 生成跨层级摘要
     */
    private func generateCrosslevelSummary(userSummary: Option<SummaryResult>, 
                                          sessionSummary: Option<SummaryResult>,
                                          agentSummary: Option<SummaryResult>): Option<SummaryResult> {
        let summaryTexts = ArrayList<String>()
        
        if (userSummary.isSome()) {
            summaryTexts.add("用户层级：" + userSummary.getOrThrow().summary)
        }
        
        if (sessionSummary.isSome()) {
            summaryTexts.add("会话层级：" + sessionSummary.getOrThrow().summary)
        }
        
        if (agentSummary.isSome()) {
            summaryTexts.add("代理层级：" + agentSummary.getOrThrow().summary)
        }
        
        if (summaryTexts.size == 0) {
            return None
        }
        
        let combinedText = joinTexts(summaryTexts.toArray())
        let combinedMemory = MemoryRecord(
            "crosslevel_summary", combinedText, "", 0.0, 0, 0, HashMap<String, String>()
        )
        
        let crosslevelConfig = SummaryConfig(SummaryStrategy.ABSTRACTIVE, 400, 0.6)
        return Some(memorySummarizer.summarizeMemory(combinedMemory, Some(crosslevelConfig)))
    }
    
    /**
     * 连接文本
     */
    private func joinTexts(texts: Array<String>): String {
        if (texts.size == 0) {
            return ""
        }
        
        var result = texts[0]
        for (i in 1..texts.size) {
            result = result + "\n\n" + texts[i]
        }
        
        return result
    }
    
    /**
     * 生成摘要缓存键
     */
    private func generateSummaryCacheKey(scope: MemoryScope): String {
        return "${scope.userId}_${scope.sessionId}_${scope.agentId}"
    }
    
    /**
     * 检查是否应该使用缓存摘要
     */
    private func shouldUseCachedSummary(cacheKey: String, currentTime: Int64): Bool {
        if (!summaryCache.contains(cacheKey)) {
            return false
        }
        
        let lastUpdate = lastUpdateTime.get(cacheKey) ?? 0
        let timeSinceUpdate = currentTime - lastUpdate
        
        return timeSinceUpdate < config.summaryUpdateInterval
    }
    
    /**
     * 按层级统计摘要数量
     */
    private func countSummariesByLevel(level: String): Int64 {
        var count: Int64 = 0
        
        for ((_, summary) in summaryCache) {
            if (level == "USER" && summary.userSummary.isSome()) {
                count = count + 1
            } else if (level == "SESSION" && summary.sessionSummary.isSome()) {
                count = count + 1
            } else if (level == "AGENT" && summary.agentSummary.isSome()) {
                count = count + 1
            }
        }
        
        return count
    }
    
    /**
     * 获取当前时间（毫秒）
     */
    private func getCurrentTimeMillis(): Int64 {
        return 1000  // 简化实现
    }
}
