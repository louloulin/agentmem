/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.hierarchy

import std.collection.HashMap
import std.collection.ArrayList
import std.math.*
import std.time.DateTime
import contextengine.models.{HierarchicalMemoryRecord, MemoryLevel}

/**
 * 重要性评分因子
 * 定义影响记忆重要性的各种因素
 */
public struct ImportanceFactors {
    public let recency: Float64         // 时间新近性 (0.0-1.0)
    public let frequency: Float64       // 访问频率 (0.0-1.0)
    public let relevance: Float64       // 内容相关性 (0.0-1.0)
    public let userInteraction: Float64 // 用户交互强度 (0.0-1.0)
    public let semanticRichness: Float64 // 语义丰富度 (0.0-1.0)
    public let contextualImportance: Float64 // 上下文重要性 (0.0-1.0)
    
    /**
     * 构造函数
     */
    public init(recency: Float64, frequency: Float64, relevance: Float64,
                userInteraction: Float64, semanticRichness: Float64, contextualImportance: Float64) {
        this.recency = recency
        this.frequency = frequency
        this.relevance = relevance
        this.userInteraction = userInteraction
        this.semanticRichness = semanticRichness
        this.contextualImportance = contextualImportance
    }
    
    /**
     * 默认构造函数
     */
    public init() {
        this.recency = 0.5
        this.frequency = 0.5
        this.relevance = 0.5
        this.userInteraction = 0.5
        this.semanticRichness = 0.5
        this.contextualImportance = 0.5
    }
    
    /**
     * 计算综合重要性评分
     */
    public func calculateOverallScore(weights: ImportanceWeights): Float64 {
        return recency * weights.recencyWeight +
               frequency * weights.frequencyWeight +
               relevance * weights.relevanceWeight +
               userInteraction * weights.userInteractionWeight +
               semanticRichness * weights.semanticRichnessWeight +
               contextualImportance * weights.contextualImportanceWeight
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "ImportanceFactors(recency=${recency}, frequency=${frequency}, relevance=${relevance})"
    }
}

/**
 * 重要性权重配置
 * 定义各个评分因子的权重
 */
public struct ImportanceWeights {
    public let recencyWeight: Float64           // 时间新近性权重
    public let frequencyWeight: Float64         // 访问频率权重
    public let relevanceWeight: Float64         // 内容相关性权重
    public let userInteractionWeight: Float64   // 用户交互权重
    public let semanticRichnessWeight: Float64  // 语义丰富度权重
    public let contextualImportanceWeight: Float64 // 上下文重要性权重
    
    /**
     * 构造函数
     */
    public init(recencyWeight: Float64, frequencyWeight: Float64, relevanceWeight: Float64,
                userInteractionWeight: Float64, semanticRichnessWeight: Float64, contextualImportanceWeight: Float64) {
        this.recencyWeight = recencyWeight
        this.frequencyWeight = frequencyWeight
        this.relevanceWeight = relevanceWeight
        this.userInteractionWeight = userInteractionWeight
        this.semanticRichnessWeight = semanticRichnessWeight
        this.contextualImportanceWeight = contextualImportanceWeight
    }
    
    /**
     * 默认权重配置
     */
    public init() {
        this.recencyWeight = 0.2
        this.frequencyWeight = 0.2
        this.relevanceWeight = 0.25
        this.userInteractionWeight = 0.15
        this.semanticRichnessWeight = 0.1
        this.contextualImportanceWeight = 0.1
    }
    
    /**
     * 用户级记忆权重配置
     */
    public static func userMemoryWeights(): ImportanceWeights {
        return ImportanceWeights(
            0.15,  // 用户记忆更注重长期性，时间新近性权重较低
            0.25,  // 访问频率很重要
            0.3,   // 内容相关性最重要
            0.2,   // 用户交互重要
            0.05,  // 语义丰富度权重较低
            0.05   // 上下文重要性权重较低
        )
    }
    
