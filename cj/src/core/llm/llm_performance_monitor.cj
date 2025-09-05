/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.llm

import std.collection.HashMap
import std.collection.ArrayList

/**
 * 性能监控级别
 */
public enum MonitoringLevel {
    | BASIC      // 基础监控
    | DETAILED   // 详细监控
    | ADVANCED   // 高级监控
    
    public func toString(): String {
        match (this) {
            case BASIC => "basic"
            case DETAILED => "detailed"
            case ADVANCED => "advanced"
        }
    }

    public operator func ==(right: MonitoringLevel): Bool {
        match (this) {
            case BASIC =>
                match (right) {
                    case BASIC => true
                    case _ => false
                }
            case DETAILED =>
                match (right) {
                    case DETAILED => true
                    case _ => false
                }
            case ADVANCED =>
                match (right) {
                    case ADVANCED => true
                    case _ => false
                }
        }
    }
}

/**
 * 告警类型
 */
public enum AlertType {
    | HIGH_COST         // 高成本告警
    | LOW_QUALITY       // 低质量告警
    | SLOW_RESPONSE     // 慢响应告警
    | HIGH_ERROR_RATE   // 高错误率告警
    | QUOTA_EXCEEDED    // 配额超限告警
    
    public func toString(): String {
        match (this) {
            case HIGH_COST => "high_cost"
            case LOW_QUALITY => "low_quality"
            case SLOW_RESPONSE => "slow_response"
            case HIGH_ERROR_RATE => "high_error_rate"
            case QUOTA_EXCEEDED => "quota_exceeded"
        }
    }

    public operator func ==(right: AlertType): Bool {
        match (this) {
            case HIGH_COST =>
                match (right) {
                    case HIGH_COST => true
                    case _ => false
                }
            case LOW_QUALITY =>
                match (right) {
                    case LOW_QUALITY => true
                    case _ => false
                }
            case SLOW_RESPONSE =>
                match (right) {
                    case SLOW_RESPONSE => true
                    case _ => false
                }
            case HIGH_ERROR_RATE =>
                match (right) {
                    case HIGH_ERROR_RATE => true
                    case _ => false
                }
            case QUOTA_EXCEEDED =>
                match (right) {
                    case QUOTA_EXCEEDED => true
                    case _ => false
                }
        }
    }

    public operator func !=(right: AlertType): Bool {
        return !(this == right)
    }
}

/**
 * 告警信息
 */
public struct Alert {
    public let alertType: AlertType                 // 告警类型
    public let message: String                      // 告警消息
    public let severity: String                     // 严重程度
    public let timestamp: Int64                     // 时间戳
    public let metadata: HashMap<String, String>    // 元数据

    public init(alertType: AlertType, message: String, severity: String, timestamp: Int64,
                metadata: HashMap<String, String>) {
        this.alertType = alertType
        this.message = message
        this.severity = severity
        this.timestamp = timestamp
        this.metadata = metadata
    }
}

/**
 * 监控配置
 */
public struct MonitoringConfig {
    public let level: MonitoringLevel               // 监控级别
    public let enableAlerts: Bool                   // 是否启用告警
    public let costThreshold: Float64               // 成本阈值
    public let qualityThreshold: Float64            // 质量阈值
    public let responseTimeThreshold: Int64         // 响应时间阈值（毫秒）
    public let errorRateThreshold: Float64          // 错误率阈值
    public let monitoringInterval: Int64            // 监控间隔（秒）
    
    public init() {
        this.level = MonitoringLevel.BASIC
        this.enableAlerts = true
        this.costThreshold = 10.0
        this.qualityThreshold = 0.7
        this.responseTimeThreshold = 5000
        this.errorRateThreshold = 0.1
        this.monitoringInterval = 60
    }
    
    public init(level: MonitoringLevel, enableAlerts: Bool, costThreshold: Float64,
                qualityThreshold: Float64, responseTimeThreshold: Int64, errorRateThreshold: Float64,
                monitoringInterval: Int64) {
        this.level = level
        this.enableAlerts = enableAlerts
        this.costThreshold = costThreshold
        this.qualityThreshold = qualityThreshold
        this.responseTimeThreshold = responseTimeThreshold
        this.errorRateThreshold = errorRateThreshold
        this.monitoringInterval = monitoringInterval
    }
}

/**
 * 实时性能数据
 */
public struct RealTimeMetrics {
    public let currentCost: Float64                 // 当前成本
    public let currentQuality: Float64              // 当前质量
    public let currentResponseTime: Int64           // 当前响应时间
    public let currentErrorRate: Float64            // 当前错误率
    public let requestsPerMinute: Int64             // 每分钟请求数
    public let tokensPerMinute: Int64               // 每分钟令牌数
    public let timestamp: Int64                     // 时间戳
    
