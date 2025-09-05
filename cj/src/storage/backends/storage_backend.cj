/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.storage.backends

import std.collection.HashMap
import contextengine.models.MemoryRecord
import contextengine.models.SearchResult

/**
 * 存储后端异常
 */
public class StorageBackendException <: Exception {
    public init(message: String) {
        super(message)
    }
    
    public init(message: String, _: Exception) {
        super(message)
    }
}

/**
 * 键值存储接口
 */
public interface KVStorage {
    /**
     * 存储键值对
     * @param key 键
     * @param value 值
     * @return 是否成功
     */
    func put(key: String, value: String): Bool
    
    /**
     * 获取值
     * @param key 键
     * @return 值（可选）
     */
    func get(key: String): Option<String>
    
    /**
     * 删除键值对
     * @param key 键
     * @return 是否成功
     */
    func delete(key: String): Bool
    
    /**
     * 检查键是否存在
     * @param key 键
     * @return 是否存在
     */
    func exists(key: String): Bool
    
    /**
     * 获取所有键
     * @return 键列表
     */
    func keys(): Array<String>
    
    /**
     * 清空所有数据
     * @return 是否成功
     */
    func clear(): Bool
    
    /**
     * 保存到文件
     * @param filePath 文件路径
     * @return 是否成功
     */
    func save(filePath: String): Bool
    
    /**
     * 从文件加载
     * @param filePath 文件路径
     * @return 是否成功
     */
    func load(filePath: String): Bool
}

// VectorStorage接口已移动到vector_storage.cj文件中

/**
 * 图存储接口
 */
public interface GraphStorage {
    /**
     * 添加节点
     * @param nodeId 节点ID
     * @param properties 节点属性
     * @return 是否成功
     */
    func addNode(nodeId: String, properties: HashMap<String, String>): Bool
    
    /**
     * 添加边
     * @param fromNodeId 起始节点ID
     * @param toNodeId 目标节点ID
     * @param relationshipType 关系类型
     * @param properties 边属性
     * @return 是否成功
     */
    func addEdge(fromNodeId: String, toNodeId: String, relationshipType: String, properties: HashMap<String, String>): Bool
    
    /**
     * 获取节点
     * @param nodeId 节点ID
     * @return 节点数据（可选）
     */
    func getNode(nodeId: String): Option<GraphNode>
    
    /**
     * 获取节点的所有邻居
     * @param nodeId 节点ID
     * @param relationshipType 关系类型（可选）
     * @return 邻居节点列表
     */
    func getNeighbors(nodeId: String, relationshipType: Option<String>): Array<GraphNode>
    
    /**
     * 删除节点
     * @param nodeId 节点ID
     * @return 是否成功
     */
    func deleteNode(nodeId: String): Bool
    
    /**
     * 删除边
     * @param fromNodeId 起始节点ID
     * @param toNodeId 目标节点ID
     * @param relationshipType 关系类型
     * @return 是否成功
     */
    func deleteEdge(fromNodeId: String, toNodeId: String, relationshipType: String): Bool
    
    /**
     * 查找路径
     * @param fromNodeId 起始节点ID
     * @param toNodeId 目标节点ID
     * @param maxDepth 最大深度
     * @return 路径列表
     */
    func findPaths(fromNodeId: String, toNodeId: String, maxDepth: Int64): Array<Array<String>>
    
    /**
     * 清空所有数据
     * @return 是否成功
     */
    func clear(): Bool
    
    /**
     * 保存到文件
     * @param filePath 文件路径
     * @return 是否成功
     */
    func save(filePath: String): Bool
    
    /**
     * 从文件加载
     * @param filePath 文件路径
     * @return 是否成功
     */
    func load(filePath: String): Bool
}

// VectorData和VectorSearchResult已移动到vector_storage.cj文件中

/**
 * 图节点结构
 */
public struct GraphNode {
    public let id: String
    public let properties: HashMap<String, String>
    
    public init(id: String, properties: HashMap<String, String>) {
        this.id = id
        this.properties = properties
    }
}

/**
 * 图边结构
 */
public struct GraphEdge {
    public let fromNodeId: String
    public let toNodeId: String
    public let relationshipType: String
    public let properties: HashMap<String, String>
    
    public init(fromNodeId: String, toNodeId: String, relationshipType: String, properties: HashMap<String, String>) {
        this.fromNodeId = fromNodeId
        this.toNodeId = toNodeId
        this.relationshipType = relationshipType
        this.properties = properties
    }
}