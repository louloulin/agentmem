/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import std.time.DateTime
import contextengine.models.MemoryRecord
// import contextengine.core.memory.SemanticSimilarityDetector  // 避免循环依赖

/**
 * 记忆合并策略枚举
 */
public enum MergeStrategy {
    | KEEP_LATEST      // 保留最新的记忆
    | KEEP_MOST_IMPORTANT  // 保留最重要的记忆
    | KEEP_MOST_COMPLETE   // 保留最完整的记忆
    | INTELLIGENT_MERGE    // 智能合并内容
    | USER_DEFINED        // 用户自定义策略
    
    public func toString(): String {
        match (this) {
            case KEEP_LATEST => "keep_latest"
            case KEEP_MOST_IMPORTANT => "keep_most_important"
            case KEEP_MOST_COMPLETE => "keep_most_complete"
            case INTELLIGENT_MERGE => "intelligent_merge"
            case USER_DEFINED => "user_defined"
        }
    }
}

/**
 * 记忆合并配置
 */
public struct MemoryMergerConfig {
    public let defaultStrategy: MergeStrategy     // 默认合并策略
    public let preserveMetadata: Bool            // 是否保留元数据
    public let mergeThreshold: Float64           // 合并阈值
    public let maxMergeSize: Int64              // 最大合并数量
    public let enableContentFusion: Bool        // 是否启用内容融合
    public let prioritizeImportance: Bool       // 是否优先考虑重要性
    public let preserveHistory: Bool            // 是否保留合并历史
    
    public init() {
        this.defaultStrategy = MergeStrategy.INTELLIGENT_MERGE
        this.preserveMetadata = true
        this.mergeThreshold = 0.85
        this.maxMergeSize = 5
        this.enableContentFusion = true
        this.prioritizeImportance = true
        this.preserveHistory = true
    }
    
    public init(strategy: MergeStrategy, threshold: Float64, maxSize: Int64) {
        this.defaultStrategy = strategy
        this.preserveMetadata = true
        this.mergeThreshold = threshold
        this.maxMergeSize = maxSize
        this.enableContentFusion = true
        this.prioritizeImportance = true
        this.preserveHistory = true
    }
}

/**
 * 合并结果
 */
public struct MergeResult {
    public let mergedMemory: MemoryRecord        // 合并后的记忆
    public let originalMemories: Array<MemoryRecord>  // 原始记忆列表
    public let strategy: MergeStrategy           // 使用的合并策略
    public let confidence: Float64               // 合并置信度
    public let mergeReason: String              // 合并原因
    public let timestamp: Int64                 // 合并时间戳
    
    public init(mergedMemory: MemoryRecord, originalMemories: Array<MemoryRecord>, 
                strategy: MergeStrategy, confidence: Float64, reason: String) {
        this.mergedMemory = mergedMemory
        this.originalMemories = originalMemories
        this.strategy = strategy
        this.confidence = confidence
        this.mergeReason = reason
        this.timestamp = 0  // 简化实现
    }
}

/**
 * 记忆合并器
 * 负责智能合并相似的记忆
 */
public class MemoryMerger {
    private let config: MemoryMergerConfig
    private var isInitialized: Bool
    private let mergeHistory: ArrayList<MergeResult>

    public init(config: MemoryMergerConfig) {
        this.config = config
        this.isInitialized = true
        this.mergeHistory = ArrayList<MergeResult>()
    }

    public init() {
        this.config = MemoryMergerConfig()
        this.isInitialized = true
        this.mergeHistory = ArrayList<MergeResult>()
    }
    
