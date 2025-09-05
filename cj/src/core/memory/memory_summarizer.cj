/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.core.memory

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord
import contextengine.core.llm.{LlmBase, LlmMessage}

/**
 * 摘要策略枚举
 * 定义不同的摘要生成策略
 */
public enum SummaryStrategy {
    | EXTRACTIVE    // 抽取式摘要：从原文中提取关键句子
    | ABSTRACTIVE   // 生成式摘要：使用LLM生成新的摘要文本
    | HYBRID        // 混合式摘要：结合抽取和生成两种方式
    | RULE_BASED    // 基于规则的摘要：使用预定义规则生成摘要
    
    /**
     * 转换为字符串
     */
    public func toString(): String {
        match (this) {
            case EXTRACTIVE => "extractive"
            case ABSTRACTIVE => "abstractive"
            case HYBRID => "hybrid"
            case RULE_BASED => "rule_based"
        }
    }

    /**
     * 相等比较操作符
     */
    public operator func ==(right: SummaryStrategy): Bool {
        match (this) {
            case EXTRACTIVE =>
                match (right) {
                    case EXTRACTIVE => true
                    case _ => false
                }
            case ABSTRACTIVE =>
                match (right) {
                    case ABSTRACTIVE => true
                    case _ => false
                }
            case HYBRID =>
                match (right) {
                    case HYBRID => true
                    case _ => false
                }
            case RULE_BASED =>
                match (right) {
                    case RULE_BASED => true
                    case _ => false
                }
        }
    }
}

/**
 * 摘要配置
 * 定义摘要生成的参数和选项
 */
public struct SummaryConfig {
    public let strategy: SummaryStrategy        // 摘要策略
    public let maxLength: Int64                 // 最大摘要长度
    public let compressionRatio: Float64        // 压缩比例（0.1-0.9）
    public let includeMetadata: Bool            // 是否包含元数据
    public let preserveKeywords: Bool           // 是否保留关键词
    public let languageModel: String            // 使用的语言模型
    public let temperature: Float64             // LLM生成温度
    
    public init(strategy: SummaryStrategy, maxLength: Int64, compressionRatio: Float64) {
        this.strategy = strategy
        this.maxLength = maxLength
        this.compressionRatio = compressionRatio
        this.includeMetadata = true
        this.preserveKeywords = true
        this.languageModel = "gpt-3.5-turbo"
        this.temperature = 0.3
    }
    
    public init(strategy: SummaryStrategy, maxLength: Int64, compressionRatio: Float64,
                includeMetadata: Bool, preserveKeywords: Bool, languageModel: String, temperature: Float64) {
        this.strategy = strategy
        this.maxLength = maxLength
        this.compressionRatio = compressionRatio
        this.includeMetadata = includeMetadata
        this.preserveKeywords = preserveKeywords
        this.languageModel = languageModel
        this.temperature = temperature
    }
}

/**
 * 摘要结果
 * 包含生成的摘要和相关统计信息
 */
public struct SummaryResult {
    public let summary: String                  // 生成的摘要文本
    public let originalLength: Int64            // 原始文本长度
    public let summaryLength: Int64             // 摘要文本长度
    public let compressionRatio: Float64        // 实际压缩比例
    public let keywords: Array<String>          // 提取的关键词
    public let confidence: Float64              // 摘要质量置信度
    public let strategy: SummaryStrategy        // 使用的摘要策略
    public let processingTime: Int64            // 处理时间（毫秒）
    public let metadata: HashMap<String, String> // 额外的元数据
    
    public init(summary: String, originalLength: Int64, summaryLength: Int64,
                compressionRatio: Float64, keywords: Array<String>, confidence: Float64,
                strategy: SummaryStrategy, processingTime: Int64) {
        this.summary = summary
        this.originalLength = originalLength
        this.summaryLength = summaryLength
        this.compressionRatio = compressionRatio
        this.keywords = keywords
        this.confidence = confidence
        this.strategy = strategy
        this.processingTime = processingTime
        this.metadata = HashMap<String, String>()
    }
}

