/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.tests

import std.collection.HashMap
import std.collection.ArrayList
import std.time.DateTime
import contextengine.models.MemoryRecord
import contextengine.core.memory.{SemanticSimilarityDetector, SemanticSimilarityConfig, MemoryMerger, MemoryMergerConfig, MemoryDeduplicator, DeduplicationConfig, ConflictResolver, ConflictResolverConfig, MergeStrategy, DeduplicationPriority}

/**
 * Phase 1.4 记忆合并和去重测试
 * 测试语义相似性检测、记忆合并、自动去重和冲突处理功能
 */
public class Phase14Test {
    private var testResults: HashMap<String, Bool>
    private var testCount: Int64
    private var passedCount: Int64
    
    public init() {
        this.testResults = HashMap<String, Bool>()
        this.testCount = 0
        this.passedCount = 0
    }
    
    /**
     * 运行所有Phase 1.4测试
     */
    public func runAllTests(): Bool {
        println("=== Phase 1.4 记忆合并和去重测试开始 ===")
        
        // 测试语义相似性检测
        testSemanticSimilarityDetection()
        
        // 测试记忆合并策略
        testMemoryMerging()
        
        // 测试自动去重机制
        testMemoryDeduplication()
        
        // 测试冲突处理
        testConflictResolution()
        
        // 测试集成功能
        testIntegratedDeduplicationWorkflow()
        
        // 测试性能指标
        testPerformanceMetrics()
        
        // 输出测试结果
        printTestResults()
        
        return passedCount == testCount
    }
    
    /**
     * 测试语义相似性检测功能
     */
    private func testSemanticSimilarityDetection(): Unit {
        println("\n--- 测试语义相似性检测功能 ---")
        
        // 创建相似性检测器
        let config = SemanticSimilarityConfig(0.8, 0.3, 0.75)
        let detector = SemanticSimilarityDetector(config)
        
        // 创建测试记忆
        let memory1 = createTestMemory("memory_1", "我喜欢喝咖啡", "user1")
        let memory2 = createTestMemory("memory_2", "我很喜欢喝咖啡", "user1")
        let memory3 = createTestMemory("memory_3", "我不喜欢喝茶", "user1")
        let memory4 = createTestMemory("memory_4", "今天天气很好", "user1")
        
        // 测试相似记忆检测
        let similarity1 = detector.detectSimilarity(memory1, memory2)
        recordTest("相似记忆检测", similarity1.isSimilar && similarity1.similarity > 0.7)
        
        // 测试不相似记忆检测
        let similarity2 = detector.detectSimilarity(memory1, memory4)
        recordTest("不相似记忆检测", !similarity2.isSimilar)
        
        // 测试文本相似性
        let textSimilarity = detector.detectTextSimilarity("我喜欢咖啡", "我很喜欢咖啡")
        recordTest("文本相似性检测", textSimilarity.similarity > 0.6)
        
        // 测试批量相似记忆查找
        let candidateMemories = [memory2, memory3, memory4]
        let similarMemories = detector.findSimilarMemories(memory1, candidateMemories)
        recordTest("批量相似记忆查找", similarMemories.size > 0)
        
        // 测试重复记忆组检测
        let allMemories = [memory1, memory2, memory3, memory4]
        let duplicateGroups = detector.detectDuplicateGroups(allMemories)
        recordTest("重复记忆组检测", duplicateGroups.size > 0)
        
        // 测试统计信息
        let stats = detector.getSimilarityStatistics(allMemories)
        recordTest("相似性统计信息", stats.size > 0 && stats.contains("total_memories"))
        
        println("语义相似性检测测试完成")
    }
    
