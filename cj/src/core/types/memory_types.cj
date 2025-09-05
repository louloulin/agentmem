/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.types

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord

/**
 * 记忆类型枚举
 * 对应Mem0的记忆类型系统
 */
public enum MemoryType {
    | WORKING        // 工作记忆 - 临时性的、短期的记忆
    | FACTUAL        // 事实记忆 - 客观的、可验证的事实信息
    | EPISODIC       // 情景记忆 - 特定时间和地点的经历
    | SEMANTIC       // 语义记忆 - 概念性的、抽象的知识
    | PROCEDURAL     // 过程记忆 - 如何做某事的知识
    | GENERAL        // 通用记忆 - 默认类型
    
    /**
     * 获取记忆类型的字符串表示
     */
    public func toString(): String {
        match (this) {
            case WORKING => "working_memory"
            case FACTUAL => "factual_memory"
            case EPISODIC => "episodic_memory"
            case SEMANTIC => "semantic_memory"
            case PROCEDURAL => "procedural_memory"
            case GENERAL => "general_memory"
        }
    }
    
    /**
     * 从字符串解析记忆类型
     */
    public static func fromString(typeStr: String): MemoryType {
        match (typeStr) {
            case "working_memory" => WORKING
            case "factual_memory" => FACTUAL
            case "episodic_memory" => EPISODIC
            case "semantic_memory" => SEMANTIC
            case "procedural_memory" => PROCEDURAL
            case "general_memory" => GENERAL
            case _ => GENERAL  // 默认类型
        }
    }
    
    /**
     * 获取记忆类型的描述
     */
    public func getDescription(): String {
        match (this) {
            case WORKING => "临时性的工作记忆，用于短期信息处理"
            case FACTUAL => "客观的事实信息，可验证的知识"
            case EPISODIC => "特定时间和地点的个人经历"
            case SEMANTIC => "概念性的抽象知识和理解"
            case PROCEDURAL => "如何执行任务的程序性知识"
            case GENERAL => "通用记忆，未分类的信息"
        }
    }
    
    /**
     * 获取记忆类型的保留期限（天数）
     */
    public func getRetentionDays(): Int64 {
        match (this) {
            case WORKING => 1        // 工作记忆保留1天
            case FACTUAL => 365      // 事实记忆保留1年
            case EPISODIC => 180     // 情景记忆保留6个月
            case SEMANTIC => 730     // 语义记忆保留2年
            case PROCEDURAL => 1095  // 过程记忆保留3年
            case GENERAL => 90       // 通用记忆保留3个月
        }
    }
    
    /**
     * 获取记忆类型的优先级（数字越大优先级越高）
     */
    public func getPriority(): Int64 {
        match (this) {
            case WORKING => 1
            case GENERAL => 2
            case EPISODIC => 3
            case FACTUAL => 4
            case SEMANTIC => 5
            case PROCEDURAL => 6
        }
    }

    /**
     * 相等比较操作符
     */
    public operator func ==(right: MemoryType): Bool {
        return this.toString() == right.toString()
    }
}

/**
 * 类型化记忆记录
 * 扩展基础MemoryRecord，添加类型信息
 */
public struct TypedMemoryRecord {
    public let id: String
    public let memory: String
    public let hash: String
    public let score: Float64
    public let createdAt: Int64
    public let updatedAt: Option<Int64>
    public let metadata: HashMap<String, String>
    public let memoryType: MemoryType
    public let expiresAt: Option<Int64>  // 过期时间
    
    public init(id: String, memory: String, hash: String, score: Float64,
                createdAt: Int64, updatedAt: Option<Int64>, 
                metadata: HashMap<String, String>, memoryType: MemoryType) {
        this.id = id
        this.memory = memory
        this.hash = hash
        this.score = score
        this.createdAt = createdAt
        this.updatedAt = updatedAt
        this.metadata = metadata
        this.memoryType = memoryType
        
        // 计算过期时间
        let retentionDays = memoryType.getRetentionDays()
        if (retentionDays > 0) {
            this.expiresAt = Some(createdAt + retentionDays * 24 * 60 * 60 * 1000)
        } else {
            this.expiresAt = None
        }
    }
    
    /**
     * 检查记忆是否已过期
     */
    public func isExpired(currentTime: Int64): Bool {
        if (expiresAt.isSome()) {
            return currentTime > expiresAt.getOrThrow()
        }
        return false
    }
    
    /**
     * 转换为基础MemoryRecord
     */
    public func toMemoryRecord(): MemoryRecord {
        // 将类型信息添加到元数据中
        let enhancedMetadata = HashMap<String, String>()
        for ((key, value) in metadata) {
            enhancedMetadata[key] = value
        }
        enhancedMetadata["memory_type"] = memoryType.toString()
        if (expiresAt.isSome()) {
            enhancedMetadata["expires_at"] = expiresAt.getOrThrow().toString()
        }
        
        return MemoryRecord(id, memory, hash, score, createdAt, updatedAt, enhancedMetadata)
    }
}

