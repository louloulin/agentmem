/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.storage.backends

import std.collection.HashMap
import std.collection.ArrayList

/**
 * 向量数据结构
 */
public struct Vector {
    public let dimensions: Int64
    public let values: Array<Float64>
    
    public init(values: Array<Float64>) {
        this.values = values
        this.dimensions = Int64(values.size)
    }
    
    /**
     * 计算与另一个向量的余弦相似度
     */
    public func cosineSimilarity(other: Vector): Float64 {
        if (this.dimensions != other.dimensions) {
            return 0.0
        }
        
        var dotProduct: Float64 = 0.0
        var normA: Float64 = 0.0
        var normB: Float64 = 0.0
        
        for (i in 0..this.values.size) {
            let a = this.values[i]
            let b = other.values[i]
            
            dotProduct = dotProduct + (a * b)
            normA = normA + (a * a)
            normB = normB + (b * b)
        }
        
        if (normA == 0.0 || normB == 0.0) {
            return 0.0
        }
        
        return dotProduct / (sqrtImpl(normA) * sqrtImpl(normB))
    }
    
    /**
     * 计算欧几里得距离
     */
    public func euclideanDistance(other: Vector): Float64 {
        if (this.dimensions != other.dimensions) {
            return 999999999.0  // 使用大数值代替MAX_VALUE
        }
        
        var sum: Float64 = 0.0
        for (i in 0..this.values.size) {
            let diff = this.values[i] - other.values[i]
            sum = sum + (diff * diff)
        }
        
        return sqrtImpl(sum)
    }
}

/**
 * 向量搜索结果
 */
public struct VectorSearchResult {
    public let id: String
    public let vector: Vector
    public let metadata: HashMap<String, String>
    public let score: Float64  // 相似度评分
    public let distance: Float64  // 距离
    
    public init(id: String, vector: Vector, metadata: HashMap<String, String>, 
                score: Float64, distance: Float64) {
        this.id = id
        this.vector = vector
        this.metadata = metadata
        this.score = score
        this.distance = distance
    }
}

/**
 * 向量存储接口
 * 定义向量数据库的基本操作
 */
public interface VectorStorage {
    /**
     * 添加向量
     * @param id 向量ID
     * @param vector 向量数据
     * @param metadata 元数据
     * @return 是否成功
     */
    func addVector(id: String, vector: Vector, metadata: HashMap<String, String>): Bool
    
    /**
     * 获取向量
     * @param id 向量ID
     * @return 向量数据（可选）
     */
    func getVector(id: String): Option<Vector>
    
    /**
     * 删除向量
     * @param id 向量ID
     * @return 是否成功
     */
    func deleteVector(id: String): Bool
    
    /**
     * 向量相似性搜索
     * @param queryVector 查询向量
     * @param topK 返回前K个结果
     * @param threshold 相似度阈值
     * @return 搜索结果列表
     */
    func searchSimilar(queryVector: Vector, topK: Int64, threshold: Float64): Array<VectorSearchResult>
    
    /**
     * 批量添加向量
     * @param vectors 向量数据映射
     * @return 成功添加的数量
     */
    func batchAddVectors(vectors: HashMap<String, Vector>, 
                        metadata: HashMap<String, HashMap<String, String>>): Int64
    
    /**
     * 获取向量数量
     * @return 向量总数
     */
    func getVectorCount(): Int64
    
    /**
     * 更新向量
     * @param id 向量ID
     * @param vector 新向量数据
     * @param metadata 新元数据
     * @return 是否成功
     */
    func updateVector(id: String, vector: Vector, metadata: HashMap<String, String>): Bool

    /**
     * 清空所有向量
     * @return 是否成功
     */
    func clearAllVectors(): Bool

    /**
     * 清空所有向量（别名方法，为了兼容性）
     * @return 是否成功
     */
    func clear(): Bool
    
    /**
     * 创建索引
     * @param indexType 索引类型
     * @return 是否成功
     */
    func createIndex(indexType: String): Bool
    
    /**
     * 获取存储统计信息
     * @return 统计信息
     */
    func getStorageStats(): HashMap<String, String>
}

/**
 * 内存向量存储实现
 * 用于测试和小规模应用
 */
public class MemoryVectorStorage <: VectorStorage {
    private let vectors: HashMap<String, Vector>
    private let metadata: HashMap<String, HashMap<String, String>>
    private var indexBuilt: Bool
    
    public init() {
        this.vectors = HashMap<String, Vector>()
        this.metadata = HashMap<String, HashMap<String, String>>()
        this.indexBuilt = false
    }
    