    /**
     * 合并相似记忆组
     * @param memoryGroup 相似记忆组
     * @param strategy 合并策略（可选）
     * @return 合并结果
     */
    public func mergeMemoryGroup(memoryGroup: Array<MemoryRecord>, 
                                strategy: Option<MergeStrategy>): Option<MergeResult> {
        if (!isInitialized || memoryGroup.size < 2) {
            return None
        }
        
        try {
            let mergeStrategy = if (strategy.isSome()) {
                strategy.getOrThrow()
            } else {
                config.defaultStrategy
            }
            
            println("MemoryMerger: 合并记忆组，数量: ${memoryGroup.size}, 策略: ${mergeStrategy.toString()}")
            
            let mergeResult = match (mergeStrategy) {
                case MergeStrategy.KEEP_LATEST =>
                    mergeKeepLatest(memoryGroup)
                case MergeStrategy.KEEP_MOST_IMPORTANT =>
                    mergeKeepMostImportant(memoryGroup)
                case MergeStrategy.KEEP_MOST_COMPLETE =>
                    mergeKeepMostComplete(memoryGroup)
                case MergeStrategy.INTELLIGENT_MERGE =>
                    mergeIntelligent(memoryGroup)
                case MergeStrategy.USER_DEFINED =>
                    mergeUserDefined(memoryGroup)
            }
            
            if (mergeResult.isSome() && config.preserveHistory) {
                mergeHistory.add(mergeResult.getOrThrow())
            }
            
            return mergeResult
            
        } catch (e: Exception) {
            println("MemoryMerger: 合并记忆组失败: ${e}")
            return None
        }
    }
    
    /**
     * 批量合并记忆（简化版本，不依赖相似性检测器）
     * @param memoryGroups 预先分组的记忆列表
     * @return 合并结果列表
     */
    public func batchMergeMemoryGroups(memoryGroups: Array<Array<MemoryRecord>>): Array<MergeResult> {
        let results = ArrayList<MergeResult>()

        try {
            println("MemoryMerger: 开始批量合并，记忆组数量: ${memoryGroups.size}")

            for (group in memoryGroups) {
                if (group.size <= config.maxMergeSize && group.size > 1) {
                    let mergeResult = mergeMemoryGroup(group, None)
                    if (mergeResult.isSome()) {
                        results.add(mergeResult.getOrThrow())
                    }
                } else {
                    println("MemoryMerger: 记忆组大小不适合合并: ${group.size}")
                }
            }

            println("MemoryMerger: 批量合并完成，成功合并: ${results.size} 组")

        } catch (e: Exception) {
            println("MemoryMerger: 批量合并失败: ${e}")
        }

        return results.toArray()
    }
    
    /**
     * 智能内容融合
     * @param memories 记忆列表
     * @return 融合后的内容
     */
    public func fuseContent(memories: Array<MemoryRecord>): String {
        if (memories.size == 0) {
            return ""
        }
        
        if (memories.size == 1) {
            return memories[0].memory
        }
        
        try {
            // 按重要性和时间排序
            let sortedMemories = sortMemoriesByPriority(memories)
            
            // 提取关键信息
            let keyInfo = extractKeyInformation(sortedMemories)
            
            // 融合内容
            return fuseKeyInformation(keyInfo)
            
        } catch (e: Exception) {
            println("MemoryMerger: 内容融合失败: ${e}")
            // 返回最重要的记忆内容
            return getMostImportantMemory(memories).memory
        }
    }
    
    /**
     * 获取合并统计信息
     * @return 统计信息
     */
    public func getMergeStatistics(): HashMap<String, String> {
        let stats = HashMap<String, String>()
        
        stats["total_merges"] = mergeHistory.size.toString()
        stats["default_strategy"] = config.defaultStrategy.toString()
        stats["merge_threshold"] = config.mergeThreshold.toString()
        stats["max_merge_size"] = config.maxMergeSize.toString()
        stats["content_fusion_enabled"] = config.enableContentFusion.toString()
        
        // 按策略统计
        let strategyStats = HashMap<String, Int64>()
        for (result in mergeHistory) {
            let strategy = result.strategy.toString()
            let count = if (strategyStats.contains(strategy)) {
                strategyStats[strategy]
            } else {
                0
            }
            strategyStats[strategy] = count + 1
        }
        
        for ((strategy, count) in strategyStats) {
            stats["strategy_${strategy}"] = count.toString()
        }
        
        // 计算平均置信度
        if (mergeHistory.size > 0) {
            var totalConfidence: Float64 = 0.0
            for (result in mergeHistory) {
                totalConfidence = totalConfidence + result.confidence
            }
            let avgConfidence = totalConfidence / Float64(mergeHistory.size)
            stats["average_confidence"] = avgConfidence.toString()
        } else {
            stats["average_confidence"] = "0.0"
        }
        
        return stats
    }
    