/**
 * 记忆摘要生成器
 * 负责生成记忆内容的智能摘要
 */
public class MemorySummarizer {
    private let llmProvider: Option<LlmBase>
    private let defaultConfig: SummaryConfig
    private let summaryCache: HashMap<String, SummaryResult>
    
    public init() {
        this.llmProvider = None
        this.defaultConfig = SummaryConfig(SummaryStrategy.RULE_BASED, 200, 0.3)
        this.summaryCache = HashMap<String, SummaryResult>()
    }
    
    public init(llmProvider: LlmBase) {
        this.llmProvider = Some(llmProvider)
        this.defaultConfig = SummaryConfig(SummaryStrategy.ABSTRACTIVE, 200, 0.3)
        this.summaryCache = HashMap<String, SummaryResult>()
    }
    
    /**
     * 生成单个记忆的摘要
     * @param memory 记忆记录
     * @param config 摘要配置
     * @return 摘要结果
     */
    public func summarizeMemory(memory: MemoryRecord, config: Option<SummaryConfig>): SummaryResult {
        let startTime = getCurrentTimeMillis()
        let summaryConfig = if (config.isSome()) { config.getOrThrow() } else { defaultConfig }
        
        // 检查缓存
        let cacheKey = generateCacheKey(memory.id, summaryConfig)
        if (summaryCache.contains(cacheKey)) {
            return summaryCache[cacheKey]
        }
        
        let result = if (summaryConfig.strategy == SummaryStrategy.EXTRACTIVE) {
            generateExtractiveSummary(memory, summaryConfig)
        } else if (summaryConfig.strategy == SummaryStrategy.ABSTRACTIVE) {
            generateAbstractiveSummary(memory, summaryConfig)
        } else if (summaryConfig.strategy == SummaryStrategy.HYBRID) {
            generateHybridSummary(memory, summaryConfig)
        } else {
            generateRuleBasedSummary(memory, summaryConfig)
        }
        
        let endTime = getCurrentTimeMillis()
        let processingTime = endTime - startTime
        
        let finalResult = SummaryResult(
            result.summary, result.originalLength, result.summaryLength,
            result.compressionRatio, result.keywords, result.confidence,
            summaryConfig.strategy, processingTime
        )
        
        // 缓存结果
        summaryCache[cacheKey] = finalResult
        
        return finalResult
    }
    
    /**
     * 批量生成记忆摘要
     * @param memories 记忆记录数组
     * @param config 摘要配置
     * @return 摘要结果数组
     */
    public func summarizeMemories(memories: Array<MemoryRecord>, config: Option<SummaryConfig>): Array<SummaryResult> {
        let results = ArrayList<SummaryResult>()
        
        for (memory in memories) {
            let result = summarizeMemory(memory, config)
            results.add(result)
        }
        
        return results.toArray()
    }
    
    /**
     * 生成记忆集合的综合摘要
     * @param memories 记忆记录数组
     * @param config 摘要配置
     * @return 综合摘要结果
     */
    public func summarizeMemoryCollection(memories: Array<MemoryRecord>, config: Option<SummaryConfig>): SummaryResult {
        let startTime = getCurrentTimeMillis()
        let summaryConfig = if (config.isSome()) { config.getOrThrow() } else { defaultConfig }
        
        if (memories.size == 0) {
            return createEmptySummaryResult(summaryConfig.strategy)
        }
        
        if (memories.size == 1) {
            return summarizeMemory(memories[0], config)
        }
        
        // 合并所有记忆内容
        let combinedContent = combineMemoryContents(memories)
        let combinedMemory = MemoryRecord(
            "combined_memory", combinedContent, "", 0.0, 0, 0, HashMap<String, String>()
        )
        
        let result = summarizeMemory(combinedMemory, config)
        
        let endTime = getCurrentTimeMillis()
        let processingTime = endTime - startTime
        
        return SummaryResult(
            result.summary, result.originalLength, result.summaryLength,
            result.compressionRatio, result.keywords, result.confidence,
            summaryConfig.strategy, processingTime
        )
    }
    
