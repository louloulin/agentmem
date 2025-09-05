/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import std.time.DateTime
import contextengine.models.MemoryRecord
// import contextengine.core.memory.{SemanticSimilarityDetector, MemoryMerger, MergeStrategy}  // 避免循环依赖

/**
 * 去重优先级规则
 */
public enum DeduplicationPriority {
    | KEEP_NEWEST      // 保留最新的
    | KEEP_OLDEST      // 保留最旧的
    | KEEP_MOST_IMPORTANT  // 保留最重要的
    | KEEP_MOST_ACCESSED   // 保留访问最多的
    | KEEP_LONGEST     // 保留内容最长的
    | INTELLIGENT      // 智能选择
    
    public func toString(): String {
        match (this) {
            case KEEP_NEWEST => "keep_newest"
            case KEEP_OLDEST => "keep_oldest"
            case KEEP_MOST_IMPORTANT => "keep_most_important"
            case KEEP_MOST_ACCESSED => "keep_most_accessed"
            case KEEP_LONGEST => "keep_longest"
            case INTELLIGENT => "intelligent"
        }
    }
}

/**
 * 去重配置
 */
public struct DeduplicationConfig {
    public let priority: DeduplicationPriority   // 去重优先级
    public let similarityThreshold: Float64      // 相似度阈值
    public let batchSize: Int64                  // 批处理大小
    public let enableMerging: Bool               // 是否启用合并
    public let preserveHistory: Bool             // 是否保留历史
    public let maxProcessingTime: Int64          // 最大处理时间（秒）
    public let enableStatistics: Bool           // 是否启用统计
    
    public init() {
        this.priority = DeduplicationPriority.INTELLIGENT
        this.similarityThreshold = 0.9
        this.batchSize = 1000
        this.enableMerging = true
        this.preserveHistory = true
        this.maxProcessingTime = 300  // 5分钟
        this.enableStatistics = true
    }
    
    public init(priority: DeduplicationPriority, threshold: Float64, batchSize: Int64) {
        this.priority = priority
        this.similarityThreshold = threshold
        this.batchSize = batchSize
        this.enableMerging = true
        this.preserveHistory = true
        this.maxProcessingTime = 300
        this.enableStatistics = true
    }
}

/**
 * 去重结果
 */
public struct DeduplicationResult {
    public let originalCount: Int64              // 原始记忆数量
    public let duplicateCount: Int64             // 重复记忆数量
    public let removedCount: Int64               // 删除的记忆数量
    public let mergedCount: Int64                // 合并的记忆数量
    public let finalCount: Int64                 // 最终记忆数量
    public let processingTime: Int64             // 处理时间（毫秒）
    public let deduplicationRate: Float64       // 去重率
    public let removedMemoryIds: Array<String>  // 被删除的记忆ID
    public let mergedGroups: Int64              // 合并组数量
    
    public init(originalCount: Int64, duplicateCount: Int64, removedCount: Int64, 
                mergedCount: Int64, finalCount: Int64, processingTime: Int64,
                removedIds: Array<String>, mergedGroups: Int64) {
        this.originalCount = originalCount
        this.duplicateCount = duplicateCount
        this.removedCount = removedCount
        this.mergedCount = mergedCount
        this.finalCount = finalCount
        this.processingTime = processingTime
        this.deduplicationRate = if (originalCount > 0) {
            Float64(removedCount) / Float64(originalCount) * 100.0
        } else {
            0.0
        }
        this.removedMemoryIds = removedIds
        this.mergedGroups = mergedGroups
    }
}

/**
 * 记忆去重器
 * 负责自动检测和处理重复记忆
 */
public class MemoryDeduplicator {
    private let config: DeduplicationConfig
    private var isInitialized: Bool
    private let deduplicationHistory: ArrayList<DeduplicationResult>
    private var totalProcessed: Int64
    private var totalRemoved: Int64

    public init(config: DeduplicationConfig) {
        this.config = config
        this.isInitialized = true
        this.deduplicationHistory = ArrayList<DeduplicationResult>()
        this.totalProcessed = 0
        this.totalRemoved = 0
    }

    public init() {
        this.config = DeduplicationConfig()
        this.isInitialized = true
        this.deduplicationHistory = ArrayList<DeduplicationResult>()
        this.totalProcessed = 0
        this.totalRemoved = 0
    }
    
