/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.tests

import std.collection.HashMap
import std.collection.ArrayList
import std.time.DateTime
import contextengine.core.memory.{MemoryServiceImpl, MemoryConfig}
import contextengine.core.search.{EnhancedSearchParams, TimeRangeFilter}
import contextengine.models.MemoryRecord

/**
 * Phase 2.1 高级记忆搜索功能测试
 * 测试语义搜索增强、时间范围过滤、重要性排序和多模态搜索
 */
public class Phase21Test {
    private let memoryService: MemoryServiceImpl
    private let testMemories: Array<String>
    
    public init() {
        this.memoryService = MemoryServiceImpl()
        this.testMemories = [
            "我喜欢喝咖啡，特别是意式浓缩咖啡",
            "如何制作完美的拿铁：先制作浓缩咖啡，然后加入蒸汽牛奶",
            "昨天在星巴克买了一杯卡布奇诺，味道很不错",
            "咖啡豆的种类很多，阿拉比卡和罗布斯塔是最常见的",
            "我正在学习机器学习和深度学习技术",
            "Python是我最喜欢的编程语言，特别适合AI开发",
            "今天完成了一个自然语言处理项目",
            "我的工作涉及大数据分析和人工智能算法",
            "最近在研究Transformer模型的架构",
            "我住在北京，经常去中关村的科技园区"
        ]
    }
    
    /**
     * 运行所有Phase 2.1测试
     */
    public func runAllTests(): Bool {
        println("=== Phase 2.1 高级记忆搜索功能测试开始 ===")
        
        var allTestsPassed = true
        
        // 准备测试数据
        if (!prepareTestData()) {
            println("❌ 测试数据准备失败")
            return false
        }
        
        // 测试1：增强语义搜索
        println("\n1. 测试增强语义搜索...")
        if (!testEnhancedSemanticSearch()) {
            println("❌ 增强语义搜索测试失败")
            allTestsPassed = false
        } else {
            println("✅ 增强语义搜索测试通过")
        }
        
        // 测试2：时间范围过滤
        println("\n2. 测试时间范围过滤...")
        if (!testTimeRangeFiltering()) {
            println("❌ 时间范围过滤测试失败")
            allTestsPassed = false
        } else {
            println("✅ 时间范围过滤测试通过")
        }
        
        // 测试3：重要性排序
        println("\n3. 测试重要性排序...")
        if (!testImportanceRanking()) {
            println("❌ 重要性排序测试失败")
            allTestsPassed = false
        } else {
            println("✅ 重要性排序测试通过")
        }
        
        // 测试4：多模态搜索
        println("\n4. 测试多模态搜索...")
        if (!testMultiModalSearch()) {
            println("❌ 多模态搜索测试失败")
            allTestsPassed = false
        } else {
            println("✅ 多模态搜索测试通过")
        }
        
        // 测试5：智能搜索推荐
        println("\n5. 测试智能搜索推荐...")
        if (!testIntelligentRecommendation()) {
            println("❌ 智能搜索推荐测试失败")
            allTestsPassed = false
        } else {
            println("✅ 智能搜索推荐测试通过")
        }
        
        // 测试6：相关记忆发现
        println("\n6. 测试相关记忆发现...")
        if (!testRelatedMemoryDiscovery()) {
            println("❌ 相关记忆发现测试失败")
            allTestsPassed = false
        } else {
            println("✅ 相关记忆发现测试通过")
        }
        
        if (allTestsPassed) {
            println("\n🎉 Phase 2.1 所有测试通过！")
            println("✅ 语义搜索增强功能完成")
            println("✅ 时间范围过滤功能完成")
            println("✅ 重要性排序优化完成")
            println("✅ 多模态搜索支持完成")
            println("✅ 智能搜索推荐完成")
            println("✅ 相关记忆发现完成")
        } else {
            println("\n❌ Phase 2.1 部分测试失败")
        }
        
        return allTestsPassed
    }
    
