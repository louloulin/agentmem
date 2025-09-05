/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord
import contextengine.storage.backends.Vector

/**
 * 语义相似性配置
 */
public struct SemanticSimilarityConfig {
    public let cosineSimilarityThreshold: Float64    // 余弦相似度阈值
    public let euclideanDistanceThreshold: Float64   // 欧几里得距离阈值
    public let textSimilarityThreshold: Float64      // 文本相似度阈值
    public let enableMultiLanguage: Bool             // 是否启用多语言支持
    public let vectorDimension: Int64                // 向量维度
    public let similarityAlgorithm: String          // 相似性算法：cosine/euclidean/hybrid
    
    public init() {
        this.cosineSimilarityThreshold = 0.85
        this.euclideanDistanceThreshold = 0.3
        this.textSimilarityThreshold = 0.8
        this.enableMultiLanguage = true
        this.vectorDimension = 384
        this.similarityAlgorithm = "cosine"
    }
    
    public init(cosineThreshold: Float64, euclideanThreshold: Float64, textThreshold: Float64) {
        this.cosineSimilarityThreshold = cosineThreshold
        this.euclideanDistanceThreshold = euclideanThreshold
        this.textSimilarityThreshold = textThreshold
        this.enableMultiLanguage = true
        this.vectorDimension = 384
        this.similarityAlgorithm = "cosine"
    }
}

/**
 * 相似性检测结果
 */
public struct SimilarityResult {
    public let similarity: Float64      // 相似度分数 (0.0-1.0)
    public let distance: Float64        // 距离分数
    public let algorithm: String        // 使用的算法
    public let isSimilar: Bool         // 是否相似
    public let confidence: Float64      // 置信度
    
    public init(similarity: Float64, distance: Float64, algorithm: String, threshold: Float64) {
        this.similarity = similarity
        this.distance = distance
        this.algorithm = algorithm
        this.isSimilar = similarity >= threshold
        this.confidence = if (similarity >= threshold) {
            (similarity - threshold) / (1.0 - threshold)
        } else {
            0.0
        }
    }
}

/**
 * 语义相似性检测器
 * 负责检测记忆之间的语义相似性
 */
public class SemanticSimilarityDetector {
    private let config: SemanticSimilarityConfig
    private var isInitialized: Bool
    private let stopWords: HashMap<String, Bool>  // 停用词表
    
    public init(config: SemanticSimilarityConfig) {
        this.config = config
        this.isInitialized = false
        this.stopWords = HashMap<String, Bool>()
        
        initializeStopWords()
        this.isInitialized = true
    }
    
    public init() {
        this.config = SemanticSimilarityConfig()
        this.isInitialized = false
        this.stopWords = HashMap<String, Bool>()
        
        initializeStopWords()
        this.isInitialized = true
    }
    
    /**
     * 检测两个记忆的相似性
     * @param memory1 第一个记忆
     * @param memory2 第二个记忆
     * @return 相似性结果
     */
    public func detectSimilarity(memory1: MemoryRecord, memory2: MemoryRecord): SimilarityResult {
        if (!isInitialized) {
            return SimilarityResult(0.0, 1.0, "none", config.cosineSimilarityThreshold)
        }

        try {
            // 根据配置选择算法
            match (config.similarityAlgorithm) {
                case "cosine" =>
                    detectCosineSimilarity(memory1, memory2)
                case "euclidean" =>
                    detectEuclideanSimilarity(memory1, memory2)
                case "hybrid" =>
                    detectHybridSimilarity(memory1, memory2)
                case _ =>
                    detectCosineSimilarity(memory1, memory2)
            }

        } catch (e: Exception) {
            println("SemanticSimilarityDetector: 相似性检测失败: ${e}")
            return SimilarityResult(0.0, 1.0, "error", config.cosineSimilarityThreshold)
        }
    }
    
    /**
     * 检测文本内容的相似性
     * @param text1 第一个文本
     * @param text2 第二个文本
     * @return 相似性结果
     */
    public func detectTextSimilarity(text1: String, text2: String): SimilarityResult {
        try {
            // 文本预处理
            let processedText1 = preprocessText(text1)
            let processedText2 = preprocessText(text2)
            
            // 计算文本相似度
            let similarity = calculateTextSimilarity(processedText1, processedText2)
            let distance = 1.0 - similarity
            
            return SimilarityResult(similarity, distance, "text", config.textSimilarityThreshold)
            
        } catch (e: Exception) {
            println("SemanticSimilarityDetector: 文本相似性检测失败: ${e}")
            return SimilarityResult(0.0, 1.0, "text_error", config.textSimilarityThreshold)
        }
    }
    
