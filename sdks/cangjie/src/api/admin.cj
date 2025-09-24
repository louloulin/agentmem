/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 管理功能 - AgentMem 仓颉 SDK 的管理和维护功能
 */

package agentmem.api

import agentmem.core.*
import agentmem.ffi.*
import agentmem.utils.{TimeUtils}

/// 管理功能管理器 - 提供系统管理和维护功能
public class AdminManager {
    /// 客户端引用
    private var _client: AgentMemClient

    public init(client: AgentMemClient) {
        this._client = client
    }

    /// 系统健康检查
    public func systemHealthCheck(): AgentMemResult<SystemHealth> {
        let monitor = PerformanceMonitor("system_health_check")

        try {
            // 检查客户端连接
            let connectionResult = this._client.healthCheck()
            let isConnected = match (connectionResult) {
                case Ok(healthy) => healthy
                case Err(_) => false
            }

            // 获取系统版本
            let version = this._client.getVersion()

            // 获取全局统计信息
            let statsResult = this._client.getGlobalStats()
            let stats = match (statsResult) {
                case Ok(s) => Some(s)
                case Err(_) => None
            }

            let health = SystemHealth(
                isConnected,
                version,
                TimeUtils.getCurrentTimestamp(),
                stats
            )

            monitor.finishAndLog()
            return AgentMemResult<SystemHealth>.Ok(health)
            
        } catch (e: Exception) {
            return AgentMemResult<SystemHealth>.Err(
                AgentMemError.InternalError("Failed to perform health check: ${e}")
            )
        }
    }

    /// 数据备份 (重载1)
    public func backupData(outputPath: String): AgentMemResult<BackupInfo> {
        return this.backupData(None<String>, outputPath, BackupFormat.Json)
    }

