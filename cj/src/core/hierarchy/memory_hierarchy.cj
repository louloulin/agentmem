/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.hierarchy

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.{MemoryRecord, MemoryLevel, MemoryScope, HierarchicalMemoryRecord}

/**
 * 记忆层级管理器
 * 负责管理不同层级记忆的存储、检索和访问控制
 */
public class MemoryHierarchyManager {
    private let userMemories: HashMap<String, HashMap<String, HierarchicalMemoryRecord>>    // userId -> memoryId -> record
    private let sessionMemories: HashMap<String, HashMap<String, HierarchicalMemoryRecord>> // sessionId -> memoryId -> record
    private let agentMemories: HashMap<String, HashMap<String, HierarchicalMemoryRecord>>   // agentId -> memoryId -> record
    private let scopeIndex: HashMap<String, MemoryScope>  // scopeId -> scope
    
    /**
     * 构造函数
     */
    public init() {
        this.userMemories = HashMap<String, HashMap<String, HierarchicalMemoryRecord>>()
        this.sessionMemories = HashMap<String, HashMap<String, HierarchicalMemoryRecord>>()
        this.agentMemories = HashMap<String, HashMap<String, HierarchicalMemoryRecord>>()
        this.scopeIndex = HashMap<String, MemoryScope>()
    }
    
    /**
     * 添加分层记忆
     */
    public func addMemory(memory: HierarchicalMemoryRecord): Bool {
        if (!memory.scope.isValid()) {
            return false
        }

        let scopeId = memory.scope.getScopeId()
        scopeIndex[scopeId] = memory.scope

        if (memory.scope.level == MemoryLevel.USER) {
            return addUserMemory(memory)
        } else if (memory.scope.level == MemoryLevel.SESSION) {
            return addSessionMemory(memory)
        } else if (memory.scope.level == MemoryLevel.AGENT) {
            return addAgentMemory(memory)
        }

        return false
    }
    
    /**
     * 获取指定作用域的记忆
     */
    public func getMemory(memoryId: String, requestScope: MemoryScope): Option<HierarchicalMemoryRecord> {
        // 首先尝试从请求作用域获取
        let directResult = getMemoryFromScope(memoryId, requestScope)
        if (directResult.isSome()) {
            return directResult
        }
        
        // 如果直接获取失败，尝试跨作用域访问
        return searchMemoryAcrossScopes(memoryId, requestScope)
    }
    
    /**
     * 从指定作用域获取记忆（私有方法）
     */
    private func getMemoryFromScope(memoryId: String, scope: MemoryScope): Option<HierarchicalMemoryRecord> {
        if (scope.level == MemoryLevel.USER) {
            return getUserMemoryFromScope(memoryId, scope)
        } else if (scope.level == MemoryLevel.SESSION) {
            return getSessionMemoryFromScope(memoryId, scope)
        } else if (scope.level == MemoryLevel.AGENT) {
            return getAgentMemoryFromScope(memoryId, scope)
        }
        return None
    }
    
    /**
     * 跨作用域搜索记忆（私有方法）
     */
    private func searchMemoryAcrossScopes(memoryId: String, requestScope: MemoryScope): Option<HierarchicalMemoryRecord> {
        // 搜索所有用户记忆
        for ((userId, memories) in userMemories) {
            if (memories.contains(memoryId)) {
                let memory = memories[memoryId]
                if (requestScope.canAccess(memory.scope)) {
                    return Some(memory)
                }
            }
        }
        
        // 搜索所有会话记忆
        for ((sessionId, memories) in sessionMemories) {
            if (memories.contains(memoryId)) {
                let memory = memories[memoryId]
                if (requestScope.canAccess(memory.scope)) {
                    return Some(memory)
                }
            }
        }
        
        // 搜索所有代理记忆
        for ((agentId, memories) in agentMemories) {
            if (memories.contains(memoryId)) {
                let memory = memories[memoryId]
                if (requestScope.canAccess(memory.scope)) {
                    return Some(memory)
                }
            }
        }
        
        return None
    }
    
    /**
     * 获取指定作用域的所有记忆
     */
    public func getMemoriesInScope(requestScope: MemoryScope): Array<HierarchicalMemoryRecord> {
        let result = ArrayList<HierarchicalMemoryRecord>()
        
        // 获取直接作用域的记忆
        let directMemories = getDirectScopeMemories(requestScope)
        for (memory in directMemories) {
            result.add(memory)
        }
        
        // 获取可访问的跨作用域记忆
        let crossScopeMemories = getCrossScopeMemories(requestScope)
        for (memory in crossScopeMemories) {
            result.add(memory)
        }
        
        return result.toArray()
    }
    
    /**
     * 获取直接作用域的记忆（私有方法）
     */
    private func getDirectScopeMemories(scope: MemoryScope): Array<HierarchicalMemoryRecord> {
        let result = ArrayList<HierarchicalMemoryRecord>()

        if (scope.level == MemoryLevel.USER) {
            addUserMemoriesToResult(scope, result)
        } else if (scope.level == MemoryLevel.SESSION) {
            addSessionMemoriesToResult(scope, result)
        } else if (scope.level == MemoryLevel.AGENT) {
            addAgentMemoriesToResult(scope, result)
        }

        return result.toArray()
    }
    
