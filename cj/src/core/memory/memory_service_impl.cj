/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.time.DateTime
import std.convert.*
import contextengine.storage.backends.*
import contextengine.models.*
import std.collection.ArrayList
import contextengine.storage.history.{HistoryManager, HistoryRecord}
import contextengine.core.llm.{LlmBase, MemoryExtractor}
import contextengine.core.factory.EnhancedLlmFactory
import contextengine.core.config.{EnhancedMemoryConfig, LlmConfig}
import contextengine.core.types.{MemoryType, MemoryTypeManager, TypedMemoryRecord}
import contextengine.core.search.{AdvancedSearchEngine, PaginationParams, PaginatedSearchResult, EnhancedSearchParams, TimeRangeFilter}
import contextengine.storage.backends.{MultiStorageManager, StorageBackendConfig, StorageBackendType}
import contextengine.core.hierarchy.{MemoryHierarchyManager, MemoryImportanceScorer}
import contextengine.models.{HierarchicalMemoryRecord, MemoryScope, MemoryLevel}

/**
 * 记忆服务实现类
 * 整合KV、向量和图存储，提供完整的记忆管理功能
 */
public class MemoryServiceImpl <: MemoryService {
    private var kvStorage: KVStorage
    private let vectorStorage: VectorStorage
    private let graphStorage: Option<GraphStorage>
    private let config: MemoryConfig
    private let historyManager: HistoryManager
    private let memoryRecords: HashMap<String, MemoryRecord>  // 直接存储完整的记录对象
    private var llm: Option<LlmBase>                          // LLM实例
    private var memoryExtractor: Option<MemoryExtractor>      // 记忆提取器
    private let typeManager: MemoryTypeManager               // 记忆类型管理器
    private let advancedSearchEngine: AdvancedSearchEngine   // 高级搜索引擎
    private let multiStorageManager: MultiStorageManager     // 多存储后端管理器
    private let hierarchyManager: MemoryHierarchyManager     // 多层记忆架构管理器
    private let importanceScorer: MemoryImportanceScorer     // 重要性评分器
    private var memoryCounter: Int64
    
    /**
     * 构造函数
     * @param kvStorage KV存储
     * @param vectorStorage 向量存储
     * @param graphStorage 图存储（可选）
     * @param config 配置
     */
    public init(kvStorage: KVStorage, vectorStorage: VectorStorage,
                graphStorage: Option<GraphStorage>, config: MemoryConfig) {
        this.kvStorage = kvStorage
        this.vectorStorage = vectorStorage
        this.graphStorage = graphStorage
        this.config = config
        this.historyManager = HistoryManager()
        this.memoryRecords = HashMap<String, MemoryRecord>()
        this.llm = None
        this.memoryExtractor = None
        this.typeManager = MemoryTypeManager()
        this.advancedSearchEngine = AdvancedSearchEngine(vectorStorage, graphStorage, typeManager, memoryRecords)
        this.multiStorageManager = MultiStorageManager()
        this.hierarchyManager = MemoryHierarchyManager()
        this.importanceScorer = MemoryImportanceScorer()
        this.memoryCounter = 0
    }
    
    /**
     * 简化构造函数（使用默认存储）
     * @param config 配置
     */
    public init(config: MemoryConfig) {
        this.kvStorage = MemoryKVStorage()
        this.vectorStorage = MemoryVectorStorage()
        this.graphStorage = None
        this.config = config
        this.historyManager = HistoryManager()
        this.memoryRecords = HashMap<String, MemoryRecord>()
        this.llm = None
        this.memoryExtractor = None
        this.typeManager = MemoryTypeManager()
        this.advancedSearchEngine = AdvancedSearchEngine(vectorStorage, graphStorage, typeManager, memoryRecords)
        this.multiStorageManager = MultiStorageManager()
        this.hierarchyManager = MemoryHierarchyManager()
        this.importanceScorer = MemoryImportanceScorer()
        this.memoryCounter = 0
    }
    
    /**
     * 增强配置构造函数
     * @param enhancedConfig 增强配置
     */
    public init(enhancedConfig: EnhancedMemoryConfig) {
        this.kvStorage = MemoryKVStorage()
        this.vectorStorage = MemoryVectorStorage()
        this.graphStorage = None
        this.config = MemoryConfig()  // 使用基础配置
        this.historyManager = HistoryManager()
        this.memoryRecords = HashMap<String, MemoryRecord>()

        // 初始化LLM
        let llmInstance = EnhancedLlmFactory.create(enhancedConfig.llm.provider, enhancedConfig.llm)
        this.llm = Some(llmInstance)
        this.memoryExtractor = Some(MemoryExtractor(llmInstance))
        this.typeManager = MemoryTypeManager()
        this.advancedSearchEngine = AdvancedSearchEngine(vectorStorage, graphStorage, typeManager, memoryRecords)
        this.multiStorageManager = MultiStorageManager()
        this.hierarchyManager = MemoryHierarchyManager()
        this.importanceScorer = MemoryImportanceScorer()

        this.memoryCounter = 0
    }

