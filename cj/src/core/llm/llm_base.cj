/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.llm

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.core.config.LlmConfig
// 移除循环依赖的导入

/**
 * LLM消息结构
 */
public struct LlmMessage {
    public let role: String      // "system", "user", "assistant"
    public let content: String   // 消息内容
    
    public init(role: String, content: String) {
        this.role = role
        this.content = content
    }
}

/**
 * LLM响应结构
 */
public struct LlmResponse {
    public let content: String           // 响应内容
    public let usage: Option<LlmUsage>   // 使用统计
    public let model: String             // 使用的模型
    
    public init(content: String, model: String) {
        this.content = content
        this.usage = None
        this.model = model
    }
    
    public init(content: String, usage: LlmUsage, model: String) {
        this.content = content
        this.usage = Some(usage)
        this.model = model
    }
}

/**
 * LLM使用统计
 */
public struct LlmUsage {
    public let promptTokens: Int64       // 输入token数
    public let completionTokens: Int64   // 输出token数
    public let totalTokens: Int64        // 总token数
    
    public init(promptTokens: Int64, completionTokens: Int64) {
        this.promptTokens = promptTokens
        this.completionTokens = completionTokens
        this.totalTokens = promptTokens + completionTokens
    }
}

/**
 * LLM基础接口
 * 对应Mem0的LLMBase类
 */
public interface LlmBase {
    /**
     * 生成响应
     * @param messages 消息列表
     * @param temperature 温度参数
     * @param maxTokens 最大token数
     * @return LLM响应
     */
    func generateResponse(messages: Array<LlmMessage>, 
                         temperature: Option<Float64>, 
                         maxTokens: Option<Int64>): LlmResponse
    
    /**
     * 生成JSON格式响应
     * @param messages 消息列表
     * @param temperature 温度参数
     * @param maxTokens 最大token数
     * @return JSON格式的响应
     */
    func generateJsonResponse(messages: Array<LlmMessage>, 
                             temperature: Option<Float64>, 
                             maxTokens: Option<Int64>): String
    
    /**
     * 验证配置
     * @return 是否有效
     */
    func validateConfig(): Bool
    
    /**
     * 获取模型名称
     * @return 模型名称
     */
    func getModelName(): String
    
    /**
     * 获取提供商名称
     * @return 提供商名称
     */
    func getProviderName(): String
}

/**
 * 模拟LLM实现
 * 用于测试和开发环境
 */
public class MockLlm <: LlmBase {
    private let config: LlmConfig
    
    public init(config: LlmConfig) {
        this.config = config
    }
    
    public func generateResponse(messages: Array<LlmMessage>, 
                                temperature: Option<Float64>, 
                                maxTokens: Option<Int64>): LlmResponse {
        // 模拟响应生成
        let lastMessage = messages[messages.size - 1]
        var response = ""
        
        if (lastMessage.content.contains("extract") || lastMessage.content.contains("facts")) {
            response = "{\"facts\": [\"模拟提取的事实信息\"]}"
        } else if (lastMessage.content.contains("action") || lastMessage.content.contains("memory")) {
            response = "{\"actions\": [{\"action\": \"ADD\", \"memory\": \"模拟记忆内容\"}]}"
        } else {
            response = "这是一个模拟的LLM响应，基于输入: " + lastMessage.content
        }
        
        let usage = LlmUsage(50, 30)
        return LlmResponse(response, usage, config.model)
    }
    
    public func generateJsonResponse(messages: Array<LlmMessage>, 
                                    temperature: Option<Float64>, 
                                    maxTokens: Option<Int64>): String {
        let response = generateResponse(messages, temperature, maxTokens)
        return response.content
    }
    
    public func validateConfig(): Bool {
        return !config.model.isEmpty() && !config.provider.isEmpty()
    }
    
    public func getModelName(): String {
        return config.model
    }
    
    public func getProviderName(): String {
        return config.provider
    }
}

/**
 * LLM工厂类
 * 对应Mem0的LlmFactory
 */
public class LlmFactory {
    /**
     * 创建LLM实例
     * @param provider 提供商名称
     * @param config LLM配置
     * @return LLM实例
     */
    public static func create(provider: String, config: LlmConfig): LlmBase {
        match (provider) {
            case "mock" => MockLlm(config)
            case "openai" => RealOpenAILlm(config)  // 使用真实的OpenAI实现
            case "anthropic" => RealAnthropicLlm(config)  // 使用真实的Anthropic实现
            case "azure" => MockLlm(config)  // Azure实现待开发
            case _ => SimpleRealLlm(config, provider)  // 默认使用简化的真实实现
        }
    }
    
    /**
     * 获取支持的提供商列表
     * @return 提供商列表
     */
    public static func getSupportedProviders(): Array<String> {
        return ["mock", "openai", "anthropic", "azure"]
    }
}

/**
 * LLM辅助工具类
 */
public class LlmUtils {
    /**
     * 创建系统消息
     * @param content 消息内容
     * @return LLM消息
     */
    public static func createSystemMessage(content: String): LlmMessage {
        return LlmMessage("system", content)
    }
    
    /**
     * 创建用户消息
     * @param content 消息内容
     * @return LLM消息
     */
    public static func createUserMessage(content: String): LlmMessage {
        return LlmMessage("user", content)
    }
    
    /**
     * 创建助手消息
     * @param content 消息内容
     * @return LLM消息
     */
    public static func createAssistantMessage(content: String): LlmMessage {
        return LlmMessage("assistant", content)
    }
    
    /**
     * 构建消息列表
     * @param systemPrompt 系统提示
     * @param userPrompt 用户提示
     * @return 消息列表
     */
    public static func buildMessages(systemPrompt: String, userPrompt: String): Array<LlmMessage> {
        let messagesList = ArrayList<LlmMessage>()
        messagesList.add(createSystemMessage(systemPrompt))
        messagesList.add(createUserMessage(userPrompt))
        return messagesList.toArray()
    }
    
    /**
     * 清理代码块标记
     * @param content 原始内容
     * @return 清理后的内容
     */
    public static func removeCodeBlocks(content: String): String {
        var cleaned = content
        
        // 移除```json和```标记
        cleaned = cleaned.replace("```json", "")
        cleaned = cleaned.replace("```", "")
        
        // 移除前后空白
        cleaned = cleaned.trimAscii()
        
        return cleaned
    }
}