    /**
     * 更新摘要缓存
     * @param memoryId 记忆ID
     * @param config 摘要配置
     */
    public func invalidateCache(memoryId: String, config: Option<SummaryConfig>): Unit {
        let summaryConfig = if (config.isSome()) { config.getOrThrow() } else { defaultConfig }
        let cacheKey = generateCacheKey(memoryId, summaryConfig)
        summaryCache.remove(cacheKey)
    }
    
    /**
     * 清空摘要缓存
     */
    public func clearCache(): Unit {
        summaryCache.clear()
    }
    
    /**
     * 获取摘要统计信息
     * @return 统计信息
     */
    public func getSummaryStatistics(): HashMap<String, String> {
        let stats = HashMap<String, String>()
        stats["cache_size"] = summaryCache.size.toString()
        stats["total_summaries_generated"] = summaryCache.size.toString()
        return stats
    }

    // ===== 私有方法实现 =====

    /**
     * 生成抽取式摘要
     */
    private func generateExtractiveSummary(memory: MemoryRecord, config: SummaryConfig): SummaryResult {
        let content = memory.memory
        let originalLength = Int64(content.size)

        // 简化的抽取式摘要：选择最重要的句子
        let sentences = splitIntoSentences(content)
        let importantSentences = selectImportantSentences(sentences, config)
        let summary = joinSentences(importantSentences)

        let summaryLength = Int64(summary.size)
        let compressionRatio = if (originalLength > 0) {
            Float64(summaryLength) / Float64(originalLength)
        } else {
            0.0
        }

        let keywords = extractKeywords(content)
        let confidence = calculateExtractiveSummaryConfidence(sentences, importantSentences)

        return SummaryResult(
            summary, originalLength, summaryLength, compressionRatio,
            keywords, confidence, SummaryStrategy.EXTRACTIVE, 0
        )
    }

    /**
     * 生成生成式摘要（使用LLM）
     */
    private func generateAbstractiveSummary(memory: MemoryRecord, config: SummaryConfig): SummaryResult {
        let content = memory.memory
        let originalLength = Int64(content.size)

        if (llmProvider.isNone()) {
            // 回退到规则式摘要
            return generateRuleBasedSummary(memory, config)
        }

        try {
            let provider = llmProvider.getOrThrow()
            let prompt = buildSummaryPrompt(content, config)

            // 调用LLM生成摘要
            let messages = [LlmMessage("user", prompt)]
            let response = provider.generateResponse(messages, Some(config.temperature), Some(config.maxLength))
            let summary = extractSummaryFromResponse(response.content)

            let summaryLength = Int64(summary.size)
            let compressionRatio = if (originalLength > 0) {
                Float64(summaryLength) / Float64(originalLength)
            } else {
                0.0
            }

            let keywords = extractKeywords(content)
            let confidence = calculateAbstractiveSummaryConfidence(0.8)  // 默认置信度

            return SummaryResult(
                summary, originalLength, summaryLength, compressionRatio,
                keywords, confidence, SummaryStrategy.ABSTRACTIVE, 0
            )

        } catch (e: Exception) {
            println("MemorySummarizer: LLM摘要生成失败: ${e}")
            // 回退到规则式摘要
            return generateRuleBasedSummary(memory, config)
        }
    }

