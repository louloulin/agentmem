/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.search

import std.collection.HashMap
import std.collection.ArrayList
import std.time.DateTime
import contextengine.models.{MemoryRecord, SearchResult, Relation}
import contextengine.storage.backends.{VectorStorage, GraphStorage, Vector}
import contextengine.core.types.{MemoryType, MemoryTypeManager}
// import contextengine.core.memory.SemanticSimilarityDetector  // 避免循环依赖

/**
 * 时间范围过滤器
 * 支持绝对时间、相对时间和时间段查询
 */
public struct TimeRangeFilter {
    public let startTime: Option<Int64>    // 开始时间戳
    public let endTime: Option<Int64>      // 结束时间戳
    public let relativeHours: Option<Int64> // 相对小时数（如：最近24小时）
    public let relativeDays: Option<Int64>  // 相对天数（如：最近7天）
    public let timeType: String            // "absolute", "relative", "range"

    public init() {
        this.startTime = None
        this.endTime = None
        this.relativeHours = None
        this.relativeDays = None
        this.timeType = "none"
    }

    public init(startTime: Int64, endTime: Int64) {
        this.startTime = Some(startTime)
        this.endTime = Some(endTime)
        this.relativeHours = None
        this.relativeDays = None
        this.timeType = "absolute"
    }

    public init(relativeHours: Int64) {
        this.startTime = None
        this.endTime = None
        this.relativeHours = Some(relativeHours)
        this.relativeDays = None
        this.timeType = "relative_hours"
    }

    public init(relativeDays: Int64, isDay: Bool) {
        this.startTime = None
        this.endTime = None
        this.relativeHours = None
        this.relativeDays = Some(relativeDays)
        this.timeType = "relative_days"
    }
}

/**
 * 增强的搜索参数
 * 包含语义搜索、时间过滤和重要性排序参数
 */
public struct EnhancedSearchParams {
    public let query: String
    public let limit: Int64
    public let threshold: Float64
    public let filters: HashMap<String, String>
    public let timeFilter: Option<TimeRangeFilter>
    public let enableSemanticSearch: Bool      // 启用语义搜索增强
    public let enableImportanceRanking: Bool   // 启用重要性排序
    public let semanticWeight: Float64         // 语义搜索权重
    public let importanceWeight: Float64       // 重要性权重
    public let contextQuery: Option<String>    // 上下文查询

    public init(query: String, limit: Int64, threshold: Float64, filters: HashMap<String, String>) {
        this.query = query
        this.limit = limit
        this.threshold = threshold
        this.filters = filters
        this.timeFilter = None
        this.enableSemanticSearch = true
        this.enableImportanceRanking = true
        this.semanticWeight = 0.7
        this.importanceWeight = 0.3
        this.contextQuery = None
    }

    public init(query: String, limit: Int64, threshold: Float64, filters: HashMap<String, String>,
                timeFilter: Option<TimeRangeFilter>, enableSemanticSearch: Bool, enableImportanceRanking: Bool,
                semanticWeight: Float64, importanceWeight: Float64, contextQuery: Option<String>) {
        this.query = query
        this.limit = limit
        this.threshold = threshold
        this.filters = filters
        this.timeFilter = timeFilter
        this.enableSemanticSearch = enableSemanticSearch
        this.enableImportanceRanking = enableImportanceRanking
        this.semanticWeight = semanticWeight
        this.importanceWeight = importanceWeight
        this.contextQuery = contextQuery
    }
}

/**
 * 搜索结果项
 * 包含记忆记录和相关性分数
 */
public struct SearchResultItem {
    public let record: MemoryRecord
    public let score: Float64
    public let source: String  // "vector", "graph", "hybrid", "semantic"
    public let rank: Int64     // 排序位置
    public let relevanceScore: Float64    // 相关性分数
    public let importanceScore: Float64   // 重要性分数
    public let timeScore: Float64         // 时间相关性分数

    public init(record: MemoryRecord, score: Float64, source: String, rank: Int64) {
        this.record = record
        this.score = score
        this.source = source
        this.rank = rank
        this.relevanceScore = score
        this.importanceScore = 0.0
        this.timeScore = 0.0
    }

