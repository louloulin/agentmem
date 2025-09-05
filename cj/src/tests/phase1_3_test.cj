/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.tests

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.storage.backends.*
import contextengine.models.MemoryRecord

/**
 * Phase 1.3 高级存储后端测试
 * 测试分布式存储、向量数据库等高级功能
 */
public class Phase13Test {
    private var testResults: HashMap<String, Bool>
    private var testCount: Int64
    private var passedCount: Int64
    
    public init() {
        this.testResults = HashMap<String, Bool>()
        this.testCount = 0
        this.passedCount = 0
    }
    
    /**
     * 运行所有Phase 1.3测试
     */
    public func runAllTests(): Bool {
        println("=== Phase 1.3 高级存储后端测试开始 ===")
        
        // 测试分布式存储
        testDistributedStorage()
        
        // 测试Pinecone向量存储
        testPineconeVectorStorage()
        
        // 测试存储配置管理器增强功能
        testStorageConfigManagerEnhancements()
        
        // 测试多存储管理器增强功能
        testMultiStorageManagerEnhancements()
        
        // 测试向量存储集成
        testVectorStorageIntegration()
        
        // 输出测试结果
        printTestResults()
        
        return passedCount == testCount
    }
    
    /**
     * 测试分布式存储功能
     */
    private func testDistributedStorage(): Unit {
        println("\n--- 测试分布式存储功能 ---")
        
        // 创建内存存储作为本地后端
        let memoryStorage = MemoryStorageBackend()
        memoryStorage.initialize()
        
        // 创建分布式存储配置
        let config = DistributedStorageConfig(3, "eventual", "consistent_hash")
        
        // 创建分布式存储
        let distributedStorage = DistributedKVStorage(config, memoryStorage)
        
        // 测试初始化
        let initResult = distributedStorage.initialize()
        recordTest("分布式存储初始化", initResult)
        
        // 测试添加节点
        let node1 = DistributedNode("node1", "192.168.1.10", 8080, 1.0, "us-west")
        let node2 = DistributedNode("node2", "192.168.1.11", 8080, 1.0, "us-west")
        let node3 = DistributedNode("node3", "192.168.1.12", 8080, 1.0, "us-east")
        
        let addNode1 = distributedStorage.addNode(node1)
        let addNode2 = distributedStorage.addNode(node2)
        let addNode3 = distributedStorage.addNode(node3)
        
        recordTest("添加分布式节点1", addNode1)
        recordTest("添加分布式节点2", addNode2)
        recordTest("添加分布式节点3", addNode3)
        
        // 测试获取节点
        let allNodes = distributedStorage.getAllNodes()
        recordTest("获取所有节点", allNodes.size == 3)
        
        let healthyNodes = distributedStorage.getHealthyNodes()
        recordTest("获取健康节点", healthyNodes.size == 3)
        
        // 测试分布式存储操作
        let putResult = distributedStorage.distributedPut("test_key", "test_value")
        recordTest("分布式存储数据", putResult)
        
        let getValue = distributedStorage.distributedGet("test_key")
        recordTest("分布式获取数据", getValue.isSome())
        
        let deleteResult = distributedStorage.distributedDelete("test_key")
        recordTest("分布式删除数据", deleteResult)
        
        // 测试统计信息
        let stats = distributedStorage.getDistributedStats()
        recordTest("获取分布式统计信息", stats.size > 0)
        
        println("分布式存储测试完成")
    }
    