    public init(currentCost: Float64, currentQuality: Float64, currentResponseTime: Int64,
                currentErrorRate: Float64, requestsPerMinute: Int64, tokensPerMinute: Int64,
                timestamp: Int64) {
        this.currentCost = currentCost
        this.currentQuality = currentQuality
        this.currentResponseTime = currentResponseTime
        this.currentErrorRate = currentErrorRate
        this.requestsPerMinute = requestsPerMinute
        this.tokensPerMinute = tokensPerMinute
        this.timestamp = timestamp
    }
}

/**
 * LLM性能监控器
 * 负责监控LLM的性能指标和告警
 */
public class LlmPerformanceMonitor {
    private let config: MonitoringConfig
    private let metrics: HashMap<String, Float64>
    private let alerts: ArrayList<Alert>
    private let metricsHistory: ArrayList<RealTimeMetrics>
    private let providerMetrics: HashMap<String, HashMap<String, Float64>>
    private let lastMonitoringTime: Int64
    
    public init() {
        this.config = MonitoringConfig()
        this.metrics = HashMap<String, Float64>()
        this.alerts = ArrayList<Alert>()
        this.metricsHistory = ArrayList<RealTimeMetrics>()
        this.providerMetrics = HashMap<String, HashMap<String, Float64>>()
        this.lastMonitoringTime = 1000  // 简化实现

        initializeMetrics()
    }

    public init(config: MonitoringConfig) {
        this.config = config
        this.metrics = HashMap<String, Float64>()
        this.alerts = ArrayList<Alert>()
        this.metricsHistory = ArrayList<RealTimeMetrics>()
        this.providerMetrics = HashMap<String, HashMap<String, Float64>>()
        this.lastMonitoringTime = 1000  // 简化实现

        initializeMetrics()
    }
    
    /**
     * 记录LLM请求
     * @param provider 提供商名称
     * @param responseTime 响应时间
     * @param tokenCount 令牌数量
     * @param cost 成本
     * @param quality 质量评分
     * @param success 是否成功
     */
    public func recordRequest(provider: String, responseTime: Int64, tokenCount: Int64,
                             cost: Float64, quality: Float64, success: Bool): Unit {
        // 更新基础指标
        updateMetric("total_requests", 1.0)
        updateMetric("total_tokens", Float64(tokenCount))
        updateMetric("total_cost", cost)
        updateMetric("total_response_time", Float64(responseTime))
        
        if (success) {
            updateMetric("successful_requests", 1.0)
            updateMetric("total_quality", quality)
        } else {
            updateMetric("failed_requests", 1.0)
        }
        
        // 更新提供商特定指标
        updateProviderMetrics(provider, responseTime, tokenCount, cost, quality, success)
        
        // 检查告警条件
        if (config.enableAlerts) {
            checkAlertConditions(provider, responseTime, cost, quality)
        }
        
        // 记录实时指标
        if (config.level == MonitoringLevel.DETAILED || config.level == MonitoringLevel.ADVANCED) {
            recordRealTimeMetrics()
        }
    }
    
    /**
     * 获取当前性能指标
     * @return 性能指标
     */
    public func getCurrentMetrics(): HashMap<String, String> {
        let result = HashMap<String, String>()
        
        let totalRequests = metrics.get("total_requests") ?? 0.0
        let successfulRequests = metrics.get("successful_requests") ?? 0.0
        let totalCost = metrics.get("total_cost") ?? 0.0
        let totalTokens = metrics.get("total_tokens") ?? 0.0
        let totalResponseTime = metrics.get("total_response_time") ?? 0.0
        let totalQuality = metrics.get("total_quality") ?? 0.0
        
        result["total_requests"] = totalRequests.toString()
        result["success_rate"] = if (totalRequests > 0.0) { (successfulRequests / totalRequests).toString() } else { "0.0" }
        result["total_cost"] = totalCost.toString()
        result["average_cost_per_request"] = if (totalRequests > 0.0) { (totalCost / totalRequests).toString() } else { "0.0" }
        result["total_tokens"] = totalTokens.toString()
        result["average_tokens_per_request"] = if (totalRequests > 0.0) { (totalTokens / totalRequests).toString() } else { "0.0" }
        result["average_response_time"] = if (totalRequests > 0.0) { (totalResponseTime / totalRequests).toString() } else { "0.0" }
        result["average_quality"] = if (successfulRequests > 0.0) { (totalQuality / successfulRequests).toString() } else { "0.0" }
        
        return result
    }
    