    /**
     * 获取合并历史
     * @return 合并历史列表
     */
    public func getMergeHistory(): Array<MergeResult> {
        return mergeHistory.toArray()
    }
    
    /**
     * 清除合并历史
     */
    public func clearMergeHistory(): Unit {
        mergeHistory.clear()
        println("MemoryMerger: 合并历史已清除")
    }
    
    // ===== 私有合并策略方法 =====
    
    /**
     * 保留最新记忆的合并策略
     */
    private func mergeKeepLatest(memories: Array<MemoryRecord>): Option<MergeResult> {
        let latestMemory = getLatestMemory(memories)
        let confidence = 0.9  // 高置信度，因为策略简单明确
        
        let result = MergeResult(
            latestMemory,
            memories,
            MergeStrategy.KEEP_LATEST,
            confidence,
            "保留最新的记忆"
        )
        
        return Some(result)
    }
    
    /**
     * 保留最重要记忆的合并策略
     */
    private func mergeKeepMostImportant(memories: Array<MemoryRecord>): Option<MergeResult> {
        let mostImportantMemory = getMostImportantMemory(memories)
        let confidence = 0.85  // 较高置信度
        
        let result = MergeResult(
            mostImportantMemory,
            memories,
            MergeStrategy.KEEP_MOST_IMPORTANT,
            confidence,
            "保留最重要的记忆"
        )
        
        return Some(result)
    }
    
    /**
     * 保留最完整记忆的合并策略
     */
    private func mergeKeepMostComplete(memories: Array<MemoryRecord>): Option<MergeResult> {
        let mostCompleteMemory = getMostCompleteMemory(memories)
        let confidence = 0.8  // 中等置信度
        
        let result = MergeResult(
            mostCompleteMemory,
            memories,
            MergeStrategy.KEEP_MOST_COMPLETE,
            confidence,
            "保留最完整的记忆"
        )
        
        return Some(result)
    }
    
    /**
     * 智能合并策略
     */
    private func mergeIntelligent(memories: Array<MemoryRecord>): Option<MergeResult> {
        try {
            // 选择基础记忆（最重要的）
            let baseMemory = getMostImportantMemory(memories)
            
            // 如果启用内容融合，融合所有记忆的内容
            let mergedContent = if (config.enableContentFusion) {
                fuseContent(memories)
            } else {
                baseMemory.memory
            }
            
            // 创建合并后的记忆
            let mergedMemory = MemoryRecord(
                baseMemory.id,
                mergedContent,
                baseMemory.hash,
                baseMemory.score,
                baseMemory.createdAt,
                baseMemory.updatedAt,
                baseMemory.metadata
            )
            
            // 更新重要性分数（取最高值）
            var maxImportance: Float64 = 0.0
            for (memory in memories) {
                if (memory.score > maxImportance) {
                    maxImportance = memory.score
                }
            }
            // 注意：MemoryRecord的字段是不可变的，这里只是示例
            
            let confidence = calculateMergeConfidence(memories)
            
            let result = MergeResult(
                mergedMemory,
                memories,
                MergeStrategy.INTELLIGENT_MERGE,
                confidence,
                "智能合并多个相似记忆"
            )
            
            return Some(result)
            
        } catch (e: Exception) {
            println("MemoryMerger: 智能合并失败: ${e}")
            return None
        }
    }
    