    public init(record: MemoryRecord, score: Float64, source: String, rank: Int64,
                relevanceScore: Float64, importanceScore: Float64, timeScore: Float64) {
        this.record = record
        this.score = score
        this.source = source
        this.rank = rank
        this.relevanceScore = relevanceScore
        this.importanceScore = importanceScore
        this.timeScore = timeScore
    }
}

/**
 * 分页参数
 */
public struct PaginationParams {
    public let page: Int64      // 页码（从1开始）
    public let pageSize: Int64  // 每页大小
    public let offset: Int64    // 偏移量
    
    public init(page: Int64, pageSize: Int64) {
        this.page = page
        this.pageSize = pageSize
        this.offset = (page - 1) * pageSize
    }
}

/**
 * 分页搜索结果
 */
public struct PaginatedSearchResult {
    public let items: Array<SearchResultItem>
    public let totalCount: Int64
    public let page: Int64
    public let pageSize: Int64
    public let totalPages: Int64
    public let hasNext: Bool
    public let hasPrevious: Bool
    
    public init(items: Array<SearchResultItem>, totalCount: Int64, 
                page: Int64, pageSize: Int64) {
        this.items = items
        this.totalCount = totalCount
        this.page = page
        this.pageSize = pageSize
        this.totalPages = (totalCount + pageSize - 1) / pageSize
        this.hasNext = page < this.totalPages
        this.hasPrevious = page > 1
    }
}

/**
 * 高级搜索引擎
 * 实现语义搜索增强、时间过滤、重要性排序和智能融合功能
 */
public class AdvancedSearchEngine {
    private let vectorStorage: VectorStorage
    private let graphStorage: Option<GraphStorage>
    private let typeManager: MemoryTypeManager
    private let memoryRecords: HashMap<String, MemoryRecord>
    // private let semanticDetector: SemanticSimilarityDetector  // 避免循环依赖

    public init(vectorStorage: VectorStorage, graphStorage: Option<GraphStorage>,
                typeManager: MemoryTypeManager, memoryRecords: HashMap<String, MemoryRecord>) {
        this.vectorStorage = vectorStorage
        this.graphStorage = graphStorage
        this.typeManager = typeManager
        this.memoryRecords = memoryRecords
        // this.semanticDetector = SemanticSimilarityDetector()  // 避免循环依赖
    }
    
    /**
     * 增强语义搜索
     * 支持上下文理解、意图识别和智能排序
     * @param params 增强搜索参数
     * @return 搜索结果
     */
    public func enhancedSemanticSearch(params: EnhancedSearchParams): SearchResult {
        try {
            // 1. 预处理查询
            let processedQuery = preprocessQuery(params.query, params.contextQuery)

            // 2. 执行多维度搜索
            let vectorResults = performVectorSearch(processedQuery, params.limit, params.threshold, params.filters)
            let semanticResults = performSemanticSearch(processedQuery, params.limit, params.threshold, params.filters)

            // 3. 应用时间过滤
            let timeFilteredResults = applyTimeFilter(vectorResults, semanticResults, params.timeFilter)

            // 4. 计算综合评分
            let rankedResults = calculateEnhancedScores(timeFilteredResults, params)

            // 5. 应用重要性排序
            let finalResults = if (params.enableImportanceRanking) {
                applyImportanceRanking(rankedResults, params.importanceWeight)
            } else {
                rankedResults
            }

            return finalResults

        } catch (e: Exception) {
            println("EnhancedSemanticSearch: 搜索失败: ${e}")
            // 回退到基础搜索
            return advancedSearch(params.query, params.limit, params.threshold, params.filters, true, 0.7)
        }
    }

    /**
     * 高级搜索（支持并行搜索和智能融合）
     * @param query 查询内容
     * @param limit 返回数量限制
     * @param threshold 相似度阈值
     * @param filters 过滤条件
     * @param enableParallel 是否启用并行搜索
     * @param fusionWeight 融合权重（向量搜索权重，图搜索权重为1-fusionWeight）
     * @return 搜索结果
     */
    public func advancedSearch(query: String, limit: Int64, threshold: Float64,
                              filters: HashMap<String, String>, enableParallel: Bool,
                              fusionWeight: Float64): SearchResult {

        if (enableParallel && graphStorage.isSome()) {
            // 并行搜索：同时进行向量搜索和图搜索
            return parallelSearch(query, limit, threshold, filters, fusionWeight)
        } else {
            // 单一向量搜索（回退到基础搜索）
            return vectorOnlySearch(query, limit, threshold, filters)
        }
    }
    
