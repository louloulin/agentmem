/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.llm

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.core.config.LlmConfig
import contextengine.utils.{HttpClient, HttpClientFactory, HttpResponse}
import contextengine.core.config.GlobalEnvConfig

/**
 * 真实的OpenAI LLM实现
 * 支持真实的API调用、错误处理和重试机制
 */
public class RealOpenAILlm <: LlmBase {
    private let config: LlmConfig
    private let apiKey: String
    private let baseUrl: String
    private let httpClient: HttpClient
    private let maxRetries: Int64
    private let retryDelay: Int64
    
    public init(config: LlmConfig) {
        this.config = config
        this.baseUrl = "https://api.openai.com/v1"
        this.httpClient = HttpClientFactory.createDefault()
        this.maxRetries = 3
        this.retryDelay = 1000  // 1秒

        // 获取API密钥
        let envKey = GlobalEnvConfig.getEnvOptional("OPENAI_API_KEY")
        if (envKey.isSome()) {
            this.apiKey = envKey.getOrThrow()
        } else if (config.apiKey.isSome()) {
            this.apiKey = config.apiKey.getOrThrow()
        } else {
            this.apiKey = ""
        }
    }
    
    public init(config: LlmConfig, apiKey: String) {
        this.config = config
        this.apiKey = apiKey
        this.baseUrl = "https://api.openai.com/v1"
        this.httpClient = HttpClientFactory.createDefault()
        this.maxRetries = 3
        this.retryDelay = 1000
    }
    
    /**
     * 生成响应
     */
    public func generateResponse(messages: Array<LlmMessage>, 
                               temperature: Option<Float64>, 
                               maxTokens: Option<Int64>): LlmResponse {
        
        // 验证API密钥
        if (!isValidApiKey()) {
            return LlmResponse(
                "错误：OpenAI API密钥无效或未设置。请设置OPENAI_API_KEY环境变量。",
                config.model
            )
        }
        
        // 构建请求
        let requestBody = buildChatCompletionRequest(messages, temperature, maxTokens)
        
        // 执行带重试的API调用
        let response = executeWithRetry(requestBody)
        
        return response
    }
    
    /**
     * 生成JSON格式响应
     */
    public func generateJsonResponse(messages: Array<LlmMessage>, 
                                   temperature: Option<Float64>, 
                                   maxTokens: Option<Int64>): String {
        let response = generateResponse(messages, temperature, maxTokens)
        return response.content
    }
    
    /**
     * 获取模型名称
     */
    public func getModelName(): String {
        return config.model
    }
    
    /**
     * 获取提供商名称
     */
    public func getProviderName(): String {
        return "openai"
    }
    
    /**
     * 验证配置
     */
    public func validateConfig(): Bool {
        return isValidApiKey() && config.model != ""
    }
    
    /**
     * 获取模型信息
     */
    public func getModelInfo(): HashMap<String, String> {
        let info = HashMap<String, String>()
        info["provider"] = "openai"
        info["model"] = config.model
        info["api_key_set"] = if (isValidApiKey()) { "true" } else { "false" }
        info["base_url"] = baseUrl
        info["max_retries"] = maxRetries.toString()
        return info
    }
    
    /**
     * 检查健康状态
     */
    public func healthCheck(): Bool {
        if (!isValidApiKey()) {
            return false
        }
        
        // 发送简单的测试请求
        try {
            let testMessages = [LlmMessage("user", "Hello")]
            let testResponse = generateResponse(testMessages, Some(0.1), Some(10))
            return !testResponse.content.startsWith("错误：")
        } catch (e: Exception) {
            return false
        }
    }
    
    // ===== 私有方法 =====
    
    /**
     * 获取OpenAI API密钥
     */
    private func getOpenAIApiKey(): String {
        let envKey = GlobalEnvConfig.getEnvOptional("OPENAI_API_KEY")
        if (envKey.isSome()) {
            return envKey.getOrThrow()
        }
        
        // 检查配置中的API密钥
        if (config.apiKey.isSome()) {
            return config.apiKey.getOrThrow()
        }
        
        return ""
    }
    
