/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord
import contextengine.models.{MemoryScope, HierarchicalMemoryRecord}

/**
 * 摘要更新策略枚举
 */
public enum SummaryUpdateStrategy {
    | INCREMENTAL    // 增量更新：只处理新增或修改的记忆
    | FULL_REBUILD   // 完全重建：重新生成所有摘要
    | SCHEDULED      // 定时更新：按计划定期更新
    | THRESHOLD      // 阈值触发：达到阈值时更新
    
    public func toString(): String {
        match (this) {
            case INCREMENTAL => "incremental"
            case FULL_REBUILD => "full_rebuild"
            case SCHEDULED => "scheduled"
            case THRESHOLD => "threshold"
        }
    }

    /**
     * 相等比较操作符
     */
    public operator func ==(right: SummaryUpdateStrategy): Bool {
        match (this) {
            case INCREMENTAL =>
                match (right) {
                    case INCREMENTAL => true
                    case _ => false
                }
            case FULL_REBUILD =>
                match (right) {
                    case FULL_REBUILD => true
                    case _ => false
                }
            case SCHEDULED =>
                match (right) {
                    case SCHEDULED => true
                    case _ => false
                }
            case THRESHOLD =>
                match (right) {
                    case THRESHOLD => true
                    case _ => false
                }
        }
    }

    /**
     * 不等比较操作符
     */
    public operator func !=(right: SummaryUpdateStrategy): Bool {
        return !(this == right)
    }
}

/**
 * 摘要更新配置
 */
public struct SummaryUpdateConfig {
    public let strategy: SummaryUpdateStrategy      // 更新策略
    public let updateInterval: Int64                // 更新间隔（秒）
    public let changeThreshold: Int64               // 变化阈值（记忆数量）
    public let maxBatchSize: Int64                  // 最大批处理大小
    public let enableVersioning: Bool               // 是否启用版本控制
    public let maxVersions: Int64                   // 最大版本数
    public let autoCleanup: Bool                    // 是否自动清理
    
    public init() {
        this.strategy = SummaryUpdateStrategy.INCREMENTAL
        this.updateInterval = 3600  // 1小时
        this.changeThreshold = 10   // 10个记忆变化
        this.maxBatchSize = 100
        this.enableVersioning = true
        this.maxVersions = 5
        this.autoCleanup = true
    }
    
    public init(strategy: SummaryUpdateStrategy, updateInterval: Int64, changeThreshold: Int64,
                maxBatchSize: Int64, enableVersioning: Bool, maxVersions: Int64, autoCleanup: Bool) {
        this.strategy = strategy
        this.updateInterval = updateInterval
        this.changeThreshold = changeThreshold
        this.maxBatchSize = maxBatchSize
        this.enableVersioning = enableVersioning
        this.maxVersions = maxVersions
        this.autoCleanup = autoCleanup
    }
}

/**
 * 摘要版本信息
 */
public struct SummaryVersion {
    public let version: Int64                       // 版本号
    public let summary: SummaryResult               // 摘要内容
    public let timestamp: Int64                     // 创建时间戳
    public let changeCount: Int64                   // 变化数量
    public let updateReason: String                 // 更新原因
    
    public init(version: Int64, summary: SummaryResult, timestamp: Int64, 
                changeCount: Int64, updateReason: String) {
        this.version = version
        this.summary = summary
        this.timestamp = timestamp
        this.changeCount = changeCount
        this.updateReason = updateReason
    }
}

/**
 * 摘要更新结果
 */
public struct SummaryUpdateResult {
    public let success: Bool                        // 是否成功
    public let updatedSummaries: Int64              // 更新的摘要数量
    public let newVersion: Option<Int64>            // 新版本号
    public let processingTime: Int64                // 处理时间
    public let updateStrategy: SummaryUpdateStrategy // 使用的更新策略
    public let changesSummary: String               // 变化摘要
    public let errors: Array<String>                // 错误信息
    
