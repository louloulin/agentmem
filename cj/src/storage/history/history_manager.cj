/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.storage.history

import std.collection.HashMap
import std.collection.ArrayList
import std.time.DateTime
import contextengine.models.*

/**
 * 历史记录结构
 * 对应Mem0的SQLiteManager中的history表结构
 */
public struct HistoryRecord {
    public let id: String                    // 历史记录唯一ID
    public let memoryId: String              // 关联的记忆ID
    public let oldMemory: Option<String>     // 旧记忆内容
    public let newMemory: Option<String>     // 新记忆内容
    public let event: String                 // 事件类型 (ADD, UPDATE, DELETE)
    public let createdAt: String             // 创建时间
    public let updatedAt: String             // 更新时间
    public let isDeleted: Int64              // 是否已删除 (0=否, 1=是)
    public let actorId: Option<String>       // 操作者ID
    public let role: Option<String>          // 操作者角色
    
    /**
     * 完整构造函数
     */
    public init(id: String, memoryId: String, oldMemory: Option<String>, 
                newMemory: Option<String>, event: String, createdAt: String, 
                updatedAt: String, isDeleted: Int64, actorId: Option<String>, 
                role: Option<String>) {
        this.id = id
        this.memoryId = memoryId
        this.oldMemory = oldMemory
        this.newMemory = newMemory
        this.event = event
        this.createdAt = createdAt
        this.updatedAt = updatedAt
        this.isDeleted = isDeleted
        this.actorId = actorId
        this.role = role
    }
    
    /**
     * 简化构造函数
     */
    public init(memoryId: String, event: String, newMemory: Option<String>) {
        let now = DateTime.now()
        let timestamp = formatDateTime(now)
        
        this.id = generateHistoryId()
        this.memoryId = memoryId
        this.oldMemory = None
        this.newMemory = newMemory
        this.event = event
        this.createdAt = timestamp
        this.updatedAt = timestamp
        this.isDeleted = 0
        this.actorId = None
        this.role = None
    }
    
    /**
     * 转换为字符串表示
     */
    public func toString(): String {
        return "HistoryRecord(id=${id}, memoryId=${memoryId}, event=${event})"
    }
}

/**
 * 历史记录管理器
 * 负责记忆操作的历史记录追踪，对应Mem0的SQLiteManager功能
 */
public class HistoryManager {
    private let historyRecords: HashMap<String, HistoryRecord>  // 内存存储历史记录
    private let memoryHistories: HashMap<String, ArrayList<String>>  // 记忆ID到历史记录ID的映射
    private var historyCounter: Int64
    
    /**
     * 构造函数
     */
    public init() {
        this.historyRecords = HashMap<String, HistoryRecord>()
        this.memoryHistories = HashMap<String, ArrayList<String>>()
        this.historyCounter = 0
    }
    
    /**
     * 添加历史记录
     * @param memoryId 记忆ID
     * @param oldMemory 旧记忆内容
     * @param newMemory 新记忆内容
     * @param event 事件类型
     * @param actorId 操作者ID
     * @param role 操作者角色
     * @return 是否成功
     */
    public func addHistory(memoryId: String, oldMemory: Option<String>,
                          newMemory: Option<String>, event: String,
                          actorId: Option<String>,
                          role: Option<String>): Bool {
        try {
            let now = DateTime.now()
            let timestamp = formatDateTime(now)
            
            // 生成历史记录ID
            historyCounter = historyCounter + 1
            let historyId = "history_" + historyCounter.toString()
            
            // 创建历史记录
            let historyRecord = HistoryRecord(
                historyId,
                memoryId,
                oldMemory,
                newMemory,
                event,
                timestamp,
                timestamp,
                0,
                actorId,
                role
            )
            
            // 存储历史记录
            historyRecords[historyId] = historyRecord
            
            // 更新记忆到历史记录的映射
            if (!memoryHistories.contains(memoryId)) {
                memoryHistories[memoryId] = ArrayList<String>()
            }
            memoryHistories[memoryId].add(historyId)
            
            return true
        } catch (e: Exception) {
            return false
        }
    }

    /**
     * 添加历史记录（简化版本）
     * @param memoryId 记忆ID
     * @param oldMemory 旧记忆内容
     * @param newMemory 新记忆内容
     * @param event 事件类型
     * @return 是否成功
     */
    public func addHistory(memoryId: String, oldMemory: Option<String>,
                          newMemory: Option<String>, event: String): Bool {
        return addHistory(memoryId, oldMemory, newMemory, event, None, None)
    }

    /**
     * 获取记忆的历史记录
     * @param memoryId 记忆ID
     * @return 历史记录列表
     */
    public func getHistory(memoryId: String): Array<HistoryRecord> {
        let resultList = ArrayList<HistoryRecord>()
        
        if (memoryHistories.contains(memoryId)) {
            let historyIds = memoryHistories[memoryId]
            for (historyId in historyIds) {
                if (historyRecords.contains(historyId)) {
                    resultList.add(historyRecords[historyId])
                }
            }
        }
        
        return resultList.toArray()
    }
    
    /**
     * 获取所有历史记录
     * @return 所有历史记录
     */
    public func getAllHistory(): Array<HistoryRecord> {
        let resultList = ArrayList<HistoryRecord>()
        for ((_, record) in historyRecords) {
            resultList.add(record)
        }
        return resultList.toArray()
    }
    
    /**
     * 删除记忆的历史记录
     * @param memoryId 记忆ID
     * @return 是否成功
     */
    public func deleteHistory(memoryId: String): Bool {
        try {
            if (memoryHistories.contains(memoryId)) {
                let historyIds = memoryHistories[memoryId]
                for (historyId in historyIds) {
                    historyRecords.remove(historyId)
                }
                memoryHistories.remove(memoryId)
            }
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 重置所有历史记录
     * @return 是否成功
     */
    public func reset(): Bool {
        try {
            historyRecords.clear()
            memoryHistories.clear()
            historyCounter = 0
            return true
        } catch (e: Exception) {
            return false
        }
    }
    
    /**
     * 获取历史记录数量
     * @return 历史记录总数
     */
    public func getHistoryCount(): Int64 {
        return historyRecords.size
    }
}

/**
 * 生成历史记录ID的辅助函数
 */
private func generateHistoryId(): String {
    let now = DateTime.now()
    let timestamp = now.year * 10000000000 + Int64(now.month.toInteger()) * 100000000 + 
                   now.dayOfMonth * 1000000 + now.hour * 10000 + now.minute * 100 + now.second
    return "hist_" + timestamp.toString()
}

/**
 * 格式化日期时间的辅助函数
 */
private func formatDateTime(dateTime: DateTime): String {
    return dateTime.year.toString() + "-" +
           padLeft(dateTime.month.toInteger().toString(), 2) + "-" +
           padLeft(dateTime.dayOfMonth.toString(), 2) + " " +
           padLeft(dateTime.hour.toString(), 2) + ":" +
           padLeft(dateTime.minute.toString(), 2) + ":" +
           padLeft(dateTime.second.toString(), 2)
}

/**
 * 左填充字符串到指定长度
 */
private func padLeft(str: String, length: Int64): String {
    if (str.size >= length) {
        return str
    }
    let padding = length - str.size
    var result = ""
    for (i in 0..padding) {
        result = result + "0"
    }
    return result + str
}