    /**
     * 添加向量
     */
    public func addVector(id: String, vector: Vector, metadata: HashMap<String, String>): Bool {
        try {
            vectors[id] = vector
            this.metadata[id] = metadata
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 获取向量
     */
    public func getVector(id: String): Option<Vector> {
        if (vectors.contains(id)) {
            return Some(vectors[id])
        }
        return None
    }
    
    /**
     * 删除向量
     */
    public func deleteVector(id: String): Bool {
        try {
            if (vectors.contains(id)) {
                vectors.remove(id)
            }
            if (metadata.contains(id)) {
                metadata.remove(id)
            }
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 向量相似性搜索
     */
    public func searchSimilar(queryVector: Vector, topK: Int64, threshold: Float64): Array<VectorSearchResult> {
        let results = ArrayList<VectorSearchResult>()
        
        // 计算与所有向量的相似度
        for ((id, vector) in vectors) {
            let similarity = queryVector.cosineSimilarity(vector)
            let distance = queryVector.euclideanDistance(vector)
            
            if (similarity >= threshold) {
                let vectorMetadata = if (metadata.contains(id)) {
                    metadata[id]
                } else {
                    HashMap<String, String>()
                }
                
                let result = VectorSearchResult(id, vector, vectorMetadata, similarity, distance)
                results.add(result)
            }
        }
        
        // 按相似度排序（简化实现）
        let sortedResults = sortResultsBySimilarity(results.toArray())
        
        // 返回前topK个结果
        let finalResults = ArrayList<VectorSearchResult>()
        let limit = min(topK, Int64(sortedResults.size))
        
        for (i in 0..limit) {
            finalResults.add(sortedResults[i])
        }
        
        return finalResults.toArray()
    }
    
    /**
     * 批量添加向量
     */
    public func batchAddVectors(vectors: HashMap<String, Vector>, 
                               metadata: HashMap<String, HashMap<String, String>>): Int64 {
        var addedCount: Int64 = 0
        
        for ((id, vector) in vectors) {
            let vectorMetadata = if (metadata.contains(id)) {
                metadata[id]
            } else {
                HashMap<String, String>()
            }
            
            if (addVector(id, vector, vectorMetadata)) {
                addedCount = addedCount + 1
            }
        }
        
        return addedCount
    }
    
    /**
     * 获取向量数量
     */
    public func getVectorCount(): Int64 {
        return Int64(vectors.size)
    }
    
    /**
     * 更新向量
     */
    public func updateVector(id: String, vector: Vector, metadata: HashMap<String, String>): Bool {
        try {
            if (vectors.contains(id)) {
                vectors[id] = vector
                this.metadata[id] = metadata
                return true
            }
            return false
        } catch (e: Exception) {
            return false
        }
    }

    /**
     * 清空所有向量
     */
    public func clearAllVectors(): Bool {
        try {
            vectors.clear()
            metadata.clear()
            indexBuilt = false
            return true
        } catch (e: Exception) {
            return false
        }
    }

    /**
     * 清空所有向量（别名方法）
     */
    public func clear(): Bool {
        return clearAllVectors()
    }
    
    /**
     * 创建索引
     */
    public func createIndex(indexType: String): Bool {
        println("MemoryVectorStorage: 创建${indexType}索引")
        indexBuilt = true
        return true
    }
    
    /**
     * 获取存储统计信息
     */
    public func getStorageStats(): HashMap<String, String> {
        let stats = HashMap<String, String>()
        
        stats["storage_type"] = "memory_vector"
        stats["vector_count"] = vectors.size.toString()
        stats["index_built"] = indexBuilt.toString()
        
        // 计算平均向量维度
        if (vectors.size > 0) {
            var totalDimensions: Int64 = 0
            for ((_, vector) in vectors) {
                totalDimensions = totalDimensions + vector.dimensions
            }
            let avgDimensions = totalDimensions / Int64(vectors.size)
            stats["average_dimensions"] = avgDimensions.toString()
        } else {
            stats["average_dimensions"] = "0"
        }
        
        return stats
    }
    
    // ===== 私有辅助方法 =====
    
    /**
     * 按相似度排序结果（简化实现）
     */
    private func sortResultsBySimilarity(results: Array<VectorSearchResult>): Array<VectorSearchResult> {
        // 简化的冒泡排序实现
        let sortedList = ArrayList<VectorSearchResult>()
        for (result in results) {
            sortedList.add(result)
        }
        
        // 按相似度降序排序
        for (i in 0..sortedList.size) {
            for (j in 0..(sortedList.size - 1 - i)) {
                if (sortedList[j].score < sortedList[j + 1].score) {
                    let temp = sortedList[j]
                    sortedList[j] = sortedList[j + 1]
                    sortedList[j + 1] = temp
                }
            }
        }
        
        return sortedList.toArray()
    }
    
    /**
     * 计算平方根（简化实现）
     */
    private func sqrt(x: Float64): Float64 {
        if (x < 0.0) {
            return 0.0
        }
        if (x == 0.0) {
            return 0.0
        }
        
        // 使用牛顿法计算平方根
        var guess: Float64 = x / 2.0
        for (i in 0..10) {  // 迭代10次
            guess = (guess + x / guess) / 2.0
        }
        
        return guess
    }
    
    /**
     * 计算最小值
     */
    private func min(a: Int64, b: Int64): Int64 {
        return if (a < b) {
            a
        } else {
            b
        }
    }
}

/**
 * 计算平方根的辅助函数
 */
func sqrtImpl(x: Float64): Float64 {
    if (x < 0.0) {
        return 0.0
    }
    if (x == 0.0) {
        return 0.0
    }

    // 使用牛顿法计算平方根
    var guess: Float64 = x / 2.0
    for (i in 0..10) {  // 迭代10次
        guess = (guess + x / guess) / 2.0
    }

    return guess
}
