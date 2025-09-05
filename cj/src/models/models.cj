/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.models

import std.collection.HashMap
import std.collection.ArrayList

/**
 * 记忆记录的核心数据结构
 * 对应Mem0的MemoryItem
 */
public struct MemoryRecord {
    public let id: String                    // 唯一标识符
    public let memory: String                // 记忆内容
    public let hash: String                  // 内容哈希值
    public let score: Float64                // 相似度分数
    public let createdAt: Int64              // 创建时间戳
    public let updatedAt: Option<Int64>      // 更新时间戳
    public let metadata: HashMap<String, String>  // 元数据
    
    /**
     * 构造函数
     */
    public init(id: String, memory: String, hash: String, score: Float64, 
                createdAt: Int64, updatedAt: Option<Int64>, 
                metadata: HashMap<String, String>) {
        this.id = id
        this.memory = memory
        this.hash = hash
        this.score = score
        this.createdAt = createdAt
        this.updatedAt = updatedAt
        this.metadata = metadata
    }
    
    /**
     * 简化构造函数
     */
    public init(id: String, memory: String, hash: String) {
        this.id = id
        this.memory = memory
        this.hash = hash
        this.score = 0.0
        this.createdAt = 0  // 使用0作为默认时间戳，实际使用时应该设置为当前时间
        this.updatedAt = None
        this.metadata = HashMap<String, String>()
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "MemoryRecord(id=${id}, memory=${memory}, score=${score})"
    }
}

/**
 * 摘要记录结构
 * 用于存储对话或文档的摘要信息
 */
public struct SummaryRecord {
    public let id: String                    // 唯一标识符
    public let summary: String               // 摘要内容
    public let sourceIds: Array<String>      // 源记忆ID列表
    public let createdAt: Int64              // 创建时间戳
    public let metadata: HashMap<String, String>  // 元数据
    
    /**
     * 构造函数
     */
    public init(id: String, summary: String, sourceIds: Array<String>, 
                createdAt: Int64, metadata: HashMap<String, String>) {
        this.id = id
        this.summary = summary
        this.sourceIds = sourceIds
        this.createdAt = createdAt
        this.metadata = metadata
    }
    
    /**
     * 简化构造函数
     */
    public init(id: String, summary: String, sourceIds: Array<String>) {
        this.id = id
        this.summary = summary
        this.sourceIds = sourceIds
        this.createdAt = 0  // 使用0作为默认时间戳
        this.metadata = HashMap<String, String>()
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "SummaryRecord(id=${id}, summary=${summary}, sourceCount=${sourceIds.size})"
    }
}

/**
 * 关系记录结构
 * 用于存储记忆之间的关联关系
 */
public struct Relation {
    public let id: String                    // 关系ID
    public let sourceId: String              // 源记忆ID
    public let targetId: String              // 目标记忆ID
    public let relationType: String          // 关系类型
    public let strength: Float64             // 关系强度
    public let createdAt: Int64              // 创建时间戳
    
    /**
     * 构造函数
     */
    public init(id: String, sourceId: String, targetId: String, 
                relationType: String, strength: Float64, createdAt: Int64) {
        this.id = id
        this.sourceId = sourceId
        this.targetId = targetId
        this.relationType = relationType
        this.strength = strength
        this.createdAt = createdAt
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "Relation(${sourceId} -[${relationType}]-> ${targetId}, strength=${strength})"
    }
}

/**
 * 搜索结果结构
 * 包含记忆记录和相关的关系信息
 */
public struct SearchResult {
    public let memories: Array<MemoryRecord>  // 匹配的记忆列表
    public let relations: Array<Relation>     // 相关的关系列表
    
    /**
     * 构造函数
     */
    public init(memories: Array<MemoryRecord>, relations: Array<Relation>) {
        this.memories = memories
        this.relations = relations
    }
    
    /**
     * 简化构造函数
     */
    public init(memories: Array<MemoryRecord>) {
        this.memories = memories
        this.relations = Array<Relation>()
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "SearchResult(memories=${memories.size}, relations=${relations.size})"
    }
}

/**
 * 记忆操作事件类型
 */
public enum MemoryEvent {
    | ADD      // 添加记忆
    | UPDATE   // 更新记忆
    | DELETE   // 删除记忆
    | SEARCH   // 搜索记忆
    
