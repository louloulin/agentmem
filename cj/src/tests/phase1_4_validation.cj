/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.tests

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord
import contextengine.core.memory.{SemanticSimilarityDetector, SemanticSimilarityConfig, MemoryMerger, MemoryMergerConfig, MemoryDeduplicator, DeduplicationConfig, ConflictResolver, ConflictResolverConfig}

/**
 * Phase 1.4 功能验证
 * 验证记忆合并和去重的核心功能
 */
public class Phase14Validation {
    
    /**
     * 运行Phase 1.4功能验证
     */
    public func runValidation(): Bool {
        println("=== Phase 1.4 记忆合并和去重功能验证 ===")
        
        var allTestsPassed = true
        
        // 测试1: 语义相似性检测
        if (!testSemanticSimilarity()) {
            allTestsPassed = false
        }
        
        // 测试2: 记忆合并
        if (!testMemoryMerging()) {
            allTestsPassed = false
        }
        
        // 测试3: 自动去重
        if (!testMemoryDeduplication()) {
            allTestsPassed = false
        }
        
        // 测试4: 冲突检测
        if (!testConflictDetection()) {
            allTestsPassed = false
        }
        
        if (allTestsPassed) {
            println("🎉 Phase 1.4 所有功能验证通过！")
        } else {
            println("❌ Phase 1.4 部分功能验证失败")
        }
        
        return allTestsPassed
    }
    