    /**
     * 会话级记忆权重配置
     */
    public static func sessionMemoryWeights(): ImportanceWeights {
        return ImportanceWeights(
            0.3,   // 会话记忆更注重时间新近性
            0.15,  // 访问频率权重较低
            0.25,  // 内容相关性重要
            0.1,   // 用户交互权重较低
            0.1,   // 语义丰富度
            0.1    // 上下文重要性
        )
    }
    
    /**
     * 代理级记忆权重配置
     */
    public static func agentMemoryWeights(): ImportanceWeights {
        return ImportanceWeights(
            0.1,   // 代理记忆时间新近性权重最低
            0.2,   // 访问频率重要
            0.2,   // 内容相关性重要
            0.05,  // 用户交互权重很低
            0.25,  // 语义丰富度很重要
            0.2    // 上下文重要性重要
        )
    }
    
    /**
     * 验证权重总和是否为1.0
     */
    public func isValid(): Bool {
        let total = recencyWeight + frequencyWeight + relevanceWeight + 
                   userInteractionWeight + semanticRichnessWeight + contextualImportanceWeight
        return abs(total - 1.0) < 0.001  // 允许小的浮点误差
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "ImportanceWeights(recency=${recencyWeight}, frequency=${frequencyWeight}, relevance=${relevanceWeight})"
    }
}

/**
 * 记忆重要性评分器
 * 负责计算和更新记忆的重要性评分
 */
public class MemoryImportanceScorer {
    private var userWeights: ImportanceWeights      // 用户级权重配置
    private var sessionWeights: ImportanceWeights  // 会话级权重配置
    private var agentWeights: ImportanceWeights    // 代理级权重配置
    private let decayRate: Float64                                // 时间衰减率
    private let minImportance: Float64                           // 最小重要性阈值
    private let maxImportance: Float64                           // 最大重要性阈值

    // Phase 1.2 增强功能
    private let accessHistory: HashMap<String, ArrayList<Int64>>  // 记忆访问历史
    private let importanceHistory: HashMap<String, ArrayList<Float64>>  // 重要性变化历史
    private let userFeedback: HashMap<String, String>            // 用户反馈记录
    private let adaptiveLearning: Bool                           // 是否启用自适应学习
    private let maxHistorySize: Int64                            // 最大历史记录数
    
    /**
     * 构造函数
     */
    public init(decayRate: Float64, minImportance: Float64, maxImportance: Float64) {
        this.decayRate = decayRate
        this.minImportance = minImportance
        this.maxImportance = maxImportance

        // 初始化默认权重
        this.userWeights = ImportanceWeights.userMemoryWeights()
        this.sessionWeights = ImportanceWeights.sessionMemoryWeights()
        this.agentWeights = ImportanceWeights.agentMemoryWeights()

        // 初始化Phase 1.2增强功能
        this.accessHistory = HashMap<String, ArrayList<Int64>>()
        this.importanceHistory = HashMap<String, ArrayList<Float64>>()
        this.userFeedback = HashMap<String, String>()
        this.adaptiveLearning = true
        this.maxHistorySize = 100
    }
    
    /**
     * 默认构造函数
     */
    public init() {
        this.decayRate = 0.1      // 默认衰减率
        this.minImportance = 0.0  // 最小重要性
        this.maxImportance = 1.0  // 最大重要性

        // 初始化默认权重
        this.userWeights = ImportanceWeights.userMemoryWeights()
        this.sessionWeights = ImportanceWeights.sessionMemoryWeights()
        this.agentWeights = ImportanceWeights.agentMemoryWeights()

        // 初始化Phase 1.2增强功能
        this.accessHistory = HashMap<String, ArrayList<Int64>>()
        this.importanceHistory = HashMap<String, ArrayList<Float64>>()
        this.userFeedback = HashMap<String, String>()
        this.adaptiveLearning = true
        this.maxHistorySize = 100
    }
    
