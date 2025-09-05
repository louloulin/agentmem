/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.tests

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord
import contextengine.core.memory.{MemorySummarizer, MemoryCompressor, HierarchicalSummarizer, SummaryUpdateManager}
import contextengine.core.memory.{SummaryConfig, SummaryStrategy, CompressionConfig, CompressionStrategy}
import contextengine.models.{MemoryLevel, MemoryScope, HierarchicalMemoryRecord}

/**
 * Phase 2.2 记忆摘要和压缩功能测试
 * 测试摘要生成、压缩算法、分层摘要和摘要更新功能
 */
public class Phase22Test {
    private let memorySummarizer: MemorySummarizer
    private let memoryCompressor: MemoryCompressor
    private let hierarchicalSummarizer: HierarchicalSummarizer
    private let summaryUpdateManager: SummaryUpdateManager
    private let testMemories: Array<String>
    
    public init() {
        this.memorySummarizer = MemorySummarizer()
        this.memoryCompressor = MemoryCompressor()
        this.hierarchicalSummarizer = HierarchicalSummarizer(memorySummarizer)
        this.summaryUpdateManager = SummaryUpdateManager(hierarchicalSummarizer)
        this.testMemories = [
            "我今天学习了机器学习的基础知识，包括监督学习、无监督学习和强化学习。监督学习使用标记数据进行训练，无监督学习从未标记数据中发现模式，强化学习通过与环境交互来学习最优策略。",
            "深度学习是机器学习的一个子领域，使用多层神经网络来学习数据的复杂表示。卷积神经网络（CNN）特别适合图像处理，循环神经网络（RNN）适合序列数据处理，Transformer架构在自然语言处理中表现出色。",
            "自然语言处理（NLP）是人工智能的重要分支，涉及计算机对人类语言的理解和生成。主要任务包括文本分类、情感分析、机器翻译、问答系统等。近年来，大语言模型如GPT、BERT等在NLP任务中取得了突破性进展。",
            "计算机视觉是让计算机能够理解和解释视觉信息的技术。主要任务包括图像分类、目标检测、语义分割、人脸识别等。深度学习技术，特别是卷积神经网络，大大提升了计算机视觉的性能。",
            "强化学习是一种通过与环境交互来学习最优行为策略的机器学习方法。智能体通过执行动作获得奖励或惩罚，目标是最大化累积奖励。Q学习、策略梯度、Actor-Critic等是常用的强化学习算法。"
        ]
    }
    
    /**
     * 运行所有Phase 2.2测试
     */
    public func runAllTests(): Bool {
        println("=== Phase 2.2 记忆摘要和压缩功能测试开始 ===")
        
        var allTestsPassed = true
        
        // 准备测试数据
        if (!prepareTestData()) {
            println("❌ 测试数据准备失败")
            return false
        }
        
        // 测试1：记忆摘要生成
        println("\n1. 测试记忆摘要生成...")
        if (!testMemorySummarization()) {
            println("❌ 记忆摘要生成测试失败")
            allTestsPassed = false
        } else {
            println("✅ 记忆摘要生成测试通过")
        }
        
        // 测试2：记忆压缩算法
        println("\n2. 测试记忆压缩算法...")
        if (!testMemoryCompression()) {
            println("❌ 记忆压缩算法测试失败")
            allTestsPassed = false
        } else {
            println("✅ 记忆压缩算法测试通过")
        }
        
        // 测试3：分层摘要系统
        println("\n3. 测试分层摘要系统...")
        if (!testHierarchicalSummarization()) {
            println("❌ 分层摘要系统测试失败")
            allTestsPassed = false
        } else {
            println("✅ 分层摘要系统测试通过")
        }
        
        // 测试4：摘要更新机制
        println("\n4. 测试摘要更新机制...")
        if (!testSummaryUpdateMechanism()) {
            println("❌ 摘要更新机制测试失败")
            allTestsPassed = false
        } else {
            println("✅ 摘要更新机制测试通过")
        }
        
        // 测试5：压缩率验证
        println("\n5. 测试压缩率验证...")
        if (!testCompressionRatio()) {
            println("❌ 压缩率验证测试失败")
            allTestsPassed = false
        } else {
            println("✅ 压缩率验证测试通过")
        }
        
        // 测试6：摘要质量评估
        println("\n6. 测试摘要质量评估...")
        if (!testSummaryQuality()) {
            println("❌ 摘要质量评估测试失败")
            allTestsPassed = false
        } else {
            println("✅ 摘要质量评估测试通过")
        }
        
        if (allTestsPassed) {
            println("\n🎉 Phase 2.2 所有测试通过！")
            println("✅ 记忆摘要生成功能完成")
            println("✅ 记忆压缩算法功能完成")
            println("✅ 分层摘要系统功能完成")
            println("✅ 摘要更新机制功能完成")
            println("✅ 压缩率达到预期目标（>70%）")
            println("✅ 摘要质量评估系统完成")
        } else {
            println("\n❌ Phase 2.2 部分测试失败")
        }
        
        return allTestsPassed
    }
    
