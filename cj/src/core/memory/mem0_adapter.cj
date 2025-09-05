/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import std.time.DateTime
import contextengine.models.*
import contextengine.storage.history.HistoryRecord

/**
 * Mem0适配器
 * 提供与Python Mem0库语义对齐的接口
 */
public class Mem0Adapter {
    private let memoryService: MemoryService
    private let config: MemoryConfig
    
    /**
     * 构造函数
     */
    public init(memoryService: MemoryService, config: MemoryConfig) {
        this.memoryService = memoryService
        this.config = config
    }
    
    /**
     * 简化构造函数
     */
    public init(memoryService: MemoryService) {
        this.memoryService = memoryService
        this.config = MemoryConfig()
    }
    
    /**
     * 添加记忆（对应Mem0的add方法）
     * @param messages 消息内容（字符串或消息列表）
     * @param userId 用户ID
     * @param agentId 代理ID
     * @param runId 运行ID
     * @param metadata 元数据
     * @param infer 是否推理
     * @return 添加结果
     */
    public func add(messages: String, userId: Option<String>, agentId: Option<String>,
                    runId: Option<String>, metadata: Option<HashMap<String, String>>,
                    infer: Bool): Mem0AddResult {

        // 处理消息内容
        let processedContent = processMessages(messages, infer)

        // 构建元数据
        let finalMetadata = buildMetadata(userId, agentId, runId, metadata)

        // 添加推理标志到元数据
        finalMetadata["infer"] = infer.toString()

        // 添加记忆
        let memoryRecord = memoryService.add(processedContent, finalMetadata)

        // 构建返回结果
        let resultsList = ArrayList<Mem0MemoryItem>()
        resultsList.add(Mem0MemoryItem(
            memoryRecord.id,
            memoryRecord.memory,
            "ADD"
        ))

        return Mem0AddResult(resultsList.toArray())
    }
    
    /**
     * 简化添加方法
     */
    public func add(messages: String): Mem0AddResult {
        return add(messages, None, None, None, None, true)
    }
    
    /**
     * 搜索记忆（对应Mem0的search方法）
     * @param query 查询内容
     * @param userId 用户ID
     * @param agentId 代理ID
     * @param runId 运行ID
     * @param limit 返回数量限制
     * @param filters 过滤条件
     * @param threshold 相似度阈值
     * @return 搜索结果
     */
    public func search(query: String, userId: Option<String>, agentId: Option<String>, 
                       runId: Option<String>, limit: Int64, 
                       filters: Option<HashMap<String, String>>, 
                       threshold: Option<Float64>): Mem0SearchResult {
        
        // 构建过滤条件
        let finalFilters = buildFilters(userId, agentId, runId, filters)
        
        // 确定阈值
        let finalThreshold = match (threshold) {
            case Some(t) => t
            case None => config.defaultThreshold
        }
        
        // 执行搜索
        let searchResult = memoryService.search(query, limit, finalThreshold, finalFilters)
        
        // 转换结果格式
        let resultsList = ArrayList<Mem0MemoryItem>()
        for (memory in searchResult.memories) {
            resultsList.add(Mem0MemoryItem(
                memory.id,
                memory.memory,
                "SEARCH",
                memory.score,
                memory.hash,
                memory.createdAt.toString(),
                match (memory.updatedAt) {
                    case Some(dt) => dt.toString()
                    case None => ""
                },
                memory.metadata
            ))
        }
        
        return Mem0SearchResult(resultsList.toArray())
    }
    
    /**
     * 简化搜索方法
     */
    public func search(query: String): Mem0SearchResult {
        return search(query, None, None, None, 100, None, None)
    }
    
    /**
     * 更新记忆（对应Mem0的update方法）
     * @param memoryId 记忆ID
     * @param data 新数据
     * @return 更新结果
     */
    public func update(memoryId: String, data: String): Mem0UpdateResult {
        let success = memoryService.update(memoryId, data)
        if (success) {
            return Mem0UpdateResult("Memory updated successfully!")
        } else {
            throw MemoryServiceException("Failed to update memory with ID: ${memoryId}")
        }
    }
    
    /**
     * 删除记忆（对应Mem0的delete方法）
     * @param memoryId 记忆ID
     * @return 删除结果
     */
    public func delete(memoryId: String): Mem0DeleteResult {
        let success = memoryService.delete(memoryId)
        if (success) {
            return Mem0DeleteResult("Memory deleted successfully!")
        } else {
            throw MemoryServiceException("Failed to delete memory with ID: ${memoryId}")
        }
    }
    
