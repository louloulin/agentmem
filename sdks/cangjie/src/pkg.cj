/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * AgentMem 仓颉 SDK 包定义
 * 
 * 这个文件定义了AgentMem SDK的包结构和导出的公共API。
 * 当前版本专注于核心功能，FFI绑定将在后续版本中完善。
 */

package agentmem

// 导出核心类型和功能
public import agentmem.core.*
public import agentmem.utils.*

// 导出API功能
public import agentmem.api.*

// 导出FFI功能（暂时禁用，需要完善集成）
// public import agentmem.ffi.*

// 导出示例代码（暂时禁用，需要修复语法问题）
// public import agentmem.examples.*
