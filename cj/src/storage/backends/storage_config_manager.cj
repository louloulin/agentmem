/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.storage.backends

import std.collection.HashMap
import std.collection.ArrayList

/**
 * 存储配置类型枚举
 */
public enum StorageConfigType {
    | KV_STORAGE      // 键值存储配置
    | VECTOR_STORAGE  // 向量存储配置
    | GRAPH_STORAGE   // 图存储配置
    | HYBRID_STORAGE  // 混合存储配置

    public func toString(): String {
        match (this) {
            case KV_STORAGE => "kv_storage"
            case VECTOR_STORAGE => "vector_storage"
            case GRAPH_STORAGE => "graph_storage"
            case HYBRID_STORAGE => "hybrid_storage"
        }
    }

    public static func fromString(typeStr: String): StorageConfigType {
        match (typeStr) {
            case "kv_storage" => KV_STORAGE
            case "vector_storage" => VECTOR_STORAGE
            case "graph_storage" => GRAPH_STORAGE
            case "hybrid_storage" => HYBRID_STORAGE
            case _ => KV_STORAGE  // 默认为键值存储
        }
    }

    /**
     * 相等比较操作符
     */
    public operator func ==(right: StorageConfigType): Bool {
        return this.toString() == right.toString()
    }
}

/**
 * 存储配置项
 */
public struct StorageConfigItem {
    public let configType: StorageConfigType
    public let backendType: StorageBackendType
    public let name: String
    public let connectionString: Option<String>
    public let filePath: Option<String>
    public let options: HashMap<String, String>
    public let isEnabled: Bool
    public let priority: Int64  // 优先级，数字越小优先级越高
    
    public init(configType: StorageConfigType, backendType: StorageBackendType, 
                name: String, isEnabled: Bool, priority: Int64) {
        this.configType = configType
        this.backendType = backendType
        this.name = name
        this.connectionString = None
        this.filePath = None
        this.options = HashMap<String, String>()
        this.isEnabled = isEnabled
        this.priority = priority
    }
    
    public init(configType: StorageConfigType, backendType: StorageBackendType, 
                name: String, connectionString: String, options: HashMap<String, String>,
                isEnabled: Bool, priority: Int64) {
        this.configType = configType
        this.backendType = backendType
        this.name = name
        this.connectionString = Some(connectionString)
        this.filePath = None
        this.options = options
        this.isEnabled = isEnabled
        this.priority = priority
    }
    
    public init(configType: StorageConfigType, backendType: StorageBackendType, 
                name: String, filePath: String, isEnabled: Bool, priority: Int64) {
        this.configType = configType
        this.backendType = backendType
        this.name = name
        this.connectionString = None
        this.filePath = Some(filePath)
        this.options = HashMap<String, String>()
        this.isEnabled = isEnabled
        this.priority = priority
    }
}

/**
 * 存储配置管理器
 * 负责管理所有存储后端的配置信息
 */
public class StorageConfigManager {
    private let configurations: HashMap<String, StorageConfigItem>
    private let configsByType: HashMap<String, ArrayList<StorageConfigItem>>  // 使用String作为键
    private var defaultKVStorage: Option<String>
    private var defaultVectorStorage: Option<String>
    private var defaultGraphStorage: Option<String>
    private let configFilePath: String
    
    public init() {
        this.configurations = HashMap<String, StorageConfigItem>()
        this.configsByType = HashMap<String, ArrayList<StorageConfigItem>>()
        this.defaultKVStorage = None
        this.defaultVectorStorage = None
        this.defaultGraphStorage = None
        this.configFilePath = "storage_config.json"

        // 初始化配置类型映射
        initializeConfigTypes()

        // 加载默认配置
        loadDefaultConfigurations()
    }

    public init(configFilePath: String) {
        this.configurations = HashMap<String, StorageConfigItem>()
        this.configsByType = HashMap<String, ArrayList<StorageConfigItem>>()
        this.defaultKVStorage = None
        this.defaultVectorStorage = None
        this.defaultGraphStorage = None
        this.configFilePath = configFilePath

        // 初始化配置类型映射
        initializeConfigTypes()

        // 加载配置文件
        loadConfigurationFromFile()
    }
    
    /**
     * 初始化配置类型映射
     */
    private func initializeConfigTypes(): Unit {
        configsByType[StorageConfigType.KV_STORAGE.toString()] = ArrayList<StorageConfigItem>()
        configsByType[StorageConfigType.VECTOR_STORAGE.toString()] = ArrayList<StorageConfigItem>()
        configsByType[StorageConfigType.GRAPH_STORAGE.toString()] = ArrayList<StorageConfigItem>()
        configsByType[StorageConfigType.HYBRID_STORAGE.toString()] = ArrayList<StorageConfigItem>()
    }
    