    /**
     * 默认构造函数
     */
    public init() {
        this.kvStorage = MemoryKVStorage()
        this.vectorStorage = MemoryVectorStorage()
        this.graphStorage = None
        this.config = MemoryConfig()
        this.historyManager = HistoryManager()
        this.memoryRecords = HashMap<String, MemoryRecord>()
        this.llm = None
        this.memoryExtractor = None
        this.typeManager = MemoryTypeManager()
        this.advancedSearchEngine = AdvancedSearchEngine(vectorStorage, graphStorage, typeManager, memoryRecords)
        this.multiStorageManager = MultiStorageManager()
        this.hierarchyManager = MemoryHierarchyManager()
        this.importanceScorer = MemoryImportanceScorer()
        this.memoryCounter = 0
    }
    
    /**
     * 添加记忆
     * @param content 记忆内容
     * @param metadata 元数据
     * @return 记忆记录
     */
    public func add(content: String, metadata: HashMap<String, String>): MemoryRecord {
        var processedContent = content

        // 如果启用了LLM，进行智能处理
        if (memoryExtractor.isSome()) {
            let extractor = memoryExtractor.getOrThrow()

            // 提取事实信息
            let facts = extractor.extractFacts(content)
            if (facts.size > 0) {
                // 使用提取的第一个事实作为处理后的内容
                processedContent = facts[0]
            }

            // 检测与现有记忆的冲突
            let existingMemories = getAllMemoryContents()
            let hasConflict = extractor.detectConflicts(processedContent, existingMemories)
            if (hasConflict) {
                // 在元数据中标记冲突
                metadata["has_conflict"] = "true"
            }
        }

        // 推断记忆类型
        let inferredType = typeManager.inferMemoryType(processedContent, metadata)
        metadata["memory_type"] = inferredType.toString()

        // 生成记忆ID
        memoryCounter = memoryCounter + 1
        let memoryId = "memory_" + memoryCounter.toString()

        // 生成哈希
        let hash = generateHash(processedContent)

        // 创建记忆记录
        let now = DateTime.now()
        let timestamp = now.year * 10000000000 + Int64(now.month.toInteger()) * 100000000 + now.dayOfMonth * 1000000 + now.hour * 10000 + now.minute * 100 + now.second
        let record = MemoryRecord(
            memoryId,
            processedContent,
            hash,
            0.0,
            timestamp,
            Some(timestamp),
            metadata
        )
        
        // 存储到KV
        let recordJson = serializeMemoryRecord(record)
        kvStorage.put(memoryId, recordJson)

        // 同时存储到内存记录中（保持完整的元数据）
        memoryRecords[memoryId] = record

        // 生成向量并存储（模拟向量化）
        let vector = generateVector(content)
        vectorStorage.addVector(memoryId, vector, metadata)

        // 创建类型化记忆记录
        let typedRecord = TypedMemoryRecord(
            memoryId,
            processedContent,
            hash,
            0.0,
            timestamp,
            Some(timestamp),
            metadata,
            inferredType
        )
        typeManager.addTypedMemory(typedRecord)

        // 添加历史记录
        historyManager.addHistory(memoryId, None, Some(content), "ADD")

        return record
    }
    
    /**
     * 简化添加记忆方法
     * @param content 记忆内容
     * @return 记忆记录
     */
    public func add(content: String): MemoryRecord {
        let emptyMetadata = HashMap<String, String>()
        return add(content, emptyMetadata)
    }
    
    /**
     * 搜索记忆
     * @param query 查询内容
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @param filters 过滤条件
     * @return 搜索结果
     */
    public func search(query: String, limit: Int64, threshold: Float64,
                       filters: HashMap<String, String>): SearchResult {

        let memoriesList = ArrayList<MemoryRecord>()

        // 如果查询为空，返回所有记忆（用于get_all功能）
        if (query.isEmpty()) {
            var count: Int64 = 0

            for ((memoryId, record) in memoryRecords) {
                if (count >= limit) {
                    break
                }

                if (matchesFilters(record, filters)) {
                    memoriesList.add(record)
                    count = count + 1
                }
            }
        } else {
            // 正常的向量搜索
            let queryVector = generateVector(query)
            let vectorResults = vectorStorage.searchSimilar(queryVector, limit, threshold)

            for (vectorResult in vectorResults) {
                let memoryData = kvStorage.get(vectorResult.id)
                if (memoryData.isSome()) {
                    let record = deserializeMemoryRecord(memoryData.getOrThrow())
                    if (matchesFilters(record, filters)) {
                        // 更新分数
                        let updatedRecord = MemoryRecord(
                            record.id,
                            record.memory,
                            record.hash,
                            vectorResult.score,
                            record.createdAt,
                            record.updatedAt,
                            record.metadata
                        )
                        memoriesList.add(updatedRecord)
                    }
                }
            }
        }

        let memories = memoriesList.toArray()
        let relations = Array<Relation>()
        return SearchResult(memories, relations)
    }
    
    /**
     * 简化搜索方法
     * @param query 查询内容
     * @return 搜索结果
     */
    public func search(query: String): SearchResult {
        let emptyFilters = HashMap<String, String>()
        return search(query, 10, config.defaultThreshold, emptyFilters)
    }
    
    /**
      * 根据ID获取记忆
      * @param memoryId 记忆ID
      * @return 记忆记录（可选）
     */
    public func get(memoryId: String): Option<MemoryRecord> {
        // 优先从内存记录获取（保持完整元数据）
        if (memoryRecords.contains(memoryId)) {
            return Some(memoryRecords[memoryId])
        }

        // 回退到KV存储
        let memoryData = kvStorage.get(memoryId)
        if (memoryData.isSome()) {
            return Some(deserializeMemoryRecord(memoryData.getOrThrow()))
        } else {
            return None
        }
    }
    