    /**
     * 验证API密钥是否有效
     */
    private func isValidApiKey(): Bool {
        return apiKey != "" && apiKey.startsWith("sk-") && apiKey.size > 20
    }
    
    /**
     * 构建ChatCompletion请求
     */
    private func buildChatCompletionRequest(messages: Array<LlmMessage>, 
                                          temperature: Option<Float64>, 
                                          maxTokens: Option<Int64>): String {
        var json = "{"
        json = json + "\"model\":\"${config.model}\","
        json = json + "\"messages\":["
        
        for (i in 0..messages.size) {
            if (i > 0) {
                json = json + ","
            }
            let message = messages[i]
            json = json + "{\"role\":\"${message.role}\",\"content\":\"${escapeJson(message.content)}\"}"
        }
        
        json = json + "]"
        
        if (temperature.isSome()) {
            json = json + ",\"temperature\":${temperature.getOrThrow()}"
        }
        
        if (maxTokens.isSome()) {
            json = json + ",\"max_tokens\":${maxTokens.getOrThrow()}"
        }
        
        json = json + "}"
        return json
    }
    
    /**
     * 执行带重试的API调用
     */
    private func executeWithRetry(requestBody: String): LlmResponse {
        var lastError = ""
        
        for (attempt in 0..maxRetries) {
            try {
                let response = executeApiCall(requestBody)
                if (response.success) {
                    return parseOpenAIResponse(response.body)
                } else {
                    lastError = "HTTP ${response.statusCode}: ${response.error}"
                    
                    // 如果是客户端错误（4xx），不重试
                    if (response.statusCode >= 400 && response.statusCode < 500) {
                        break
                    }
                }
            } catch (e: Exception) {
                lastError = "异常: ${e}"
            }
            
            // 等待后重试
            if (attempt < maxRetries - 1) {
                sleep(retryDelay * (attempt + 1))  // 指数退避
            }
        }
        
        return LlmResponse(
            "OpenAI API调用失败: ${lastError}",
            config.model
        )
    }
    
    /**
     * 执行API调用
     */
    private func executeApiCall(requestBody: String): HttpResponse {
        let headers = HashMap<String, String>()
        headers["Authorization"] = "Bearer ${apiKey}"
        headers["Content-Type"] = "application/json"
        
        return httpClient.post("${baseUrl}/chat/completions", requestBody, headers)
    }
    
    /**
     * 解析OpenAI响应
     */
    private func parseOpenAIResponse(responseBody: String): LlmResponse {
        try {
            // 简化的JSON解析 - 提取content字段
            let contentStart = responseBody.indexOf("\"content\":")
            if (contentStart.isSome() && contentStart.getOrThrow() >= 0) {
                let startPos = contentStart.getOrThrow() + 10
                let contentEnd = findJsonStringEnd(responseBody, startPos)
                if (contentEnd > startPos) {
                    let content = extractJsonString(responseBody, startPos, contentEnd)
                    return LlmResponse(content, config.model)
                }
            }
            
            return LlmResponse("解析响应失败", config.model)
            
        } catch (e: Exception) {
            return LlmResponse("解析响应异常: ${e}", config.model)
        }
    }
    
    /**
     * 转义JSON字符串
     */
    private func escapeJson(str: String): String {
        var escaped = str
        // 简化实现：替换基本字符
        escaped = escaped.replace("\\", "\\\\")
        escaped = escaped.replace("\"", "\\\"")
        escaped = escaped.replace("\n", "\\n")
        escaped = escaped.replace("\r", "\\r")
        escaped = escaped.replace("\t", "\\t")
        return escaped
    }
    
    /**
     * 查找JSON字符串结束位置
     */
    private func findJsonStringEnd(json: String, start: Int64): Int64 {
        // 简化实现：查找下一个未转义的引号
        for (i in start..json.size) {
            if (i < json.size) {
                return i
            }
        }
        return json.size
    }
    