    public init(success: Bool, updatedSummaries: Int64, newVersion: Option<Int64>,
                processingTime: Int64, updateStrategy: SummaryUpdateStrategy,
                changesSummary: String, errors: Array<String>) {
        this.success = success
        this.updatedSummaries = updatedSummaries
        this.newVersion = newVersion
        this.processingTime = processingTime
        this.updateStrategy = updateStrategy
        this.changesSummary = changesSummary
        this.errors = errors
    }
}

/**
 * 摘要更新管理器
 * 负责管理摘要的更新、版本控制和调度
 */
public class SummaryUpdateManager {
    private let hierarchicalSummarizer: HierarchicalSummarizer
    private let config: SummaryUpdateConfig
    private let summaryVersions: HashMap<String, Array<SummaryVersion>>
    private let changeTracker: HashMap<String, Int64>
    private let lastUpdateTime: HashMap<String, Int64>
    private let pendingUpdates: HashMap<String, Array<HierarchicalMemoryRecord>>
    
    public init(hierarchicalSummarizer: HierarchicalSummarizer) {
        this.hierarchicalSummarizer = hierarchicalSummarizer
        this.config = SummaryUpdateConfig()
        this.summaryVersions = HashMap<String, Array<SummaryVersion>>()
        this.changeTracker = HashMap<String, Int64>()
        this.lastUpdateTime = HashMap<String, Int64>()
        this.pendingUpdates = HashMap<String, Array<HierarchicalMemoryRecord>>()
    }
    
    public init(hierarchicalSummarizer: HierarchicalSummarizer, config: SummaryUpdateConfig) {
        this.hierarchicalSummarizer = hierarchicalSummarizer
        this.config = config
        this.summaryVersions = HashMap<String, Array<SummaryVersion>>()
        this.changeTracker = HashMap<String, Int64>()
        this.lastUpdateTime = HashMap<String, Int64>()
        this.pendingUpdates = HashMap<String, Array<HierarchicalMemoryRecord>>()
    }
    
    /**
     * 触发摘要更新
     * @param scope 记忆作用域
     * @param memories 记忆记录
     * @param forceUpdate 是否强制更新
     * @return 更新结果
     */
    public func triggerSummaryUpdate(scope: MemoryScope, memories: Array<HierarchicalMemoryRecord>,
                                    forceUpdate: Bool): SummaryUpdateResult {
        let startTime = getCurrentTimeMillis()
        let scopeKey = generateScopeKey(scope)
        
        try {
            // 检查是否需要更新
            if (!forceUpdate && !shouldUpdateSummary(scopeKey)) {
                return createNoUpdateResult()
            }
            
            // 选择更新策略
            let updateStrategy = determineUpdateStrategy(scopeKey, memories)
            
            // 执行更新
            let updateResult = executeUpdate(scope, memories, updateStrategy)
            
            // 更新版本信息
            if (updateResult.success && config.enableVersioning) {
                addSummaryVersion(scopeKey, updateResult, updateStrategy)
            }
            
            // 更新跟踪信息
            updateTrackingInfo(scopeKey)
            
            let endTime = getCurrentTimeMillis()
            let processingTime = endTime - startTime
            
            return SummaryUpdateResult(
                updateResult.success, updateResult.updatedSummaries, updateResult.newVersion,
                processingTime, updateStrategy, updateResult.changesSummary, updateResult.errors
            )
            
        } catch (e: Exception) {
            let endTime = getCurrentTimeMillis()
            let processingTime = endTime - startTime
            
            return SummaryUpdateResult(
                false, 0, None, processingTime, config.strategy,
                "更新失败", [e.toString()]
            )
        }
    }
    