    /**
     * 获取可访问的跨作用域记忆（私有方法）
     */
    private func getCrossScopeMemories(requestScope: MemoryScope): Array<HierarchicalMemoryRecord> {
        let result = ArrayList<HierarchicalMemoryRecord>()
        
        // 检查所有用户记忆
        for ((userId, memories) in userMemories) {
            for ((_, memory) in memories) {
                if (requestScope.canAccess(memory.scope) && memory.scope.getScopeId() != requestScope.getScopeId()) {
                    result.add(memory)
                }
            }
        }
        
        // 检查所有代理记忆
        for ((agentId, memories) in agentMemories) {
            for ((_, memory) in memories) {
                if (requestScope.canAccess(memory.scope) && memory.scope.getScopeId() != requestScope.getScopeId()) {
                    result.add(memory)
                }
            }
        }
        
        return result.toArray()
    }
    
    /**
     * 删除记忆
     */
    public func deleteMemory(memoryId: String, scope: MemoryScope): Bool {
        if (scope.level == MemoryLevel.USER) {
            return deleteUserMemory(memoryId, scope)
        } else if (scope.level == MemoryLevel.SESSION) {
            return deleteSessionMemory(memoryId, scope)
        } else if (scope.level == MemoryLevel.AGENT) {
            return deleteAgentMemory(memoryId, scope)
        }
        return false
    }
    
    /**
     * 清理过期记忆
     */
    public func cleanupExpiredMemories(currentTime: Int64): Int64 {
        var deletedCount: Int64 = 0
        
        // 清理用户记忆
        for ((userId, memories) in userMemories) {
            let toDelete = ArrayList<String>()
            for ((memoryId, memory) in memories) {
                if (memory.isExpired(currentTime)) {
                    toDelete.add(memoryId)
                }
            }
            for (memoryId in toDelete) {
                memories.remove(memoryId)
                deletedCount = deletedCount + 1
            }
        }
        
        // 清理会话记忆
        for ((sessionId, memories) in sessionMemories) {
            let toDelete = ArrayList<String>()
            for ((memoryId, memory) in memories) {
                if (memory.isExpired(currentTime)) {
                    toDelete.add(memoryId)
                }
            }
            for (memoryId in toDelete) {
                memories.remove(memoryId)
                deletedCount = deletedCount + 1
            }
        }
        
        // 清理代理记忆
        for ((agentId, memories) in agentMemories) {
            let toDelete = ArrayList<String>()
            for ((memoryId, memory) in memories) {
                if (memory.isExpired(currentTime)) {
                    toDelete.add(memoryId)
                }
            }
            for (memoryId in toDelete) {
                memories.remove(memoryId)
                deletedCount = deletedCount + 1
            }
        }
        
        return deletedCount
    }
    
    /**
     * 获取统计信息
     */
    public func getStatistics(): HashMap<String, Int64> {
        let stats = HashMap<String, Int64>()
        
        var totalUserMemories: Int64 = 0
        for ((_, memories) in userMemories) {
            totalUserMemories = totalUserMemories + memories.size
        }
        
        var totalSessionMemories: Int64 = 0
        for ((_, memories) in sessionMemories) {
            totalSessionMemories = totalSessionMemories + memories.size
        }
        
        var totalAgentMemories: Int64 = 0
        for ((_, memories) in agentMemories) {
            totalAgentMemories = totalAgentMemories + memories.size
        }
        
        stats["user_memories"] = totalUserMemories
        stats["session_memories"] = totalSessionMemories
        stats["agent_memories"] = totalAgentMemories
        stats["total_memories"] = totalUserMemories + totalSessionMemories + totalAgentMemories
        stats["total_users"] = userMemories.size
        stats["total_sessions"] = sessionMemories.size
        stats["total_agents"] = agentMemories.size
        
        return stats
    }

    /**
     * 添加用户记忆（私有辅助方法）
     */
    private func addUserMemory(memory: HierarchicalMemoryRecord): Bool {
        match (memory.scope.userId) {
            case Some(userIdStr) =>
                if (!userMemories.contains(userIdStr)) {
                    userMemories[userIdStr] = HashMap<String, HierarchicalMemoryRecord>()
                }
                userMemories[userIdStr][memory.baseRecord.id] = memory
                true
            case None => false
        }
    }

    /**
     * 添加会话记忆（私有辅助方法）
     */
    private func addSessionMemory(memory: HierarchicalMemoryRecord): Bool {
        match (memory.scope.sessionId) {
            case Some(sessionIdStr) =>
                if (!sessionMemories.contains(sessionIdStr)) {
                    sessionMemories[sessionIdStr] = HashMap<String, HierarchicalMemoryRecord>()
                }
                sessionMemories[sessionIdStr][memory.baseRecord.id] = memory
                true
            case None => false
        }
    }