    /**
     * 计算记忆的重要性评分
     */
    public func calculateImportance(memory: HierarchicalMemoryRecord, currentTime: Int64, 
                                  contextMemories: Array<HierarchicalMemoryRecord>): Float64 {
        let factors = calculateImportanceFactors(memory, currentTime, contextMemories)
        let levelWeights = if (memory.scope.level == MemoryLevel.USER) {
            userWeights
        } else if (memory.scope.level == MemoryLevel.SESSION) {
            sessionWeights
        } else if (memory.scope.level == MemoryLevel.AGENT) {
            agentWeights
        } else {
            ImportanceWeights()
        }
        
        let rawScore = factors.calculateOverallScore(levelWeights)
        
        // 应用边界限制
        if (rawScore < minImportance) {
            return minImportance
        } else if (rawScore > maxImportance) {
            return maxImportance
        } else {
            return rawScore
        }
    }
    
    /**
     * 计算重要性因子
     */
    private func calculateImportanceFactors(memory: HierarchicalMemoryRecord, currentTime: Int64,
                                          contextMemories: Array<HierarchicalMemoryRecord>): ImportanceFactors {
        let recency = calculateRecencyScore(memory, currentTime)
        let frequency = calculateFrequencyScore(memory)
        let relevance = calculateRelevanceScore(memory, contextMemories)
        let userInteraction = calculateUserInteractionScore(memory)
        let semanticRichness = calculateSemanticRichnessScore(memory)
        let contextualImportance = calculateContextualImportanceScore(memory, contextMemories)
        
        return ImportanceFactors(recency, frequency, relevance, userInteraction, semanticRichness, contextualImportance)
    }
    
    /**
     * 计算时间新近性评分
     */
    private func calculateRecencyScore(memory: HierarchicalMemoryRecord, currentTime: Int64): Float64 {
        let timeDiff = currentTime - memory.lastAccessTime
        let daysDiff = Float64(timeDiff) / (24.0 * 60.0 * 60.0)  // 转换为浮点数

        // 使用简化的衰减函数
        let decayFactor = decayRate * daysDiff
        if (decayFactor > 10.0) {
            return 0.0
        } else {
            return 1.0 / (1.0 + decayFactor)  // 简化的衰减函数
        }
    }
    
    /**
     * 计算访问频率评分
     */
    private func calculateFrequencyScore(memory: HierarchicalMemoryRecord): Float64 {
        // 简化的频率评分
        if (memory.accessCount <= 0) {
            return 0.0
        }

        let accessCountFloat = Float64(memory.accessCount)
        let normalizedCount = accessCountFloat / 100.0  // 假设最大访问次数为100

        if (normalizedCount > 1.0) {
            return 1.0
        } else {
            return normalizedCount
        }
    }
    
    /**
     * 计算内容相关性评分
     */
    private func calculateRelevanceScore(memory: HierarchicalMemoryRecord, 
                                       contextMemories: Array<HierarchicalMemoryRecord>): Float64 {
        if (contextMemories.size == 0) {
            return 0.5  // 默认中等相关性
        }
        
        var totalSimilarity: Float64 = 0.0
        var count: Int64 = 0
        
        for (contextMemory in contextMemories) {
            if (contextMemory.baseRecord.id != memory.baseRecord.id) {
                let similarity = calculateContentSimilarity(memory.baseRecord.memory, contextMemory.baseRecord.memory)
                totalSimilarity = totalSimilarity + similarity
                count = count + 1
            }
        }
        
        if (count > 0) {
            return totalSimilarity / Float64(count)
        } else {
            return 0.5
        }
    }
    
    /**
     * 计算用户交互强度评分
     */
    private func calculateUserInteractionScore(memory: HierarchicalMemoryRecord): Float64 {
        // 基于元数据中的用户交互信息
        let metadata = memory.baseRecord.metadata
        
        var score: Float64 = 0.0
        
        // 检查是否有用户反馈
        if (metadata.contains("user_feedback")) {
            let feedback = if (metadata.contains("user_feedback")) {
                metadata["user_feedback"]
            } else {
                ""
            }
            // 简化的反馈处理
            if (feedback == "positive") {
                score = score + 0.3
            } else if (feedback == "negative") {
                score = score - 0.2
            } else {
                score = score + 0.1
            }
        }
        
        // 检查是否有用户标记
        if (metadata.contains("user_marked")) {
            score = score + 0.2
        }
        
        // 检查是否有用户编辑
        if (metadata.contains("user_edited")) {
            score = score + 0.3
        }
        
        // 检查是否有用户分享
        if (metadata.contains("user_shared")) {
            score = score + 0.2
        }
        
        return min(1.0, max(0.0, score))
    }
    