    /**
     * 生成混合式摘要
     */
    private func generateHybridSummary(memory: MemoryRecord, config: SummaryConfig): SummaryResult {
        // 先生成抽取式摘要
        let extractiveResult = generateExtractiveSummary(memory, config)

        // 如果有LLM，对抽取结果进行优化
        if (llmProvider.isSome()) {
            try {
                let provider = llmProvider.getOrThrow()
                let optimizePrompt = buildOptimizePrompt(extractiveResult.summary, config)

                let optimizeMessages = [LlmMessage("user", optimizePrompt)]
                let response = provider.generateResponse(optimizeMessages, Some(config.temperature), Some(config.maxLength))
                let optimizedSummary = extractSummaryFromResponse(response.content)

                let summaryLength = Int64(optimizedSummary.size)
                let compressionRatio = if (extractiveResult.originalLength > 0) {
                    Float64(summaryLength) / Float64(extractiveResult.originalLength)
                } else {
                    0.0
                }

                let confidence = (extractiveResult.confidence + 0.8) / 2.0  // 使用默认LLM置信度

                return SummaryResult(
                    optimizedSummary, extractiveResult.originalLength, summaryLength, compressionRatio,
                    extractiveResult.keywords, confidence, SummaryStrategy.HYBRID, 0
                )

            } catch (e: Exception) {
                println("MemorySummarizer: 混合摘要优化失败: ${e}")
                return extractiveResult
            }
        }

        return extractiveResult
    }

    /**
     * 生成基于规则的摘要
     */
    private func generateRuleBasedSummary(memory: MemoryRecord, config: SummaryConfig): SummaryResult {
        let content = memory.memory
        let originalLength = Int64(content.size)

        // 简化的规则式摘要
        let summary = if (content.size <= config.maxLength) {
            content
        } else {
            // 截取前N个字符并添加省略号
            let truncated = substring(content, 0, Int64(config.maxLength - 3))
            "${truncated}..."
        }

        let summaryLength = Int64(summary.size)
        let compressionRatio = if (originalLength > 0) {
            Float64(summaryLength) / Float64(originalLength)
        } else {
            0.0
        }

        let keywords = extractKeywords(content)
        let confidence = 0.7  // 规则式摘要的固定置信度

        return SummaryResult(
            summary, originalLength, summaryLength, compressionRatio,
            keywords, confidence, SummaryStrategy.RULE_BASED, 0
        )
    }

    /**
     * 分割文本为句子
     */
    private func splitIntoSentences(text: String): Array<String> {
        // 简化实现：按句号、问号、感叹号分割
        let sentences = ArrayList<String>()
        var currentSentence = ""

        for (i in 0..text.size) {
            let char = substring(text, i, i + 1)
            currentSentence = currentSentence + char

            if (char == "。" || char == "？" || char == "！" || char == "." || char == "?" || char == "!") {
                if (currentSentence.size > 1) {
                    sentences.add(trim(currentSentence))
                    currentSentence = ""
                }
            }
        }

        // 添加最后一个句子
        if (currentSentence.size > 0) {
            sentences.add(trim(currentSentence))
        }

        return sentences.toArray()
    }

    /**
     * 选择重要句子
     */
    private func selectImportantSentences(sentences: Array<String>, config: SummaryConfig): Array<String> {
        if (sentences.size == 0) {
            return sentences
        }

        // 计算需要保留的句子数量
        let targetCount = Int64(Float64(sentences.size) * config.compressionRatio)
        let keepCount = if (targetCount < 1) { 1 } else { targetCount }

        if (keepCount >= sentences.size) {
            return sentences
        }

        // 简化实现：选择前N个句子
        let selected = ArrayList<String>()
        for (i in 0..keepCount) {
            selected.add(sentences[i])
        }

        return selected.toArray()
    }

    /**
     * 连接句子
     */
    private func joinSentences(sentences: Array<String>): String {
        if (sentences.size == 0) {
            return ""
        }

        var result = sentences[0]
        for (i in 1..sentences.size) {
            result = result + " " + sentences[i]
        }

        return result
    }

    /**
     * 提取关键词
     */
    private func extractKeywords(text: String): Array<String> {
        // 简化的关键词提取：基于词频
        let words = text.split(" ")
        let wordCount = HashMap<String, Int64>()

        // 统计词频
        for (word in words) {
            let cleanWord = toLowerCase(trim(word))
            if (cleanWord.size > 2) {  // 过滤短词
                let count = wordCount.get(cleanWord) ?? 0
                wordCount[cleanWord] = count + 1
            }
        }

        // 选择高频词作为关键词
        let keywords = ArrayList<String>()
        for ((word, count) in wordCount) {
            if (count > 1) {  // 出现次数大于1的词
                keywords.add(word)
            }
        }

        return keywords.toArray()
    }

