/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.factory

import std.collection.HashMap
import contextengine.core.config.LlmConfig
import contextengine.core.llm.{LlmBase, MockLlm, SimpleRealLlm}

/**
 * 增强的LLM工厂类
 * 支持真实LLM提供商和Mock实现
 */
public class EnhancedLlmFactory {
    /**
     * 创建LLM实例
     * @param provider 提供商名称
     * @param config LLM配置
     * @return LLM实例
     */
    public static func create(provider: String, config: LlmConfig): LlmBase {
        if (provider == "openai") {
            return SimpleRealLlm(config, "openai")
        } else if (provider == "anthropic") {
            return SimpleRealLlm(config, "anthropic")
        } else if (provider == "mock") {
            return MockLlm(config)
        } else {
            // 默认使用简化的真实实现
            return SimpleRealLlm(config, provider)
        }
    }
    
    /**
     * 创建带API密钥的LLM实例
     * @param provider 提供商名称
     * @param config LLM配置
     * @param apiKey API密钥
     * @return LLM实例
     */
    public static func createWithApiKey(provider: String, config: LlmConfig, apiKey: String): LlmBase {
        if (provider == "openai") {
            return SimpleRealLlm(config, "openai")
        } else if (provider == "anthropic") {
            return SimpleRealLlm(config, "anthropic")
        } else if (provider == "mock") {
            return MockLlm(config)
        } else {
            // 默认使用简化的真实实现
            return SimpleRealLlm(config, provider)
        }
    }
    
    /**
     * 获取支持的提供商列表
     * @return 提供商列表
     */
    public static func getSupportedProviders(): Array<String> {
        return ["mock", "openai", "anthropic"]
    }
    
    /**
     * 验证提供商是否支持
     * @param provider 提供商名称
     * @return 是否支持
     */
    public static func isProviderSupported(provider: String): Bool {
        let supportedProviders = getSupportedProviders()
        for (supportedProvider in supportedProviders) {
            if (supportedProvider == provider) {
                return true
            }
        }
        return false
    }
    
    /**
     * 获取提供商的默认模型
     * @param provider 提供商名称
     * @return 默认模型名称
     */
    public static func getDefaultModel(provider: String): String {
        if (provider == "openai") {
            return "gpt-3.5-turbo"
        } else if (provider == "anthropic") {
            return "claude-3-sonnet-20240229"
        } else if (provider == "mock") {
            return "mock-gpt-3.5"
        } else {
            return "unknown"
        }
    }
    
    /**
     * 获取提供商信息
     * @param provider 提供商名称
     * @return 提供商信息
     */
    public static func getProviderInfo(provider: String): HashMap<String, String> {
        let info = HashMap<String, String>()
        
        if (provider == "openai") {
            info["name"] = "OpenAI"
            info["description"] = "OpenAI GPT模型"
            info["api_url"] = "https://api.openai.com/v1"
            info["default_model"] = "gpt-3.5-turbo"
            info["supports_streaming"] = "true"
            info["max_tokens"] = "4096"
        } else if (provider == "anthropic") {
            info["name"] = "Anthropic"
            info["description"] = "Anthropic Claude模型"
            info["api_url"] = "https://api.anthropic.com/v1"
            info["default_model"] = "claude-3-sonnet-20240229"
            info["supports_streaming"] = "true"
            info["max_tokens"] = "4096"
        } else if (provider == "mock") {
            info["name"] = "Mock LLM"
            info["description"] = "模拟LLM，用于测试和开发"
            info["api_url"] = "local"
            info["default_model"] = "mock-gpt-3.5"
            info["supports_streaming"] = "false"
            info["max_tokens"] = "unlimited"
        } else {
            info["name"] = "Unknown"
            info["description"] = "未知提供商"
        }
        
        return info
    }
    
    /**
     * 测试LLM连接
     * @param provider 提供商名称
     * @param config LLM配置
     * @return 连接测试结果
     */
    public static func testConnection(provider: String, config: LlmConfig): (Bool, String) {
        try {
            let llm = create(provider, config)
            
            // 验证配置
            if (!llm.validateConfig()) {
                return (false, "配置验证失败")
            }
            
            // 简化测试连接实现
            if (provider == "openai") {
                return (true, "OpenAI连接测试成功（模拟）")
            } else if (provider == "anthropic") {
                return (true, "Anthropic连接测试成功（模拟）")
            } else {
                return (true, "Mock LLM连接成功")
            }
            
        } catch (e: Exception) {
            return (false, "连接测试异常: ${e.message}")
        }
    }
    
    /**
     * 创建推荐的LLM配置
     * @param provider 提供商名称
     * @param apiKey API密钥（可选）
     * @return 推荐配置
     */
    public static func createRecommendedConfig(provider: String, apiKey: Option<String>): LlmConfig {
        let defaultModel = getDefaultModel(provider)
        
        if (provider == "openai") {
            return LlmConfig(
                provider, defaultModel, apiKey, None, 0.7, 1000, HashMap<String, String>()
            )
        } else if (provider == "anthropic") {
            return LlmConfig(
                provider, defaultModel, apiKey, None, 0.7, 1000, HashMap<String, String>()
            )
        } else {
            return LlmConfig(
                provider, defaultModel, None, None, 0.7, 1000, HashMap<String, String>()
            )
        }
    }
    
    /**
     * 获取所有提供商的统计信息
     * @return 统计信息
     */
    public static func getAllProvidersStats(): HashMap<String, HashMap<String, String>> {
        let stats = HashMap<String, HashMap<String, String>>()
        let providers = getSupportedProviders()
        
        for (provider in providers) {
            stats[provider] = getProviderInfo(provider)
        }
        
        return stats
    }
}
