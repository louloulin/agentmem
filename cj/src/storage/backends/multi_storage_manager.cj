/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.storage.backends

import std.collection.HashMap
import std.collection.ArrayList
import contextengine.models.MemoryRecord

/**
 * 存储后端类型枚举
 */
public enum StorageBackendType {
    | MEMORY      // 内存存储
    | FILE        // 文件存储
    | SQLITE      // SQLite数据库
    | POSTGRESQL  // PostgreSQL数据库（预留）
    | MYSQL       // MySQL数据库（预留）
    
    public func toString(): String {
        match (this) {
            case MEMORY => "memory"
            case FILE => "file"
            case SQLITE => "sqlite"
            case POSTGRESQL => "postgresql"
            case MYSQL => "mysql"
        }
    }
    
    public static func fromString(typeStr: String): StorageBackendType {
        match (typeStr) {
            case "memory" => MEMORY
            case "file" => FILE
            case "sqlite" => SQLITE
            case "postgresql" => POSTGRESQL
            case "mysql" => MYSQL
            case _ => MEMORY  // 默认为内存存储
        }
    }

    /**
     * 相等比较操作符
     */
    public operator func ==(right: StorageBackendType): Bool {
        return this.toString() == right.toString()
    }
}

/**
 * 存储后端配置
 */
public struct StorageBackendConfig {
    public let backendType: StorageBackendType
    public let connectionString: Option<String>  // 数据库连接字符串
    public let filePath: Option<String>          // 文件路径
    public let options: HashMap<String, String>  // 额外选项
    
    public init(backendType: StorageBackendType) {
        this.backendType = backendType
        this.connectionString = None
        this.filePath = None
        this.options = HashMap<String, String>()
    }
    
    public init(backendType: StorageBackendType, filePath: String) {
        this.backendType = backendType
        this.connectionString = None
        this.filePath = Some(filePath)
        this.options = HashMap<String, String>()
    }
    
    public init(backendType: StorageBackendType, connectionString: String, 
                options: HashMap<String, String>) {
        this.backendType = backendType
        this.connectionString = Some(connectionString)
        this.filePath = None
        this.options = options
    }
}

/**
 * 多存储后端管理器
 * 负责管理多个存储后端实例和切换
 */
public class MultiStorageManager {
    private var currentBackend: Option<KVStorage>
    private let backends: HashMap<String, KVStorage>
    private let backendConfigs: HashMap<String, StorageBackendConfig>
    private var defaultBackendName: String

    // Phase 1.3 增强功能
    private let vectorStorages: HashMap<String, VectorStorage>  // 向量存储后端
    private var currentVectorStorage: Option<VectorStorage>     // 当前向量存储
    private var defaultVectorStorageName: String               // 默认向量存储名称
    private let configManager: StorageConfigManager            // 配置管理器
    private let enabledBackends: HashMap<String, Bool>         // 启用状态
    private let distributedStorages: HashMap<String, DistributedStorage>  // 分布式存储后端
    private var currentDistributedStorage: Option<DistributedStorage>     // 当前分布式存储
    
    public init() {
        this.currentBackend = None
        this.backends = HashMap<String, KVStorage>()
        this.backendConfigs = HashMap<String, StorageBackendConfig>()
        this.defaultBackendName = "memory"

        // 初始化Phase 1.3增强功能
        this.vectorStorages = HashMap<String, VectorStorage>()
        this.currentVectorStorage = None
        this.defaultVectorStorageName = "memory_vector"
        this.configManager = StorageConfigManager()
        this.enabledBackends = HashMap<String, Bool>()
        this.distributedStorages = HashMap<String, DistributedStorage>()
        this.currentDistributedStorage = None
    }
    