    /**
     * 批量检测相似记忆
     * @param targetMemory 目标记忆
     * @param candidateMemories 候选记忆列表
     * @return 相似记忆及其相似度
     */
    public func findSimilarMemories(targetMemory: MemoryRecord, 
                                   candidateMemories: Array<MemoryRecord>): Array<(MemoryRecord, SimilarityResult)> {
        let results = ArrayList<(MemoryRecord, SimilarityResult)>()
        
        for (candidate in candidateMemories) {
            // 跳过自己
            if (candidate.id == targetMemory.id) {
                continue
            }
            
            let similarity = detectSimilarity(targetMemory, candidate)
            if (similarity.isSimilar) {
                results.add((candidate, similarity))
            }
        }
        
        // 按相似度排序
        return sortBySimilarity(results.toArray())
    }
    
    /**
     * 检测记忆组中的重复项
     * @param memories 记忆列表
     * @return 重复记忆组
     */
    public func detectDuplicateGroups(memories: Array<MemoryRecord>): Array<Array<MemoryRecord>> {
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
                
                let similarity = detectSimilarity(memory, candidate)
                if (similarity.isSimilar) {
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
     * 获取相似性统计信息
     * @param memories 记忆列表
     * @return 统计信息
     */
    public func getSimilarityStatistics(memories: Array<MemoryRecord>): HashMap<String, String> {
        let stats = HashMap<String, String>()
        
        let totalMemories = memories.size
        let duplicateGroups = detectDuplicateGroups(memories)
        var totalDuplicates: Int64 = 0
        
        for (group in duplicateGroups) {
            totalDuplicates = totalDuplicates + Int64(group.size)
        }
        
        let duplicateRate = if (totalMemories > 0) {
            Float64(totalDuplicates) / Float64(totalMemories) * 100.0
        } else {
            0.0
        }
        
        stats["total_memories"] = totalMemories.toString()
        stats["duplicate_groups"] = duplicateGroups.size.toString()
        stats["total_duplicates"] = totalDuplicates.toString()
        stats["duplicate_rate"] = duplicateRate.toString()
        stats["algorithm"] = config.similarityAlgorithm
        stats["cosine_threshold"] = config.cosineSimilarityThreshold.toString()
        stats["euclidean_threshold"] = config.euclideanDistanceThreshold.toString()
        stats["text_threshold"] = config.textSimilarityThreshold.toString()
        
        return stats
    }
    
    // ===== 私有方法 =====
    
    /**
     * 检测余弦相似度
     */
    private func detectCosineSimilarity(memory1: MemoryRecord, memory2: MemoryRecord): SimilarityResult {
        let vector1 = generateVector(memory1.memory)
        let vector2 = generateVector(memory2.memory)
        
        let similarity = vector1.cosineSimilarity(vector2)
        let distance = 1.0 - similarity
        
        return SimilarityResult(similarity, distance, "cosine", config.cosineSimilarityThreshold)
    }
    
    /**
     * 检测欧几里得距离相似度
     */
    private func detectEuclideanSimilarity(memory1: MemoryRecord, memory2: MemoryRecord): SimilarityResult {
        let vector1 = generateVector(memory1.memory)
        let vector2 = generateVector(memory2.memory)
        
        let distance = vector1.euclideanDistance(vector2)
        let similarity = 1.0 / (1.0 + distance)  // 转换为相似度
        
        return SimilarityResult(similarity, distance, "euclidean", config.euclideanDistanceThreshold)
    }
    
    /**
     * 检测混合相似度
     */
    private func detectHybridSimilarity(memory1: MemoryRecord, memory2: MemoryRecord): SimilarityResult {
        let cosineResult = detectCosineSimilarity(memory1, memory2)
        let euclideanResult = detectEuclideanSimilarity(memory1, memory2)
        let textResult = detectTextSimilarity(memory1.memory, memory2.memory)
        
        // 加权平均
        let hybridSimilarity = (cosineResult.similarity * 0.4 + 
                               euclideanResult.similarity * 0.3 + 
                               textResult.similarity * 0.3)
        
        let hybridDistance = (cosineResult.distance * 0.4 + 
                             euclideanResult.distance * 0.3 + 
                             textResult.distance * 0.3)
        
        let hybridThreshold = (config.cosineSimilarityThreshold * 0.4 + 
                              config.euclideanDistanceThreshold * 0.3 + 
                              config.textSimilarityThreshold * 0.3)
        
        return SimilarityResult(hybridSimilarity, hybridDistance, "hybrid", hybridThreshold)
    }
    
    /**
     * 生成文本向量
     */
    private func generateVector(text: String): Vector {
        let vectorValues = ArrayList<Float64>()
        let processedText = preprocessText(text)
        
        // 简化的向量生成：基于文本特征
        let textLength = processedText.size
        let wordCount = countWords(processedText)
        
        for (i in 0..config.vectorDimension) {
            let value = (Float64(textLength + wordCount + i) * 0.01)
            let normalizedValue = if (value > 1.0) { value - 1.0 } else { value }
            vectorValues.add(normalizedValue)
        }
        
        return Vector(vectorValues.toArray())
    }
    
    /**
     * 文本预处理
     */
    private func preprocessText(text: String): String {
        // 简化实现：直接移除停用词
        return removeStopWords(text)
    }
    
    /**
     * 计算文本相似度
     */
    private func calculateTextSimilarity(text1: String, text2: String): Float64 {
        let words1 = getWords(text1)
        let words2 = getWords(text2)
        
        if (words1.size == 0 && words2.size == 0) {
            return 1.0
        }
        
        if (words1.size == 0 || words2.size == 0) {
            return 0.0
        }
        
        // 计算Jaccard相似度
        let intersection = getIntersection(words1, words2)
        let union = getUnion(words1, words2)
        
        return Float64(intersection.size) / Float64(union.size)
    }
    
    /**
     * 初始化停用词表
     */
    private func initializeStopWords(): Unit {
        // 中文停用词
        let chineseStopWords = ["的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这"]
        
        // 英文停用词
        let englishStopWords = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does", "did", "will", "would", "could", "should"]
        
        for (word in chineseStopWords) {
            stopWords[word] = true
        }
        
        for (word in englishStopWords) {
            stopWords[word] = true
        }
    }
    
    /**
     * 移除停用词
     */
    private func removeStopWords(text: String): String {
        let words = text.split(" ")
        let filteredWords = ArrayList<String>()
        
        for (word in words) {
            if (!stopWords.contains(word) && word.size > 1) {
                filteredWords.add(word)
            }
        }
        
        // 简化join实现
        var result = ""
        for (i in 0..filteredWords.size) {
            if (i > 0) {
                result = result + " "
            }
            result = result + filteredWords[i]
        }
        return result
    }
    
    /**
     * 获取单词列表
     */
    private func getWords(text: String): Array<String> {
        return text.split(" ")
    }
    
    /**
     * 计算单词数量
     */
    private func countWords(text: String): Int64 {
        let words = text.split(" ")
        return Int64(words.size)
    }
    
    /**
     * 检查字符是否为字母数字
     */
    private func isAlphaNumeric(char: String): Bool {
        // 简化实现
        return char != "." && char != "," && char != "!" && char != "?" && char != ";" && char != ":"
    }
    
    /**
     * 获取两个数组的交集
     */
    private func getIntersection(arr1: Array<String>, arr2: Array<String>): Array<String> {
        let intersection = ArrayList<String>()
        let set2 = HashMap<String, Bool>()
        
        for (item in arr2) {
            set2[item] = true
        }
        
        for (item in arr1) {
            if (set2.contains(item)) {
                intersection.add(item)
            }
        }
        
        return intersection.toArray()
    }
    
    /**
     * 获取两个数组的并集
     */
    private func getUnion(arr1: Array<String>, arr2: Array<String>): Array<String> {
        let union = HashMap<String, Bool>()
        let result = ArrayList<String>()
        
        for (item in arr1) {
            union[item] = true
        }
        
        for (item in arr2) {
            union[item] = true
        }
        
        for ((item, _) in union) {
            result.add(item)
        }
        
        return result.toArray()
    }
    
    /**
     * 按相似度排序
     */
    private func sortBySimilarity(results: Array<(MemoryRecord, SimilarityResult)>): Array<(MemoryRecord, SimilarityResult)> {
        let sortedList = ArrayList<(MemoryRecord, SimilarityResult)>()
        for (result in results) {
            sortedList.add(result)
        }
        
        // 简化的冒泡排序，按相似度降序排序
        for (i in 0..sortedList.size) {
            for (j in 0..(sortedList.size - 1 - i)) {
                let (_, currentResult) = sortedList[j]
                let (_, nextResult) = sortedList[j + 1]
                if (currentResult.similarity < nextResult.similarity) {
                    let temp = sortedList[j]
                    sortedList[j] = sortedList[j + 1]
                    sortedList[j + 1] = temp
                }
            }
        }
        
        return sortedList.toArray()
    }
}