    /**
     * 计算语义丰富度评分
     */
    private func calculateSemanticRichnessScore(memory: HierarchicalMemoryRecord): Float64 {
        let content = memory.baseRecord.memory
        let metadata = memory.baseRecord.metadata
        
        var score: Float64 = 0.0
        
        // 基于内容长度
        let contentLength = content.size
        if (contentLength > 100) {
            score = score + 0.3
        } else if (contentLength > 50) {
            score = score + 0.2
        } else {
            score = score + 0.1
        }
        
        // 基于实体数量（简化实现）
        if (metadata.contains("entity_count")) {
            score = score + 0.2
        }

        // 基于关键词数量（简化实现）
        if (metadata.contains("keyword_count")) {
            score = score + 0.1
        }

        // 基于记忆类型（简化实现）
        if (metadata.contains("memory_type")) {
            let memoryType = if (metadata.contains("memory_type")) {
                metadata["memory_type"]
            } else {
                ""
            }
            if (memoryType == "factual_memory") {
                score = score + 0.2
            } else if (memoryType == "episodic_memory") {
                score = score + 0.15
            } else if (memoryType == "semantic_memory") {
                score = score + 0.25
            } else if (memoryType == "procedural_memory") {
                score = score + 0.2
            } else {
                score = score + 0.1
            }
        }

        if (score > 1.0) {
            return 1.0
        } else {
            return score
        }
    }
    
    /**
     * 计算上下文重要性评分
     */
    private func calculateContextualImportanceScore(memory: HierarchicalMemoryRecord,
                                                  contextMemories: Array<HierarchicalMemoryRecord>): Float64 {
        var score: Float64 = 0.0
        
        // 基于记忆层级
        match (memory.scope.level) {
            case MemoryLevel.USER => score = score + 0.3      // 用户记忆上下文重要性高
            case MemoryLevel.SESSION => score = score + 0.2   // 会话记忆上下文重要性中等
            case MemoryLevel.AGENT => score = score + 0.4     // 代理记忆上下文重要性最高
        }
        
        // 基于关联记忆数量（简化实现）
        let relatedCount = countRelatedMemories(memory, contextMemories)
        let relatedScore = Float64(relatedCount) * 0.05
        if (relatedScore > 0.3) {
            score = score + 0.3
        } else {
            score = score + relatedScore
        }

        // 基于是否有父子关系
        if (memory.parentMemoryId.isSome()) {
            score = score + 0.1  // 有父记忆
        }
        if (memory.childMemoryIds.size > 0) {
            let childScore = Float64(memory.childMemoryIds.size) * 0.05
            if (childScore > 0.2) {
                score = score + 0.2
            } else {
                score = score + childScore
            }
        }

        if (score > 1.0) {
            return 1.0
        } else {
            return score
        }
    }
    
    /**
     * 计算内容相似性（简化实现）
     */
    private func calculateContentSimilarity(content1: String, content2: String): Float64 {
        // 简化的相似性计算
        if (content1 == content2) {
            return 1.0
        }

        // 基于长度的简单相似性
        let len1 = content1.size
        let len2 = content2.size
        let maxLen = if (len1 > len2) { len1 } else { len2 }
        let minLen = if (len1 < len2) { len1 } else { len2 }

        if (maxLen == 0) {
            return 0.0
        }

        return Float64(minLen) / Float64(maxLen)
    }
    