    /**
     * 更新记忆
     * @param memoryId 记忆ID
     * @param newMemory 新记忆内容
     * @return 是否成功
     */
    public func update(memoryId: String, newMemory: String): Bool {
        let existingData = kvStorage.get(memoryId)
        if (existingData.isSome()) {
            let existingRecord = deserializeMemoryRecord(existingData.getOrThrow())
            
            // 创建更新的记录
            let now = DateTime.now()
            let newHash = generateHash(newMemory)
            let updatedRecord = MemoryRecord(
                existingRecord.id,
                newMemory,
                newHash,
                existingRecord.score,
                existingRecord.createdAt,
                Some(now.year * 10000000000 + Int64(now.month.toInteger()) * 100000000 + now.dayOfMonth * 1000000 + now.hour * 10000 + now.minute * 100 + now.second),
                existingRecord.metadata
            )
            
            // 更新KV存储
            let recordJson = serializeMemoryRecord(updatedRecord)
            kvStorage.put(memoryId, recordJson)

            // 同时更新内存记录
            memoryRecords[memoryId] = updatedRecord

            // 更新向量存储
            let newVector = generateVector(newMemory)
            vectorStorage.updateVector(memoryId, newVector, existingRecord.metadata)

            // 添加历史记录
            historyManager.addHistory(memoryId, Some(existingRecord.memory), Some(newMemory), "UPDATE")

            return true
        } else {
            return false
        }
    }
    
    /**
     * 删除记忆
     * @param memoryId 记忆ID
     * @return 是否成功
     */
    public func delete(memoryId: String): Bool {
        // 先获取记忆内容用于历史记录
        let existingData = kvStorage.get(memoryId)
        var oldMemory: Option<String> = None
        if (existingData.isSome()) {
            let existingRecord = deserializeMemoryRecord(existingData.getOrThrow())
            oldMemory = Some(existingRecord.memory)
        }

        let kvSuccess = kvStorage.delete(memoryId)
        let vectorSuccess = vectorStorage.deleteVector(memoryId)

        // 从内存记录中删除
        if (kvSuccess && vectorSuccess) {
            memoryRecords.remove(memoryId)
            // 添加历史记录
            historyManager.addHistory(memoryId, oldMemory, None, "DELETE")
        }

        return kvSuccess && vectorSuccess
    }
    
    /**
     * 删除所有记忆
     * @param filters 过滤条件
     * @return 删除数量
     */
    public func deleteAll(filters: HashMap<String, String>): Int64 {
        let allKeys = kvStorage.keys()
        var deletedCount: Int64 = 0
        
        for (key in allKeys) {
            let memoryData = kvStorage.get(key)
            if (memoryData.isSome()) {
                let record = deserializeMemoryRecord(memoryData.getOrThrow())
                if (matchesFilters(record, filters)) {
                    if (delete(key)) {
                        deletedCount = deletedCount + 1
                    }
                }
            }
        }
        
        return deletedCount
    }
    
    /**
     * 获取记忆历史
     * @param memoryId 记忆ID
     * @return 历史记录
     */
    public func getHistory(memoryId: String): Array<HistoryRecord> {
        return historyManager.getHistory(memoryId)
    }
    
    /**
     * 创建摘要
     * @param memoryIds 记忆ID列表
     * @return 摘要记录
     */
    public func createSummary(memoryIds: Array<String>): SummaryRecord {
        let memoriesList = ArrayList<MemoryRecord>()
        for (id in memoryIds) {
            let memory = get(id)
            if (memory.isSome()) {
                memoriesList.add(memory.getOrThrow())
            }
        }
        let memories = memoriesList.toArray()
        var content = "Summary of " + memories.size.toString() + " memories:\n"
        for (memory in memories) {
            content = content + "- " + memory.memory + "\n"
        }
        
        let now = DateTime.now()
        let sourceIds = Array<String>()
        return SummaryRecord(
            "summary_" + (now.year * 10000000000 + Int64(now.month.toInteger()) * 100000000 + now.dayOfMonth * 1000000 + now.hour * 10000 + now.minute * 100 + now.second).toString(),
            content,
            sourceIds
        )
    }
    
    /**
     * 重置所有记忆
     * @return 是否成功
     */
    public func reset(): Bool {
        let kvSuccess = kvStorage.clear()
        let vectorSuccess = vectorStorage.clear()
        memoryCounter = 0
        return kvSuccess && vectorSuccess
    }
    
    /**
     * 生成哈希值
     * @param content 内容
     * @return 哈希值
     */
    private func generateHash(content: String): String {
        // 简化实现：使用内容长度和前几个字符
        let prefix = if (content.size > 10) { "prefix_" + content.size.toString() } else { content }
        return "hash_" + content.size.toString() + "_" + prefix
    }
    
    /**
     * 生成向量（模拟）
     * @param content 内容
     * @return 向量
     */
    private func generateVector(content: String): Vector {
        let vectorList = ArrayList<Float64>()
        let dimension = config.vectorDimension

        // 简化实现：基于内容生成伪随机向量
        let seed = content.size
        for (i in 0..dimension) {
            let value = Float64(seed + i) / 100.0
            vectorList.add(value)
        }

        return Vector(vectorList.toArray())
    }
    