    /**
     * 执行记忆去重
     * @param memories 记忆列表
     * @return 去重结果和处理后的记忆列表
     */
    public func deduplicateMemories(memories: Array<MemoryRecord>): (DeduplicationResult, Array<MemoryRecord>) {
        if (!isInitialized || memories.size == 0) {
            let emptyResult = DeduplicationResult(0, 0, 0, 0, 0, 0, Array<String>(), 0)
            return (emptyResult, memories)
        }
        
        let startTime = DateTime.now().toUnixTimeStamp()
        
        try {
            println("MemoryDeduplicator: 开始去重处理，记忆数量: ${memories.size}")
            
            let originalCount = Int64(memories.size)
            totalProcessed = totalProcessed + originalCount
            
            // 使用简单的内容比较检测重复记忆组
            let duplicateGroups = detectSimpleDuplicateGroups(memories)
            println("MemoryDeduplicator: 发现 ${duplicateGroups.size} 个重复记忆组")
            
            var processedMemories = ArrayList<MemoryRecord>()
            var removedIds = ArrayList<String>()
            var mergedCount: Int64 = 0
            var removedCount: Int64 = 0
            var mergedGroups: Int64 = 0
            
            // 创建已处理记忆的映射
            let processedMap = HashMap<String, Bool>()
            
            // 处理重复记忆组
            for (group in duplicateGroups) {
                // 简化实现：仅去重，不合并
                let selectedMemory = selectMemoryByPriority(group)
                processedMemories.add(selectedMemory)

                for (memory in group) {
                    processedMap[memory.id] = true
                    if (memory.id != selectedMemory.id) {
                        removedIds.add(memory.id)
                        removedCount = removedCount + 1
                    }
                }

                println("MemoryDeduplicator: 处理记忆组，原始数量: ${group.size}, 保留: 1")
            }
            
            // 添加未处理的记忆
            for (memory in memories) {
                if (!processedMap.contains(memory.id)) {
                    processedMemories.add(memory)
                }
            }
            
            let endTime = DateTime.now().toUnixTimeStamp()
            let processingTime = endTime - startTime
            
            let finalCount = Int64(processedMemories.size)
            let duplicateCount = originalCount - finalCount + mergedCount
            
            totalRemoved = totalRemoved + removedCount
            
            let result = DeduplicationResult(
                originalCount,
                duplicateCount,
                removedCount,
                mergedCount,
                finalCount,
                0,  // 简化处理时间
                removedIds.toArray(),
                mergedGroups
            )
            
            if (config.preserveHistory) {
                deduplicationHistory.add(result)
            }
            
            println("MemoryDeduplicator: 去重完成")
            println("  - 原始数量: ${originalCount}")
            println("  - 最终数量: ${finalCount}")
            println("  - 删除数量: ${removedCount}")
            println("  - 合并数量: ${mergedCount}")
            println("  - 去重率: ${result.deduplicationRate}%")
            println("  - 处理时间: ${processingTime}ms")
            
            return (result, processedMemories.toArray())
            
        } catch (e: Exception) {
            println("MemoryDeduplicator: 去重处理失败: ${e}")
            let errorResult = DeduplicationResult(Int64(memories.size), 0, 0, 0, Int64(memories.size), 0, Array<String>(), 0)
            return (errorResult, memories)
        }
    }
    
    /**
     * 批量去重处理
     * @param memories 记忆列表
     * @return 去重结果和处理后的记忆列表
     */
    public func batchDeduplicate(memories: Array<MemoryRecord>): (DeduplicationResult, Array<MemoryRecord>) {
        if (memories.size <= config.batchSize) {
            return deduplicateMemories(memories)
        }
        
        println("MemoryDeduplicator: 开始批量去重，总数量: ${memories.size}, 批大小: ${config.batchSize}")
        
        var allProcessedMemories = ArrayList<MemoryRecord>()
        var totalOriginal: Int64 = 0
        var totalDuplicate: Int64 = 0
        var totalRemoved: Int64 = 0
        var totalMerged: Int64 = 0
        var totalProcessingTime: Int64 = 0
        var allRemovedIds = ArrayList<String>()
        var totalMergedGroups: Int64 = 0
        
        // 分批处理
        var processed: Int64 = 0
        while (processed < memories.size) {
            let batchEnd = if (processed + config.batchSize > memories.size) {
                memories.size
            } else {
                processed + config.batchSize
            }
            
            let batch = ArrayList<MemoryRecord>()
            for (i in processed..batchEnd) {
                batch.add(memories[i])
            }
            
            let (batchResult, batchProcessed) = deduplicateMemories(batch.toArray())
            
            for (memory in batchProcessed) {
                allProcessedMemories.add(memory)
            }
            
            totalOriginal = totalOriginal + batchResult.originalCount
            totalDuplicate = totalDuplicate + batchResult.duplicateCount
            totalRemoved = totalRemoved + batchResult.removedCount
            totalMerged = totalMerged + batchResult.mergedCount
            totalProcessingTime = totalProcessingTime + batchResult.processingTime
            totalMergedGroups = totalMergedGroups + batchResult.mergedGroups
            
            for (id in batchResult.removedMemoryIds) {
                allRemovedIds.add(id)
            }
            
            processed = batchEnd
        }
        
        let finalResult = DeduplicationResult(
            totalOriginal,
            totalDuplicate,
            totalRemoved,
            totalMerged,
            Int64(allProcessedMemories.size),
            totalProcessingTime,
            allRemovedIds.toArray(),
            totalMergedGroups
        )
        
        println("MemoryDeduplicator: 批量去重完成")
        println("  - 总原始数量: ${totalOriginal}")
        println("  - 总最终数量: ${allProcessedMemories.size}")
        println("  - 总去重率: ${finalResult.deduplicationRate}%")
        
        return (finalResult, allProcessedMemories.toArray())
    }
    