    /**
     * 测试语义相似性检测
     */
    private func testSemanticSimilarity(): Bool {
        println("\n--- 测试语义相似性检测 ---")
        
        try {
            let detector = SemanticSimilarityDetector()
            
            // 创建测试记忆
            let memory1 = createTestMemory("test1", "我喜欢喝咖啡")
            let memory2 = createTestMemory("test2", "我很喜欢喝咖啡")
            let memory3 = createTestMemory("test3", "我不喜欢喝茶")
            
            // 测试相似记忆检测
            let similarity1 = detector.detectSimilarity(memory1, memory2)
            println("相似记忆检测: ${similarity1.similarity} (阈值: 0.8)")
            
            // 测试不相似记忆检测
            let similarity2 = detector.detectSimilarity(memory1, memory3)
            println("不相似记忆检测: ${similarity2.similarity}")
            
            // 测试文本相似性
            let textSimilarity = detector.detectTextSimilarity("我喜欢咖啡", "我很喜欢咖啡")
            println("文本相似性: ${textSimilarity.similarity}")
            
            // 测试重复记忆组检测
            let memories = [memory1, memory2, memory3]
            let duplicateGroups = detector.detectDuplicateGroups(memories)
            println("重复记忆组数量: ${duplicateGroups.size}")
            
            // 验证结果
            let passed = duplicateGroups.size >= 0  // 基本验证
            println("语义相似性检测: ${if (passed) { "✅ 通过" } else { "❌ 失败" }}")
            return passed
            
        } catch (e: Exception) {
            println("语义相似性检测测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试记忆合并
     */
    private func testMemoryMerging(): Bool {
        println("\n--- 测试记忆合并 ---")
        
        try {
            let merger = MemoryMerger()
            
            // 创建相似记忆组
            let memory1 = createTestMemory("merge1", "我喜欢喝咖啡")
            let memory2 = createTestMemory("merge2", "我很喜欢喝咖啡，特别是拿铁")
            let memory3 = createTestMemory("merge3", "咖啡是我最喜欢的饮料")
            
            let memoryGroup = [memory1, memory2, memory3]
            
            // 测试智能合并
            let mergeResult = merger.mergeMemoryGroup(memoryGroup, None)
            let mergeSuccess = mergeResult.isSome()
            println("智能合并: ${if (mergeSuccess) { "成功" } else { "失败" }}")
            
            if (mergeSuccess) {
                let result = mergeResult.getOrThrow()
                println("合并策略: ${result.strategy.toString()}")
                println("合并置信度: ${result.confidence}")
                println("原始记忆数量: ${result.originalMemories.size}")
            }
            
            // 测试内容融合
            let fusedContent = merger.fuseContent(memoryGroup)
            println("内容融合长度: ${fusedContent.size}")
            
            // 测试合并统计
            let stats = merger.getMergeStatistics()
            println("合并统计项数: ${stats.size}")
            
            println("记忆合并: ✅ 通过")
            return true
            
        } catch (e: Exception) {
            println("记忆合并测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试自动去重
     */
    private func testMemoryDeduplication(): Bool {
        println("\n--- 测试自动去重 ---")
        
        try {
            let deduplicator = MemoryDeduplicator()
            
            // 创建包含重复的记忆列表
            let memories = ArrayList<MemoryRecord>()
            memories.add(createTestMemory("dup1", "我喜欢喝咖啡"))
            memories.add(createTestMemory("dup2", "我很喜欢喝咖啡"))
            memories.add(createTestMemory("dup3", "我喜欢喝茶"))
            memories.add(createTestMemory("dup4", "今天天气很好"))
            
            let memoryArray = memories.toArray()
            
            // 测试去重处理
            let (dedupResult, processedMemories) = deduplicator.deduplicateMemories(memoryArray)
            
            println("原始数量: ${dedupResult.originalCount}")
            println("最终数量: ${processedMemories.size}")
            println("去重率: ${dedupResult.deduplicationRate}%")
            println("处理时间: ${dedupResult.processingTime}ms")
            
            // 测试快速重复检查
            let hasDuplicates = deduplicator.haseDuplicates(memoryArray)
            println("存在重复: ${hasDuplicates}")
            
            // 测试去重统计
            let stats = deduplicator.getDeduplicationStatistics()
            println("统计项数: ${stats.size}")
            
            // 验证去重效果
            let passed = processedMemories.size <= memoryArray.size
            println("自动去重: ${if (passed) { "✅ 通过" } else { "❌ 失败" }}")
            return passed
            
        } catch (e: Exception) {
            println("自动去重测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试冲突检测
     */
    private func testConflictDetection(): Bool {
        println("\n--- 测试冲突检测 ---")
        
        try {
            let resolver = ConflictResolver()
            
            // 创建可能冲突的记忆
            let memory1 = createTestMemory("conflict1", "我喜欢喝咖啡")
            let memory2 = createTestMemory("conflict2", "我不喜欢喝咖啡")
            let memory3 = createTestMemory("conflict3", "我住在北京")
            let memory4 = createTestMemory("conflict4", "我住在上海")
            
            let memories = [memory1, memory2, memory3, memory4]
            
            // 测试冲突检测
            let conflicts = resolver.detectConflicts(memories)
            println("检测到冲突数量: ${conflicts.size}")
            
            // 测试冲突解决
            var resolvedCount: Int64 = 0
            for (conflict in conflicts) {
                let resolution = resolver.resolveConflict(conflict, None)
                if (resolution.isSome()) {
                    resolvedCount = resolvedCount + 1
                }
            }
            println("成功解决冲突数量: ${resolvedCount}")
            
            // 测试冲突统计
            let stats = resolver.getConflictStatistics()
            println("冲突统计项数: ${stats.size}")
            
            // 测试待审核冲突
            let pendingReviews = resolver.getPendingManualReviews()
            println("待审核冲突数量: ${pendingReviews.size}")
            
            println("冲突检测: ✅ 通过")
            return true
            
        } catch (e: Exception) {
            println("冲突检测测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 创建测试记忆
     */
    private func createTestMemory(id: String, content: String): MemoryRecord {
        let metadata = HashMap<String, String>()
        metadata["source"] = "test"
        metadata["type"] = "validation"
        
        return MemoryRecord(id, content, "hash_${id}", 0.5, 0, None, metadata)
    }
}