    /**
     * 序列化记忆记录
     * @param record 记忆记录
     * @return JSON字符串
     */
    private func serializeMemoryRecord(record: MemoryRecord): String {
        var json = "{"
        json = json + "\"id\":\"" + record.id + "\","
        json = json + "\"memory\":\"" + record.memory + "\","
        json = json + "\"hash\":\"" + record.hash + "\","
        json = json + "\"createdAt\":\"" + record.createdAt.toString() + "\","

        if (record.updatedAt.isSome()) {
            json = json + "\"updatedAt\":\"" + record.updatedAt.getOrThrow().toString() + "\","
        }

        json = json + "\"score\":" + record.score.toString() + ","

        // 序列化元数据
        json = json + "\"metadata\":{"
        var first = true
        for ((key, value) in record.metadata) {
            if (!first) {
                json = json + ","
            }
            json = json + "\"" + key + "\":\"" + value + "\""
            first = false
        }
        json = json + "}"

        json = json + "}"

        return json
    }
    
    /**
     * 反序列化记忆记录
     * @param json JSON字符串
     * @return 记忆记录
     */
    private func deserializeMemoryRecord(json: String): MemoryRecord {
        // 简化的JSON解析实现
        var id = "default_id"
        var memory = "default_memory"
        var hash = "default_hash"
        var score = 0.0
        var createdAt: Int64 = 0
        var updatedAt: Option<Int64> = None
        let metadata = HashMap<String, String>()

        let jsonArray = json.toArray()

        // 提取id
        let idStart = json.indexOf("\"id\":\"")
        if (idStart.isSome()) {
            let idValueStart = idStart.getOrThrow() + 6
            let idEnd = json.indexOf("\"", idValueStart)
            if (idEnd.isSome() && idEnd.getOrThrow() > idValueStart) {
                let idArray = jsonArray.slice(idValueStart, idEnd.getOrThrow() - idValueStart)
                id = String.fromUtf8(idArray)
            }
        }
        
        // 提取memory
        let memoryStart = json.indexOf("\"memory\":\"")
        if (memoryStart.isSome()) {
            let memoryValueStart = memoryStart.getOrThrow() + 10
            let memoryEnd = json.indexOf("\"", memoryValueStart)
            if (memoryEnd.isSome() && memoryEnd.getOrThrow() > memoryValueStart) {
                let memoryArray = jsonArray.slice(memoryValueStart, memoryEnd.getOrThrow() - memoryValueStart)
                memory = String.fromUtf8(memoryArray)
            }
        }
        
        // 提取hash
        let hashStart = json.indexOf("\"hash\":\"")
        if (hashStart.isSome()) {
            let hashValueStart = hashStart.getOrThrow() + 8
            let hashEnd = json.indexOf("\"", hashValueStart)
            if (hashEnd.isSome() && hashEnd.getOrThrow() > hashValueStart) {
                let hashArray = jsonArray.slice(hashValueStart, hashEnd.getOrThrow() - hashValueStart)
                hash = String.fromUtf8(hashArray)
            }
        }
        
        // 提取createdAt
        let createdAtStart = json.indexOf("\"createdAt\":\"")
        if (createdAtStart.isSome()) {
            let createdAtValueStart = createdAtStart.getOrThrow() + 13
            let createdAtEnd = json.indexOf("\"", createdAtValueStart)
            if (createdAtEnd.isSome() && createdAtEnd.getOrThrow() > createdAtValueStart) {
                let createdAtArray = jsonArray.slice(createdAtValueStart, createdAtEnd.getOrThrow() - createdAtValueStart)
                let createdAtStr = String.fromUtf8(createdAtArray)
                let parsedCreatedAt = Int64.tryParse(createdAtStr.trimAscii())
                createdAt = parsedCreatedAt ?? 0
            }
        }
        
        // 提取score
        let scoreStart = json.indexOf("\"score\":")
        if (scoreStart.isSome()) {
            let scoreValueStart = scoreStart.getOrThrow() + 8
            let scoreEnd = json.indexOf("}", scoreValueStart)
            if (scoreEnd.isSome() && scoreEnd.getOrThrow() > scoreValueStart) {
                let scoreArray = jsonArray.slice(scoreValueStart, scoreEnd.getOrThrow() - scoreValueStart)
                let scoreStr = String.fromUtf8(scoreArray)
                let parsedScore = Float64.tryParse(scoreStr.trimAscii())
                score = parsedScore ?? 0.0
            }
        }
        
        // 简化的元数据解析（实际应该解析JSON中的metadata字段）
        // 这里暂时使用空的元数据，后续可以改进JSON解析

        return MemoryRecord(
            id,
            memory,
            hash,
            score,
            createdAt,
            updatedAt,
            metadata
        )
    }
    