    /**
     * 用户自定义合并策略
     */
    private func mergeUserDefined(memories: Array<MemoryRecord>): Option<MergeResult> {
        // 默认使用智能合并策略
        return mergeIntelligent(memories)
    }
    
    // ===== 私有辅助方法 =====
    
    /**
     * 获取最新的记忆
     */
    private func getLatestMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var latestMemory = memories[0]
        for (memory in memories) {
            if (memory.createdAt > latestMemory.createdAt) {
                latestMemory = memory
            }
        }
        return latestMemory
    }
    
    /**
     * 获取最重要的记忆
     */
    private func getMostImportantMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var mostImportantMemory = memories[0]
        for (memory in memories) {
            if (memory.score > mostImportantMemory.score) {
                mostImportantMemory = memory
            }
        }
        return mostImportantMemory
    }
    
    /**
     * 获取最完整的记忆（内容最长）
     */
    private func getMostCompleteMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var mostCompleteMemory = memories[0]
        for (memory in memories) {
            if (memory.memory.size > mostCompleteMemory.memory.size) {
                mostCompleteMemory = memory
            }
        }
        return mostCompleteMemory
    }
    
    /**
     * 按优先级排序记忆
     */
    private func sortMemoriesByPriority(memories: Array<MemoryRecord>): Array<MemoryRecord> {
        let sortedList = ArrayList<MemoryRecord>()
        for (memory in memories) {
            sortedList.add(memory)
        }
        
        // 按重要性和时间排序
        for (i in 0..sortedList.size) {
            for (j in 0..(sortedList.size - 1 - i)) {
                let current = sortedList[j]
                let next = sortedList[j + 1]
                
                // 首先按重要性排序，然后按时间排序
                let shouldSwap = if (config.prioritizeImportance) {
                    current.score < next.score ||
                    (current.score == next.score && current.createdAt < next.createdAt)
                } else {
                    current.createdAt < next.createdAt
                }
                
                if (shouldSwap) {
                    sortedList[j] = next
                    sortedList[j + 1] = current
                }
            }
        }
        
        return sortedList.toArray()
    }
    
    /**
     * 提取关键信息
     */
    private func extractKeyInformation(memories: Array<MemoryRecord>): Array<String> {
        let keyInfo = ArrayList<String>()
        
        for (memory in memories) {
            // 简化实现：提取每个记忆的关键句子
            let sentences = memory.memory.split("。")
            for (sentence in sentences) {
                if (sentence.size > 10) {  // 过滤短句
                    keyInfo.add(sentence)
                }
            }
        }
        
        return keyInfo.toArray()
    }
    
    /**
     * 融合关键信息
     */
    private func fuseKeyInformation(keyInfo: Array<String>): String {
        if (keyInfo.size == 0) {
            return ""
        }
        
        // 简化实现：去重并连接
        let uniqueInfo = HashMap<String, Bool>()
        let fusedInfo = ArrayList<String>()
        
        for (info in keyInfo) {
            if (!uniqueInfo.contains(info)) {
                uniqueInfo[info] = true
                fusedInfo.add(info)
            }
        }
        
        // 简化join实现
        var result = ""
        for (i in 0..fusedInfo.size) {
            if (i > 0) {
                result = result + "。"
            }
            result = result + fusedInfo[i]
        }
        return result + "。"
    }
    
    /**
     * 计算合并置信度
     */
    private func calculateMergeConfidence(memories: Array<MemoryRecord>): Float64 {
        if (memories.size < 2) {
            return 1.0
        }
        
        // 基于记忆数量和相似性计算置信度
        let baseConfidence = 0.7
        let sizeBonus = 0.1 * Float64(memories.size - 2)  // 记忆越多，置信度稍微提高
        let maxBonus = 0.2
        
        let finalConfidence = baseConfidence + (if (sizeBonus > maxBonus) { maxBonus } else { sizeBonus })
        
        return if (finalConfidence > 1.0) { 1.0 } else { finalConfidence }
    }
}