    /**
     * 转换为字符串
     */
    public func toString(): String {
        match (this) {
            case ADD => "ADD"
            case UPDATE => "UPDATE"
            case DELETE => "DELETE"
            case SEARCH => "SEARCH"
        }
    }
}

/**
 * 记忆层级枚举
 * 定义三级记忆架构：User、Session、Agent
 */
public enum MemoryLevel {
    | USER      // 用户级记忆 - 跨会话持久化的用户个人信息
    | SESSION   // 会话级记忆 - 单次会话内的临时记忆
    | AGENT     // 代理级记忆 - 代理的全局知识和经验

    /**
     * 获取记忆层级的字符串表示
     */
    public func toString(): String {
        match (this) {
            case USER => "user"
            case SESSION => "session"
            case AGENT => "agent"
        }
    }

    /**
     * 从字符串解析记忆层级
     */
    public static func fromString(level: String): Option<MemoryLevel> {
        let lowerLevel = level  // 简化实现，暂时不使用toLowerCase
        match (lowerLevel) {
            case "user" => Some(USER)
            case "session" => Some(SESSION)
            case "agent" => Some(AGENT)
            case _ => None
        }
    }

    /**
     * 获取记忆层级的优先级（数值越小优先级越高）
     */
    public func getPriority(): Int64 {
        match (this) {
            case USER => 1      // 用户记忆优先级最高
            case SESSION => 2   // 会话记忆次之
            case AGENT => 3     // 代理记忆优先级最低
        }
    }

    /**
     * 相等比较操作符
     */
    public operator func ==(right: MemoryLevel): Bool {
        match (this) {
            case USER =>
                match (right) {
                    case USER => true
                    case _ => false
                }
            case SESSION =>
                match (right) {
                    case SESSION => true
                    case _ => false
                }
            case AGENT =>
                match (right) {
                    case AGENT => true
                    case _ => false
                }
        }
    }
}

/**
 * 记忆作用域定义
 * 控制记忆的访问范围和共享策略
 */
public struct MemoryScope {
    public let level: MemoryLevel           // 记忆层级
    public let userId: Option<String>       // 用户ID（USER级必需）
    public let sessionId: Option<String>    // 会话ID（SESSION级必需）
    public let agentId: Option<String>      // 代理ID（AGENT级必需）
    public let isShared: Bool               // 是否允许跨作用域共享
    public let accessLevel: String          // 访问级别：private, shared, public

    /**
     * 构造函数
     */
    public init(level: MemoryLevel, userId: Option<String>, sessionId: Option<String>,
                agentId: Option<String>, isShared: Bool, accessLevel: String) {
        this.level = level
        this.userId = userId
        this.sessionId = sessionId
        this.agentId = agentId
        this.isShared = isShared
        this.accessLevel = accessLevel
    }

    /**
     * 创建用户级记忆作用域
     */
    public static func createUserScope(userId: String, isShared: Bool): MemoryScope {
        let accessLevel = if (isShared) { "shared" } else { "private" }
        return MemoryScope(
            MemoryLevel.USER,
            Some(userId),
            None,
            None,
            isShared,
            accessLevel
        )
    }

    /**
     * 创建会话级记忆作用域
     */
    public static func createSessionScope(userId: String, sessionId: String): MemoryScope {
        return MemoryScope(
            MemoryLevel.SESSION,
            Some(userId),
            Some(sessionId),
            None,
            false,  // 会话记忆默认不跨作用域共享
            "private"
        )
    }

    /**
     * 创建代理级记忆作用域
     */
    public static func createAgentScope(agentId: String, isShared: Bool): MemoryScope {
        let accessLevel = if (isShared) { "public" } else { "private" }
        return MemoryScope(
            MemoryLevel.AGENT,
            None,
            None,
            Some(agentId),
            isShared,
            accessLevel
        )
    }

    /**
     * 验证作用域的有效性
     */
    public func isValid(): Bool {
        match (level) {
            case MemoryLevel.USER => userId.isSome()
            case MemoryLevel.SESSION => userId.isSome() && sessionId.isSome()
            case MemoryLevel.AGENT => agentId.isSome()
        }
    }