    /**
     * 检查记录是否匹配过滤条件
     * @param record 记忆记录
     * @param filters 过滤条件
     * @return 是否匹配
     */
    private func matchesFilters(record: MemoryRecord, filters: HashMap<String, String>): Bool {
        for ((key, value) in filters) {
            // 支持特殊的内置字段过滤
            if (key == "memory_id" || key == "id") {
                if (record.id != value) {
                    return false
                }
            } else if (key == "content" || key == "memory") {
                if (!record.memory.contains(value)) {
                    return false
                }
            } else if (key == "hash") {
                if (record.hash != value) {
                    return false
                }
            } else if (key == "created_after") {
                // 支持时间范围过滤
                let filterTime = Int64.tryParse(value) ?? 0
                if (record.createdAt <= filterTime) {
                    return false
                }
            } else if (key == "created_before") {
                let filterTime = Int64.tryParse(value) ?? 9223372036854775807
                if (record.createdAt >= filterTime) {
                    return false
                }
            } else {
                // 元数据字段过滤
                if (record.metadata.contains(key)) {
                    if (record.metadata[key] != value) {
                        return false
                    }
                } else {
                    return false
                }
            }
        }
        return true
    }

    /**
     * 获取所有记忆内容（用于LLM分析）
     * @return 记忆内容列表
     */
    private func getAllMemoryContents(): Array<String> {
        let contentsList = ArrayList<String>()
        for ((_, record) in memoryRecords) {
            contentsList.add(record.memory)
        }
        return contentsList.toArray()
    }

    /**
     * 按类型获取记忆
     * @param memoryType 记忆类型
     * @return 记忆记录列表
     */
    public func getMemoriesByType(memoryType: MemoryType): Array<MemoryRecord> {
        let typedMemories = typeManager.getMemoriesByType(memoryType)
        let resultList = ArrayList<MemoryRecord>()
        for (typedMemory in typedMemories) {
            resultList.add(typedMemory.toMemoryRecord())
        }
        return resultList.toArray()
    }

    /**
     * 获取记忆类型统计
     * @return 类型统计信息
     */
    public func getTypeStatistics(): HashMap<String, Int64> {
        return typeManager.getTypeStatistics()
    }

    /**
     * 清理过期记忆
     * @return 清理的记忆数量
     */
    public func cleanupExpiredMemories(): Int64 {
        let now = DateTime.now()
        let currentTime = now.year * 10000000000 + Int64(now.month.toInteger()) * 100000000 +
                         now.dayOfMonth * 1000000 + now.hour * 10000 + now.minute * 100 + now.second

        // 清理类型管理器中的过期记忆
        let expiredCount = typeManager.cleanupExpiredMemories(currentTime)

        // 同时从其他存储中删除过期记忆
        // 这里简化实现，实际应该遍历所有记忆检查过期时间

        return expiredCount
    }

    /**
     * 按优先级获取记忆
     * @return 按优先级排序的记忆列表
     */
    public func getMemoriesByPriority(): Array<MemoryRecord> {
        let prioritizedMemories = typeManager.getMemoriesByPriority()
        let resultList = ArrayList<MemoryRecord>()
        for (typedMemory in prioritizedMemories) {
            resultList.add(typedMemory.toMemoryRecord())
        }
        return resultList.toArray()
    }

    /**
     * 高级搜索（支持并行搜索和智能融合）
     * @param query 查询内容
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @param filters 过滤条件
     * @param enableParallel 是否启用并行搜索
     * @param fusionWeight 融合权重
     * @return 搜索结果
     */
    public func advancedSearch(query: String, limit: Int64, threshold: Float64,
                              filters: HashMap<String, String>, enableParallel: Bool,
                              fusionWeight: Float64): SearchResult {
        return advancedSearchEngine.advancedSearch(query, limit, threshold, filters, enableParallel, fusionWeight)
    }

    /**
     * 分页搜索
     * @param query 查询内容
     * @param page 页码
     * @param pageSize 每页大小
     * @param threshold 相似度阈值
     * @param filters 过滤条件
     * @return 分页搜索结果
     */
    public func paginatedSearch(query: String, page: Int64, pageSize: Int64,
                               threshold: Float64, filters: HashMap<String, String>): PaginatedSearchResult {
        let pagination = PaginationParams(page, pageSize)
        return advancedSearchEngine.paginatedSearch(query, pagination, threshold, filters)
    }

    /**
     * 按记忆类型搜索
     * @param query 查询内容
     * @param memoryType 记忆类型
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @return 搜索结果
     */
    public func searchByType(query: String, memoryType: MemoryType,
                            limit: Int64, threshold: Float64): SearchResult {
        return advancedSearchEngine.searchByType(query, memoryType, limit, threshold)
    }

    // ===== Phase 2.1 增强搜索功能 =====

    /**
     * 增强语义搜索
     * 支持上下文理解、意图识别和智能排序
     * @param params 增强搜索参数
     * @return 搜索结果
     */
    public func enhancedSemanticSearch(params: EnhancedSearchParams): SearchResult {
        return advancedSearchEngine.enhancedSemanticSearch(params)
    }

    /**
     * 时间范围搜索
     * @param query 查询内容
     * @param timeFilter 时间过滤器
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @return 搜索结果
     */
    public func searchByTimeRange(query: String, timeFilter: TimeRangeFilter,
                                 limit: Int64, threshold: Float64): SearchResult {
        return advancedSearchEngine.searchByTimeRange(query, timeFilter, limit, threshold)
    }

    /**
     * 多模态搜索
     * 支持文本、元数据、标签等多维度搜索
     * @param textQuery 文本查询
     * @param metadataFilters 元数据过滤器
     * @param tags 标签列表
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @return 搜索结果
     */
    public func multiModalSearch(textQuery: String, metadataFilters: HashMap<String, String>,
                                tags: Array<String>, limit: Int64, threshold: Float64): SearchResult {
        return advancedSearchEngine.multiModalSearch(textQuery, metadataFilters, tags, limit, threshold)
    }