    /**
     * 计算相关记忆数量
     */
    private func countRelatedMemories(memory: HierarchicalMemoryRecord,
                                    contextMemories: Array<HierarchicalMemoryRecord>): Int64 {
        var count: Int64 = 0
        let threshold: Float64 = 0.3  // 相似性阈值
        
        for (contextMemory in contextMemories) {
            if (contextMemory.baseRecord.id != memory.baseRecord.id) {
                let similarity = calculateContentSimilarity(memory.baseRecord.memory, contextMemory.baseRecord.memory)
                if (similarity > threshold) {
                    count = count + 1
                }
            }
        }
        
        return count
    }
    
    /**
     * 应用时间衰减
     */
    public func applyTimeDecay(memory: HierarchicalMemoryRecord, currentTime: Int64): Float64 {
        let timeDiff = currentTime - memory.lastAccessTime
        let daysDiff = Float64(timeDiff) / (24.0 * 60.0 * 60.0)

        // 简化的衰减计算
        let decayFactor = 1.0 / (1.0 + decayRate * daysDiff)
        let newImportance = memory.importance * decayFactor

        if (newImportance < minImportance) {
            return minImportance
        } else {
            return newImportance
        }
    }
    
    /**
     * 批量更新记忆重要性
     */
    public func batchUpdateImportance(memories: Array<HierarchicalMemoryRecord>, 
                                    currentTime: Int64): Array<HierarchicalMemoryRecord> {
        let result = ArrayList<HierarchicalMemoryRecord>()
        
        for (memory in memories) {
            let newImportance = calculateImportance(memory, currentTime, memories)
            let updatedMemory = memory.updateImportance(newImportance)
            result.add(updatedMemory)
        }
        
        return result.toArray()
    }
    
    /**
     * 设置层级权重
     */
    public func setLevelWeights(level: MemoryLevel, weights: ImportanceWeights): Bool {
        if (!weights.isValid()) {
            return false
        }

        if (level == MemoryLevel.USER) {
            this.userWeights = weights
        } else if (level == MemoryLevel.SESSION) {
            this.sessionWeights = weights
        } else if (level == MemoryLevel.AGENT) {
            this.agentWeights = weights
        } else {
            return false
        }
        return true
    }
    
    /**
     * 获取层级权重
     */
    public func getLevelWeights(level: MemoryLevel): Option<ImportanceWeights> {
        if (level == MemoryLevel.USER) {
            return Some(userWeights)
        } else if (level == MemoryLevel.SESSION) {
            return Some(sessionWeights)
        } else if (level == MemoryLevel.AGENT) {
            return Some(agentWeights)
        } else {
            return None<ImportanceWeights>
        }
    }
    
    /**
     * 获取重要性统计信息
     */
    public func getImportanceStatistics(memories: Array<HierarchicalMemoryRecord>): HashMap<String, Float64> {
        let stats = HashMap<String, Float64>()
        
        if (memories.size == 0) {
            return stats
        }
        
        var totalImportance: Float64 = 0.0
        var minImportanceValue: Float64 = 1.0
        var maxImportanceValue: Float64 = 0.0
        
        for (memory in memories) {
            let importance = memory.importance
            totalImportance = totalImportance + importance
            minImportanceValue = min(minImportanceValue, importance)
            maxImportanceValue = max(maxImportanceValue, importance)
        }
        
        stats["average_importance"] = totalImportance / Float64(memories.size)
        stats["min_importance"] = minImportanceValue
        stats["max_importance"] = maxImportanceValue
        stats["total_memories"] = Float64(memories.size)
        
        return stats
    }

    // ===== Phase 1.2 增强功能方法 =====

    /**
     * 记录记忆访问
     * @param memoryId 记忆ID
     * @param accessTime 访问时间
     */
    public func recordAccess(memoryId: String, accessTime: Int64): Unit {
        if (!accessHistory.contains(memoryId)) {
            accessHistory[memoryId] = ArrayList<Int64>()
        }

        let history = accessHistory[memoryId]
        history.add(accessTime)

        // 限制历史记录大小
        while (history.size > maxHistorySize) {
            let newHistory = ArrayList<Int64>()
            for (i in 1..history.size) {
                newHistory.add(history[i])
            }
            accessHistory[memoryId] = newHistory
            break
        }
    }

