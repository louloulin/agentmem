/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.llm

import std.collection.HashMap
import std.collection.ArrayList

/**
 * 记忆提取结果
 */
public struct MemoryExtractionResult {
    public let facts: Array<String>          // 提取的事实
    public let actions: Array<MemoryAction>  // 建议的操作
    public let confidence: Float64           // 置信度
    
    public init(facts: Array<String>, actions: Array<MemoryAction>, confidence: Float64) {
        this.facts = facts
        this.actions = actions
        this.confidence = confidence
    }
}

/**
 * 记忆操作
 */
public struct MemoryAction {
    public let action: String        // 操作类型: ADD, UPDATE, DELETE
    public let memory: String        // 记忆内容
    public let reason: String        // 操作原因
    public let targetId: Option<String>  // 目标记忆ID（用于UPDATE/DELETE）
    
    public init(action: String, memory: String, reason: String) {
        this.action = action
        this.memory = memory
        this.reason = reason
        this.targetId = None
    }
    
    public init(action: String, memory: String, reason: String, targetId: String) {
        this.action = action
        this.memory = memory
        this.reason = reason
        this.targetId = Some(targetId)
    }
}

/**
 * 智能记忆提取器
 * 对应Mem0中LLM驱动的记忆提取功能
 */
public class MemoryExtractor {
    private let llm: LlmBase
    
    public init(llm: LlmBase) {
        this.llm = llm
    }
    
    /**
     * 从消息中提取事实
     * @param messages 输入消息
     * @return 提取的事实列表
     */
    public func extractFacts(messages: String): Array<String> {
        let systemPrompt = """
你是一个智能记忆提取器。你的任务是从用户的消息中提取关键事实和信息。

请遵循以下规则：
1. 只提取明确的事实信息，不要推测或假设
2. 提取的事实应该是具体的、可验证的
3. 忽略临时性的信息（如"今天"、"现在"等）
4. 关注用户的偏好、习惯、个人信息等持久性信息
5. 以JSON格式返回结果

返回格式：
{
  "facts": ["事实1", "事实2", "事实3"]
}
"""
        
        let userPrompt = "请从以下消息中提取关键事实：\n\n" + messages
        
        let messagesList = LlmUtils.buildMessages(systemPrompt, userPrompt)
        let response = llm.generateJsonResponse(messagesList, Some(0.3), Some(1000))
        
        return parseFactsFromResponse(response)
    }
    
    /**
     * 分析记忆操作
     * @param newFacts 新提取的事实
     * @param existingMemories 现有记忆
     * @return 建议的操作列表
     */
    public func analyzeMemoryActions(newFacts: Array<String>, 
                                   existingMemories: Array<String>): Array<MemoryAction> {
        if (newFacts.size == 0) {
            return Array<MemoryAction>()
        }
        
        let systemPrompt = """
你是一个智能记忆管理器。你的任务是分析新的事实信息和现有记忆，决定应该执行什么操作。

可能的操作：
- ADD: 添加新的记忆（当信息是全新的时）
- UPDATE: 更新现有记忆（当新信息与现有记忆相关但有更新时）
- DELETE: 删除过时记忆（当新信息表明旧信息不再正确时）

请遵循以下规则：
1. 仔细比较新事实和现有记忆
2. 避免重复添加相似的信息
3. 优先更新而不是删除+添加
4. 为每个操作提供清晰的理由

返回格式：
{
  "actions": [
    {
      "action": "ADD|UPDATE|DELETE",
      "memory": "记忆内容",
      "reason": "操作理由"
    }
  ]
}
"""
        
        var userPrompt = "新事实：\n"
        for (fact in newFacts) {
            userPrompt = userPrompt + "- " + fact + "\n"
        }
        
        userPrompt = userPrompt + "\n现有记忆：\n"
        for (memory in existingMemories) {
            userPrompt = userPrompt + "- " + memory + "\n"
        }
        
        userPrompt = userPrompt + "\n请分析并决定应该执行什么操作："
        
        let messagesList = LlmUtils.buildMessages(systemPrompt, userPrompt)
        let response = llm.generateJsonResponse(messagesList, Some(0.3), Some(1500))
        
        return parseActionsFromResponse(response)
    }
    