    /**
     * 添加代理记忆（私有辅助方法）
     */
    private func addAgentMemory(memory: HierarchicalMemoryRecord): Bool {
        match (memory.scope.agentId) {
            case Some(agentIdStr) =>
                if (!agentMemories.contains(agentIdStr)) {
                    agentMemories[agentIdStr] = HashMap<String, HierarchicalMemoryRecord>()
                }
                agentMemories[agentIdStr][memory.baseRecord.id] = memory
                true
            case None => false
        }
    }

    /**
     * 从用户作用域获取记忆（私有辅助方法）
     */
    private func getUserMemoryFromScope(memoryId: String, scope: MemoryScope): Option<HierarchicalMemoryRecord> {
        match (scope.userId) {
            case Some(userIdStr) =>
                if (userMemories.contains(userIdStr) && userMemories[userIdStr].contains(memoryId)) {
                    Some(userMemories[userIdStr][memoryId])
                } else {
                    None<HierarchicalMemoryRecord>
                }
            case None => None<HierarchicalMemoryRecord>
        }
    }

    /**
     * 从会话作用域获取记忆（私有辅助方法）
     */
    private func getSessionMemoryFromScope(memoryId: String, scope: MemoryScope): Option<HierarchicalMemoryRecord> {
        match (scope.sessionId) {
            case Some(sessionIdStr) =>
                if (sessionMemories.contains(sessionIdStr) && sessionMemories[sessionIdStr].contains(memoryId)) {
                    Some(sessionMemories[sessionIdStr][memoryId])
                } else {
                    None<HierarchicalMemoryRecord>
                }
            case None => None<HierarchicalMemoryRecord>
        }
    }

    /**
     * 从代理作用域获取记忆（私有辅助方法）
     */
    private func getAgentMemoryFromScope(memoryId: String, scope: MemoryScope): Option<HierarchicalMemoryRecord> {
        match (scope.agentId) {
            case Some(agentIdStr) =>
                if (agentMemories.contains(agentIdStr) && agentMemories[agentIdStr].contains(memoryId)) {
                    Some(agentMemories[agentIdStr][memoryId])
                } else {
                    None<HierarchicalMemoryRecord>
                }
            case None => None<HierarchicalMemoryRecord>
        }
    }

    /**
     * 添加用户记忆到结果列表（私有辅助方法）
     */
    private func addUserMemoriesToResult(scope: MemoryScope, result: ArrayList<HierarchicalMemoryRecord>): Unit {
        match (scope.userId) {
            case Some(userIdStr) =>
                if (userMemories.contains(userIdStr)) {
                    for ((_, memory) in userMemories[userIdStr]) {
                        result.add(memory)
                    }
                }
            case None => ()
        }
    }

    /**
     * 添加会话记忆到结果列表（私有辅助方法）
     */
    private func addSessionMemoriesToResult(scope: MemoryScope, result: ArrayList<HierarchicalMemoryRecord>): Unit {
        match (scope.sessionId) {
            case Some(sessionIdStr) =>
                if (sessionMemories.contains(sessionIdStr)) {
                    for ((_, memory) in sessionMemories[sessionIdStr]) {
                        result.add(memory)
                    }
                }
            case None => ()
        }
    }

    /**
     * 添加代理记忆到结果列表（私有辅助方法）
     */
    private func addAgentMemoriesToResult(scope: MemoryScope, result: ArrayList<HierarchicalMemoryRecord>): Unit {
        match (scope.agentId) {
            case Some(agentIdStr) =>
                if (agentMemories.contains(agentIdStr)) {
                    for ((_, memory) in agentMemories[agentIdStr]) {
                        result.add(memory)
                    }
                }
            case None => ()
        }
    }

    /**
     * 删除用户记忆（私有辅助方法）
     */
    private func deleteUserMemory(memoryId: String, scope: MemoryScope): Bool {
        match (scope.userId) {
            case Some(userIdStr) =>
                if (userMemories.contains(userIdStr) && userMemories[userIdStr].contains(memoryId)) {
                    userMemories[userIdStr].remove(memoryId)
                    true
                } else {
                    false
                }
            case None => false
        }
    }

    /**
     * 删除会话记忆（私有辅助方法）
     */
    private func deleteSessionMemory(memoryId: String, scope: MemoryScope): Bool {
        match (scope.sessionId) {
            case Some(sessionIdStr) =>
                if (sessionMemories.contains(sessionIdStr) && sessionMemories[sessionIdStr].contains(memoryId)) {
                    sessionMemories[sessionIdStr].remove(memoryId)
                    true
                } else {
                    false
                }
            case None => false
        }
    }

    /**
     * 删除代理记忆（私有辅助方法）
     */
    private func deleteAgentMemory(memoryId: String, scope: MemoryScope): Bool {
        match (scope.agentId) {
            case Some(agentIdStr) =>
                if (agentMemories.contains(agentIdStr) && agentMemories[agentIdStr].contains(memoryId)) {
                    agentMemories[agentIdStr].remove(memoryId)
                    true
                } else {
                    false
                }
            case None => false
        }
    }
}