/**
 * 记忆类型管理器
 * 负责类型化记忆的管理和操作
 */
public class MemoryTypeManager {
    private let typedMemories: HashMap<String, TypedMemoryRecord>
    
    public init() {
        this.typedMemories = HashMap<String, TypedMemoryRecord>()
    }
    
    /**
     * 添加类型化记忆
     */
    public func addTypedMemory(record: TypedMemoryRecord): Bool {
        typedMemories[record.id] = record
        return true
    }
    
    /**
     * 获取类型化记忆
     */
    public func getTypedMemory(memoryId: String): Option<TypedMemoryRecord> {
        if (typedMemories.contains(memoryId)) {
            return Some(typedMemories[memoryId])
        }
        return None
    }
    
    /**
     * 按类型获取记忆
     */
    public func getMemoriesByType(memoryType: MemoryType): Array<TypedMemoryRecord> {
        let resultList = ArrayList<TypedMemoryRecord>()
        for ((_, record) in typedMemories) {
            if (record.memoryType == memoryType) {
                resultList.add(record)
            }
        }
        return resultList.toArray()
    }
    
    /**
     * 获取所有记忆类型的统计
     */
    public func getTypeStatistics(): HashMap<String, Int64> {
        let stats = HashMap<String, Int64>()
        
        // 初始化所有类型的计数
        stats["working_memory"] = 0
        stats["factual_memory"] = 0
        stats["episodic_memory"] = 0
        stats["semantic_memory"] = 0
        stats["procedural_memory"] = 0
        stats["general_memory"] = 0
        
        // 统计每种类型的数量
        for ((_, record) in typedMemories) {
            let typeStr = record.memoryType.toString()
            let currentCount = stats[typeStr]
            stats[typeStr] = currentCount + 1
        }
        
        return stats
    }
    
    /**
     * 智能推断记忆类型
     */
    public func inferMemoryType(content: String, metadata: HashMap<String, String>): MemoryType {
        // 检查元数据中是否已指定类型
        if (metadata.contains("memory_type")) {
            return MemoryType.fromString(metadata["memory_type"])
        }
        
        // 基于内容的简单推断规则
        let lowerContent = content
        
        // 过程记忆关键词
        if (lowerContent.contains("如何") || lowerContent.contains("步骤") || 
            lowerContent.contains("方法") || lowerContent.contains("流程")) {
            return MemoryType.PROCEDURAL
        }
        
        // 事实记忆关键词
        if (lowerContent.contains("是") || lowerContent.contains("定义") || 
            lowerContent.contains("概念") || lowerContent.contains("原理")) {
            return MemoryType.FACTUAL
        }
        
        // 情景记忆关键词
        if (lowerContent.contains("昨天") || lowerContent.contains("今天") || 
            lowerContent.contains("时候") || lowerContent.contains("经历")) {
            return MemoryType.EPISODIC
        }
        
        // 工作记忆关键词
        if (lowerContent.contains("临时") || lowerContent.contains("暂时") || 
            lowerContent.contains("现在") || lowerContent.contains("当前")) {
            return MemoryType.WORKING
        }
        
        // 语义记忆关键词
        if (lowerContent.contains("理解") || lowerContent.contains("意义") || 
            lowerContent.contains("关系") || lowerContent.contains("概念")) {
            return MemoryType.SEMANTIC
        }
        
        // 默认为通用记忆
        return MemoryType.GENERAL
    }
    
    /**
     * 重置所有类型化记忆
     */
    public func reset(): Bool {
        typedMemories.clear()
        return true
    }
    
    /**
     * 清理过期记忆
     */
    public func cleanupExpiredMemories(currentTime: Int64): Int64 {
        let expiredIds = ArrayList<String>()

        // 找出过期的记忆
        for ((id, record) in typedMemories) {
            if (record.isExpired(currentTime)) {
                expiredIds.add(id)
            }
        }

        // 删除过期记忆
        for (id in expiredIds.toArray()) {
            typedMemories.remove(id)
        }

        return Int64(expiredIds.size)
    }

    /**
     * 按优先级排序记忆
     */
    public func getMemoriesByPriority(): Array<TypedMemoryRecord> {
        let allMemories = ArrayList<TypedMemoryRecord>()
        for ((_, record) in typedMemories) {
            allMemories.add(record)
        }

        // 简单的优先级排序（按类型优先级降序）
        let sortedList = ArrayList<TypedMemoryRecord>()
        let memories = allMemories.toArray()

        // 按优先级分组
        for (priority in [6, 5, 4, 3, 2, 1]) {
            for (memory in memories) {
                if (memory.memoryType.getPriority() == priority) {
                    sortedList.add(memory)
                }
            }
        }

        return sortedList.toArray()
    }

    /**
     * 获取记忆总数
     */
    public func getTotalCount(): Int64 {
        return Int64(typedMemories.size)
    }
}