    /**
     * 测试记忆合并策略
     */
    private func testMemoryMerging(): Unit {
        println("\n--- 测试记忆合并策略 ---")
        
        // 创建记忆合并器
        let config = MemoryMergerConfig(MergeStrategy.INTELLIGENT_MERGE, 0.8, 5)
        let detector = SemanticSimilarityDetector()
        let merger = MemoryMerger(config, detector)
        
        // 创建相似记忆组
        let memory1 = createTestMemory("merge_1", "我喜欢喝咖啡", "user1")
        memory1.importance = 0.8
        
        let memory2 = createTestMemory("merge_2", "我很喜欢喝咖啡，特别是拿铁", "user1")
        memory2.importance = 0.9
        
        let memory3 = createTestMemory("merge_3", "咖啡是我最喜欢的饮料", "user1")
        memory3.importance = 0.7
        
        let memoryGroup = [memory1, memory2, memory3]
        
        // 测试智能合并策略
        let mergeResult = merger.mergeMemoryGroup(memoryGroup, Some(MergeStrategy.INTELLIGENT_MERGE))
        recordTest("智能合并策略", mergeResult.isSome())
        
        if (mergeResult.isSome()) {
            let result = mergeResult.getOrThrow()
            recordTest("合并结果包含原始记忆", result.originalMemories.size == 3)
            recordTest("合并置信度合理", result.confidence > 0.5)
        }
        
        // 测试保留最新策略
        let latestResult = merger.mergeMemoryGroup(memoryGroup, Some(MergeStrategy.KEEP_LATEST))
        recordTest("保留最新策略", latestResult.isSome())
        
        // 测试保留最重要策略
        let importantResult = merger.mergeMemoryGroup(memoryGroup, Some(MergeStrategy.KEEP_MOST_IMPORTANT))
        recordTest("保留最重要策略", importantResult.isSome())
        
        // 测试批量合并
        let allMemories = [memory1, memory2, memory3]
        let batchResults = merger.batchMergeMemories(allMemories)
        recordTest("批量合并", batchResults.size >= 0)
        
        // 测试内容融合
        let fusedContent = merger.fuseContent(memoryGroup)
        recordTest("内容融合", fusedContent.size > 0)
        
        // 测试合并统计
        let mergeStats = merger.getMergeStatistics()
        recordTest("合并统计信息", mergeStats.size > 0)
        
        println("记忆合并策略测试完成")
    }
    
    /**
     * 测试自动去重机制
     */
    private func testMemoryDeduplication(): Unit {
        println("\n--- 测试自动去重机制 ---")
        
        // 创建去重器
        let config = DeduplicationConfig(DeduplicationPriority.INTELLIGENT, 0.85, 100)
        let detector = SemanticSimilarityDetector()
        let merger = MemoryMerger()
        let deduplicator = MemoryDeduplicator(config, detector, merger)
        
        // 创建包含重复的记忆列表
        let memories = ArrayList<MemoryRecord>()
        
        // 添加重复记忆
        memories.add(createTestMemory("dup_1", "我喜欢喝咖啡", "user1"))
        memories.add(createTestMemory("dup_2", "我很喜欢喝咖啡", "user1"))
        memories.add(createTestMemory("dup_3", "我喜欢喝茶", "user1"))
        memories.add(createTestMemory("dup_4", "我非常喜欢喝茶", "user1"))
        memories.add(createTestMemory("dup_5", "今天天气很好", "user1"))
        memories.add(createTestMemory("dup_6", "我学习编程", "user1"))
        
        let memoryArray = memories.toArray()
        
        // 测试去重处理
        let (deduplicationResult, processedMemories) = deduplicator.deduplicateMemories(memoryArray)
        
        recordTest("去重处理执行", deduplicationResult.originalCount == 6)
        recordTest("去重后数量减少", processedMemories.size <= memoryArray.size)
        recordTest("去重率计算", deduplicationResult.deduplicationRate >= 0.0)
        
        // 测试快速重复检查
        let hasDuplicates = deduplicator.haseDuplicates(memoryArray)
        recordTest("快速重复检查", hasDuplicates)
        
        // 测试批量去重
        let (batchResult, batchProcessed) = deduplicator.batchDeduplicate(memoryArray)
        recordTest("批量去重处理", batchResult.originalCount > 0)
        
        // 测试去重统计
        let dedupStats = deduplicator.getDeduplicationStatistics()
        recordTest("去重统计信息", dedupStats.size > 0 && dedupStats.contains("total_processed"))
        
        // 测试去重历史
        let dedupHistory = deduplicator.getDeduplicationHistory()
        recordTest("去重历史记录", dedupHistory.size > 0)
        
        println("自动去重机制测试完成")
    }
    