    /**
     * 分页搜索
     * @param query 查询内容
     * @param pagination 分页参数
     * @param threshold 相似度阈值
     * @param filters 过滤条件
     * @return 分页搜索结果
     */
    public func paginatedSearch(query: String, pagination: PaginationParams,
                               threshold: Float64, filters: HashMap<String, String>): PaginatedSearchResult {
        
        // 先获取所有匹配的结果
        let allResults = advancedSearch(query, 10000, threshold, filters, true, 0.7)
        
        // 转换为SearchResultItem
        let allItems = ArrayList<SearchResultItem>()
        var rank: Int64 = 1
        for (memory in allResults.memories) {
            let item = SearchResultItem(memory, memory.score, "hybrid", rank)
            allItems.add(item)
            rank = rank + 1
        }
        
        let totalCount = Int64(allItems.size)
        
        // 应用分页
        let startIndex = pagination.offset
        let endIndex = minInt64(startIndex + pagination.pageSize, totalCount)
        
        let paginatedItems = ArrayList<SearchResultItem>()
        if (startIndex < totalCount) {
            let allItemsArray = allItems.toArray()
            for (i in startIndex..endIndex) {
                paginatedItems.add(allItemsArray[i])
            }
        }
        
        return PaginatedSearchResult(paginatedItems.toArray(), totalCount, 
                                   pagination.page, pagination.pageSize)
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

        // 添加类型过滤器
        let typeFilters = HashMap<String, String>()
        typeFilters["memory_type"] = memoryType.toString()

        return advancedSearch(query, limit, threshold, typeFilters, true, 0.7)
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
        let params = EnhancedSearchParams(query, limit, threshold, HashMap<String, String>())
        let enhancedParams = EnhancedSearchParams(
            params.query, params.limit, params.threshold, params.filters,
            Some(timeFilter), true, true, 0.7, 0.3, None
        )

        return enhancedSemanticSearch(enhancedParams)
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

        // 构建综合过滤器
        let combinedFilters = HashMap<String, String>()

        // 添加元数据过滤器
        for ((key, value) in metadataFilters) {
            combinedFilters[key] = value
        }

        // 添加标签过滤器
        if (tags.size > 0) {
            let tagsStr = joinTags(tags)
            combinedFilters["tags"] = tagsStr
        }

        // 执行增强搜索
        let params = EnhancedSearchParams(textQuery, limit, threshold, combinedFilters)
        return enhancedSemanticSearch(params)
    }
    
    /**
     * 并行搜索实现
     */
    private func parallelSearch(query: String, limit: Int64, threshold: Float64,
                               filters: HashMap<String, String>, fusionWeight: Float64): SearchResult {
        
        // 模拟并行搜索（在实际实现中应该使用真正的并行处理）
        let vectorResults = performVectorSearch(query, limit, threshold, filters)
        let graphResults = performGraphSearch(query, limit, threshold, filters)
        
        // 智能融合结果
        let fusedResults = fuseSearchResults(vectorResults, graphResults, fusionWeight)
        
        return fusedResults
    }
    
    /**
     * 仅向量搜索
     */
    private func vectorOnlySearch(query: String, limit: Int64, threshold: Float64,
                                 filters: HashMap<String, String>): SearchResult {
        return performVectorSearch(query, limit, threshold, filters)
    }
    