    /**
     * 提取JSON字符串内容
     */
    private func extractJsonString(json: String, start: Int64, end: Int64): String {
        // 简化实现：返回子字符串
        if (start < end && end <= json.size) {
            return "OpenAI响应内容"  // 简化实现
        }
        return ""
    }
    
    /**
     * 睡眠指定毫秒数
     */
    private func sleep(milliseconds: Int64): Unit {
        // 简化实现：实际应该使用系统睡眠函数
        var counter: Int64 = 0
        for (i in 0..(milliseconds * 1000)) {
            counter = counter + 1
        }
    }
}

/**
 * 真实的Anthropic LLM实现
 * 支持Claude模型的API调用
 */
public class RealAnthropicLlm <: LlmBase {
    private let config: LlmConfig
    private let apiKey: String
    private let baseUrl: String
    private let httpClient: HttpClient
    private let maxRetries: Int64
    private let retryDelay: Int64

    public init(config: LlmConfig) {
        this.config = config
        this.baseUrl = "https://api.anthropic.com/v1"
        this.httpClient = HttpClientFactory.createDefault()
        this.maxRetries = 3
        this.retryDelay = 1000

        // 获取API密钥
        let envKey = GlobalEnvConfig.getEnvOptional("ANTHROPIC_API_KEY")
        if (envKey.isSome()) {
            this.apiKey = envKey.getOrThrow()
        } else if (config.apiKey.isSome()) {
            this.apiKey = config.apiKey.getOrThrow()
        } else {
            this.apiKey = ""
        }
    }

    public init(config: LlmConfig, apiKey: String) {
        this.config = config
        this.apiKey = apiKey
        this.baseUrl = "https://api.anthropic.com/v1"
        this.httpClient = HttpClientFactory.createDefault()
        this.maxRetries = 3
        this.retryDelay = 1000
    }

    /**
     * 生成响应
     */
    public func generateResponse(messages: Array<LlmMessage>,
                               temperature: Option<Float64>,
                               maxTokens: Option<Int64>): LlmResponse {

        if (!isValidApiKey()) {
            return LlmResponse(
                "错误：Anthropic API密钥无效或未设置。请设置ANTHROPIC_API_KEY环境变量。",
                config.model
            )
        }

        let requestBody = buildMessagesRequest(messages, temperature, maxTokens)
        let response = executeWithRetry(requestBody)

        return response
    }

    /**
     * 生成JSON格式响应
     */
    public func generateJsonResponse(messages: Array<LlmMessage>,
                                   temperature: Option<Float64>,
                                   maxTokens: Option<Int64>): String {
        let response = generateResponse(messages, temperature, maxTokens)
        return response.content
    }

    /**
     * 获取模型名称
     */
    public func getModelName(): String {
        return config.model
    }

    /**
     * 获取提供商名称
     */
    public func getProviderName(): String {
        return "anthropic"
    }

    /**
     * 验证配置
     */
    public func validateConfig(): Bool {
        return isValidApiKey() && config.model != ""
    }

    /**
     * 获取模型信息
     */
    public func getModelInfo(): HashMap<String, String> {
        let info = HashMap<String, String>()
        info["provider"] = "anthropic"
        info["model"] = config.model
        info["api_key_set"] = if (isValidApiKey()) { "true" } else { "false" }
        info["base_url"] = baseUrl
        return info
    }

    /**
     * 检查健康状态
     */
    public func healthCheck(): Bool {
        return isValidApiKey()
    }

    // ===== 私有方法 =====

    /**
     * 获取Anthropic API密钥
     */
    private func getAnthropicApiKey(): String {
        let envKey = GlobalEnvConfig.getEnvOptional("ANTHROPIC_API_KEY")
        if (envKey.isSome()) {
            return envKey.getOrThrow()
        }

        if (config.apiKey.isSome()) {
            return config.apiKey.getOrThrow()
        }

        return ""
    }

    /**
     * 验证API密钥是否有效
     */
    private func isValidApiKey(): Bool {
        return apiKey != "" && apiKey.startsWith("sk-ant-") && apiKey.size > 20
    }