    /**
     * 生成作用域的唯一标识符
     */
    public func getScopeId(): String {
        match (level) {
            case MemoryLevel.USER =>
                match (userId) {
                    case Some(id) => "user:" + id
                    case None => "user:unknown"
                }
            case MemoryLevel.SESSION =>
                match (userId) {
                    case Some(uid) =>
                        match (sessionId) {
                            case Some(sid) => "session:" + uid + ":" + sid
                            case None => "session:" + uid + ":unknown"
                        }
                    case None =>
                        match (sessionId) {
                            case Some(sid) => "session:unknown:" + sid
                            case None => "session:unknown:unknown"
                        }
                }
            case MemoryLevel.AGENT =>
                match (agentId) {
                    case Some(id) => "agent:" + id
                    case None => "agent:unknown"
                }
        }
    }

    /**
     * 检查是否可以访问指定的记忆作用域
     */
    public func canAccess(targetScope: MemoryScope): Bool {
        // 相同作用域总是可以访问
        if (this.getScopeId() == targetScope.getScopeId()) {
            return true
        }

        // 检查共享权限
        if (!targetScope.isShared) {
            return false
        }

        // 根据访问级别判断
        match (targetScope.accessLevel) {
            case "public" => true
            case "shared" =>
                // 共享级别需要有相同的用户或代理
                match (targetScope.level) {
                    case MemoryLevel.USER => this.userId == targetScope.userId
                    case MemoryLevel.AGENT => this.agentId == targetScope.agentId
                    case MemoryLevel.SESSION => false  // 会话记忆不允许跨会话访问
                }
            case "private" => false
            case _ => false
        }
    }

    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "MemoryScope(${level.toString()}, ${getScopeId()}, ${accessLevel})"
    }
}

/**
 * 分层记忆记录
 * 扩展基础MemoryRecord，添加层级和作用域信息
 */
public struct HierarchicalMemoryRecord {
    public let baseRecord: MemoryRecord     // 基础记忆记录
    public let scope: MemoryScope           // 记忆作用域
    public let importance: Float64          // 重要性评分 (0.0-1.0)
    public let accessCount: Int64           // 访问次数
    public let lastAccessTime: Int64        // 最后访问时间
    public let expirationTime: Option<Int64> // 过期时间（可选）
    public let parentMemoryId: Option<String> // 父记忆ID（用于记忆层级关系）
    public let childMemoryIds: Array<String>  // 子记忆ID列表

    /**
     * 构造函数
     */
    public init(baseRecord: MemoryRecord, scope: MemoryScope, importance: Float64,
                accessCount: Int64, lastAccessTime: Int64, expirationTime: Option<Int64>,
                parentMemoryId: Option<String>, childMemoryIds: Array<String>) {
        this.baseRecord = baseRecord
        this.scope = scope
        this.importance = importance
        this.accessCount = accessCount
        this.lastAccessTime = lastAccessTime
        this.expirationTime = expirationTime
        this.parentMemoryId = parentMemoryId
        this.childMemoryIds = childMemoryIds
    }

    /**
     * 从基础记忆记录创建分层记忆记录
     */
    public static func fromBaseRecord(baseRecord: MemoryRecord, scope: MemoryScope): HierarchicalMemoryRecord {
        return HierarchicalMemoryRecord(
            baseRecord,
            scope,
            0.5,  // 默认重要性
            0,    // 初始访问次数
            baseRecord.createdAt,  // 最后访问时间设为创建时间
            None, // 默认无过期时间
            None, // 默认无父记忆
            Array<String>()  // 默认无子记忆
        )
    }

    /**
     * 更新访问信息
     */
    public func updateAccess(currentTime: Int64): HierarchicalMemoryRecord {
        return HierarchicalMemoryRecord(
            baseRecord,
            scope,
            importance,
            accessCount + 1,
            currentTime,
            expirationTime,
            parentMemoryId,
            childMemoryIds
        )
    }

    /**
     * 更新重要性评分
     */
    public func updateImportance(newImportance: Float64): HierarchicalMemoryRecord {
        return HierarchicalMemoryRecord(
            baseRecord,
            scope,
            newImportance,
            accessCount,
            lastAccessTime,
            expirationTime,
            parentMemoryId,
            childMemoryIds
        )
    }

    /**
     * 检查记忆是否已过期
     */
    public func isExpired(currentTime: Int64): Bool {
        match (expirationTime) {
            case Some(expTime) => currentTime > expTime
            case None => false
        }
    }