    /**
     * 获取提供商性能对比
     * @return 提供商性能数据
     */
    public func getProviderComparison(): HashMap<String, HashMap<String, String>> {
        let comparison = HashMap<String, HashMap<String, String>>()
        
        for ((provider, metrics) in providerMetrics) {
            let providerStats = HashMap<String, String>()
            
            let requests = metrics.get("requests") ?? 0.0
            let cost = metrics.get("cost") ?? 0.0
            let responseTime = metrics.get("response_time") ?? 0.0
            let quality = metrics.get("quality") ?? 0.0
            let successRate = metrics.get("success_rate") ?? 0.0
            
            providerStats["requests"] = requests.toString()
            providerStats["average_cost"] = if (requests > 0.0) { (cost / requests).toString() } else { "0.0" }
            providerStats["average_response_time"] = if (requests > 0.0) { (responseTime / requests).toString() } else { "0.0" }
            providerStats["average_quality"] = if (requests > 0.0) { (quality / requests).toString() } else { "0.0" }
            providerStats["success_rate"] = successRate.toString()
            
            comparison[provider] = providerStats
        }
        
        return comparison
    }
    
    /**
     * 获取活跃告警
     * @return 告警列表
     */
    public func getActiveAlerts(): Array<Alert> {
        return alerts.toArray()
    }
    
    /**
     * 清除告警
     * @param alertType 告警类型（可选）
     */
    public func clearAlerts(alertType: Option<AlertType>): Unit {
        if (alertType.isSome()) {
            let targetType = alertType.getOrThrow()
            let filteredAlerts = ArrayList<Alert>()
            
            for (alert in alerts.toArray()) {
                if (alert.alertType != targetType) {
                    filteredAlerts.add(alert)
                }
            }
            
            alerts.clear()
            for (alert in filteredAlerts.toArray()) {
                alerts.add(alert)
            }
        } else {
            alerts.clear()
        }
    }
    
    /**
     * 获取实时指标历史
     * @param limit 限制数量
     * @return 历史指标
     */
    public func getMetricsHistory(limit: Option<Int64>): Array<RealTimeMetrics> {
        let historyArray = metricsHistory.toArray()
        let actualLimit = if (limit.isSome()) { limit.getOrThrow() } else { Int64(historyArray.size) }
        
        if (actualLimit >= historyArray.size) {
            return historyArray
        }
        
        let result = ArrayList<RealTimeMetrics>()
        let startIndex = historyArray.size - actualLimit
        
        for (i in startIndex..historyArray.size) {
            result.add(historyArray[i])
        }
        
        return result.toArray()
    }
    
    /**
     * 重置监控数据
     */
    public func resetMonitoring(): Unit {
        metrics.clear()
        alerts.clear()
        metricsHistory.clear()
        providerMetrics.clear()
        initializeMetrics()
    }
    
    /**
     * 生成监控报告
     * @return 监控报告
     */
    public func generateReport(): String {
        let currentMetrics = getCurrentMetrics()
        let providerComparison = getProviderComparison()
        let activeAlerts = getActiveAlerts()
        
        var report = "=== LLM性能监控报告 ===\n\n"
        
        // 基础指标
        report = report + "基础指标:\n"
        for ((key, value) in currentMetrics) {
            report = report + "  ${key}: ${value}\n"
        }
        
        // 提供商对比
        report = report + "\n提供商性能对比:\n"
        for ((provider, stats) in providerComparison) {
            report = report + "  ${provider}:\n"
            for ((metric, value) in stats) {
                report = report + "    ${metric}: ${value}\n"
            }
        }
        
        // 活跃告警
        report = report + "\n活跃告警 (${activeAlerts.size}个):\n"
        for (alert in activeAlerts) {
            report = report + "  [${alert.severity}] ${alert.alertType.toString()}: ${alert.message}\n"
        }
        
        return report
    }

    // ===== 私有方法 =====

    /**
     * 初始化指标
     */
    private func initializeMetrics(): Unit {
        metrics["total_requests"] = 0.0
        metrics["successful_requests"] = 0.0
        metrics["failed_requests"] = 0.0
        metrics["total_tokens"] = 0.0
        metrics["total_cost"] = 0.0
        metrics["total_response_time"] = 0.0
        metrics["total_quality"] = 0.0
    }

    /**
     * 更新指标
     */
    private func updateMetric(key: String, value: Float64): Unit {
        let currentValue = metrics.get(key) ?? 0.0
        metrics[key] = currentValue + value
    }