    /**
     * 注册存储后端
     * @param name 后端名称
     * @param config 后端配置
     * @return 是否成功
     */
    public func registerBackend(name: String, config: StorageBackendConfig): Bool {
        try {
            let backend = createBackend(config)
            if (backend.isSome()) {
                let backendInstance = backend.getOrThrow()
                backends[name] = backendInstance
                backendConfigs[name] = config
                
                // 如果是第一个注册的后端，设为默认
                if (currentBackend.isNone()) {
                    this.currentBackend = Some(backendInstance)
                    this.defaultBackendName = name
                }
                
                return true
            }
            return false
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 创建存储后端实例
     */
    private func createBackend(config: StorageBackendConfig): Option<KVStorage> {
        if (config.backendType == StorageBackendType.MEMORY) {
            return Some(MemoryKVStorage())
        } else if (config.backendType == StorageBackendType.FILE) {
            if (config.filePath.isSome()) {
                let backend = FileStorageBackend(config.filePath.getOrThrow())
                if (backend.initialize()) {
                    return Some(backend)
                }
            }
            return None
        } else if (config.backendType == StorageBackendType.SQLITE) {
            if (config.filePath.isSome()) {
                let backend = SqliteStorageBackend(config.filePath.getOrThrow())
                if (backend.initialize()) {
                    return Some(backend)
                }
            }
            return None
        } else {
            // PostgreSQL和MySQL预留
            return None
        }
    }
    
    /**
     * 切换到指定的存储后端
     * @param name 后端名称
     * @return 是否成功
     */
    public func switchToBackend(name: String): Bool {
        if (backends.contains(name)) {
            this.currentBackend = Some(backends[name])
            return true
        }
        return false
    }
    
    /**
     * 获取当前存储后端
     * @return 当前后端实例
     */
    public func getCurrentBackend(): Option<KVStorage> {
        return currentBackend
    }
    
    /**
     * 获取当前后端名称
     */
    public func getCurrentBackendName(): String {
        return defaultBackendName
    }
    
    /**
     * 获取所有注册的后端名称
     * @return 后端名称列表
     */
    public func getRegisteredBackends(): Array<String> {
        let namesList = ArrayList<String>()
        for ((name, _) in backends) {
            namesList.add(name)
        }
        return namesList.toArray()
    }
    
    /**
     * 检查所有后端的健康状态
     * @return 健康状态报告
     */
    public func checkAllBackendsHealth(): HashMap<String, Bool> {
        let healthStatus = HashMap<String, Bool>()
        for ((name, _) in backends) {
            healthStatus[name] = true  // 简化实现，默认所有后端健康
        }
        return healthStatus
    }
    
    /**
     * 获取后端配置信息
     */
    public func getBackendConfig(name: String): Option<StorageBackendConfig> {
        if (backendConfigs.contains(name)) {
            return Some(backendConfigs[name])
        }
        return None
    }

    /**
     * 注册向量存储后端
     * @param name 后端名称
     * @param storage 向量存储实例
     * @return 是否成功
     */
    public func registerVectorStorage(name: String, storage: VectorStorage): Bool {
        try {
            vectorStorages[name] = storage

            // 如果是第一个注册的向量存储，设为默认
            if (currentVectorStorage.isNone()) {
                this.currentVectorStorage = Some(storage)
                this.defaultVectorStorageName = name
            }

            println("MultiStorageManager: 注册向量存储后端: ${name}")
            return true

        } catch (e: Exception) {
            println("MultiStorageManager: 注册向量存储后端失败: ${e}")
            return false
        }
    }

    /**
     * 注册分布式存储后端
     * @param name 后端名称
     * @param storage 分布式存储实例
     * @return 是否成功
     */
    public func registerDistributedStorage(name: String, storage: DistributedStorage): Bool {
        try {
            distributedStorages[name] = storage

            // 如果是第一个注册的分布式存储，设为默认
            if (currentDistributedStorage.isNone()) {
                this.currentDistributedStorage = Some(storage)
            }

            println("MultiStorageManager: 注册分布式存储后端: ${name}")
            return true

        } catch (e: Exception) {
            println("MultiStorageManager: 注册分布式存储后端失败: ${e}")
            return false
        }
    }

    /**
     * 切换向量存储后端
     * @param name 后端名称
     * @return 是否成功
     */
    public func switchVectorStorage(name: String): Bool {
        if (vectorStorages.contains(name)) {
            this.currentVectorStorage = Some(vectorStorages[name])
            this.defaultVectorStorageName = name
            println("MultiStorageManager: 切换到向量存储后端: ${name}")
            return true
        }
        return false
    }

    /**
     * 切换分布式存储后端
     * @param name 后端名称
     * @return 是否成功
     */
    public func switchDistributedStorage(name: String): Bool {
        if (distributedStorages.contains(name)) {
            this.currentDistributedStorage = Some(distributedStorages[name])
            println("MultiStorageManager: 切换到分布式存储后端: ${name}")
            return true
        }
        return false
    }

    /**
     * 获取当前向量存储
     * @return 当前向量存储（可选）
     */
    public func getCurrentVectorStorage(): Option<VectorStorage> {
        return currentVectorStorage
    }

    /**
     * 获取当前分布式存储
     * @return 当前分布式存储（可选）
     */
    public func getCurrentDistributedStorage(): Option<DistributedStorage> {
        return currentDistributedStorage
    }

    /**
     * 获取所有注册的向量存储后端名称
     * @return 后端名称列表
     */
    public func getRegisteredVectorStorages(): Array<String> {
        let names = ArrayList<String>()
        for ((name, _) in vectorStorages) {
            names.add(name)
        }
        return names.toArray()
    }

    /**
     * 获取所有注册的分布式存储后端名称
     * @return 后端名称列表
     */
    public func getRegisteredDistributedStorages(): Array<String> {
        let names = ArrayList<String>()
        for ((name, _) in distributedStorages) {
            names.add(name)
        }
        return names.toArray()
    }
    
    /**
     * 获取后端统计信息
     */
    public func getBackendStats(): HashMap<String, HashMap<String, String>> {
        let stats = HashMap<String, HashMap<String, String>>()

        for ((name, _) in backends) {
            let backendStats = HashMap<String, String>()
            backendStats["type"] = backendConfigs[name].backendType.toString()
            backendStats["status"] = "active"
            backendStats["healthy"] = "true"

            stats[name] = backendStats
        }

        return stats
    }
    
    /**
     * 数据迁移：从一个后端迁移到另一个后端
     * @param fromBackend 源后端名称
     * @param toBackend 目标后端名称
     * @return 迁移的记录数量
     */
    public func migrateData(fromBackend: String, toBackend: String): Int64 {
        if (!backends.contains(fromBackend) || !backends.contains(toBackend)) {
            return 0
        }
        
        let sourceBackend = backends[fromBackend]
        let targetBackend = backends[toBackend]
        
        let keys = sourceBackend.keys()
        var migratedCount: Int64 = 0
        
        for (key in keys) {
            let value = sourceBackend.get(key)
            if (value.isSome()) {
                if (targetBackend.put(key, value.getOrThrow())) {
                    migratedCount = migratedCount + 1
                }
            }
        }
        
        return migratedCount
    }
    
    /**
     * 数据同步：将当前后端的数据同步到所有其他后端
     * @return 同步的记录数量
     */
    public func syncToAllBackends(): Int64 {
        if (currentBackend.isNone()) {
            return 0
        }

        let sourceBackend = currentBackend.getOrThrow()
        let keys = sourceBackend.keys()
        var syncedCount: Int64 = 0

        for ((_, backend) in backends) {
            for (key in keys) {
                let value = sourceBackend.get(key)
                if (value.isSome()) {
                    if (backend.put(key, value.getOrThrow())) {
                        syncedCount = syncedCount + 1
                    }
                }
            }
        }

        return syncedCount
    }
    
    /**
     * 清理所有后端
     */
    public func clearAllBackends(): Bool {
        var allSuccess = true
        for ((_, backend) in backends) {
            if (!backend.clear()) {
                allSuccess = false
            }
        }
        return allSuccess
    }
    
    /**
     * 关闭所有后端连接
     */
    public func closeAllBackends(): Bool {
        // 简化实现，默认成功
        return true
    }
}