    /**
     * 智能搜索推荐
     * 基于用户历史和上下文提供搜索建议
     * @param query 查询内容
     * @param userId 用户ID
     * @param limit 返回数量限制
     * @return 搜索结果
     */
    public func intelligentSearchRecommendation(query: String, userId: String, limit: Int64): SearchResult {
        // 构建上下文感知的搜索参数
        let userContext = buildUserContext(userId)
        let contextQuery = generateContextQuery(query, userContext)

        let params = EnhancedSearchParams(
            query, limit, 0.7, HashMap<String, String>(),
            None, true, true, 0.8, 0.2, Some(contextQuery)
        )

        return enhancedSemanticSearch(params)
    }

    /**
     * 相关记忆发现
     * 基于给定记忆发现相关的其他记忆
     * @param memoryId 基准记忆ID
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @return 搜索结果
     */
    public func discoverRelatedMemories(memoryId: String, limit: Int64, threshold: Float64): SearchResult {
        let baseMemory = get(memoryId)
        if (baseMemory.isNone()) {
            return SearchResult(Array<MemoryRecord>(), Array<Relation>())
        }

        let memory = baseMemory.getOrThrow()
        let params = EnhancedSearchParams(
            memory.memory, limit, threshold, HashMap<String, String>(),
            None, true, true, 0.9, 0.1, None
        )

        let results = enhancedSemanticSearch(params)

        // 过滤掉原始记忆
        let filteredMemories = ArrayList<MemoryRecord>()
        for (resultMemory in results.memories) {
            if (resultMemory.id != memoryId) {
                filteredMemories.add(resultMemory)
            }
        }

        return SearchResult(filteredMemories.toArray(), results.relations)
    }

    /**
     * 注册存储后端
     * @param name 后端名称
     * @param backendType 后端类型
     * @param filePath 文件路径（可选）
     * @return 是否成功
     */
    public func registerStorageBackend(name: String, backendType: StorageBackendType,
                                      filePath: Option<String>): Bool {
        let config = if (filePath.isSome()) {
            StorageBackendConfig(backendType, filePath.getOrThrow())
        } else {
            StorageBackendConfig(backendType)
        }

        return multiStorageManager.registerBackend(name, config)
    }

    /**
     * 切换存储后端
     * @param name 后端名称
     * @return 是否成功
     */
    public func switchStorageBackend(name: String): Bool {
        let success = multiStorageManager.switchToBackend(name)
        if (success) {
            // 更新当前使用的KV存储
            let currentBackend = multiStorageManager.getCurrentBackend()
            if (currentBackend.isSome()) {
                this.kvStorage = currentBackend.getOrThrow()
            }
        }
        return success
    }

    /**
     * 获取当前存储后端名称
     * @return 后端名称
     */
    public func getCurrentStorageBackend(): String {
        return multiStorageManager.getCurrentBackendName()
    }

    /**
     * 获取所有注册的存储后端
     * @return 后端名称列表
     */
    public func getRegisteredStorageBackends(): Array<String> {
        return multiStorageManager.getRegisteredBackends()
    }

    /**
     * 检查所有存储后端健康状态
     * @return 健康状态报告
     */
    public func checkStorageBackendsHealth(): HashMap<String, Bool> {
        return multiStorageManager.checkAllBackendsHealth()
    }

    /**
     * 获取存储后端统计信息
     * @return 统计信息
     */
    public func getStorageBackendStats(): HashMap<String, HashMap<String, String>> {
        return multiStorageManager.getBackendStats()
    }

    /**
     * 数据迁移：从一个后端迁移到另一个后端
     * @param fromBackend 源后端名称
     * @param toBackend 目标后端名称
     * @return 迁移的记录数量
     */
    public func migrateStorageData(fromBackend: String, toBackend: String): Int64 {
        return multiStorageManager.migrateData(fromBackend, toBackend)
    }

    /**
     * 数据同步：将当前后端的数据同步到所有其他后端
     * @return 同步的记录数量
     */
    public func syncStorageToAllBackends(): Int64 {
        return multiStorageManager.syncToAllBackends()
    }

    /**
     * 清理所有存储后端
     * @return 是否成功
     */
    public func clearAllStorageBackends(): Bool {
        return multiStorageManager.clearAllBackends()
    }

    /**
     * 切换LLM提供商
     * @param provider 提供商名称
     * @param apiKey API密钥（可选）
     * @return 是否成功
     */
    public func switchLlmProvider(provider: String, apiKey: Option<String>): Bool {
        try {
            // 创建新的LLM配置
            let newConfig = EnhancedLlmFactory.createRecommendedConfig(provider, apiKey)

            // 创建新的LLM实例
            let newLlmInstance = if (apiKey.isSome()) {
                EnhancedLlmFactory.createWithApiKey(provider, newConfig, apiKey.getOrThrow())
            } else {
                EnhancedLlmFactory.create(provider, newConfig)
            }

            // 验证配置
            if (!newLlmInstance.validateConfig()) {
                return false
            }

            // 更新LLM实例
            this.llm = Some(newLlmInstance)
            this.memoryExtractor = Some(MemoryExtractor(newLlmInstance))

            return true
        } catch (e: Exception) {
            return false
        }
    }