    /**
     * 删除所有记忆（对应Mem0的delete_all方法）
     */
    public func deleteAll(userId: Option<String>, agentId: Option<String>, 
                          runId: Option<String>): Mem0DeleteResult {
        let filters = buildFilters(userId, agentId, runId, None)
        let deletedCount = memoryService.deleteAll(filters)
        return Mem0DeleteResult("Deleted ${deletedCount} memories successfully!")
    }
    
    /**
     * 重置所有记忆（对应Mem0的reset方法）
     */
    public func reset(): Mem0ResetResult {
        let success = memoryService.reset()
        if (success) {
            return Mem0ResetResult("All memories reset successfully!")
        } else {
            throw MemoryServiceException("Failed to reset memories")
        }
    }

    /**
     * 获取记忆历史记录（对应Mem0的history方法）
     * @param memoryId 记忆ID
     * @return 历史记录列表
     */
    public func history(memoryId: String): Array<Mem0HistoryItem> {
        let historyRecords = memoryService.getHistory(memoryId)
        let resultsList = ArrayList<Mem0HistoryItem>()

        for (record in historyRecords) {
            resultsList.add(Mem0HistoryItem(
                record.id,
                record.memoryId,
                record.oldMemory,
                record.newMemory,
                record.event,
                record.createdAt,
                record.updatedAt,
                record.isDeleted,
                record.actorId,
                record.role
            ))
        }

        return resultsList.toArray()
    }

    /**
     * 获取所有记忆（对应Mem0的get_all方法）
     * @param userId 用户ID
     * @param agentId 代理ID
     * @param runId 运行ID
     * @param limit 返回数量限制
     * @param filters 过滤条件
     * @return 记忆列表
     */
    public func getAll(userId: Option<String>, agentId: Option<String>,
                       runId: Option<String>, limit: Int64,
                       filters: Option<HashMap<String, String>>): Mem0SearchResult {

        // 构建过滤条件
        let finalFilters = buildFilters(userId, agentId, runId, filters)

        // 获取所有记忆（通过搜索空字符串实现）
        let searchResult = memoryService.search("", limit, 0.0, finalFilters)

        // 转换结果格式
        let resultsList = ArrayList<Mem0MemoryItem>()
        for (memory in searchResult.memories) {
            resultsList.add(Mem0MemoryItem(
                memory.id,
                memory.memory,
                "GET",
                memory.score,
                memory.hash,
                memory.createdAt.toString(),
                match (memory.updatedAt) {
                    case Some(dt) => dt.toString()
                    case None => ""
                },
                memory.metadata
            ))
        }

        return Mem0SearchResult(resultsList.toArray())
    }

    /**
     * 简化获取所有记忆方法
     */
    public func getAll(): Mem0SearchResult {
        return getAll(None, None, None, 100, None)
    }

    /**
     * 删除所有记忆（对应Mem0的delete_all方法）
     * @param userId 用户ID
     * @param agentId 代理ID
     * @param runId 运行ID
     * @param filters 过滤条件
     * @return 删除结果
     */
    public func deleteAll(userId: Option<String>, agentId: Option<String>,
                          runId: Option<String>,
                          filters: Option<HashMap<String, String>>): Mem0DeleteResult {

        // 构建过滤条件
        let finalFilters = buildFilters(userId, agentId, runId, filters)

        // 执行批量删除
        let deletedCount = memoryService.deleteAll(finalFilters)

        return Mem0DeleteResult("Successfully deleted ${deletedCount} memories!")
    }

    /**
     * 简化删除所有记忆方法
     */
    public func deleteAll(): Mem0DeleteResult {
        return deleteAll(None, None, None, None)
    }

    /**
     * 构建元数据
     */
    private func buildMetadata(userId: Option<String>, agentId: Option<String>,
                               runId: Option<String>,
                               inputMetadata: Option<HashMap<String, String>>): HashMap<String, String> {
        let metadata = HashMap<String, String>()

        // 复制输入的元数据
        if (inputMetadata.isSome()) {
            let m = inputMetadata.getOrThrow()
            for ((key, value) in m) {
                metadata[key] = value
            }
        }

        if (userId.isSome()) {
            metadata["user_id"] = userId.getOrThrow()
        }

        if (agentId.isSome()) {
            metadata["agent_id"] = agentId.getOrThrow()
        }

        if (runId.isSome()) {
            metadata["run_id"] = runId.getOrThrow()
        }

        return metadata
    }
    