    /**
     * 加载默认配置
     */
    private func loadDefaultConfigurations(): Unit {
        println("StorageConfigManager: 加载默认配置...")
        
        // 默认内存KV存储
        let memoryKVConfig = StorageConfigItem(
            StorageConfigType.KV_STORAGE,
            StorageBackendType.MEMORY,
            "default_memory_kv",
            true,
            1
        )
        addConfiguration(memoryKVConfig)
        setDefaultKVStorage("default_memory_kv")
        
        // 默认SQLite存储
        let sqliteOptions = HashMap<String, String>()
        sqliteOptions["journal_mode"] = "WAL"
        sqliteOptions["synchronous"] = "NORMAL"
        sqliteOptions["cache_size"] = "10000"
        
        let sqliteConfig = StorageConfigItem(
            StorageConfigType.KV_STORAGE,
            StorageBackendType.SQLITE,
            "default_sqlite",
            "memory.db",
            true,
            2
        )
        addConfiguration(sqliteConfig)
        
        // 默认PostgreSQL存储
        let pgOptions = HashMap<String, String>()
        pgOptions["max_connections"] = "20"
        pgOptions["connection_timeout"] = "30"
        pgOptions["enable_wal"] = "true"
        
        let pgConfig = StorageConfigItem(
            StorageConfigType.KV_STORAGE,
            StorageBackendType.POSTGRESQL,
            "default_postgresql",
            "postgresql://localhost:5432/contextengine",
            pgOptions,
            false,  // 默认禁用，需要手动启用
            3
        )
        addConfiguration(pgConfig)
        
        // 默认内存向量存储
        let memoryVectorConfig = StorageConfigItem(
            StorageConfigType.VECTOR_STORAGE,
            StorageBackendType.MEMORY,
            "default_memory_vector",
            true,
            1
        )
        addConfiguration(memoryVectorConfig)
        setDefaultVectorStorage("default_memory_vector")
        
        // Chroma向量存储
        let chromaOptions = HashMap<String, String>()
        chromaOptions["server_url"] = "http://localhost:8000"
        chromaOptions["collection_name"] = "contextengine_memories"
        chromaOptions["max_retries"] = "3"
        chromaOptions["request_timeout"] = "30"
        chromaOptions["batch_size"] = "100"

        let chromaConfig = StorageConfigItem(
            StorageConfigType.VECTOR_STORAGE,
            StorageBackendType.MEMORY,  // 使用MEMORY作为占位符，实际是Chroma
            "default_chroma",
            "http://localhost:8000",
            chromaOptions,
            false,  // 默认禁用
            2
        )
        addConfiguration(chromaConfig)

        // Pinecone向量存储
        let pineconeOptions = HashMap<String, String>()
        pineconeOptions["environment"] = "us-west1-gcp"
        pineconeOptions["project_id"] = ""
        pineconeOptions["dimension"] = "384"
        pineconeOptions["max_retries"] = "3"
        pineconeOptions["request_timeout"] = "30"
        pineconeOptions["batch_size"] = "100"

        let pineconeConfig = StorageConfigItem(
            StorageConfigType.VECTOR_STORAGE,
            StorageBackendType.MEMORY,  // 使用MEMORY作为占位符，实际是Pinecone
            "default_pinecone",
            "",  // API密钥通过环境变量设置
            pineconeOptions,
            false,  // 默认禁用，需要配置API密钥
            3
        )
        addConfiguration(pineconeConfig)

        // 分布式存储配置
        let distributedOptions = HashMap<String, String>()
        distributedOptions["replication_factor"] = "3"
        distributedOptions["consistency_level"] = "eventual"
        distributedOptions["partition_strategy"] = "consistent_hash"
        distributedOptions["enable_sharding"] = "true"
        distributedOptions["max_retries"] = "3"
        distributedOptions["timeout_ms"] = "5000"

        let distributedConfig = StorageConfigItem(
            StorageConfigType.HYBRID_STORAGE,
            StorageBackendType.MEMORY,  // 使用MEMORY作为占位符，实际是分布式存储
            "default_distributed",
            "",
            distributedOptions,
            false,  // 默认禁用
            4
        )
        addConfiguration(distributedConfig)

        println("StorageConfigManager: 默认配置加载完成")
    }
    
    /**
     * 从文件加载配置
     */
    private func loadConfigurationFromFile(): Unit {
        println("StorageConfigManager: 从文件加载配置: ${configFilePath}")
        
        // 模拟从JSON文件加载配置
        // 在实际实现中，这里会解析JSON配置文件
        
        // 如果文件不存在，加载默认配置
        loadDefaultConfigurations()
        
        println("StorageConfigManager: 配置文件加载完成")
    }
    