    /**
     * 记录重要性变化
     * @param memoryId 记忆ID
     * @param importance 新的重要性评分
     */
    public func recordImportanceChange(memoryId: String, importance: Float64): Unit {
        if (!importanceHistory.contains(memoryId)) {
            importanceHistory[memoryId] = ArrayList<Float64>()
        }

        let history = importanceHistory[memoryId]
        history.add(importance)

        // 限制历史记录大小
        while (history.size > maxHistorySize) {
            let newHistory = ArrayList<Float64>()
            for (i in 1..history.size) {
                newHistory.add(history[i])
            }
            importanceHistory[memoryId] = newHistory
            break
        }
    }

    /**
     * 记录用户反馈
     * @param memoryId 记忆ID
     * @param feedback 反馈内容（positive, negative, neutral）
     */
    public func recordUserFeedback(memoryId: String, feedback: String): Unit {
        userFeedback[memoryId] = feedback
    }

    /**
     * 计算访问频率评分
     * @param memoryId 记忆ID
     * @param currentTime 当前时间
     * @return 频率评分 (0.0-1.0)
     */
    public func calculateFrequencyScore(memoryId: String, currentTime: Int64): Float64 {
        if (!accessHistory.contains(memoryId)) {
            return 0.0
        }

        let history = accessHistory[memoryId]
        if (history.size == 0) {
            return 0.0
        }

        // 计算最近访问的权重
        var weightedScore: Float64 = 0.0
        var totalWeight: Float64 = 0.0

        for (i in 0..history.size) {
            let accessTime = history[i]
            let timeDiff = currentTime - accessTime
            let weight = exp(-Float64(timeDiff) * decayRate / 86400.0)  // 按天衰减

            weightedScore = weightedScore + weight
            totalWeight = totalWeight + weight
        }

        if (totalWeight > 0.0) {
            let normalizedScore = weightedScore / totalWeight
            // 根据访问次数进行调整
            let frequencyBonus = min(Float64(history.size) * 0.1, 0.5)
            return min(normalizedScore + frequencyBonus, 1.0)
        } else {
            return 0.0
        }
    }

    /**
     * 计算重要性趋势
     * @param memoryId 记忆ID
     * @return 趋势评分 (-1.0到1.0，负值表示下降，正值表示上升)
     */
    public func calculateImportanceTrend(memoryId: String): Float64 {
        if (!importanceHistory.contains(memoryId)) {
            return 0.0
        }

        let history = importanceHistory[memoryId]
        if (history.size < 2) {
            return 0.0
        }

        // 计算简单的线性趋势
        let firstHalf = history.size / 2
        var firstHalfSum: Float64 = 0.0
        var secondHalfSum: Float64 = 0.0

        for (i in 0..firstHalf) {
            firstHalfSum = firstHalfSum + history[i]
        }

        for (i in firstHalf..history.size) {
            secondHalfSum = secondHalfSum + history[i]
        }

        let firstHalfAvg = firstHalfSum / Float64(firstHalf)
        let secondHalfAvg = secondHalfSum / Float64(history.size - firstHalf)

        return secondHalfAvg - firstHalfAvg
    }

    /**
     * 根据用户反馈调整重要性
     * @param memoryId 记忆ID
     * @param currentImportance 当前重要性
     * @return 调整后的重要性
     */
    public func adjustImportanceByFeedback(memoryId: String, currentImportance: Float64): Float64 {
        if (!userFeedback.contains(memoryId)) {
            return currentImportance
        }

        let feedback = userFeedback[memoryId]
        var adjustment: Float64 = 0.0

        if (feedback == "positive") {
            adjustment = 0.1
        } else if (feedback == "negative") {
            adjustment = -0.1
        } else if (feedback == "neutral") {
            adjustment = 0.0
        }

        let newImportance = currentImportance + adjustment
        return max(minImportance, min(maxImportance, newImportance))
    }

