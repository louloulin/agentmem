/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 核心类型定义 - AgentMem 仓颉 SDK 的基础数据类型
 */

package agentmem.core

/// 记忆类型枚举
public enum MemoryType {
    | Episodic      // 情景记忆
    | Semantic      // 语义记忆
    | Procedural    // 程序记忆
    | Working       // 工作记忆

    /// 转换为字符串表示
    public func toString(): String {
        return match (this) {
            case Episodic => "episodic"
            case Semantic => "semantic"
            case Procedural => "procedural"
            case Working => "working"
        }
    }

    /// 比较操作
    public operator func ==(right: MemoryType): Bool {
        return match (this) {
            case Episodic => match (right) {
                case Episodic => true
                case _ => false
            }
            case Semantic => match (right) {
                case Semantic => true
                case _ => false
            }
            case Procedural => match (right) {
                case Procedural => true
                case _ => false
            }
            case Working => match (right) {
                case Working => true
                case _ => false
            }
        }
    }

    /// 从字符串创建记忆类型
    public static func fromString(s: String): Option<MemoryType> {
        return match (s) {
            case "episodic" => Some(Episodic)
            case "semantic" => Some(Semantic)
            case "procedural" => Some(Procedural)
            case "working" => Some(Working)
            case _ => None
        }
    }

    /// 转换为数值
    public func toUInt32(): UInt32 {
        return match (this) {
            case Episodic => 1
            case Semantic => 2
            case Procedural => 3
            case Working => 4
        }
    }
}

/// 重要性级别枚举
public enum ImportanceLevel {
    | Low           // 低重要性
    | Medium        // 中等重要性
    | High          // 高重要性
    | Critical      // 关键重要性

    /// 转换为浮点数表示
    public func toFloat32(): Float32 {
        return match (this) {
            case Low => 0.25f32
            case Medium => 0.5f32
            case High => 0.75f32
            case Critical => 1.0f32
        }
    }

    /// 从分数值创建重要性级别
    public static func fromScore(score: Float32): ImportanceLevel {
        if (score >= 0.8f32) {
            return Critical
        } else if (score >= 0.6f32) {
            return High
        } else if (score >= 0.4f32) {
            return Medium
        } else {
            return Low
        }
    }

    /// 转换为字符串
    public func toString(): String {
        return match (this) {
            case Low => "low"
            case Medium => "medium"
            case High => "high"
            case Critical => "critical"
        }
    }
}

/// 搜索过滤器
public class SearchFilter {
    public var memoryTypes: Option<Array<MemoryType>> = None
    public var importanceRange: Option<(Float32, Float32)> = None
    public var timeRange: Option<(UInt64, UInt64)> = None
    public var agentId: Option<String> = None
    public var userId: Option<String> = None
    public var sessionId: Option<String> = None
    public var tags: Option<Array<String>> = None
    public var limit: Option<UInt32> = None

    public init() {}
}

/// 记忆搜索结果
public class MemorySearchResult {
    public var memory: Memory
    public var score: Float32
    public var relevance: Float32

    public init(memory: Memory, score: Float32, relevance: Float32) {
        this.memory = memory
        this.score = score
        this.relevance = relevance
    }
}