    /**
     * 增量更新摘要
     * @param scope 记忆作用域
     * @param newMemories 新增记忆
     * @return 更新结果
     */
    public func incrementalUpdate(scope: MemoryScope, newMemories: Array<HierarchicalMemoryRecord>): SummaryUpdateResult {
        let scopeKey = generateScopeKey(scope)
        
        // 添加到待更新队列
        let existing = pendingUpdates.get(scopeKey) ?? Array<HierarchicalMemoryRecord>()
        let combined = combineMemoryArrays(existing, newMemories)
        pendingUpdates[scopeKey] = combined
        
        // 更新变化计数
        let currentChanges = changeTracker.get(scopeKey) ?? 0
        changeTracker[scopeKey] = currentChanges + Int64(newMemories.size)
        
        // 检查是否达到阈值
        if (currentChanges + Int64(newMemories.size) >= config.changeThreshold) {
            return triggerSummaryUpdate(scope, combined, false)
        }
        
        return createPendingUpdateResult(Int64(newMemories.size))
    }
    
    /**
     * 定时更新检查
     * @return 更新的作用域数量
     */
    public func scheduledUpdateCheck(): Int64 {
        let currentTime = getCurrentTimeMillis()
        var updatedCount: Int64 = 0
        
        for ((scopeKey, lastUpdate) in lastUpdateTime) {
            let timeSinceUpdate = currentTime - lastUpdate
            
            if (timeSinceUpdate >= config.updateInterval) {
                let pendingMemories = pendingUpdates.get(scopeKey)
                if (pendingMemories.isSome() && pendingMemories.getOrThrow().size > 0) {
                    // 解析作用域并执行更新
                    let scope = parseScopeFromKey(scopeKey)
                    if (scope.isSome()) {
                        let result = triggerSummaryUpdate(scope.getOrThrow(), pendingMemories.getOrThrow(), false)
                        if (result.success) {
                            updatedCount = updatedCount + 1
                        }
                    }
                }
            }
        }
        
        return updatedCount
    }
    
    /**
     * 获取摘要版本历史
     * @param scope 记忆作用域
     * @return 版本历史
     */
    public func getSummaryVersionHistory(scope: MemoryScope): Array<SummaryVersion> {
        let scopeKey = generateScopeKey(scope)
        return summaryVersions.get(scopeKey) ?? Array<SummaryVersion>()
    }
    
    /**
     * 回滚到指定版本
     * @param scope 记忆作用域
     * @param version 目标版本号
     * @return 是否成功
     */
    public func rollbackToVersion(scope: MemoryScope, version: Int64): Bool {
        let scopeKey = generateScopeKey(scope)
        let versions = summaryVersions.get(scopeKey) ?? Array<SummaryVersion>()
        
        for (versionInfo in versions) {
            if (versionInfo.version == version) {
                // 这里应该恢复到指定版本的摘要
                // 简化实现：只返回成功状态
                return true
            }
        }
        
        return false
    }
    
    /**
     * 清理旧版本
     * @return 清理的版本数量
     */
    public func cleanupOldVersions(): Int64 {
        var cleanedCount: Int64 = 0
        
        for ((scopeKey, versions) in summaryVersions) {
            if (versions.size > config.maxVersions) {
                // 保留最新的版本
                let sortedVersions = sortVersionsByTimestamp(versions)
                let toKeep = Int64(config.maxVersions)
                let newVersions = ArrayList<SummaryVersion>()
                
                for (i in 0..toKeep) {
                    if (i < sortedVersions.size) {
                        newVersions.add(sortedVersions[i])
                    }
                }
                
                let removedCount = versions.size - newVersions.size
                summaryVersions[scopeKey] = newVersions.toArray()
                cleanedCount = cleanedCount + Int64(removedCount)
            }
        }
        
        return cleanedCount
    }
    
    /**
     * 获取更新管理器统计信息
     * @return 统计信息
     */
    public func getUpdateManagerStatistics(): HashMap<String, String> {
        let stats = HashMap<String, String>()
        
        stats["tracked_scopes"] = changeTracker.size.toString()
        stats["pending_updates"] = pendingUpdates.size.toString()
        stats["total_versions"] = getTotalVersionCount().toString()
        
        // 计算平均版本数
        if (summaryVersions.size > 0) {
            let avgVersions = getTotalVersionCount() / Int64(summaryVersions.size)
            stats["avg_versions_per_scope"] = avgVersions.toString()
        }
        
        return stats
    }