    /**
     * 快速去重检查
     * @param memories 记忆列表
     * @return 是否存在重复记忆
     */
    public func haseDuplicates(memories: Array<MemoryRecord>): Bool {
        if (memories.size < 2) {
            return false
        }

        // 快速检查：使用简单的内容比较
        for (i in 0..memories.size) {
            for (j in (i + 1)..memories.size) {
                if (isContentSimilar(memories[i].memory, memories[j].memory)) {
                    return true
                }
            }
        }

        return false
    }
    
    /**
     * 获取去重统计信息
     * @return 统计信息
     */
    public func getDeduplicationStatistics(): HashMap<String, String> {
        let stats = HashMap<String, String>()
        
        stats["total_processed"] = totalProcessed.toString()
        stats["total_removed"] = totalRemoved.toString()
        stats["total_sessions"] = deduplicationHistory.size.toString()
        stats["priority_strategy"] = config.priority.toString()
        stats["similarity_threshold"] = config.similarityThreshold.toString()
        stats["batch_size"] = config.batchSize.toString()
        stats["merging_enabled"] = config.enableMerging.toString()
        
        if (totalProcessed > 0) {
            let overallDeduplicationRate = Float64(totalRemoved) / Float64(totalProcessed) * 100.0
            stats["overall_deduplication_rate"] = overallDeduplicationRate.toString()
        } else {
            stats["overall_deduplication_rate"] = "0.0"
        }
        
        // 计算平均处理时间
        if (deduplicationHistory.size > 0) {
            var totalTime: Int64 = 0
            for (result in deduplicationHistory) {
                totalTime = totalTime + result.processingTime
            }
            let avgTime = totalTime / Int64(deduplicationHistory.size)
            stats["average_processing_time"] = avgTime.toString()
        } else {
            stats["average_processing_time"] = "0"
        }
        
        return stats
    }
    
    /**
     * 获取去重历史
     * @return 去重历史列表
     */
    public func getDeduplicationHistory(): Array<DeduplicationResult> {
        return deduplicationHistory.toArray()
    }
    
    /**
     * 清除去重历史
     */
    public func clearDeduplicationHistory(): Unit {
        deduplicationHistory.clear()
        totalProcessed = 0
        totalRemoved = 0
        println("MemoryDeduplicator: 去重历史已清除")
    }
    
    // ===== 私有方法 =====
    
    /**
     * 根据优先级选择记忆
     */
    private func selectMemoryByPriority(memories: Array<MemoryRecord>): MemoryRecord {
        if (memories.size == 1) {
            return memories[0]
        }
        
        match (config.priority) {
            case DeduplicationPriority.KEEP_NEWEST =>
                getNewestMemory(memories)
            case DeduplicationPriority.KEEP_OLDEST =>
                getOldestMemory(memories)
            case DeduplicationPriority.KEEP_MOST_IMPORTANT =>
                getMostImportantMemory(memories)
            case DeduplicationPriority.KEEP_MOST_ACCESSED =>
                getMostAccessedMemory(memories)
            case DeduplicationPriority.KEEP_LONGEST =>
                getLongestMemory(memories)
            case DeduplicationPriority.INTELLIGENT =>
                selectIntelligently(memories)
        }
    }
    