    /**
     * 添加存储配置
     */
    public func addConfiguration(config: StorageConfigItem): Bool {
        try {
            configurations[config.name] = config
            
            // 添加到类型映射
            let typeKey = config.configType.toString()
            if (configsByType.contains(typeKey)) {
                configsByType[typeKey].add(config)
            }
            
            println("StorageConfigManager: 添加配置: ${config.name}")
            return true
            
        } catch (e: Exception) {
            println("StorageConfigManager: 添加配置失败: ${e}")
            return false
        }
    }
    
    /**
     * 获取存储配置
     */
    public func getConfiguration(name: String): Option<StorageConfigItem> {
        if (configurations.contains(name)) {
            return Some(configurations[name])
        }
        return None
    }
    
    /**
     * 获取指定类型的所有配置
     */
    public func getConfigurationsByType(configType: StorageConfigType): Array<StorageConfigItem> {
        let typeKey = configType.toString()
        if (configsByType.contains(typeKey)) {
            return configsByType[typeKey].toArray()
        }
        return Array<StorageConfigItem>()
    }

    /**
     * 获取启用的配置
     */
    public func getEnabledConfigurations(configType: StorageConfigType): Array<StorageConfigItem> {
        let enabledConfigs = ArrayList<StorageConfigItem>()
        let typeKey = configType.toString()

        if (configsByType.contains(typeKey)) {
            let configs = configsByType[typeKey]
            for (config in configs) {
                if (config.isEnabled) {
                    enabledConfigs.add(config)
                }
            }
        }

        return enabledConfigs.toArray()
    }
    
    /**
     * 按优先级排序配置
     */
    public func getConfigurationsByPriority(configType: StorageConfigType): Array<StorageConfigItem> {
        let configs = getConfigurationsByType(configType)
        return sortConfigsByPriority(configs)
    }
    
    /**
     * 设置默认KV存储
     */
    public func setDefaultKVStorage(name: String): Bool {
        if (configurations.contains(name)) {
            let config = configurations[name]
            if (config.configType == StorageConfigType.KV_STORAGE) {
                defaultKVStorage = Some(name)
                println("StorageConfigManager: 设置默认KV存储: ${name}")
                return true
            }
        }
        return false
    }
    
    /**
     * 设置默认向量存储
     */
    public func setDefaultVectorStorage(name: String): Bool {
        if (configurations.contains(name)) {
            let config = configurations[name]
            if (config.configType == StorageConfigType.VECTOR_STORAGE) {
                defaultVectorStorage = Some(name)
                println("StorageConfigManager: 设置默认向量存储: ${name}")
                return true
            }
        }
        return false
    }
    
    /**
     * 设置默认图存储
     */
    public func setDefaultGraphStorage(name: String): Bool {
        if (configurations.contains(name)) {
            let config = configurations[name]
            if (config.configType == StorageConfigType.GRAPH_STORAGE) {
                defaultGraphStorage = Some(name)
                println("StorageConfigManager: 设置默认图存储: ${name}")
                return true
            }
        }
        return false
    }
    
    /**
     * 获取默认KV存储配置
     */
    public func getDefaultKVStorageConfig(): Option<StorageConfigItem> {
        if (defaultKVStorage.isSome()) {
            return getConfiguration(defaultKVStorage.getOrThrow())
        }
        return None
    }

    /**
     * 获取默认向量存储配置
     */
    public func getDefaultVectorStorageConfig(): Option<StorageConfigItem> {
        if (defaultVectorStorage.isSome()) {
            return getConfiguration(defaultVectorStorage.getOrThrow())
        }
        return None
    }

    /**
     * 获取默认图存储配置
     */
    public func getDefaultGraphStorageConfig(): Option<StorageConfigItem> {
        if (defaultGraphStorage.isSome()) {
            return getConfiguration(defaultGraphStorage.getOrThrow())
        }
        return None
    }
    
    /**
     * 启用配置
     */
    public func enableConfiguration(name: String): Bool {
        if (configurations.contains(name)) {
            let config = configurations[name]
            let connectionStr = if (config.connectionString.isSome()) {
                config.connectionString.getOrThrow()
            } else {
                ""
            }
            let updatedConfig = StorageConfigItem(
                config.configType,
                config.backendType,
                config.name,
                connectionStr,
                config.options,
                true,  // 启用
                config.priority
            )
            configurations[name] = updatedConfig
            
            // 更新类型映射
            updateConfigInTypeMapping(updatedConfig)
            
            println("StorageConfigManager: 启用配置: ${name}")
            return true
        }
        return false
    }
    