    /**
     * 测试Pinecone向量存储
     */
    private func testPineconeVectorStorage(): Unit {
        println("\n--- 测试Pinecone向量存储 ---")
        
        // 创建Pinecone存储（使用模拟配置）
        let pinecone = PineconeVectorStorage("test-api-key", "us-west1-gcp", "test-index", "test-project", 384)
        
        // 测试初始化
        let initResult = pinecone.initialize()
        recordTest("Pinecone初始化", initResult)
        
        // 创建测试向量
        let vectorValues = ArrayList<Float64>()
        for (i in 0..384) {
            vectorValues.add(0.1 * Float64(i))
        }
        let testVector = Vector(vectorValues.toArray())
        
        // 创建测试元数据
        let metadata = HashMap<String, String>()
        metadata["source"] = "test"
        metadata["type"] = "memory"
        
        // 测试添加向量
        let addResult = pinecone.addVector("test_vector_1", testVector, metadata)
        recordTest("Pinecone添加向量", addResult)
        
        // 测试获取向量
        let getResult = pinecone.getVector("test_vector_1")
        recordTest("Pinecone获取向量", getResult.isSome())
        
        // 测试相似性搜索
        let searchResults = pinecone.searchSimilar(testVector, 5, 0.8)
        recordTest("Pinecone相似性搜索", searchResults.size > 0)
        
        // 测试批量操作
        let batchVectors = HashMap<String, Vector>()
        let batchMetadata = HashMap<String, HashMap<String, String>>()
        
        for (i in 0..3) {
            let batchVectorValues = ArrayList<Float64>()
            for (j in 0..384) {
                batchVectorValues.add(0.1 * Float64(j + i))
            }
            let batchVector = Vector(batchVectorValues.toArray())
            batchVectors["batch_vector_${i}"] = batchVector
            
            let batchMeta = HashMap<String, String>()
            batchMeta["batch"] = "true"
            batchMeta["index"] = i.toString()
            batchMetadata["batch_vector_${i}"] = batchMeta
        }
        
        let batchResult = pinecone.batchAddVectors(batchVectors, batchMetadata)
        recordTest("Pinecone批量添加向量", batchResult > 0)
        
        // 测试获取统计信息
        let stats = pinecone.getStorageStats()
        recordTest("Pinecone获取统计信息", stats.size > 0)
        
        println("Pinecone向量存储测试完成")
    }
    
    /**
     * 测试存储配置管理器增强功能
     */
    private func testStorageConfigManagerEnhancements(): Unit {
        println("\n--- 测试存储配置管理器增强功能 ---")
        
        let configManager = StorageConfigManager()
        
        // 测试获取配置统计
        let stats = configManager.getConfigurationStats()
        recordTest("获取配置统计信息", stats.size > 0)
        
        // 测试获取不同类型的配置
        let kvConfigs = configManager.getConfigurationsByType(StorageConfigType.KV_STORAGE)
        recordTest("获取KV存储配置", kvConfigs.size > 0)
        
        let vectorConfigs = configManager.getConfigurationsByType(StorageConfigType.VECTOR_STORAGE)
        recordTest("获取向量存储配置", vectorConfigs.size > 0)
        
        // 测试启用/禁用配置
        let enableResult = configManager.enableConfiguration("default_chroma")
        recordTest("启用Chroma配置", enableResult)
        
        let disableResult = configManager.disableConfiguration("default_chroma")
        recordTest("禁用Chroma配置", disableResult)
        
        // 测试获取启用的配置
        let enabledConfigs = configManager.getEnabledConfigurations(StorageConfigType.KV_STORAGE)
        recordTest("获取启用的KV配置", enabledConfigs.size > 0)
        
        // 测试按优先级排序
        let priorityConfigs = configManager.getConfigurationsByPriority(StorageConfigType.KV_STORAGE)
        recordTest("按优先级获取配置", priorityConfigs.size > 0)
        
        println("存储配置管理器增强功能测试完成")
    }
    