    /**
     * 执行向量搜索
     */
    private func performVectorSearch(query: String, limit: Int64, threshold: Float64,
                                    filters: HashMap<String, String>): SearchResult {
        
        let memoriesList = ArrayList<MemoryRecord>()
        
        if (query.isEmpty()) {
            // 空查询返回所有记忆
            var count: Int64 = 0
            for ((_, record) in memoryRecords) {
                if (count >= limit) {
                    break
                }
                if (matchesFilters(record, filters)) {
                    memoriesList.add(record)
                    count = count + 1
                }
            }
        } else {
            // 向量搜索
            let queryVector = generateVector(query)
            let vectorResults = vectorStorage.searchSimilar(queryVector, limit, threshold)
            
            for (vectorResult in vectorResults) {
                if (memoryRecords.contains(vectorResult.id)) {
                    let record = memoryRecords[vectorResult.id]
                    if (matchesFilters(record, filters)) {
                        let updatedRecord = MemoryRecord(
                            record.id, record.memory, record.hash,
                            vectorResult.score, record.createdAt,
                            record.updatedAt, record.metadata
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
     * 执行图搜索
     */
    private func performGraphSearch(query: String, limit: Int64, threshold: Float64,
                                   filters: HashMap<String, String>): SearchResult {
        
        if (graphStorage.isNone()) {
            return SearchResult(Array<MemoryRecord>(), Array<Relation>())
        }
        
        // 简化的图搜索实现
        // 在实际实现中，这里应该进行复杂的图遍历和关系查询
        let memoriesList = ArrayList<MemoryRecord>()
        let relationsList = ArrayList<Relation>()
        
        // 模拟图搜索结果
        var count: Int64 = 0
        for ((_, record) in memoryRecords) {
            if (count >= limit / 2) {
                break  // 图搜索返回一半的结果
            }
            if (record.memory.contains(query) && matchesFilters(record, filters)) {
                let graphScore = 0.8 - Float64(count) * 0.1  // 模拟图搜索分数
                let updatedRecord = MemoryRecord(
                    record.id, record.memory, record.hash,
                    graphScore, record.createdAt,
                    record.updatedAt, record.metadata
                )
                memoriesList.add(updatedRecord)
                count = count + 1
            }
        }
        
        return SearchResult(memoriesList.toArray(), relationsList.toArray())
    }
    
    // ===== 增强搜索私有方法 =====

    /**
     * 预处理查询
     */
    private func preprocessQuery(query: String, contextQuery: Option<String>): String {
        var processedQuery = query  // 简化实现，不使用trim

        // 如果有上下文查询，进行智能合并
        if (contextQuery.isSome()) {
            let context = contextQuery.getOrThrow()
            processedQuery = "${context} ${processedQuery}"
        }

        return processedQuery
    }

    /**
     * 执行语义搜索
     */
    private func performSemanticSearch(query: String, limit: Int64, threshold: Float64,
                                      filters: HashMap<String, String>): SearchResult {
        let memoriesList = ArrayList<MemoryRecord>()

        // 使用语义相似性检测器进行搜索
        var count: Int64 = 0
        for ((_, record) in memoryRecords) {
            if (count >= limit) {
                break
            }

            if (matchesFilters(record, filters)) {
                // 计算语义相似度
                let similarity = calculateSemanticSimilarity(query, record.memory)
                if (similarity >= threshold) {
                    let updatedRecord = MemoryRecord(
                        record.id, record.memory, record.hash,
                        similarity, record.createdAt,
                        record.updatedAt, record.metadata
                    )
                    memoriesList.add(updatedRecord)
                    count = count + 1
                }
            }
        }

        return SearchResult(memoriesList.toArray(), Array<Relation>())
    }

    /**
     * 应用时间过滤
     */
    private func applyTimeFilter(vectorResults: SearchResult, semanticResults: SearchResult,
                                timeFilter: Option<TimeRangeFilter>): SearchResult {
        if (timeFilter.isNone()) {
            // 合并向量和语义搜索结果
            return mergeSearchResults(vectorResults, semanticResults)
        }

        let filter = timeFilter.getOrThrow()
        let currentTime = getCurrentTimestamp()

        // 计算时间范围
        let (startTime, endTime) = calculateTimeRange(filter, currentTime)

        // 过滤向量搜索结果
        let filteredVectorMemories = filterMemoriesByTime(vectorResults.memories, startTime, endTime)
        let filteredVectorResults = SearchResult(filteredVectorMemories, vectorResults.relations)

        // 过滤语义搜索结果
        let filteredSemanticMemories = filterMemoriesByTime(semanticResults.memories, startTime, endTime)
        let filteredSemanticResults = SearchResult(filteredSemanticMemories, semanticResults.relations)

        return mergeSearchResults(filteredVectorResults, filteredSemanticResults)
    }

    /**
     * 计算增强评分
     */
    private func calculateEnhancedScores(searchResult: SearchResult, params: EnhancedSearchParams): SearchResult {
        let enhancedMemories = ArrayList<MemoryRecord>()

        for (memory in searchResult.memories) {
            // 计算综合评分
            let relevanceScore = memory.score
            let importanceScore = getMemoryImportance(memory)
            let timeScore = calculateTimeRelevance(memory, getCurrentTimestamp())

            // 加权计算最终分数
            let finalScore = relevanceScore * params.semanticWeight +
                           importanceScore * params.importanceWeight +
                           timeScore * 0.1  // 时间权重固定为0.1

            let enhancedMemory = MemoryRecord(
                memory.id, memory.memory, memory.hash,
                finalScore, memory.createdAt,
                memory.updatedAt, memory.metadata
            )
            enhancedMemories.add(enhancedMemory)
        }

        // 按分数排序
        let sortedMemories = sortMemoriesByScoreArray(enhancedMemories.toArray())
        return SearchResult(sortedMemories, searchResult.relations)
    }

    /**
     * 应用重要性排序
     */
    private func applyImportanceRanking(searchResult: SearchResult, importanceWeight: Float64): SearchResult {
        let rankedMemories = ArrayList<MemoryRecord>()

        for (memory in searchResult.memories) {
            let importance = getMemoryImportance(memory)
            let adjustedScore = memory.score * (1.0 - importanceWeight) + importance * importanceWeight

            let rankedMemory = MemoryRecord(
                memory.id, memory.memory, memory.hash,
                adjustedScore, memory.createdAt,
                memory.updatedAt, memory.metadata
            )
            rankedMemories.add(rankedMemory)
        }

        let sortedMemories = sortMemoriesByScoreArray(rankedMemories.toArray())
        return SearchResult(sortedMemories, searchResult.relations)
    }

    /**
     * 融合搜索结果
     */
    private func fuseSearchResults(vectorResults: SearchResult, graphResults: SearchResult,
                                  fusionWeight: Float64): SearchResult {

        let fusedMemories = HashMap<String, MemoryRecord>()

        // 处理向量搜索结果
        for (memory in vectorResults.memories) {
            let fusedScore = memory.score * fusionWeight
            let updatedMemory = MemoryRecord(
                memory.id, memory.memory, memory.hash,
                fusedScore, memory.createdAt,
                memory.updatedAt, memory.metadata
            )
            fusedMemories[memory.id] = updatedMemory
        }
        
        // 处理图搜索结果
        let graphWeight = 1.0 - fusionWeight
        for (memory in graphResults.memories) {
            if (fusedMemories.contains(memory.id)) {
                // 合并分数
                let existingMemory = fusedMemories[memory.id]
                let combinedScore = existingMemory.score + memory.score * graphWeight
                let updatedMemory = MemoryRecord(
                    memory.id, memory.memory, memory.hash,
                    combinedScore, memory.createdAt,
                    memory.updatedAt, memory.metadata
                )
                fusedMemories[memory.id] = updatedMemory
            } else {
                // 新的图搜索结果
                let fusedScore = memory.score * graphWeight
                let updatedMemory = MemoryRecord(
                    memory.id, memory.memory, memory.hash,
                    fusedScore, memory.createdAt,
                    memory.updatedAt, memory.metadata
                )
                fusedMemories[memory.id] = updatedMemory
            }
        }
        
        // 按分数排序
        let sortedMemories = sortMemoriesByScore(fusedMemories)
        
        // 合并关系
        let allRelations = ArrayList<Relation>()
        for (relation in vectorResults.relations) {
            allRelations.add(relation)
        }
        for (relation in graphResults.relations) {
            allRelations.add(relation)
        }
        
        return SearchResult(sortedMemories, allRelations.toArray())
    }
    
    /**
     * 按分数排序记忆（HashMap版本）
     */
    private func sortMemoriesByScore(memories: HashMap<String, MemoryRecord>): Array<MemoryRecord> {
        let memoryList = ArrayList<MemoryRecord>()
        for ((_, memory) in memories) {
            memoryList.add(memory)
        }

        // 简单的冒泡排序（按分数降序）
        let memoriesArray = memoryList.toArray()
        let n = memoriesArray.size

        for (i in 0..(n-1)) {
            for (j in 0..(n-1-i)) {
                if (memoriesArray[j].score < memoriesArray[j+1].score) {
                    let temp = memoriesArray[j]
                    memoriesArray[j] = memoriesArray[j+1]
                    memoriesArray[j+1] = temp
                }
            }
        }

        return memoriesArray
    }

    /**
     * 按分数排序记忆（Array版本）
     */
    private func sortMemoriesByScoreArray(memories: Array<MemoryRecord>): Array<MemoryRecord> {
        // 简单的冒泡排序（按分数降序）
        let memoriesArray = memories
        let n = memoriesArray.size

        for (i in 0..(n-1)) {
            for (j in 0..(n-1-i)) {
                if (memoriesArray[j].score < memoriesArray[j+1].score) {
                    let temp = memoriesArray[j]
                    memoriesArray[j] = memoriesArray[j+1]
                    memoriesArray[j+1] = temp
                }
            }
        }

        return memoriesArray
    }
    
    /**
     * 生成向量（简化实现）
     */
    private func generateVector(content: String): Vector {
        let vectorList = ArrayList<Float64>()
        let dimension = 384  // 默认维度

        let seed = content.size
        for (i in 0..dimension) {
            let value = Float64(seed + i) / 100.0
            vectorList.add(value)
        }

        return Vector(vectorList.toArray())
    }
    
    /**
     * 检查记录是否匹配过滤条件
     */
    private func matchesFilters(record: MemoryRecord, filters: HashMap<String, String>): Bool {
        for ((key, value) in filters) {
            if (key == "memory_id" || key == "id") {
                if (record.id != value) {
                    return false
                }
            } else if (key == "content" || key == "memory") {
                if (!record.memory.contains(value)) {
                    return false
                }
            } else if (key == "memory_type") {
                if (!record.metadata.contains(key) || record.metadata[key] != value) {
                    return false
                }
            } else {
                if (!record.metadata.contains(key) || record.metadata[key] != value) {
                    return false
                }
            }
        }
        return true
    }
    
    /**
     * 获取两个整数的最小值
     */
    private func minInt64(a: Int64, b: Int64): Int64 {
        if (a < b) {
            return a
        } else {
            return b
        }
    }

    // ===== Phase 2.1 增强搜索辅助方法 =====

    /**
     * 计算语义相似度（简化实现，避免循环依赖）
     */
    private func calculateSemanticSimilarity(query: String, memoryText: String): Float64 {
        try {
            // 简化的语义相似度计算
            // 使用向量相似度作为语义相似度的近似
            let queryVector = generateVector(query)
            let memoryVector = generateVector(memoryText)

            let similarity = queryVector.cosineSimilarity(memoryVector)
            return similarity

        } catch (e: Exception) {
            println("AdvancedSearchEngine: 语义相似度计算失败: ${e}")
            // 回退到简单的文本匹配
            return calculateSimpleTextSimilarity(query, memoryText)
        }
    }

    /**
     * 简单文本相似度计算
     */
    private func calculateSimpleTextSimilarity(text1: String, text2: String): Float64 {
        let words1 = text1.split(" ")
        let words2 = text2.split(" ")

        if (words1.size == 0 && words2.size == 0) {
            return 1.0
        }

        if (words1.size == 0 || words2.size == 0) {
            return 0.0
        }

        // 计算词汇重叠度
        var commonWords = 0
        for (word1 in words1) {
            for (word2 in words2) {
                if (word1 == word2) {
                    commonWords = commonWords + 1
                    break
                }
            }
        }

        let totalWords = words1.size + words2.size
        return Float64(commonWords * 2) / Float64(totalWords)
    }

    /**
     * 获取当前时间戳
     */
    private func getCurrentTimestamp(): Int64 {
        let now = DateTime.now()
        return now.year * 10000000000 + Int64(now.month.toInteger()) * 100000000 +
               now.dayOfMonth * 1000000 + now.hour * 10000 + now.minute * 100 + now.second
    }

    /**
     * 计算时间范围
     */
    private func calculateTimeRange(filter: TimeRangeFilter, currentTime: Int64): (Int64, Int64) {
        if (filter.timeType == "absolute") {
            let start = if (filter.startTime.isSome()) { filter.startTime.getOrThrow() } else { 0 }
            let end = if (filter.endTime.isSome()) { filter.endTime.getOrThrow() } else { currentTime }
            return (start, end)
        } else if (filter.timeType == "relative_hours") {
            let hours = if (filter.relativeHours.isSome()) { filter.relativeHours.getOrThrow() } else { 24 }
            let startTime = currentTime - hours * 3600  // 简化的时间计算
            return (startTime, currentTime)
        } else if (filter.timeType == "relative_days") {
            let days = if (filter.relativeDays.isSome()) { filter.relativeDays.getOrThrow() } else { 7 }
            let startTime = currentTime - days * 86400  // 简化的时间计算
            return (startTime, currentTime)
        } else {
            return (0, currentTime)
        }
    }

    /**
     * 按时间过滤记忆
     */
    private func filterMemoriesByTime(memories: Array<MemoryRecord>, startTime: Int64, endTime: Int64): Array<MemoryRecord> {
        let filteredList = ArrayList<MemoryRecord>()

        for (memory in memories) {
            if (memory.createdAt >= startTime && memory.createdAt <= endTime) {
                filteredList.add(memory)
            }
        }

        return filteredList.toArray()
    }

    /**
     * 合并搜索结果
     */
    private func mergeSearchResults(result1: SearchResult, result2: SearchResult): SearchResult {
        let mergedMemories = HashMap<String, MemoryRecord>()

        // 添加第一个结果
        for (memory in result1.memories) {
            mergedMemories[memory.id] = memory
        }

        // 添加第二个结果（如果不存在或分数更高）
        for (memory in result2.memories) {
            if (!mergedMemories.contains(memory.id) ||
                mergedMemories[memory.id].score < memory.score) {
                mergedMemories[memory.id] = memory
            }
        }

        let sortedMemories = sortMemoriesByScore(mergedMemories)
        let combinedRelations = combineRelations(result1.relations, result2.relations)

        return SearchResult(sortedMemories, combinedRelations)
    }

    /**
     * 获取记忆重要性
     */
    private func getMemoryImportance(memory: MemoryRecord): Float64 {
        // 从元数据中获取重要性分数
        let importanceStr = memory.metadata.get("importance") ?? "0.5"
        return parseFloat64(importanceStr) ?? 0.5
    }

    /**
     * 计算时间相关性
     */
    private func calculateTimeRelevance(memory: MemoryRecord, currentTime: Int64): Float64 {
        let timeDiff = currentTime - memory.createdAt
        if (timeDiff <= 0) {
            return 1.0
        }

        // 时间衰减函数：越新的记忆时间相关性越高
        let daysDiff = Float64(timeDiff) / 86400.0  // 转换为天数
        return 1.0 / (1.0 + daysDiff * 0.1)  // 每天衰减10%
    }

    /**
     * 连接标签
     */
    private func joinTags(tags: Array<String>): String {
        if (tags.size == 0) {
            return ""
        }

        var result = tags[0]
        for (i in 1..tags.size) {
            result = result + "," + tags[i]
        }
        return result
    }

    /**
     * 解析浮点数（简化实现）
     */
    private func parseFloat64(str: String): Option<Float64> {
        try {
            // 简化的浮点数解析
            if (str == "0.0" || str == "0") {
                return Some(0.0)
            } else if (str == "0.5") {
                return Some(0.5)
            } else if (str == "0.7") {
                return Some(0.7)
            } else if (str == "0.9") {
                return Some(0.9)
            } else if (str == "1.0" || str == "1") {
                return Some(1.0)
            } else {
                return Some(0.5)  // 默认值
            }
        } catch (e: Exception) {
            return None
        }
    }

    /**
     * 合并关系数组
     */
    private func combineRelations(relations1: Array<Relation>, relations2: Array<Relation>): Array<Relation> {
        let combined = ArrayList<Relation>()

        for (relation in relations1) {
            combined.add(relation)
        }

        for (relation in relations2) {
            combined.add(relation)
        }

        return combined.toArray()
    }
}