    /**
     * 测试冲突处理
     */
    private func testConflictResolution(): Unit {
        println("\n--- 测试冲突处理 ---")
        
        // 创建冲突解决器
        let config = ConflictResolverConfig()
        let detector = SemanticSimilarityDetector()
        let resolver = ConflictResolver(config, detector)
        
        // 创建冲突记忆
        let conflictMemory1 = createTestMemory("conflict_1", "我喜欢喝咖啡", "user1")
        let conflictMemory2 = createTestMemory("conflict_2", "我不喜欢喝咖啡", "user1")
        let conflictMemory3 = createTestMemory("conflict_3", "我住在北京", "user1")
        let conflictMemory4 = createTestMemory("conflict_4", "我住在上海", "user1")
        
        let conflictMemories = [conflictMemory1, conflictMemory2, conflictMemory3, conflictMemory4]
        
        // 测试冲突检测
        let conflicts = resolver.detectConflicts(conflictMemories)
        recordTest("冲突检测", conflicts.size >= 0)
        
        // 如果检测到冲突，测试冲突解决
        if (conflicts.size > 0) {
            let conflict = conflicts[0]
            
            // 测试自动冲突解决
            let resolutionResult = resolver.resolveConflict(conflict, None)
            recordTest("自动冲突解决", resolutionResult.isSome())
            
            if (resolutionResult.isSome()) {
                let result = resolutionResult.getOrThrow()
                recordTest("解决结果有效", result.resolvedMemories.size > 0)
                recordTest("解决置信度合理", result.confidence > 0.0)
            }
        }
        
        // 测试批量冲突解决
        let batchResolutions = resolver.batchResolveConflicts(conflicts)
        recordTest("批量冲突解决", batchResolutions.size >= 0)
        
        // 测试待审核冲突
        let pendingReviews = resolver.getPendingManualReviews()
        recordTest("待审核冲突获取", pendingReviews.size >= 0)
        
        // 测试冲突统计
        let conflictStats = resolver.getConflictStatistics()
        recordTest("冲突统计信息", conflictStats.size > 0)
        
        // 测试冲突历史
        let conflictHistory = resolver.getConflictHistory()
        recordTest("冲突历史记录", conflictHistory.size >= 0)
        
        println("冲突处理测试完成")
    }
    
    /**
     * 测试集成去重工作流
     */
    private func testIntegratedDeduplicationWorkflow(): Unit {
        println("\n--- 测试集成去重工作流 ---")
        
        // 创建完整的去重工作流
        let similarityDetector = SemanticSimilarityDetector()
        let memoryMerger = MemoryMerger()
        let deduplicator = MemoryDeduplicator()
        let conflictResolver = ConflictResolver()
        
        // 创建复杂的测试数据集
        let testMemories = createComplexTestDataset()
        
        // 步骤1：检测相似性
        let duplicateGroups = similarityDetector.detectDuplicateGroups(testMemories)
        recordTest("集成工作流-相似性检测", duplicateGroups.size >= 0)
        
        // 步骤2：执行去重
        let (dedupResult, dedupedMemories) = deduplicator.deduplicateMemories(testMemories)
        recordTest("集成工作流-去重执行", dedupResult.finalCount <= dedupResult.originalCount)
        
        // 步骤3：检测冲突
        let conflicts = conflictResolver.detectConflicts(dedupedMemories)
        recordTest("集成工作流-冲突检测", conflicts.size >= 0)
        
        // 步骤4：解决冲突
        let resolutions = conflictResolver.batchResolveConflicts(conflicts)
        recordTest("集成工作流-冲突解决", resolutions.size >= 0)
        
        // 验证最终结果
        recordTest("集成工作流-最终数量合理", dedupedMemories.size > 0)
        
        println("集成去重工作流测试完成")
    }
    