    // ===== 私有方法 =====

    /**
     * 生成作用域键
     */
    private func generateScopeKey(scope: MemoryScope): String {
        return "${scope.userId}_${scope.sessionId}_${scope.agentId}"
    }

    /**
     * 从键解析作用域
     */
    private func parseScopeFromKey(scopeKey: String): Option<MemoryScope> {
        let parts = scopeKey.split("_")
        if (parts.size >= 3) {
            return Some(MemoryScope.createUserScope(parts[0], false))
        }
        return None
    }

    /**
     * 检查是否应该更新摘要
     */
    private func shouldUpdateSummary(scopeKey: String): Bool {
        let currentTime = getCurrentTimeMillis()
        let lastUpdate = lastUpdateTime.get(scopeKey) ?? 0
        let timeSinceUpdate = currentTime - lastUpdate

        // 检查时间间隔
        if (timeSinceUpdate >= config.updateInterval) {
            return true
        }

        // 检查变化阈值
        let changes = changeTracker.get(scopeKey) ?? 0
        if (changes >= config.changeThreshold) {
            return true
        }

        return false
    }

    /**
     * 确定更新策略
     */
    private func determineUpdateStrategy(scopeKey: String, memories: Array<HierarchicalMemoryRecord>): SummaryUpdateStrategy {
        // 如果配置了特定策略，使用配置的策略
        if (config.strategy != SummaryUpdateStrategy.THRESHOLD) {
            return config.strategy
        }

        // 根据记忆数量和变化情况自动选择策略
        let changes = changeTracker.get(scopeKey) ?? 0
        let memoryCount = Int64(memories.size)

        if (changes > memoryCount / 2) {
            return SummaryUpdateStrategy.FULL_REBUILD
        } else {
            return SummaryUpdateStrategy.INCREMENTAL
        }
    }

    /**
     * 执行更新
     */
    private func executeUpdate(scope: MemoryScope, memories: Array<HierarchicalMemoryRecord>,
                              strategy: SummaryUpdateStrategy): SummaryUpdateResult {
        try {
            let result = if (strategy == SummaryUpdateStrategy.FULL_REBUILD) {
                executeFullRebuild(scope, memories)
            } else {
                executeIncrementalUpdate(scope, memories)
            }

            return result

        } catch (e: Exception) {
            return SummaryUpdateResult(
                false, 0, None, 0, strategy, "执行更新失败", [e.toString()]
            )
        }
    }

    /**
     * 执行完全重建
     */
    private func executeFullRebuild(scope: MemoryScope, memories: Array<HierarchicalMemoryRecord>): SummaryUpdateResult {
        let summaryResult = hierarchicalSummarizer.generateHierarchicalSummary(memories, scope)

        return SummaryUpdateResult(
            true, 1, Some(getNextVersionNumber(generateScopeKey(scope))),
            summaryResult.processingTime, SummaryUpdateStrategy.FULL_REBUILD,
            "完全重建摘要", Array<String>()
        )
    }

    /**
     * 执行增量更新
     */
    private func executeIncrementalUpdate(scope: MemoryScope, memories: Array<HierarchicalMemoryRecord>): SummaryUpdateResult {
        let summaryResult = hierarchicalSummarizer.updateHierarchicalSummary(scope, memories)

        return SummaryUpdateResult(
            true, 1, Some(getNextVersionNumber(generateScopeKey(scope))),
            summaryResult.processingTime, SummaryUpdateStrategy.INCREMENTAL,
            "增量更新摘要", Array<String>()
        )
    }