    /**
     * 增强的重要性计算（包含Phase 1.2功能）
     * @param memory 记忆记录
     * @param currentTime 当前时间
     * @param contextMemories 上下文记忆
     * @return 增强的重要性评分
     */
    public func calculateEnhancedImportance(memory: HierarchicalMemoryRecord,
                                          currentTime: Int64,
                                          contextMemories: Array<HierarchicalMemoryRecord>): Float64 {
        // 计算基础重要性
        let baseImportance = calculateImportance(memory, currentTime, contextMemories)

        // 记录访问
        recordAccess(memory.baseRecord.id, currentTime)

        // 计算频率加成
        let frequencyScore = calculateFrequencyScore(memory.baseRecord.id, currentTime)
        let frequencyBonus = frequencyScore * 0.2  // 频率最多贡献20%

        // 计算趋势调整
        let trend = calculateImportanceTrend(memory.baseRecord.id)
        let trendAdjustment = trend * 0.1  // 趋势最多调整10%

        // 应用用户反馈
        var enhancedImportance = baseImportance + frequencyBonus + trendAdjustment
        enhancedImportance = adjustImportanceByFeedback(memory.baseRecord.id, enhancedImportance)

        // 记录重要性变化
        recordImportanceChange(memory.baseRecord.id, enhancedImportance)

        // 确保在有效范围内
        return max(minImportance, min(maxImportance, enhancedImportance))
    }

    /**
     * 获取热点记忆（高重要性和高频访问）
     * @param memories 记忆列表
     * @param currentTime 当前时间
     * @param threshold 重要性阈值
     * @return 热点记忆列表
     */
    public func getHotMemories(memories: Array<HierarchicalMemoryRecord>,
                             currentTime: Int64,
                             threshold: Float64): Array<HierarchicalMemoryRecord> {
        let result = ArrayList<HierarchicalMemoryRecord>()

        for (memory in memories) {
            let enhancedImportance = calculateEnhancedImportance(memory, currentTime, memories)
            let frequencyScore = calculateFrequencyScore(memory.baseRecord.id, currentTime)

            // 综合重要性和频率评分
            let hotScore = enhancedImportance * 0.7 + frequencyScore * 0.3

            if (hotScore >= threshold) {
                result.add(memory)
            }
        }

        return result.toArray()
    }

    /**
     * 获取冷记忆（低重要性和低频访问）
     * @param memories 记忆列表
     * @param currentTime 当前时间
     * @param threshold 重要性阈值
     * @return 冷记忆列表
     */
    public func getColdMemories(memories: Array<HierarchicalMemoryRecord>,
                              currentTime: Int64,
                              threshold: Float64): Array<HierarchicalMemoryRecord> {
        let result = ArrayList<HierarchicalMemoryRecord>()

        for (memory in memories) {
            let enhancedImportance = calculateEnhancedImportance(memory, currentTime, memories)
            let frequencyScore = calculateFrequencyScore(memory.baseRecord.id, currentTime)

            // 综合重要性和频率评分
            let coldScore = enhancedImportance * 0.7 + frequencyScore * 0.3

            if (coldScore <= threshold) {
                result.add(memory)
            }
        }

        return result.toArray()
    }

    /**
     * 动态调整权重（自适应学习）
     * @param memories 记忆列表
     * @param currentTime 当前时间
     */
    public func adaptWeights(memories: Array<HierarchicalMemoryRecord>, currentTime: Int64): Unit {
        if (!adaptiveLearning || memories.size < 10) {
            return  // 需要足够的数据才能进行自适应学习
        }

        // 分析用户反馈模式
        var positiveCount: Int64 = 0
        var negativeCount: Int64 = 0
        var totalFeedback: Int64 = 0

        for ((memoryId, feedback) in userFeedback) {
            totalFeedback = totalFeedback + 1
            if (feedback == "positive") {
                positiveCount = positiveCount + 1
            } else if (feedback == "negative") {
                negativeCount = negativeCount + 1
            }
        }

        if (totalFeedback > 0) {
            let positiveRatio = Float64(positiveCount) / Float64(totalFeedback)
            let negativeRatio = Float64(negativeCount) / Float64(totalFeedback)

            // 根据反馈调整权重（简化实现）
            if (positiveRatio > 0.7) {
                // 用户满意度高，增加频率权重
                adjustWeightsByLevel(MemoryLevel.USER, 0.05, "frequency")
            } else if (negativeRatio > 0.3) {
                // 用户满意度低，增加相关性权重
                adjustWeightsByLevel(MemoryLevel.USER, 0.05, "relevance")
            }
        }
    }