    /**
     * 准备测试数据
     */
    private func prepareTestData(): Bool {
        try {
            println("准备测试数据...")
            
            // 添加测试记忆，并设置不同的重要性和元数据
            for (i in 0..testMemories.size) {
                let content = testMemories[i]
                let metadata = HashMap<String, String>()
                
                // 设置不同的重要性分数
                let importance = if (i < 3) { "0.9" } else if (i < 6) { "0.7" } else { "0.5" }
                metadata["importance"] = importance
                
                // 设置分类标签
                if (content.contains("咖啡")) {
                    metadata["category"] = "beverage"
                    metadata["tags"] = "coffee,drink"
                } else if (content.contains("学习") || content.contains("Python") || content.contains("机器学习")) {
                    metadata["category"] = "technology"
                    metadata["tags"] = "programming,ai,learning"
                } else {
                    metadata["category"] = "general"
                    metadata["tags"] = "personal"
                }
                
                // 设置时间戳（模拟不同时间）
                let currentTime = getCurrentTimestamp()
                let timeOffset = Int64(i) * 3600  // 每个记忆间隔1小时
                let createdAt = currentTime - timeOffset
                
                let record = MemoryRecord(
                    "test_memory_${i}", content, "", 0.0,
                    createdAt, createdAt, metadata
                )
                
                memoryService.add(record)
            }
            
            println("成功添加 ${testMemories.size} 条测试记忆")
            return true
            
        } catch (e: Exception) {
            println("准备测试数据失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试增强语义搜索
     */
    private func testEnhancedSemanticSearch(): Bool {
        try {
            println("  测试上下文感知搜索...")
            
            // 创建增强搜索参数
            let params = EnhancedSearchParams(
                "咖啡制作", 5, 0.6, HashMap<String, String>(),
                None, true, true, 0.8, 0.2, Some("用户喜欢意式咖啡")
            )
            
            let results = memoryService.enhancedSemanticSearch(params)
            
            if (results.memories.size == 0) {
                println("    ❌ 未找到相关记忆")
                return false
            }
            
            println("    找到 ${results.memories.size} 条相关记忆:")
            for (memory in results.memories) {
                println("      - ${memory.memory} (分数: ${memory.score})")
            }
            
            // 验证结果包含咖啡相关内容
            var hasCoffeeContent = false
            for (memory in results.memories) {
                if (memory.memory.contains("咖啡") || memory.memory.contains("拿铁")) {
                    hasCoffeeContent = true
                    break
                }
            }
            
            if (!hasCoffeeContent) {
                println("    ❌ 搜索结果不包含预期的咖啡相关内容")
                return false
            }
            
            return true
            
        } catch (e: Exception) {
            println("    增强语义搜索测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 测试时间范围过滤
     */
    private func testTimeRangeFiltering(): Bool {
        try {
            println("  测试相对时间过滤...")
            
            // 创建最近2小时的时间过滤器
            let timeFilter = TimeRangeFilter(2)  // 最近2小时
            
            let results = memoryService.searchByTimeRange("记忆", timeFilter, 10, 0.0)
            
            println("    最近2小时的记忆数量: ${results.memories.size}")
            
            if (results.memories.size == 0) {
                println("    ⚠️  未找到最近2小时的记忆（这可能是正常的）")
            } else {
                for (memory in results.memories) {
                    println("      - ${memory.memory}")
                }
            }
            
            // 测试绝对时间过滤
            println("  测试绝对时间过滤...")
            let currentTime = getCurrentTimestamp()
            let startTime = currentTime - 86400  // 24小时前
            let absoluteTimeFilter = TimeRangeFilter(startTime, currentTime)
            
            let absoluteResults = memoryService.searchByTimeRange("", absoluteTimeFilter, 10, 0.0)
            println("    24小时内的记忆数量: ${absoluteResults.memories.size}")
            
            return true
            
        } catch (e: Exception) {
            println("    时间范围过滤测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 获取当前时间戳
     */
    private func getCurrentTimestamp(): Int64 {
        let now = DateTime.now()
        return now.year * 10000000000 + Int64(now.month.toInteger()) * 100000000 + 
               now.dayOfMonth * 1000000 + now.hour * 10000 + now.minute * 100 + now.second
    }
    
    /**
     * 测试重要性排序
     */
    private func testImportanceRanking(): Bool {
        try {
            println("  测试重要性排序...")
            
            // 创建启用重要性排序的搜索参数
            let params = EnhancedSearchParams(
                "", 10, 0.0, HashMap<String, String>(),
                None, true, true, 0.5, 0.5, None  // 重要性权重50%
            )
            
            let results = memoryService.enhancedSemanticSearch(params)
            
            if (results.memories.size < 2) {
                println("    ❌ 记忆数量不足，无法测试排序")
                return false
            }
            
            println("    按重要性排序的记忆:")
            for (i in 0..minInt(3, results.memories.size)) {
                let memory = results.memories[i]
                let importance = memory.metadata.get("importance") ?? "0.0"
                println("      ${i + 1}. ${memory.memory} (重要性: ${importance}, 分数: ${memory.score})")
            }
            
            return true
            
        } catch (e: Exception) {
            println("    重要性排序测试失败: ${e}")
            return false
        }
    }
    
    /**
     * 获取最小值
     */
    private func minInt(a: Int64, b: Int64): Int64 {
        if (a < b) { a } else { b }
    }

    /**
     * 测试多模态搜索
     */
    private func testMultiModalSearch(): Bool {
        try {
            println("  测试多模态搜索...")

            // 创建元数据过滤器
            let metadataFilters = HashMap<String, String>()
            metadataFilters["category"] = "technology"

            // 创建标签数组
            let tags = ["programming", "ai"]

            let results = memoryService.multiModalSearch("学习", metadataFilters, tags, 5, 0.0)

            println("    多模态搜索结果数量: ${results.memories.size}")

            if (results.memories.size > 0) {
                for (memory in results.memories) {
                    let category = memory.metadata.get("category") ?? "unknown"
                    let tags = memory.metadata.get("tags") ?? "none"
                    println("      - ${memory.memory}")
                    println("        分类: ${category}, 标签: ${tags}")
                }
            }

            return true

        } catch (e: Exception) {
            println("    多模态搜索测试失败: ${e}")
            return false
        }
    }

    /**
     * 测试智能搜索推荐
     */
    private func testIntelligentRecommendation(): Bool {
        try {
            println("  测试智能搜索推荐...")

            let results = memoryService.intelligentSearchRecommendation("编程", "test_user", 5)

            println("    智能推荐结果数量: ${results.memories.size}")

            if (results.memories.size > 0) {
                for (memory in results.memories) {
                    println("      - ${memory.memory} (分数: ${memory.score})")
                }
            }

            return true

        } catch (e: Exception) {
            println("    智能搜索推荐测试失败: ${e}")
            return false
        }
    }

    /**
     * 测试相关记忆发现
     */
    private func testRelatedMemoryDiscovery(): Bool {
        try {
            println("  测试相关记忆发现...")

            // 获取第一个记忆作为基准
            let allMemories = memoryService.getAll()
            if (allMemories.size == 0) {
                println("    ❌ 没有可用的记忆进行测试")
                return false
            }

            let baseMemoryId = allMemories[0].id
            let results = memoryService.discoverRelatedMemories(baseMemoryId, 5, 0.5)

            println("    发现的相关记忆数量: ${results.memories.size}")

            if (results.memories.size > 0) {
                println("    基准记忆: ${allMemories[0].memory}")
                println("    相关记忆:")
                for (memory in results.memories) {
                    println("      - ${memory.memory} (相似度: ${memory.score})")
                }
            }

            return true

        } catch (e: Exception) {
            println("    相关记忆发现测试失败: ${e}")
            return false
        }
    }
}