    /**
     * 获取最新的记忆
     */
    private func getNewestMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var newest = memories[0]
        for (memory in memories) {
            if (memory.createdAt > newest.createdAt) {
                newest = memory
            }
        }
        return newest
    }
    
    /**
     * 获取最旧的记忆
     */
    private func getOldestMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var oldest = memories[0]
        for (memory in memories) {
            if (memory.createdAt < oldest.createdAt) {
                oldest = memory
            }
        }
        return oldest
    }
    
    /**
     * 获取最重要的记忆
     */
    private func getMostImportantMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var mostImportant = memories[0]
        for (memory in memories) {
            if (memory.score > mostImportant.score) {
                mostImportant = memory
            }
        }
        return mostImportant
    }
    
    /**
     * 获取访问最多的记忆
     */
    private func getMostAccessedMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var mostAccessed = memories[0]
        for (memory in memories) {
            let currentAccess = if (memory.metadata.contains("access_count")) {
                1  // 简化实现
            } else {
                0
            }

            let maxAccess = if (mostAccessed.metadata.contains("access_count")) {
                1  // 简化实现
            } else {
                0
            }
            
            if (currentAccess > maxAccess) {
                mostAccessed = memory
            }
        }
        return mostAccessed
    }
    
    /**
     * 获取内容最长的记忆
     */
    private func getLongestMemory(memories: Array<MemoryRecord>): MemoryRecord {
        var longest = memories[0]
        for (memory in memories) {
            if (memory.memory.size > longest.memory.size) {
                longest = memory
            }
        }
        return longest
    }
    
    /**
     * 智能选择记忆
     */
    private func selectIntelligently(memories: Array<MemoryRecord>): MemoryRecord {
        // 综合考虑重要性、时间和内容长度
        var bestMemory = memories[0]
        var bestScore: Float64 = 0.0
        
        for (memory in memories) {
            let importanceScore = memory.score * 0.5
            let timeScore = (Float64(memory.createdAt) / 1000000000.0) * 0.3  // 归一化时间戳
            let lengthScore = (Float64(memory.memory.size) / 1000.0) * 0.2   // 归一化内容长度
            
            let totalScore = importanceScore + timeScore + lengthScore
            
            if (totalScore > bestScore) {
                bestScore = totalScore
                bestMemory = memory
            }
        }
        
        return bestMemory
    }

    /**
     * 简单的重复记忆组检测
     */
    private func detectSimpleDuplicateGroups(memories: Array<MemoryRecord>): Array<Array<MemoryRecord>> {
        let duplicateGroups = ArrayList<Array<MemoryRecord>>()
        let processed = HashMap<String, Bool>()

        for (i in 0..memories.size) {
            let memory = memories[i]

            // 跳过已处理的记忆
            if (processed.contains(memory.id)) {
                continue
            }

            let duplicateGroup = ArrayList<MemoryRecord>()
            duplicateGroup.add(memory)
            processed[memory.id] = true

            // 查找相似的记忆
            for (j in (i + 1)..memories.size) {
                let candidate = memories[j]

                if (processed.contains(candidate.id)) {
                    continue
                }

                if (isContentSimilar(memory.memory, candidate.memory)) {
                    duplicateGroup.add(candidate)
                    processed[candidate.id] = true
                }
            }

            // 如果找到重复项，添加到结果中
            if (duplicateGroup.size > 1) {
                duplicateGroups.add(duplicateGroup.toArray())
            }
        }

        return duplicateGroups.toArray()
    }

    /**
     * 简单的内容相似性检测
     */
    private func isContentSimilar(content1: String, content2: String): Bool {
        // 简化实现：基于内容长度和字符相似性
        if (content1 == content2) {
            return true
        }

        let lengthDiff = if (content1.size > content2.size) {
            content1.size - content2.size
        } else {
            content2.size - content1.size
        }

        // 如果长度差异太大，认为不相似
        if (lengthDiff > 10) {
            return false
        }

        // 简单的字符匹配检查
        let minLength = if (content1.size < content2.size) {
            content1.size
        } else {
            content2.size
        }

        if (minLength == 0) {
            return false
        }

        var matchCount: Int64 = 0
        let checkLength = if (minLength > 20) { 20 } else { minLength }

        for (i in 0..checkLength) {
            if (content1[i] == content2[i]) {
                matchCount = matchCount + 1
            }
        }

        let similarity = Float64(matchCount) / Float64(checkLength)
        return similarity >= config.similarityThreshold
    }
}