    /**
     * 测试多存储管理器增强功能
     */
    private func testMultiStorageManagerEnhancements(): Unit {
        println("\n--- 测试多存储管理器增强功能 ---")
        
        let multiManager = MultiStorageManager()
        
        // 创建向量存储实例
        let memoryVectorStorage = MemoryVectorStorage()
        memoryVectorStorage.initialize()
        
        let chromaStorage = ChromaVectorStorage()
        chromaStorage.initialize()
        
        // 测试注册向量存储
        let registerMemory = multiManager.registerVectorStorage("memory_vector", memoryVectorStorage)
        recordTest("注册内存向量存储", registerMemory)
        
        let registerChroma = multiManager.registerVectorStorage("chroma_vector", chromaStorage)
        recordTest("注册Chroma向量存储", registerChroma)
        
        // 测试获取注册的向量存储
        let vectorStorageNames = multiManager.getRegisteredVectorStorages()
        recordTest("获取注册的向量存储", vectorStorageNames.size == 2)
        
        // 测试切换向量存储
        let switchResult = multiManager.switchVectorStorage("chroma_vector")
        recordTest("切换向量存储", switchResult)
        
        // 测试获取当前向量存储
        let currentVector = multiManager.getCurrentVectorStorage()
        recordTest("获取当前向量存储", currentVector.isSome())
        
        // 创建分布式存储实例
        let memoryBackend = MemoryStorageBackend()
        memoryBackend.initialize()
        let distributedConfig = DistributedStorageConfig(2, "eventual", "hash")
        let distributedStorage = DistributedKVStorage(distributedConfig, memoryBackend)
        distributedStorage.initialize()
        
        // 测试注册分布式存储
        let registerDistributed = multiManager.registerDistributedStorage("distributed_kv", distributedStorage)
        recordTest("注册分布式存储", registerDistributed)
        
        // 测试获取注册的分布式存储
        let distributedNames = multiManager.getRegisteredDistributedStorages()
        recordTest("获取注册的分布式存储", distributedNames.size == 1)
        
        println("多存储管理器增强功能测试完成")
    }
    
    /**
     * 测试向量存储集成
     */
    private func testVectorStorageIntegration(): Unit {
        println("\n--- 测试向量存储集成 ---")
        
        // 创建向量存储实例
        let vectorStorage = MemoryVectorStorage()
        let initResult = vectorStorage.initialize()
        recordTest("向量存储初始化", initResult)
        
        // 创建测试向量
        let vectorValues = ArrayList<Float64>()
        for (i in 0..128) {
            vectorValues.add(Float64(i) * 0.01)
        }
        let testVector = Vector(vectorValues.toArray())
        
        // 创建元数据
        let metadata = HashMap<String, String>()
        metadata["type"] = "test"
        metadata["source"] = "integration_test"
        
        // 测试添加向量
        let addResult = vectorStorage.addVector("integration_test_1", testVector, metadata)
        recordTest("集成测试-添加向量", addResult)
        
        // 测试获取向量
        let getResult = vectorStorage.getVector("integration_test_1")
        recordTest("集成测试-获取向量", getResult.isSome())
        
        // 测试更新向量
        let newMetadata = HashMap<String, String>()
        newMetadata["type"] = "updated"
        newMetadata["source"] = "integration_test"
        
        let updateResult = vectorStorage.updateVector("integration_test_1", testVector, newMetadata)
        recordTest("集成测试-更新向量", updateResult)
        
        // 测试相似性搜索
        let searchResults = vectorStorage.searchSimilar(testVector, 5, 0.5)
        recordTest("集成测试-相似性搜索", searchResults.size > 0)
        
        // 测试获取向量数量
        let count = vectorStorage.getVectorCount()
        recordTest("集成测试-获取向量数量", count > 0)
        
        // 测试删除向量
        let deleteResult = vectorStorage.deleteVector("integration_test_1")
        recordTest("集成测试-删除向量", deleteResult)
        
        println("向量存储集成测试完成")
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
        println("\n=== Phase 1.3 测试结果摘要 ===")
        println("总测试数: ${testCount}")
        println("通过数: ${passedCount}")
        println("失败数: ${testCount - passedCount}")
        println("通过率: ${(Float64(passedCount) / Float64(testCount) * 100.0).toString()}%")
        
        if (passedCount == testCount) {
            println("🎉 所有测试通过！Phase 1.3 高级存储后端实现成功！")
        } else {
            println("⚠️  部分测试失败，需要进一步检查")
        }
        
        println("=== Phase 1.3 测试完成 ===")
    }
}