    /**
     * 构建过滤条件
     */
    private func buildFilters(userId: Option<String>, agentId: Option<String>, 
                              runId: Option<String>, 
                              inputFilters: Option<HashMap<String, String>>): HashMap<String, String> {
        let filters = match (inputFilters) {
            case Some(f) => f
            case None => HashMap<String, String>()
        }
        
        if (userId.isSome()) {
            filters["user_id"] = userId.getOrThrow()
        }
        
        if (agentId.isSome()) {
            filters["agent_id"] = agentId.getOrThrow()
        }
        
        if (runId.isSome()) {
            filters["run_id"] = runId.getOrThrow()
        }
        
        return filters
    }

    /**
     * 处理消息内容
     * @param messages 原始消息内容
     * @param infer 是否进行推理处理
     * @return 处理后的内容
     */
    private func processMessages(messages: String, infer: Bool): String {
        // 如果不需要推理，直接返回原始内容
        if (!infer) {
            return messages
        }

        // 简化的消息处理逻辑
        // 在实际实现中，这里应该使用LLM进行智能提取
        var processedContent = messages

        // 基本的文本清理
        processedContent = processedContent.trimAscii()

        // 移除多余的空白字符（简化实现）
        while (processedContent.contains("  ")) {
            processedContent = processedContent.replace("  ", " ")
        }

        // 如果内容为空，使用默认内容
        if (processedContent.isEmpty()) {
            processedContent = "Empty message content"
        }

        return processedContent
    }
}

/**
 * Mem0记忆项结构
 */
public struct Mem0MemoryItem {
    public let id: String
    public let memory: String
    public let event: String
    public let score: Float64
    public let hash: String
    public let createdAt: String
    public let updatedAt: String
    public let metadata: HashMap<String, String>
    
    /**
     * 完整构造函数
     */
    public init(id: String, memory: String, event: String, score: Float64, 
                hash: String, createdAt: String, updatedAt: String, 
                metadata: HashMap<String, String>) {
        this.id = id
        this.memory = memory
        this.event = event
        this.score = score
        this.hash = hash
        this.createdAt = createdAt
        this.updatedAt = updatedAt
        this.metadata = metadata
    }
    
    /**
     * 简化构造函数（用于ADD事件）
     */
    public init(id: String, memory: String, event: String) {
        this.id = id
        this.memory = memory
        this.event = event
        this.score = 0.0
        this.hash = ""
        this.createdAt = ""
        this.updatedAt = ""
        this.metadata = HashMap<String, String>()
    }
}

/**
 * Mem0添加结果
 */
public struct Mem0AddResult {
    public let results: Array<Mem0MemoryItem>
    
    public init(results: Array<Mem0MemoryItem>) {
        this.results = results
    }
}

/**
 * Mem0搜索结果
 */
public struct Mem0SearchResult {
    public let results: Array<Mem0MemoryItem>
    
    public init(results: Array<Mem0MemoryItem>) {
        this.results = results
    }
}

/**
 * Mem0更新结果
 */
public struct Mem0UpdateResult {
    public let message: String
    
    public init(message: String) {
        this.message = message
    }
}

/**
 * Mem0删除结果
 */
public struct Mem0DeleteResult {
    public let message: String
    
    public init(message: String) {
        this.message = message
    }
}

/**
 * Mem0重置结果
 */
public struct Mem0ResetResult {
    public let message: String

    public init(message: String) {
        this.message = message
    }
}

/**
 * Mem0历史记录项结构
 */
public struct Mem0HistoryItem {
    public let id: String
    public let memoryId: String
    public let oldMemory: Option<String>
    public let newMemory: Option<String>
    public let event: String
    public let createdAt: String
    public let updatedAt: String
    public let isDeleted: Int64
    public let actorId: Option<String>
    public let role: Option<String>

    public init(id: String, memoryId: String, oldMemory: Option<String>,
                newMemory: Option<String>, event: String, createdAt: String,
                updatedAt: String, isDeleted: Int64, actorId: Option<String>,
                role: Option<String>) {
        this.id = id
        this.memoryId = memoryId
        this.oldMemory = oldMemory
        this.newMemory = newMemory
        this.event = event
        this.createdAt = createdAt
        this.updatedAt = updatedAt
        this.isDeleted = isDeleted
        this.actorId = actorId
        this.role = role
    }
}