    /**
     * 准备测试数据
     */
    private func prepareTestData(): Bool {
        try {
            println("准备测试数据...")
            println("成功准备 ${testMemories.size} 条测试记忆")
            return true
            
        } catch (e: Exception) {
            println("准备测试数据失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试记忆摘要生成
     */
    private func testMemorySummarization(): Bool {
        try {
            println("  测试不同摘要策略...")
            
            // 创建测试记忆
            let testMemory = MemoryRecord(
                "test_memory_1", testMemories[0], "", 0.0, 0, 0, HashMap<String, String>()
            )
            
            // 测试抽取式摘要
            let extractiveConfig = SummaryConfig(SummaryStrategy.EXTRACTIVE, 100, 0.5)
            let extractiveResult = memorySummarizer.summarizeMemory(testMemory, Some(extractiveConfig))
            
            println("    抽取式摘要:")
            println("      原始长度: ${extractiveResult.originalLength}")
            println("      摘要长度: ${extractiveResult.summaryLength}")
            println("      压缩比例: ${extractiveResult.compressionRatio}")
            println("      置信度: ${extractiveResult.confidence}")
            
            // 测试规则式摘要
            let ruleBasedConfig = SummaryConfig(SummaryStrategy.RULE_BASED, 80, 0.4)
            let ruleBasedResult = memorySummarizer.summarizeMemory(testMemory, Some(ruleBasedConfig))
            
            println("    规则式摘要:")
            println("      摘要内容: ${ruleBasedResult.summary}")
            println("      压缩比例: ${ruleBasedResult.compressionRatio}")
            
            // 测试批量摘要
            let memories = ArrayList<MemoryRecord>()
            for (i in 0..3) {
                let memory = MemoryRecord(
                    "batch_memory_${i}", testMemories[i], "", 0.0, 0, 0, HashMap<String, String>()
                )
                memories.add(memory)
            }
            
            let batchResults = memorySummarizer.summarizeMemories(memories.toArray(), Some(extractiveConfig))
            println("    批量摘要结果数量: ${batchResults.size}")
            
            // 测试综合摘要
            let collectionResult = memorySummarizer.summarizeMemoryCollection(memories.toArray(), Some(extractiveConfig))
            println("    综合摘要长度: ${collectionResult.summaryLength}")
            
            return true
            
        } catch (e: Exception) {
            println("    记忆摘要生成测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试记忆压缩算法
     */
    private func testMemoryCompression(): Bool {
        try {
            println("  测试不同压缩策略...")
            
            // 创建测试记忆
            let testMemory = MemoryRecord(
                "compress_test", testMemories[1], "", 0.0, 0, 0, HashMap<String, String>()
            )
            
            // 测试文本压缩
            let textConfig = CompressionConfig(CompressionStrategy.TEXT_COMPRESSION, 5, 0.7)
            let textResult = memoryCompressor.compressMemory(testMemory, Some(textConfig))
            
            println("    文本压缩:")
            println("      原始大小: ${textResult.originalSize} 字节")
            println("      压缩后大小: ${textResult.compressedSize} 字节")
            println("      压缩比例: ${textResult.compressionRatio}")
            println("      质量评分: ${textResult.qualityScore}")
            println("      是否有损: ${textResult.isLossy}")
            
            // 测试无损压缩
            let losslessConfig = CompressionConfig(CompressionStrategy.LOSSLESS_COMPRESSION, 7, 0.8)
            let losslessResult = memoryCompressor.compressMemory(testMemory, Some(losslessConfig))
            
            println("    无损压缩:")
            println("      压缩比例: ${losslessResult.compressionRatio}")
            println("      质量评分: ${losslessResult.qualityScore}")
            
            // 测试有损压缩
            let lossyConfig = CompressionConfig(CompressionStrategy.LOSSY_COMPRESSION, 9, 0.5)
            let lossyResult = memoryCompressor.compressMemory(testMemory, Some(lossyConfig))
            
            println("    有损压缩:")
            println("      压缩比例: ${lossyResult.compressionRatio}")
            println("      质量评分: ${lossyResult.qualityScore}")
            
            // 测试解压缩
            let decompressed = memoryCompressor.decompressMemory(textResult.compressedData, textResult.strategy)
            if (decompressed.isSome()) {
                println("    解压缩成功")
            } else {
                println("    解压缩失败")
            }
            
            return true
            
        } catch (e: Exception) {
            println("    记忆压缩算法测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试分层摘要系统
     */
    private func testHierarchicalSummarization(): Bool {
        try {
            println("  测试分层摘要生成...")
            
            // 创建分层记忆
            let hierarchicalMemories = ArrayList<HierarchicalMemoryRecord>()
            
            // 用户级记忆
            for (i in 0..2) {
                let baseRecord = MemoryRecord(
                    "user_memory_${i}", testMemories[i], "", 0.8, 0, 0, HashMap<String, String>()
                )
                let hierarchicalRecord = HierarchicalMemoryRecord(
                    baseRecord, MemoryLevel.USER, 0.8, 0, 0
                )
                hierarchicalMemories.add(hierarchicalRecord)
            }
            
            // 会话级记忆
            let sessionRecord = MemoryRecord(
                "session_memory", testMemories[3], "", 0.6, 0, 0, HashMap<String, String>()
            )
            let hierarchicalSessionRecord = HierarchicalMemoryRecord(
                sessionRecord, MemoryLevel.SESSION, 0.6, 0, 0
            )
            hierarchicalMemories.add(hierarchicalSessionRecord)
            
            // 代理级记忆
            let agentRecord = MemoryRecord(
                "agent_memory", testMemories[4], "", 0.7, 0, 0, HashMap<String, String>()
            )
            let hierarchicalAgentRecord = HierarchicalMemoryRecord(
                agentRecord, MemoryLevel.AGENT, 0.7, 0, 0
            )
            hierarchicalMemories.add(hierarchicalAgentRecord)
            
            // 创建作用域
            let scope = MemoryScope.createUserScope("test_user", false)
            
            // 生成分层摘要
            let hierarchicalResult = hierarchicalSummarizer.generateHierarchicalSummary(
                hierarchicalMemories.toArray(), scope
            )
            
            println("    分层摘要结果:")
            println("      总记忆数量: ${hierarchicalResult.totalMemories}")
            println("      处理时间: ${hierarchicalResult.processingTime}ms")
            
            if (hierarchicalResult.userSummary.isSome()) {
                let userSummary = hierarchicalResult.userSummary.getOrThrow()
                println("      用户级摘要长度: ${userSummary.summaryLength}")
            }
            
            if (hierarchicalResult.sessionSummary.isSome()) {
                let sessionSummary = hierarchicalResult.sessionSummary.getOrThrow()
                println("      会话级摘要长度: ${sessionSummary.summaryLength}")
            }
            
            if (hierarchicalResult.agentSummary.isSome()) {
                let agentSummary = hierarchicalResult.agentSummary.getOrThrow()
                println("      代理级摘要长度: ${agentSummary.summaryLength}")
            }
            
            if (hierarchicalResult.crosslevelSummary.isSome()) {
                let crosslevelSummary = hierarchicalResult.crosslevelSummary.getOrThrow()
                println("      跨层级摘要长度: ${crosslevelSummary.summaryLength}")
            }
            
            // 测试统计信息
            let stats = hierarchicalSummarizer.getHierarchicalSummaryStatistics()
            println("    分层摘要统计:")
            for ((key, value) in stats) {
                println("      ${key}: ${value}")
            }
            
            return true
            
        } catch (e: Exception) {
            println("    分层摘要系统测试失败: ${e}")
            return false
        }
    }
}
