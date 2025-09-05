/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.storage.backends

import std.collection.HashMap
import std.collection.ArrayList
import std.fs.*

/**
 * 内存图存储实现
 * 基于HashMap的简单图存储，支持节点和边的管理
 */
public class MemoryGraphStorage <: GraphStorage {
    private let nodes: HashMap<String, GraphNode>
    private let edges: HashMap<String, ArrayList<GraphEdge>>
    
    /**
     * 构造函数
     */
    public init() {
        this.nodes = HashMap<String, GraphNode>()
        this.edges = HashMap<String, ArrayList<GraphEdge>>()
    }
    
    /**
     * 添加节点
     * @param nodeId 节点ID
     * @param properties 节点属性
     * @return 是否成功
     */
    public func addNode(nodeId: String, properties: HashMap<String, String>): Bool {
        try {
            let node = GraphNode(nodeId, properties)
            nodes[nodeId] = node
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 添加边
     * @param fromNodeId 起始节点ID
     * @param toNodeId 目标节点ID
     * @param relationshipType 关系类型
     * @param properties 边属性
     * @return 是否成功
     */
    public func addEdge(fromNodeId: String, toNodeId: String, relationshipType: String, properties: HashMap<String, String>): Bool {
        try {
            let edge = GraphEdge(fromNodeId, toNodeId, relationshipType, properties)
            
            if (!edges.contains(fromNodeId)) {
                edges[fromNodeId] = ArrayList<GraphEdge>()
            }
            
            edges[fromNodeId].add(edge)
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 获取节点
     * @param nodeId 节点ID
     * @return 节点数据（可选）
     */
    public func getNode(nodeId: String): Option<GraphNode> {
        if (nodes.contains(nodeId)) {
            return Option<GraphNode>.Some(nodes[nodeId])
        }
        return Option<GraphNode>.None
    }
    
    /**
     * 获取节点的所有邻居
     * @param nodeId 节点ID
     * @param relationshipType 关系类型（可选）
     * @return 邻居节点列表
     */
    public func getNeighbors(nodeId: String, relationshipType: Option<String>): Array<GraphNode> {
        let neighbors = ArrayList<GraphNode>()
        
        if (edges.contains(nodeId)) {
            let nodeEdges = edges[nodeId]
            for (edge in nodeEdges) {
                match (relationshipType) {
                    case Some(relType) => 
                        if (edge.relationshipType == relType && nodes.contains(edge.toNodeId)) {
                            neighbors.add(nodes[edge.toNodeId])
                        }
                    case None => 
                        if (nodes.contains(edge.toNodeId)) {
                            neighbors.add(nodes[edge.toNodeId])
                        }
                }
            }
        }
        
        return neighbors.toArray()
    }
    
    /**
     * 删除节点
     * @param nodeId 节点ID
     * @return 是否成功
     */
    public func deleteNode(nodeId: String): Bool {
        try {
            nodes.remove(nodeId)
            edges.remove(nodeId)
            
            // 删除指向该节点的边
            let keysToUpdate = ArrayList<String>()
            for (key in edges.keys()) {
                keysToUpdate.add(key)
            }
            
            for (fromId in keysToUpdate) {
                let newEdges = ArrayList<GraphEdge>()
                for (edge in edges[fromId]) {
                    if (edge.toNodeId != nodeId) {
                        newEdges.add(edge)
                    }
                }
                edges[fromId] = newEdges
            }
            
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 删除边
     * @param fromNodeId 起始节点ID
     * @param toNodeId 目标节点ID
     * @param relationshipType 关系类型
     * @return 是否成功
     */
    public func deleteEdge(fromNodeId: String, toNodeId: String, relationshipType: String): Bool {
        try {
            if (edges.contains(fromNodeId)) {
                let newEdges = ArrayList<GraphEdge>()
                
                for (edge in edges[fromNodeId]) {
                    if (!(edge.toNodeId == toNodeId && edge.relationshipType == relationshipType)) {
                        newEdges.add(edge)
                    }
                }
                
                edges[fromNodeId] = newEdges
            }
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 查找路径
     * @param fromNodeId 起始节点ID
     * @param toNodeId 目标节点ID
     * @param maxDepth 最大深度
     * @return 路径列表
     */
    public func findPaths(fromNodeId: String, toNodeId: String, maxDepth: Int64): Array<Array<String>> {
        let paths = ArrayList<Array<String>>()
        let currentPath = ArrayList<String>()
        let visited = HashMap<String, Bool>()
        
        findPathsRecursive(fromNodeId, toNodeId, maxDepth, 0, currentPath, visited, paths)
        
        return paths.toArray()
    }
    
    /**
     * 递归查找路径
     */
    private func findPathsRecursive(currentNodeId: String, targetNodeId: String, maxDepth: Int64, currentDepth: Int64,
                                   currentPath: ArrayList<String>, visited: HashMap<String, Bool>, 
                                   paths: ArrayList<Array<String>>): Unit {
        if (currentDepth > maxDepth) {
            return
        }
        
        currentPath.add(currentNodeId)
        visited[currentNodeId] = true
        
        if (currentNodeId == targetNodeId) {
            paths.add(currentPath.toArray())
        } else if (edges.contains(currentNodeId)) {
            let edgeList = edges[currentNodeId]
            for (edge in edgeList) {
                if (!visited.contains(edge.toNodeId) || !visited[edge.toNodeId]) {
                    findPathsRecursive(edge.toNodeId, targetNodeId, maxDepth, currentDepth + 1, currentPath, visited, paths)
                }
            }
        }
        
        if (currentPath.size > 0) {
            let lastIndex = currentPath.size - 1
            currentPath.remove(at: lastIndex)
        }
        visited[currentNodeId] = false
    }
    
    /**
     * 清空所有数据
     * @return 是否成功
     */
    public func clear(): Bool {
        try {
            nodes.clear()
            edges.clear()
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 保存到文件
     * @param filePath 文件路径
     * @return 是否成功
     */
    public func save(_: String): Bool {
        // 简单实现，实际应该序列化为JSON或其他格式
        return true
    }
    
    /**
     * 从文件加载
     * @param filePath 文件路径
     * @return 是否成功
     */
    public func load(_: String): Bool {
        // 简单实现，实际应该从JSON或其他格式反序列化
        return true
    }
}