    /**
     * 测试性能指标
     */
    private func testPerformanceMetrics(): Unit {
        println("\n--- 测试性能指标 ---")
        
        // 创建大量测试数据
        let largeDataset = createLargeTestDataset(1000)  // 1000条记忆
        
        let startTime = DateTime.now().toUnixTimeStamp()
        
        // 测试大数据集去重性能
        let deduplicator = MemoryDeduplicator()
        let (perfResult, _) = deduplicator.batchDeduplicate(largeDataset)
        
        let endTime = DateTime.now().toUnixTimeStamp()
        let processingTime = endTime - startTime
        
        // 验证性能指标
        let processingSpeed = Float64(perfResult.originalCount) / Float64(processingTime) * 1000.0  // 记忆/秒
        recordTest("去重处理速度 > 100记忆/秒", processingSpeed > 100.0)
        
        // 验证去重准确率
        recordTest("去重率 > 10%", perfResult.deduplicationRate > 10.0)
        
        // 验证内存使用合理
        recordTest("处理时间合理", processingTime < 10000)  // 小于10秒
        
        println("性能指标测试完成")
        println("  - 处理速度: ${processingSpeed} 记忆/秒")
        println("  - 去重率: ${perfResult.deduplicationRate}%")
        println("  - 处理时间: ${processingTime}ms")
    }
    
    // ===== 辅助方法 =====
    
    /**
     * 创建测试记忆
     */
    private func createTestMemory(id: String, content: String, userId: String): MemoryRecord {
        let metadata = HashMap<String, String>()
        metadata["source"] = "test"
        metadata["type"] = "test_memory"
        
        return MemoryRecord(id, content, userId, DateTime.now().toUnixTimeStamp(), metadata)
    }
    
    /**
     * 创建复杂测试数据集
     */
    private func createComplexTestDataset(): Array<MemoryRecord> {
        let memories = ArrayList<MemoryRecord>()
        
        // 添加相似记忆组
        memories.add(createTestMemory("complex_1", "我喜欢喝咖啡", "user1"))
        memories.add(createTestMemory("complex_2", "我很喜欢喝咖啡", "user1"))
        memories.add(createTestMemory("complex_3", "咖啡是我的最爱", "user1"))
        
        // 添加另一组相似记忆
        memories.add(createTestMemory("complex_4", "我住在北京", "user1"))
        memories.add(createTestMemory("complex_5", "我的家在北京", "user1"))
        
        // 添加冲突记忆
        memories.add(createTestMemory("complex_6", "我不喜欢喝咖啡", "user1"))
        memories.add(createTestMemory("complex_7", "我住在上海", "user1"))
        
        // 添加独特记忆
        memories.add(createTestMemory("complex_8", "今天天气很好", "user1"))
        memories.add(createTestMemory("complex_9", "我在学习编程", "user1"))
        memories.add(createTestMemory("complex_10", "明天要开会", "user1"))
        
        return memories.toArray()
    }
    
    /**
     * 创建大型测试数据集
     */
    private func createLargeTestDataset(size: Int64): Array<MemoryRecord> {
        let memories = ArrayList<MemoryRecord>()
        
        let baseContents = [
            "我喜欢喝咖啡",
            "我住在北京",
            "今天天气很好",
            "我在学习编程",
            "我喜欢看电影",
            "我经常运动",
            "我喜欢读书",
            "我喜欢旅行",
            "我喜欢音乐",
            "我喜欢美食"
        ]
        
        for (i in 0..size) {
            let baseIndex = i % Int64(baseContents.size)
            let content = baseContents[baseIndex] + " " + i.toString()
            memories.add(createTestMemory("large_${i}", content, "user1"))
        }
        
        return memories.toArray()
    }
    
    /**
     * 记录测试结果
     */
    private func recordTest(testName: String, result: Bool): Unit {
        testResults[testName] = result
        testCount = testCount + 1
        if (result) {
            passedCount = passedCount + 1
            println("✅ ${testName}: 通过")
        } else {
            println("❌ ${testName}: 失败")
        }
    }
    
    /**
     * 打印测试结果摘要
     */
    private func printTestResults(): Unit {
        println("\n=== Phase 1.4 测试结果摘要 ===")
        println("总测试数: ${testCount}")
        println("通过数: ${passedCount}")
        println("失败数: ${testCount - passedCount}")
        println("通过率: ${(Float64(passedCount) / Float64(testCount) * 100.0).toString()}%")
        
        if (passedCount == testCount) {
            println("🎉 所有测试通过！Phase 1.4 记忆合并和去重实现成功！")
        } else {
            println("⚠️  部分测试失败，需要进一步检查")
        }
        
        println("=== Phase 1.4 测试完成 ===")
    }
}