    /**
     * 获取当前LLM提供商信息
     * @return 提供商信息
     */
    public func getCurrentLlmProvider(): HashMap<String, String> {
        if (llm.isSome()) {
            let llmInstance = llm.getOrThrow()
            let info = HashMap<String, String>()
            info["provider"] = llmInstance.getProviderName()
            info["model"] = llmInstance.getModelName()
            info["status"] = "active"
            return info
        } else {
            let info = HashMap<String, String>()
            info["provider"] = "none"
            info["model"] = "none"
            info["status"] = "inactive"
            return info
        }
    }

    /**
     * 获取支持的LLM提供商列表
     * @return 提供商列表
     */
    public func getSupportedLlmProviders(): Array<String> {
        return EnhancedLlmFactory.getSupportedProviders()
    }

    /**
     * 测试LLM连接
     * @param provider 提供商名称
     * @param apiKey API密钥（可选）
     * @return 测试结果 (成功, 消息)
     */
    public func testLlmConnection(provider: String, apiKey: Option<String>): (Bool, String) {
        let config = EnhancedLlmFactory.createRecommendedConfig(provider, apiKey)
        return EnhancedLlmFactory.testConnection(provider, config)
    }

    /**
     * 获取LLM提供商信息
     * @param provider 提供商名称
     * @return 提供商详细信息
     */
    public func getLlmProviderInfo(provider: String): HashMap<String, String> {
        return EnhancedLlmFactory.getProviderInfo(provider)
    }

    /**
     * 获取所有LLM提供商统计信息
     * @return 统计信息
     */
    public func getAllLlmProvidersStats(): HashMap<String, HashMap<String, String>> {
        return EnhancedLlmFactory.getAllProvidersStats()
    }

    /**
     * 验证当前LLM配置
     * @return 是否有效
     */
    public func validateCurrentLlmConfig(): Bool {
        if (llm.isSome()) {
            return llm.getOrThrow().validateConfig()
        }
        return false
    }

    // ========== 分层记忆架构方法 ==========

    /**
     * 添加分层记忆
     * @param content 记忆内容
     * @param scope 记忆作用域
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    public func addHierarchicalMemory(content: String, scope: MemoryScope, metadata: HashMap<String, String>): HierarchicalMemoryRecord {
        // 首先创建基础记忆记录
        let baseRecord = add(content, metadata)

        // 创建分层记忆记录
        let hierarchicalRecord = HierarchicalMemoryRecord.fromBaseRecord(baseRecord, scope)

        // 计算初始重要性评分
        let currentTime = DateTime.now().year * 10000000000 + Int64(DateTime.now().month.toInteger()) * 100000000 + DateTime.now().dayOfMonth * 1000000 + DateTime.now().hour * 10000 + DateTime.now().minute * 100 + DateTime.now().second
        let contextMemories = hierarchyManager.getMemoriesInScope(scope)
        let importance = importanceScorer.calculateImportance(hierarchicalRecord, currentTime, contextMemories)
        let updatedRecord = hierarchicalRecord.updateImportance(importance)

        // 添加到层级管理器
        hierarchyManager.addMemory(updatedRecord)

        return updatedRecord
    }

    /**
     * 简化添加分层记忆方法
     * @param content 记忆内容
     * @param scope 记忆作用域
     * @return 分层记忆记录
     */
    public func addHierarchicalMemory(content: String, scope: MemoryScope): HierarchicalMemoryRecord {
        let emptyMetadata = HashMap<String, String>()
        return addHierarchicalMemory(content, scope, emptyMetadata)
    }

    /**
     * 添加用户级记忆
     * @param content 记忆内容
     * @param userId 用户ID
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    public func addUserMemory(content: String, userId: String, metadata: HashMap<String, String>): HierarchicalMemoryRecord {
        let scope = MemoryScope.createUserScope(userId, false)
        return addHierarchicalMemory(content, scope, metadata)
    }

    /**
     * 添加会话级记忆
     * @param content 记忆内容
     * @param userId 用户ID
     * @param sessionId 会话ID
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    public func addSessionMemory(content: String, userId: String, sessionId: String, metadata: HashMap<String, String>): HierarchicalMemoryRecord {
        let scope = MemoryScope.createSessionScope(userId, sessionId)
        return addHierarchicalMemory(content, scope, metadata)
    }

    /**
     * 添加代理级记忆
     * @param content 记忆内容
     * @param agentId 代理ID
     * @param metadata 元数据
     * @return 分层记忆记录
     */
    public func addAgentMemory(content: String, agentId: String, metadata: HashMap<String, String>): HierarchicalMemoryRecord {
        let scope = MemoryScope.createAgentScope(agentId, true)  // 代理记忆默认共享
        return addHierarchicalMemory(content, scope, metadata)
    }

    /**
     * 获取分层记忆
     * @param memoryId 记忆ID
     * @param requestScope 请求作用域
     * @return 分层记忆记录（可选）
     */
    public func getHierarchicalMemory(memoryId: String, requestScope: MemoryScope): Option<HierarchicalMemoryRecord> {
        return hierarchyManager.getMemory(memoryId, requestScope)
    }