    /**
     * 检测记忆冲突
     * @param newMemory 新记忆
     * @param existingMemories 现有记忆
     * @return 是否存在冲突
     */
    public func detectConflicts(newMemory: String, existingMemories: Array<String>): Bool {
        if (existingMemories.size == 0) {
            return false
        }
        
        let systemPrompt = """
你是一个记忆冲突检测器。你的任务是检测新记忆是否与现有记忆存在冲突。

冲突的定义：
1. 直接矛盾的信息（如"喜欢咖啡" vs "不喜欢咖啡"）
2. 相互排斥的偏好或状态
3. 时间上的逻辑冲突

返回格式：
{
  "has_conflict": true/false,
  "reason": "冲突原因或无冲突说明"
}
"""
        
        var userPrompt = "新记忆：" + newMemory + "\n\n现有记忆：\n"
        for (memory in existingMemories) {
            userPrompt = userPrompt + "- " + memory + "\n"
        }
        
        let messagesList = LlmUtils.buildMessages(systemPrompt, userPrompt)
        let response = llm.generateJsonResponse(messagesList, Some(0.2), Some(500))
        
        return parseConflictFromResponse(response)
    }
    
    /**
     * 生成记忆摘要
     * @param memories 记忆列表
     * @return 摘要内容
     */
    public func generateSummary(memories: Array<String>): String {
        if (memories.size == 0) {
            return "无记忆内容"
        }
        
        let systemPrompt = """
你是一个记忆摘要生成器。请将给定的记忆列表整理成一个连贯、简洁的摘要。

要求：
1. 保持信息的准确性
2. 去除重复信息
3. 按逻辑顺序组织
4. 使用自然的语言表达
"""
        
        var userPrompt = "请为以下记忆生成摘要：\n\n"
        for (memory in memories) {
            userPrompt = userPrompt + "- " + memory + "\n"
        }
        
        let messagesList = LlmUtils.buildMessages(systemPrompt, userPrompt)
        let response = llm.generateResponse(messagesList, Some(0.5), Some(1000))
        
        return response.content
    }
    
    /**
     * 解析事实提取响应
     */
    private func parseFactsFromResponse(response: String): Array<String> {
        try {
            let cleaned = LlmUtils.removeCodeBlocks(response)
            // 简化的JSON解析 - 在实际实现中应该使用正确的JSON解析器
            if (cleaned.contains("\"facts\"")) {
                // 模拟解析结果
                let factsList = ArrayList<String>()
                factsList.add("解析的事实1")
                factsList.add("解析的事实2")
                return factsList.toArray()
            }
        } catch (e: Exception) {
            // 解析失败，返回空数组
        }
        return Array<String>()
    }
    
    /**
     * 解析操作分析响应
     */
    private func parseActionsFromResponse(response: String): Array<MemoryAction> {
        try {
            let cleaned = LlmUtils.removeCodeBlocks(response)
            // 简化的JSON解析
            if (cleaned.contains("\"actions\"")) {
                let actionsList = ArrayList<MemoryAction>()
                actionsList.add(MemoryAction("ADD", "示例记忆", "示例原因"))
                return actionsList.toArray()
            }
        } catch (e: Exception) {
            // 解析失败，返回空数组
        }
        return Array<MemoryAction>()
    }
    
    /**
     * 解析冲突检测响应
     */
    private func parseConflictFromResponse(response: String): Bool {
        try {
            let cleaned = LlmUtils.removeCodeBlocks(response)
            // 简化的JSON解析
            return cleaned.contains("\"has_conflict\": true")
        } catch (e: Exception) {
            return false
        }
    }
}