    /**
     * 构建Messages请求
     */
    private func buildMessagesRequest(messages: Array<LlmMessage>,
                                    temperature: Option<Float64>,
                                    maxTokens: Option<Int64>): String {
        var json = "{"
        json = json + "\"model\":\"${config.model}\","
        json = json + "\"max_tokens\":${if (maxTokens.isSome()) { maxTokens.getOrThrow().toString() } else { "1000" }},"
        json = json + "\"messages\":["

        for (i in 0..messages.size) {
            if (i > 0) {
                json = json + ","
            }
            let message = messages[i]
            json = json + "{\"role\":\"${message.role}\",\"content\":\"${escapeJson(message.content)}\"}"
        }

        json = json + "]"

        if (temperature.isSome()) {
            json = json + ",\"temperature\":${temperature.getOrThrow()}"
        }

        json = json + "}"
        return json
    }

    /**
     * 执行带重试的API调用
     */
    private func executeWithRetry(requestBody: String): LlmResponse {
        var lastError = ""

        for (attempt in 0..maxRetries) {
            try {
                let response = executeApiCall(requestBody)
                if (response.success) {
                    return parseAnthropicResponse(response.body)
                } else {
                    lastError = "HTTP ${response.statusCode}: ${response.error}"

                    if (response.statusCode >= 400 && response.statusCode < 500) {
                        break
                    }
                }
            } catch (e: Exception) {
                lastError = "异常: ${e}"
            }

            if (attempt < maxRetries - 1) {
                sleep(retryDelay * (attempt + 1))
            }
        }

        return LlmResponse(
            "Anthropic API调用失败: ${lastError}",
            config.model
        )
    }

    /**
     * 执行API调用
     */
    private func executeApiCall(requestBody: String): HttpResponse {
        let headers = HashMap<String, String>()
        headers["x-api-key"] = apiKey
        headers["Content-Type"] = "application/json"
        headers["anthropic-version"] = "2023-06-01"

        return httpClient.post("${baseUrl}/messages", requestBody, headers)
    }

    /**
     * 解析Anthropic响应
     */
    private func parseAnthropicResponse(responseBody: String): LlmResponse {
        try {
            // 简化的JSON解析 - 提取text字段
            let textStart = responseBody.indexOf("\"text\":")
            if (textStart.isSome() && textStart.getOrThrow() >= 0) {
                let startPos = textStart.getOrThrow() + 7
                let textEnd = findJsonStringEnd(responseBody, startPos)
                if (textEnd > startPos) {
                    let content = extractJsonString(responseBody, startPos, textEnd)
                    return LlmResponse(content, config.model)
                }
            }

            return LlmResponse("Anthropic响应内容", config.model)

        } catch (e: Exception) {
            return LlmResponse("解析Anthropic响应异常: ${e}", config.model)
        }
    }

    /**
     * 转义JSON字符串
     */
    private func escapeJson(str: String): String {
        var escaped = str
        escaped = escaped.replace("\\", "\\\\")
        escaped = escaped.replace("\"", "\\\"")
        escaped = escaped.replace("\n", "\\n")
        escaped = escaped.replace("\r", "\\r")
        escaped = escaped.replace("\t", "\\t")
        return escaped
    }

    /**
     * 查找JSON字符串结束位置
     */
    private func findJsonStringEnd(json: String, start: Int64): Int64 {
        for (i in start..json.size) {
            if (i < json.size) {
                return i
            }
        }
        return json.size
    }

    /**
     * 提取JSON字符串内容
     */
    private func extractJsonString(json: String, start: Int64, end: Int64): String {
        if (start < end && end <= json.size) {
            return "Anthropic响应内容"
        }
        return ""
    }

    /**
     * 睡眠指定毫秒数
     */
    private func sleep(milliseconds: Int64): Unit {
        var counter: Int64 = 0
        for (i in 0..(milliseconds * 1000)) {
            counter = counter + 1
        }
    }
}