    /**
     * 获取指定作用域的所有记忆
     * @param requestScope 请求作用域
     * @return 分层记忆列表
     */
    public func getMemoriesInScope(requestScope: MemoryScope): Array<HierarchicalMemoryRecord> {
        return hierarchyManager.getMemoriesInScope(requestScope)
    }

    /**
     * 获取用户的所有记忆
     * @param userId 用户ID
     * @param includeShared 是否包含共享记忆
     * @return 分层记忆列表
     */
    public func getUserMemories(userId: String, includeShared: Bool): Array<HierarchicalMemoryRecord> {
        let scope = MemoryScope.createUserScope(userId, includeShared)
        return hierarchyManager.getMemoriesInScope(scope)
    }

    /**
     * 获取会话的所有记忆
     * @param userId 用户ID
     * @param sessionId 会话ID
     * @return 分层记忆列表
     */
    public func getSessionMemories(userId: String, sessionId: String): Array<HierarchicalMemoryRecord> {
        let scope = MemoryScope.createSessionScope(userId, sessionId)
        return hierarchyManager.getMemoriesInScope(scope)
    }

    /**
     * 获取代理的所有记忆
     * @param agentId 代理ID
     * @param includeShared 是否包含共享记忆
     * @return 分层记忆列表
     */
    public func getAgentMemories(agentId: String, includeShared: Bool): Array<HierarchicalMemoryRecord> {
        let scope = MemoryScope.createAgentScope(agentId, includeShared)
        return hierarchyManager.getMemoriesInScope(scope)
    }

    /**
     * 更新记忆重要性
     * @param memoryId 记忆ID
     * @param scope 记忆作用域
     * @param newImportance 新的重要性评分
     * @return 是否更新成功
     */
    public func updateMemoryImportance(memoryId: String, scope: MemoryScope, newImportance: Float64): Bool {
        let memoryOpt = hierarchyManager.getMemory(memoryId, scope)
        if (memoryOpt.isSome()) {
            let memory = memoryOpt.getOrThrow()
            let updatedMemory = memory.updateImportance(newImportance)
            return hierarchyManager.addMemory(updatedMemory)  // 重新添加更新后的记忆
        }
        return false
    }

    /**
     * 批量更新记忆重要性
     * @param requestScope 请求作用域
     * @return 更新的记忆数量
     */
    public func batchUpdateImportance(requestScope: MemoryScope): Int64 {
        let memories = hierarchyManager.getMemoriesInScope(requestScope)
        let currentTime = DateTime.now().year * 10000000000 + Int64(DateTime.now().month.toInteger()) * 100000000 + DateTime.now().dayOfMonth * 1000000 + DateTime.now().hour * 10000 + DateTime.now().minute * 100 + DateTime.now().second
        let updatedMemories = importanceScorer.batchUpdateImportance(memories, currentTime)

        var updateCount: Int64 = 0
        for (updatedMemory in updatedMemories) {
            if (hierarchyManager.addMemory(updatedMemory)) {
                updateCount = updateCount + 1
            }
        }

        return updateCount
    }

    /**
     * 删除分层记忆
     * @param memoryId 记忆ID
     * @param scope 记忆作用域
     * @return 是否删除成功
     */
    public func deleteHierarchicalMemory(memoryId: String, scope: MemoryScope): Bool {
        // 从层级管理器删除
        let hierarchyDeleted = hierarchyManager.deleteMemory(memoryId, scope)

        // 从基础存储删除
        let baseDeleted = delete(memoryId)

        return hierarchyDeleted && baseDeleted
    }

    /**
     * 清理过期记忆
     * @param currentTime 当前时间
     * @return 清理的记忆数量
     */
    public func cleanupExpiredMemories(currentTime: Int64): Int64 {
        return hierarchyManager.cleanupExpiredMemories(currentTime)
    }

    /**
     * 获取记忆层级统计信息
     * @return 统计信息
     */
    public func getHierarchyStatistics(): HashMap<String, Int64> {
        return hierarchyManager.getStatistics()
    }

    // ===== Phase 2.1 智能搜索推荐辅助方法 =====

    /**
     * 构建用户上下文
     * @param userId 用户ID
     * @return 用户上下文信息
     */
    private func buildUserContext(userId: String): String {
        try {
            let userMemories = getUserMemories(userId, false)

            // 获取最近的重要记忆作为上下文
            let recentImportantMemories = ArrayList<String>()
            var count = 0

            for (memory in userMemories) {
                if (count >= 5) {  // 最多取5个上下文记忆
                    break
                }

                if (memory.importance > 0.7) {  // 只取重要性高的记忆
                    recentImportantMemories.add(memory.baseRecord.memory)
                    count = count + 1
                }
            }

            // 构建上下文字符串
            if (recentImportantMemories.size > 0) {
                var context = "用户背景："
                for (memory in recentImportantMemories.toArray()) {
                    context = context + " " + memory + ";"
                }
                return context
            }

            return ""

        } catch (e: Exception) {
            println("MemoryServiceImpl: 构建用户上下文失败: ${e}")
            return ""
        }
    }

    /**
     * 生成上下文查询
     * @param originalQuery 原始查询
     * @param userContext 用户上下文
     * @return 增强的查询
     */
    private func generateContextQuery(originalQuery: String, userContext: String): String {
        if (userContext.isEmpty()) {
            return originalQuery
        }

        // 智能合并查询和上下文
        return "${userContext} 当前查询：${originalQuery}"
    }
}