    /**
     * 更新提供商指标
     */
    private func updateProviderMetrics(provider: String, responseTime: Int64, tokenCount: Int64,
                                      cost: Float64, quality: Float64, success: Bool): Unit {
        let providerStats = providerMetrics.get(provider) ?? HashMap<String, Float64>()

        let requests = providerStats.get("requests") ?? 0.0
        let totalCost = providerStats.get("cost") ?? 0.0
        let totalResponseTime = providerStats.get("response_time") ?? 0.0
        let totalQuality = providerStats.get("quality") ?? 0.0
        let successfulRequests = providerStats.get("successful_requests") ?? 0.0

        providerStats["requests"] = requests + 1.0
        providerStats["cost"] = totalCost + cost
        providerStats["response_time"] = totalResponseTime + Float64(responseTime)

        if (success) {
            providerStats["successful_requests"] = successfulRequests + 1.0
            providerStats["quality"] = totalQuality + quality
        }

        let newSuccessfulRequests = providerStats.get("successful_requests") ?? 0.0
        let newTotalRequests = providerStats.get("requests") ?? 1.0
        providerStats["success_rate"] = newSuccessfulRequests / newTotalRequests

        providerMetrics[provider] = providerStats
    }

    /**
     * 检查告警条件
     */
    private func checkAlertConditions(provider: String, responseTime: Int64, cost: Float64, quality: Float64): Unit {
        let currentTime = getCurrentTimeMillis()

        // 检查高成本告警
        if (cost > config.costThreshold) {
            let alert = Alert(
                AlertType.HIGH_COST,
                "提供商 ${provider} 单次请求成本过高: ${cost}",
                "WARNING",
                currentTime,
                HashMap<String, String>()
            )
            alerts.add(alert)
        }

        // 检查低质量告警
        if (quality < config.qualityThreshold) {
            let alert = Alert(
                AlertType.LOW_QUALITY,
                "提供商 ${provider} 响应质量过低: ${quality}",
                "WARNING",
                currentTime,
                HashMap<String, String>()
            )
            alerts.add(alert)
        }

        // 检查慢响应告警
        if (responseTime > config.responseTimeThreshold) {
            let alert = Alert(
                AlertType.SLOW_RESPONSE,
                "提供商 ${provider} 响应时间过长: ${responseTime}ms",
                "INFO",
                currentTime,
                HashMap<String, String>()
            )
            alerts.add(alert)
        }

        // 检查错误率告警
        let totalRequests = metrics.get("total_requests") ?? 1.0
        let failedRequests = metrics.get("failed_requests") ?? 0.0
        let errorRate = failedRequests / totalRequests

        if (errorRate > config.errorRateThreshold) {
            let alert = Alert(
                AlertType.HIGH_ERROR_RATE,
                "整体错误率过高: ${errorRate}",
                "ERROR",
                currentTime,
                HashMap<String, String>()
            )
            alerts.add(alert)
        }
    }

    /**
     * 记录实时指标
     */
    private func recordRealTimeMetrics(): Unit {
        let currentTime = getCurrentTimeMillis()

        // 计算当前指标
        let totalRequests = metrics.get("total_requests") ?? 0.0
        let totalCost = metrics.get("total_cost") ?? 0.0
        let totalResponseTime = metrics.get("total_response_time") ?? 0.0
        let successfulRequests = metrics.get("successful_requests") ?? 0.0
        let failedRequests = metrics.get("failed_requests") ?? 0.0
        let totalQuality = metrics.get("total_quality") ?? 0.0

        let currentCost = if (totalRequests > 0.0) { totalCost / totalRequests } else { 0.0 }
        let currentQuality = if (successfulRequests > 0.0) { totalQuality / successfulRequests } else { 0.0 }
        let currentResponseTime = if (totalRequests > 0.0) { Int64(totalResponseTime / totalRequests) } else { 0 }
        let currentErrorRate = if (totalRequests > 0.0) { failedRequests / totalRequests } else { 0.0 }

        // 计算每分钟指标
        let timeDiff = currentTime - lastMonitoringTime
        let minutesFactor = if (timeDiff > 0) { 60000.0 / Float64(timeDiff) } else { 1.0 }
        let requestsPerMinute = Int64(totalRequests * minutesFactor)
        let tokensPerMinute = Int64((metrics.get("total_tokens") ?? 0.0) * minutesFactor)

        let realTimeMetrics = RealTimeMetrics(
            currentCost, currentQuality, currentResponseTime, currentErrorRate,
            requestsPerMinute, tokensPerMinute, currentTime
        )

        metricsHistory.add(realTimeMetrics)

        // 限制历史记录数量
        if (metricsHistory.size > 1000) {
            let newHistory = ArrayList<RealTimeMetrics>()
            let historyArray = metricsHistory.toArray()

            for (i in 100..historyArray.size) {
                newHistory.add(historyArray[i])
            }

            metricsHistory.clear()
            for (metric in newHistory.toArray()) {
                metricsHistory.add(metric)
            }
        }
    }

    /**
     * 获取当前时间（毫秒）
     */
    private func getCurrentTimeMillis(): Int64 {
        return 1000  // 简化实现
    }
}