    /**
     * 添加摘要版本
     */
    private func addSummaryVersion(scopeKey: String, updateResult: SummaryUpdateResult, strategy: SummaryUpdateStrategy): Unit {
        if (updateResult.newVersion.isNone()) {
            return
        }

        let version = updateResult.newVersion.getOrThrow()
        let currentTime = getCurrentTimeMillis()
        let changes = changeTracker.get(scopeKey) ?? 0

        // 创建一个简化的摘要结果
        let dummySummary = SummaryResult(
            updateResult.changesSummary, 0, Int64(updateResult.changesSummary.size),
            1.0, Array<String>(), 0.8, SummaryStrategy.RULE_BASED, updateResult.processingTime
        )

        let summaryVersion = SummaryVersion(
            version, dummySummary, currentTime, changes, strategy.toString()
        )

        let existingVersions = summaryVersions.get(scopeKey) ?? Array<SummaryVersion>()
        let newVersions = ArrayList<SummaryVersion>()

        for (existingVersion in existingVersions) {
            newVersions.add(existingVersion)
        }
        newVersions.add(summaryVersion)

        summaryVersions[scopeKey] = newVersions.toArray()

        // 如果启用自动清理，检查版本数量
        if (config.autoCleanup && newVersions.size > config.maxVersions) {
            cleanupVersionsForScope(scopeKey)
        }
    }

    /**
     * 更新跟踪信息
     */
    private func updateTrackingInfo(scopeKey: String): Unit {
        let currentTime = getCurrentTimeMillis()
        lastUpdateTime[scopeKey] = currentTime
        changeTracker[scopeKey] = 0  // 重置变化计数
        pendingUpdates.remove(scopeKey)  // 清除待更新队列
    }

    /**
     * 创建无更新结果
     */
    private func createNoUpdateResult(): SummaryUpdateResult {
        return SummaryUpdateResult(
            true, 0, None, 0, config.strategy, "无需更新", Array<String>()
        )
    }

    /**
     * 创建待更新结果
     */
    private func createPendingUpdateResult(changeCount: Int64): SummaryUpdateResult {
        return SummaryUpdateResult(
            true, 0, None, 0, SummaryUpdateStrategy.INCREMENTAL,
            "已添加到待更新队列，变化数量：${changeCount}", Array<String>()
        )
    }

    /**
     * 合并记忆数组
     */
    private func combineMemoryArrays(existing: Array<HierarchicalMemoryRecord>,
                                    newMemories: Array<HierarchicalMemoryRecord>): Array<HierarchicalMemoryRecord> {
        let combined = ArrayList<HierarchicalMemoryRecord>()

        for (memory in existing) {
            combined.add(memory)
        }

        for (memory in newMemories) {
            combined.add(memory)
        }

        return combined.toArray()
    }

    /**
     * 按时间戳排序版本
     */
    private func sortVersionsByTimestamp(versions: Array<SummaryVersion>): Array<SummaryVersion> {
        // 简化实现：返回原数组（实际应该按时间戳排序）
        return versions
    }

    /**
     * 获取下一个版本号
     */
    private func getNextVersionNumber(scopeKey: String): Int64 {
        let versions = summaryVersions.get(scopeKey) ?? Array<SummaryVersion>()
        var maxVersion: Int64 = 0

        for (version in versions) {
            if (version.version > maxVersion) {
                maxVersion = version.version
            }
        }

        return maxVersion + 1
    }

    /**
     * 清理指定作用域的版本
     */
    private func cleanupVersionsForScope(scopeKey: String): Unit {
        let versions = summaryVersions.get(scopeKey) ?? Array<SummaryVersion>()
        if (versions.size <= config.maxVersions) {
            return
        }

        let sortedVersions = sortVersionsByTimestamp(versions)
        let toKeep = Int64(config.maxVersions)
        let newVersions = ArrayList<SummaryVersion>()

        for (i in 0..toKeep) {
            if (i < sortedVersions.size) {
                newVersions.add(sortedVersions[i])
            }
        }

        summaryVersions[scopeKey] = newVersions.toArray()
    }

    /**
     * 获取总版本数
     */
    private func getTotalVersionCount(): Int64 {
        var total: Int64 = 0

        for ((_, versions) in summaryVersions) {
            total = total + Int64(versions.size)
        }

        return total
    }

    /**
     * 获取当前时间（毫秒）
     */
    private func getCurrentTimeMillis(): Int64 {
        return 1000  // 简化实现
    }
}