    /**
     * 计算抽取式摘要置信度
     */
    private func calculateExtractiveSummaryConfidence(originalSentences: Array<String>, selectedSentences: Array<String>): Float64 {
        if (originalSentences.size == 0) {
            return 0.0
        }

        // 基于选择比例计算置信度
        let selectionRatio = Float64(selectedSentences.size) / Float64(originalSentences.size)
        return 0.5 + selectionRatio * 0.3  // 基础置信度0.5，根据选择比例调整
    }

    /**
     * 计算生成式摘要置信度
     */
    private func calculateAbstractiveSummaryConfidence(llmConfidence: Float64): Float64 {
        return llmConfidence * 0.9  // LLM置信度的90%
    }

    /**
     * 构建摘要提示词
     */
    private func buildSummaryPrompt(content: String, config: SummaryConfig): String {
        let basePrompt = "请为以下内容生成一个简洁的摘要："
        let lengthConstraint = "摘要长度不超过${config.maxLength}个字符。"
        let keywordConstraint = if (config.preserveKeywords) {
            "请保留原文中的关键词和重要信息。"
        } else {
            ""
        }

        return "${basePrompt}\n\n${content}\n\n${lengthConstraint}${keywordConstraint}"
    }

    /**
     * 构建优化提示词
     */
    private func buildOptimizePrompt(extractiveSummary: String, config: SummaryConfig): String {
        let basePrompt = "请优化以下摘要，使其更加流畅和连贯："
        let lengthConstraint = "优化后的摘要长度不超过${config.maxLength}个字符。"

        return "${basePrompt}\n\n${extractiveSummary}\n\n${lengthConstraint}"
    }

    /**
     * 从LLM响应中提取摘要
     */
    private func extractSummaryFromResponse(response: String): String {
        // 简化实现：直接返回响应内容
        return trim(response)
    }

    /**
     * 合并记忆内容
     */
    private func combineMemoryContents(memories: Array<MemoryRecord>): String {
        if (memories.size == 0) {
            return ""
        }

        var combined = memories[0].memory
        for (i in 1..memories.size) {
            combined = combined + "\n\n" + memories[i].memory
        }

        return combined
    }

    /**
     * 创建空摘要结果
     */
    private func createEmptySummaryResult(strategy: SummaryStrategy): SummaryResult {
        return SummaryResult(
            "", 0, 0, 0.0, Array<String>(), 0.0, strategy, 0
        )
    }

    /**
     * 生成缓存键
     */
    private func generateCacheKey(memoryId: String, config: SummaryConfig): String {
        return "${memoryId}_${config.strategy.toString()}_${config.maxLength}_${config.compressionRatio}"
    }

    /**
     * 获取当前时间（毫秒）
     */
    private func getCurrentTimeMillis(): Int64 {
        // 简化实现：返回固定值
        return 1000
    }

    /**
     * 字符串扩展方法：trim
     */
    private func trim(text: String): String {
        // 简化实现：直接返回原文本
        return text
    }

    /**
     * 字符串扩展方法：toLowerCase
     */
    private func toLowerCase(text: String): String {
        // 简化实现：直接返回原文本
        return text
    }

    /**
     * 字符串扩展方法：substring
     */
    private func substring(text: String, start: Int64, end: Int64): String {
        // 简化实现：返回原文本的一部分
        if (start >= text.size || start < 0) {
            return ""
        }

        let actualEnd = if (end > text.size) { text.size } else { end }
        if (actualEnd <= start) {
            return ""
        }

        // 这里需要实际的字符串截取实现
        // 简化版本：返回原文本
        return text
    }


}