    /**
     * 禁用配置
     */
    public func disableConfiguration(name: String): Bool {
        if (configurations.contains(name)) {
            let config = configurations[name]
            let connectionStr = if (config.connectionString.isSome()) {
                config.connectionString.getOrThrow()
            } else {
                ""
            }
            let updatedConfig = StorageConfigItem(
                config.configType,
                config.backendType,
                config.name,
                connectionStr,
                config.options,
                false,  // 禁用
                config.priority
            )
            configurations[name] = updatedConfig
            
            // 更新类型映射
            updateConfigInTypeMapping(updatedConfig)
            
            println("StorageConfigManager: 禁用配置: ${name}")
            return true
        }
        return false
    }
    
    /**
     * 删除配置
     */
    public func removeConfiguration(name: String): Bool {
        if (configurations.contains(name)) {
            let config = configurations[name]
            configurations.remove(name)
            
            // 从类型映射中删除
            removeConfigFromTypeMapping(config)
            
            println("StorageConfigManager: 删除配置: ${name}")
            return true
        }
        return false
    }
    
    /**
     * 获取所有配置名称
     */
    public func getAllConfigurationNames(): Array<String> {
        let names = ArrayList<String>()
        for ((name, _) in configurations) {
            names.add(name)
        }
        return names.toArray()
    }
    
    /**
     * 获取配置统计信息
     */
    public func getConfigurationStats(): HashMap<String, String> {
        let stats = HashMap<String, String>()
        
        stats["total_configurations"] = configurations.size.toString()
        stats["default_kv_storage"] = if (defaultKVStorage.isSome()) {
            defaultKVStorage.getOrThrow()
        } else {
            "none"
        }
        stats["default_vector_storage"] = if (defaultVectorStorage.isSome()) {
            defaultVectorStorage.getOrThrow()
        } else {
            "none"
        }
        stats["default_graph_storage"] = if (defaultGraphStorage.isSome()) {
            defaultGraphStorage.getOrThrow()
        } else {
            "none"
        }
        
        // 按类型统计
        for ((configType, configList) in configsByType) {
            let typeName = configType.toString()
            stats["${typeName}_count"] = configList.size.toString()
            
            var enabledCount: Int64 = 0
            for (config in configList) {
                if (config.isEnabled) {
                    enabledCount = enabledCount + 1
                }
            }
            stats["${typeName}_enabled"] = enabledCount.toString()
        }
        
        return stats
    }
    
    /**
     * 保存配置到文件
     */
    public func saveConfigurationToFile(): Bool {
        try {
            println("StorageConfigManager: 保存配置到文件: ${configFilePath}")
            
            // 模拟保存到JSON文件
            // 在实际实现中，这里会序列化配置并写入文件
            
            println("StorageConfigManager: 配置保存完成")
            return true
            
        } catch (e: Exception) {
            println("StorageConfigManager: 保存配置失败: ${e}")
            return false
        }
    }
    
    // ===== 私有辅助方法 =====
    
    /**
     * 按优先级排序配置
     */
    private func sortConfigsByPriority(configs: Array<StorageConfigItem>): Array<StorageConfigItem> {
        let sortedList = ArrayList<StorageConfigItem>()
        for (config in configs) {
            sortedList.add(config)
        }
        
        // 简化的冒泡排序，按优先级升序排序
        for (i in 0..sortedList.size) {
            for (j in 0..(sortedList.size - 1 - i)) {
                if (sortedList[j].priority > sortedList[j + 1].priority) {
                    let temp = sortedList[j]
                    sortedList[j] = sortedList[j + 1]
                    sortedList[j + 1] = temp
                }
            }
        }
        
        return sortedList.toArray()
    }
    
    /**
     * 更新类型映射中的配置
     */
    private func updateConfigInTypeMapping(config: StorageConfigItem): Unit {
        let typeKey = config.configType.toString()
        if (configsByType.contains(typeKey)) {
            let configList = configsByType[typeKey]

            // 查找并更新现有配置
            for (i in 0..configList.size) {
                if (configList[i].name == config.name) {
                    configList[i] = config
                    return
                }
            }

            // 如果没找到，添加新配置
            configList.add(config)
        }
    }

    /**
     * 从类型映射中删除配置
     */
    private func removeConfigFromTypeMapping(config: StorageConfigItem): Unit {
        let typeKey = config.configType.toString()
        if (configsByType.contains(typeKey)) {
            let configList = configsByType[typeKey]
            let newList = ArrayList<StorageConfigItem>()

            for (existingConfig in configList) {
                if (existingConfig.name != config.name) {
                    newList.add(existingConfig)
                }
            }

            configsByType[typeKey] = newList
        }
    }
}