    /**
     * 按层级调整权重
     * @param level 记忆层级
     * @param adjustment 调整量
     * @param targetFactor 目标因子
     */
    private func adjustWeightsByLevel(level: MemoryLevel, adjustment: Float64, targetFactor: String): Unit {
        // 简化实现：这里应该有更复杂的权重调整逻辑
        // 在实际应用中，可以使用机器学习算法来优化权重

        if (level == MemoryLevel.USER) {
            if (targetFactor == "frequency") {
                let newFrequencyWeight = min(userWeights.frequencyWeight + adjustment, 0.5)
                let adjustment_actual = newFrequencyWeight - userWeights.frequencyWeight
                let otherWeightsAdjustment = adjustment_actual / 5.0

                userWeights = ImportanceWeights(
                    userWeights.recencyWeight - otherWeightsAdjustment,
                    newFrequencyWeight,
                    userWeights.relevanceWeight - otherWeightsAdjustment,
                    userWeights.userInteractionWeight - otherWeightsAdjustment,
                    userWeights.semanticRichnessWeight - otherWeightsAdjustment,
                    userWeights.contextualImportanceWeight - otherWeightsAdjustment
                )
            }
        }
    }

    /**
     * 获取增强统计信息
     * @param memories 记忆列表
     * @param currentTime 当前时间
     * @return 统计信息
     */
    public func getEnhancedStatistics(memories: Array<HierarchicalMemoryRecord>, currentTime: Int64): HashMap<String, Float64> {
        let stats = getImportanceStatistics(memories)

        // 添加频率统计
        var totalFrequencyScore: Float64 = 0.0
        var accessedMemoriesCount: Int64 = 0

        for (memory in memories) {
            let frequencyScore = calculateFrequencyScore(memory.baseRecord.id, currentTime)
            totalFrequencyScore = totalFrequencyScore + frequencyScore

            if (accessHistory.contains(memory.baseRecord.id)) {
                accessedMemoriesCount = accessedMemoriesCount + 1
            }
        }

        if (memories.size > 0) {
            stats["average_frequency_score"] = totalFrequencyScore / Float64(memories.size)
            stats["accessed_memories_ratio"] = Float64(accessedMemoriesCount) / Float64(memories.size)
        }

        // 添加反馈统计
        var positiveCount: Int64 = 0
        var negativeCount: Int64 = 0

        for ((_, feedback) in userFeedback) {
            if (feedback == "positive") {
                positiveCount = positiveCount + 1
            } else if (feedback == "negative") {
                negativeCount = negativeCount + 1
            }
        }

        stats["positive_feedback_count"] = Float64(positiveCount)
        stats["negative_feedback_count"] = Float64(negativeCount)
        stats["total_feedback_count"] = Float64(userFeedback.size)

        return stats
    }

    /**
     * 清理过期的历史记录
     * @param currentTime 当前时间
     * @param maxAge 最大保留时间（秒）
     */
    public func cleanupOldHistory(currentTime: Int64, maxAge: Int64): Unit {
        let cutoffTime = currentTime - maxAge

        // 清理访问历史
        for ((memoryId, history) in accessHistory) {
            let newHistory = ArrayList<Int64>()
            for (accessTime in history) {
                if (accessTime >= cutoffTime) {
                    newHistory.add(accessTime)
                }
            }
            accessHistory[memoryId] = newHistory
        }

        // 移除空的历史记录
        let keysToRemove = ArrayList<String>()
        for ((memoryId, history) in accessHistory) {
            if (history.size == 0) {
                keysToRemove.add(memoryId)
            }
        }

        for (key in keysToRemove) {
            accessHistory.remove(key)
        }
    }
}