    /// 数据备份 (重载2)
    public func backupData(
        agentId: Option<String>,
        outputPath: String,
        format: BackupFormat
    ): AgentMemResult<BackupInfo> {
        let monitor = PerformanceMonitor("backup_data")

        try {
            let targetAgentId = agentId.getOrElse("*")
            let formatStr = format.toString()
            
            let exportResult = this._client.exportMemories(targetAgentId, formatStr, outputPath)
            if (exportResult.isOk()) {
                let backupInfo = BackupInfo(
                    outputPath,
                    format,
                    TimeUtils.getCurrentTimestamp(),
                    targetAgentId
                )
                monitor.finishAndLog()
                return AgentMemResult<BackupInfo>.Ok(backupInfo)
            } else {
                let error = exportResult.getErr()
                return AgentMemResult<BackupInfo>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<BackupInfo>.Err(
                AgentMemError.InternalError("Failed to backup data: ${e}")
            )
        }
    }

    /// 数据恢复 (重载1)
    public func restoreData(agentId: String, inputPath: String): AgentMemResult<RestoreInfo> {
        return this.restoreData(agentId, inputPath, BackupFormat.Json, false)
    }

    /// 数据恢复 (重载2)
    public func restoreData(
        agentId: String,
        inputPath: String,
        format: BackupFormat,
        overwrite: Bool
    ): AgentMemResult<RestoreInfo> {
        let monitor = PerformanceMonitor("restore_data")

        try {
            let formatStr = format.toString()
            
            let importResult = this._client.importMemories(agentId, formatStr, inputPath)
            if (importResult.isOk()) {
                let batchResult = importResult.getOk()
                let restoreInfo = RestoreInfo(
                    inputPath,
                    format,
                    TimeUtils.getCurrentTimestamp(),
                    agentId,
                    batchResult.successes.size,
                    batchResult.failures.size,
                    batchResult.failures
                )
                monitor.finishAndLog()
                return AgentMemResult<RestoreInfo>.Ok(restoreInfo)
            } else {
                let error = importResult.getErr()
                return AgentMemResult<RestoreInfo>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<RestoreInfo>.Err(
                AgentMemError.InternalError("Failed to restore data: ${e}")
            )
        }
    }

    /// 数据压缩 (重载1)
    public func compressData(agentId: String): AgentMemResult<CompressionInfo> {
        return this.compressData(agentId, 0.8)
    }

    /// 数据压缩 (重载2)
    public func compressData(
        agentId: String,
        compressionRatio: Float32
    ): AgentMemResult<CompressionInfo> {
        let monitor = PerformanceMonitor("compress_data")

        try {
            // 获取压缩前的统计信息
            let beforeStatsResult = this._client.getMemoryStats(agentId)
            let beforeStats = if (beforeStatsResult.isOk()) {
                Some(beforeStatsResult.getOk())
            } else {
                None<MemoryStats>
            }

            // 执行压缩
            let compressResult = this._client.compressMemories(agentId, compressionRatio)
            if (compressResult.isOk()) {
                // 获取压缩后的统计信息
                let afterStatsResult = this._client.getMemoryStats(agentId)
                let afterStats = if (afterStatsResult.isOk()) {
                    Some(afterStatsResult.getOk())
                } else {
                    None<MemoryStats>
                }

                let compressionInfo = CompressionInfo(
                    agentId,
                    compressionRatio,
                    TimeUtils.getCurrentTimestamp(),
                    beforeStats,
                    afterStats
                )

                monitor.finishAndLog()
                return AgentMemResult<CompressionInfo>.Ok(compressionInfo)
            } else {
                let error = compressResult.getErr()
                return AgentMemResult<CompressionInfo>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<CompressionInfo>.Err(
                AgentMemError.InternalError("Failed to compress data: ${e}")
            )
        }
    }

    /// 数据清理 - 删除过期或低重要性的记忆
    public func cleanupData(
        agentId: String,
        criteria: CleanupCriteria
    ): AgentMemResult<CleanupInfo> {
        let monitor = PerformanceMonitor("cleanup_data")

        try {
            // 构建搜索过滤器来找到需要清理的记忆
            var filter = SearchFilter()
            
            if (criteria.maxAge.isSome()) {
                let cutoffTime = TimeUtils.getCurrentTimestamp() - criteria.maxAge.getOrThrow()
                filter.timeRange = Some((0, cutoffTime))
            }
            
            if (criteria.maxImportance.isSome()) {
                filter.importanceRange = Some((0.0, criteria.maxImportance.getOrThrow()))
            }
            
            filter.agentIds = Some([agentId])

            // 搜索需要清理的记忆
            let searchResult = this._client.searchMemoriesFiltered("*", filter, 1000)
            if (searchResult.isOk()) {
                let searchResults = searchResult.getOk()
                var deletedCount: UInt32 = 0
                var failedCount: UInt32 = 0
                var errors = Array<String>()

                // 删除找到的记忆
                for (result in searchResults) {
                    let deleteResult = this._client.deleteMemory(result.memory.id)
                    if (deleteResult.isOk()) {
                        deletedCount = deletedCount + 1
                    } else {
                        let error = deleteResult.getErr()
                        failedCount = failedCount + 1
                        errors = ArrayUtils.append(errors, error.getMessage())
                    }
                }

                let cleanupInfo = CleanupInfo(
                    agentId,
                    criteria,
                    TimeUtils.getCurrentTimestamp(),
                    deletedCount,
                    failedCount,
                    errors
                )

                monitor.finishAndLog()
                return AgentMemResult<CleanupInfo>.Ok(cleanupInfo)
            } else {
                let error = searchResult.getErr()
                return AgentMemResult<CleanupInfo>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<CleanupInfo>.Err(
                AgentMemError.InternalError("Failed to cleanup data: ${e}")
            )
        }
    }

    /// 性能监控 (重载1)
    public func getPerformanceMetrics(): AgentMemResult<PerformanceMetrics> {
        return this.getPerformanceMetrics(None<String>)
    }

    /// 性能监控 (重载2)
    public func getPerformanceMetrics(
        agentId: Option<String>
    ): AgentMemResult<PerformanceMetrics> {
        try {
            let stats = if (agentId.isSome()) {
                this._client.getMemoryStats(agentId.getOrThrow())
            } else {
                this._client.getGlobalStats()
            }

            if (stats.isOk()) {
                let memoryStats = stats.getOk()
                let metrics = PerformanceMetrics(
                    TimeUtils.getCurrentTimestamp(),
                    memoryStats,
                    this.calculateResponseTime(),
                    this.calculateThroughput(),
                    this.getMemoryUsage()
                )
                return AgentMemResult<PerformanceMetrics>.Ok(metrics)
            } else {
                let error = stats.getErr()
                return AgentMemResult<PerformanceMetrics>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<PerformanceMetrics>.Err(
                AgentMemError.InternalError("Failed to get performance metrics: ${e}")
            )
        }
    }

    /// 配置管理
    public func updateSystemConfig(config: SystemConfig): AgentMemResult<Unit> {
        try {
            // 设置日志级别
            this._client.setLogLevel(config.logLevel)
            
            // 这里可以添加更多配置更新逻辑
            
            return AgentMemResult<Unit>.Ok(())
            
        } catch (e: Exception) {
            return AgentMemResult<Unit>.Err(
                AgentMemError.InternalError("Failed to update system config: ${e}")
            )
        }
    }

    /// 获取系统信息
    public func getSystemInfo(): AgentMemResult<SystemInfo> {
        try {
            let version = this._client.getVersion()
            let config = this._client.getConfig()
            let isConnected = this._client.isConnected()
            
            let systemInfo = SystemInfo(
                version,
                config,
                isConnected,
                TimeUtils.getCurrentTimestamp()
            )
            
            return AgentMemResult<SystemInfo>.Ok(systemInfo)
            
        } catch (e: Exception) {
            return AgentMemResult<SystemInfo>.Err(
                AgentMemError.InternalError("Failed to get system info: ${e}")
            )
        }
    }

    // 私有辅助方法
    private func calculateResponseTime(): Float32 {
        // 简化实现，返回模拟的响应时间
        return 2.5
    }

    private func calculateThroughput(): Float32 {
        // 简化实现，返回模拟的吞吐量
        return 1000.0
    }

    private func getMemoryUsage(): UInt64 {
        // 简化实现，返回模拟的内存使用量
        return 104857600 // 100MB
    }
}

/// 系统健康状态
public struct SystemHealth {
    public var isConnected: Bool
    public var version: String
    public var checkTime: UInt64
    public var stats: Option<MemoryStats>

    public init(isConnected: Bool, version: String, checkTime: UInt64, stats: Option<MemoryStats>) {
        this.isConnected = isConnected
        this.version = version
        this.checkTime = checkTime
        this.stats = stats
    }
}

/// 备份格式枚举
public enum BackupFormat {
    | Json
    | Binary
    | Csv

    public func toString(): String {
        return match (this) {
            case Json => "json"
            case Binary => "binary"
            case Csv => "csv"
        }
    }
}

/// 备份信息
public struct BackupInfo {
    public var outputPath: String
    public var format: BackupFormat
    public var backupTime: UInt64
    public var agentId: String

    public init(outputPath: String, format: BackupFormat, backupTime: UInt64, agentId: String) {
        this.outputPath = outputPath
        this.format = format
        this.backupTime = backupTime
        this.agentId = agentId
    }
}

/// 恢复信息
public struct RestoreInfo {
    public var inputPath: String
    public var format: BackupFormat
    public var restoreTime: UInt64
    public var agentId: String
    public var successCount: Int32
    public var failureCount: Int32
    public var errors: Array<String>

    public init(
        inputPath: String, 
        format: BackupFormat, 
        restoreTime: UInt64,
        agentId: String,
        successCount: Int32,
        failureCount: Int32,
        errors: Array<String>
    ) {
        this.inputPath = inputPath
        this.format = format
        this.restoreTime = restoreTime
        this.agentId = agentId
        this.successCount = successCount
        this.failureCount = failureCount
        this.errors = errors
    }
}

/// 压缩信息
public struct CompressionInfo {
    public var agentId: String
    public var compressionRatio: Float32
    public var compressionTime: UInt64
    public var beforeStats: Option<MemoryStats>
    public var afterStats: Option<MemoryStats>

    public init(
        agentId: String,
        compressionRatio: Float32,
        compressionTime: UInt64,
        beforeStats: Option<MemoryStats>,
        afterStats: Option<MemoryStats>
    ) {
        this.agentId = agentId
        this.compressionRatio = compressionRatio
        this.compressionTime = compressionTime
        this.beforeStats = beforeStats
        this.afterStats = afterStats
    }
}

/// 清理标准
public struct CleanupCriteria {
    public var maxAge: Option<Int64> = None  // 最大年龄（秒）
    public var maxImportance: Option<Float32> = None  // 最大重要性
    public var memoryTypes: Option<Array<MemoryType>> = None  // 记忆类型

    public init() {}
}

/// 清理信息
public struct CleanupInfo {
    public var agentId: String
    public var criteria: CleanupCriteria
    public var cleanupTime: UInt64
    public var deletedCount: UInt32
    public var failedCount: UInt32
    public var errors: Array<String>

    public init(
        agentId: String,
        criteria: CleanupCriteria,
        cleanupTime: UInt64,
        deletedCount: UInt32,
        failedCount: UInt32,
        errors: Array<String>
    ) {
        this.agentId = agentId
        this.criteria = criteria
        this.cleanupTime = cleanupTime
        this.deletedCount = deletedCount
        this.failedCount = failedCount
        this.errors = errors
    }
}

/// 性能指标
public struct PerformanceMetrics {
    public var timestamp: UInt64
    public var memoryStats: MemoryStats
    public var averageResponseTime: Float32  // 毫秒
    public var throughput: Float32  // 请求/秒
    public var memoryUsage: UInt64  // 字节

    public init(
        timestamp: UInt64,
        memoryStats: MemoryStats,
        averageResponseTime: Float32,
        throughput: Float32,
        memoryUsage: UInt64
    ) {
        this.timestamp = timestamp
        this.memoryStats = memoryStats
        this.averageResponseTime = averageResponseTime
        this.throughput = throughput
        this.memoryUsage = memoryUsage
    }
}

/// 系统配置
public struct SystemConfig {
    public var logLevel: LogLevel
    public var cacheEnabled: Bool
    public var compressionEnabled: Bool

    public init() {
        this.logLevel = LogLevel.Info
        this.cacheEnabled = true
        this.compressionEnabled = true
    }

    public init(logLevel: LogLevel, cacheEnabled: Bool, compressionEnabled: Bool) {
        this.logLevel = logLevel
        this.cacheEnabled = cacheEnabled
        this.compressionEnabled = compressionEnabled
    }
}

/// 系统信息
public struct SystemInfo {
    public var version: String
    public var config: ClientConfig
    public var isConnected: Bool
    public var timestamp: UInt64

    public init(version: String, config: ClientConfig, isConnected: Bool, timestamp: UInt64) {
        this.version = version
        this.config = config
        this.isConnected = isConnected
        this.timestamp = timestamp
    }
}