    /**
     * 获取记忆的完整ID（包含作用域信息）
     */
    public func getFullId(): String {
        return scope.getScopeId() + ":" + baseRecord.id
    }

    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "HierarchicalMemoryRecord(${baseRecord.id}, ${scope.toString()}, importance=${importance})"
    }
}

/**
 * 上下文信息
 * 记录当前的上下文环境
 */
public struct ContextInfo {
    public let currentTime: Int64               // 当前时间
    public let sessionId: String                // 会话ID
    public let userActivity: String             // 用户活动
    public let environmentFactors: HashMap<String, String>  // 环境因素
    public let recentQueries: Array<String>     // 最近的查询
    public let activeTopics: Array<String>      // 活跃话题

    /**
     * 构造函数
     */
    public init(currentTime: Int64, sessionId: String, userActivity: String,
                environmentFactors: HashMap<String, String>, recentQueries: Array<String>,
                activeTopics: Array<String>) {
        this.currentTime = currentTime
        this.sessionId = sessionId
        this.userActivity = userActivity
        this.environmentFactors = environmentFactors
        this.recentQueries = recentQueries
        this.activeTopics = activeTopics
    }

    /**
     * 简化构造函数
     */
    public init(currentTime: Int64, sessionId: String) {
        this.currentTime = currentTime
        this.sessionId = sessionId
        this.userActivity = "general"
        this.environmentFactors = HashMap<String, String>()
        this.recentQueries = Array<String>()
        this.activeTopics = Array<String>()
    }
}

/**
 * 用户行为模式
 * 记录用户的行为特征和偏好
 */
public struct UserBehaviorPattern {
    public let userId: String                   // 用户ID
    public let preferredMemoryTypes: Array<String>  // 偏好的记忆类型
    public let accessTimePatterns: Array<Int64>     // 访问时间模式
    public let interactionIntensity: Float64        // 交互强度
    public let feedbackHistory: HashMap<String, String>  // 反馈历史
    public let learningRate: Float64                // 学习率
    public let adaptationSpeed: Float64             // 适应速度

    /**
     * 构造函数
     */
    public init(userId: String, preferredMemoryTypes: Array<String>, accessTimePatterns: Array<Int64>,
                interactionIntensity: Float64, feedbackHistory: HashMap<String, String>,
                learningRate: Float64, adaptationSpeed: Float64) {
        this.userId = userId
        this.preferredMemoryTypes = preferredMemoryTypes
        this.accessTimePatterns = accessTimePatterns
        this.interactionIntensity = interactionIntensity
        this.feedbackHistory = feedbackHistory
        this.learningRate = learningRate
        this.adaptationSpeed = adaptationSpeed
    }

    /**
     * 默认构造函数
     */
    public init(userId: String) {
        this.userId = userId
        this.preferredMemoryTypes = Array<String>()
        this.accessTimePatterns = Array<Int64>()
        this.interactionIntensity = 0.5
        this.feedbackHistory = HashMap<String, String>()
        this.learningRate = 0.1
        this.adaptationSpeed = 0.05
    }
}

/**
 * 访问频率统计
 * 记录记忆的访问频率相关信息
 */
public struct FrequencyStats {
    public let memoryId: String                 // 记忆ID
    public let totalAccesses: Int64             // 总访问次数
    public let dailyAccesses: Float64           // 日均访问次数
    public let weeklyAccesses: Float64          // 周均访问次数
    public let monthlyAccesses: Float64         // 月均访问次数
    public let accessTrend: String              // 访问趋势：increasing, decreasing, stable
    public let peakAccessTime: Int64            // 峰值访问时间
    public let lastAccessTime: Int64            // 最后访问时间
    public let frequencyScore: Float64          // 频率评分 (0.0-1.0)

    /**
     * 构造函数
     */
    public init(memoryId: String, totalAccesses: Int64, dailyAccesses: Float64,
                weeklyAccesses: Float64, monthlyAccesses: Float64, accessTrend: String,
                peakAccessTime: Int64, lastAccessTime: Int64, frequencyScore: Float64) {
        this.memoryId = memoryId
        this.totalAccesses = totalAccesses
        this.dailyAccesses = dailyAccesses
        this.weeklyAccesses = weeklyAccesses
        this.monthlyAccesses = monthlyAccesses
        this.accessTrend = accessTrend
        this.peakAccessTime = peakAccessTime
        this.lastAccessTime = lastAccessTime
        this.frequencyScore = frequencyScore
    }
}