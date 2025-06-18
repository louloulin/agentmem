// Agent状态数据库基础使用示例
const std = @import("std");
const AgentDB = @import("../src/agent_db.zig").AgentDB;
const StateType = @import("../src/agent_state.zig").StateType;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("=== Agent状态数据库基础使用示例 ===\n\n");

    // 1. 初始化数据库
    std.debug.print("1. 初始化数据库...\n");
    const db_path = "example_agent.db";
    var db = AgentDB.init(db_path, allocator) catch |err| {
        std.debug.print("初始化数据库失败: {}\n", .{err});
        return;
    };
    defer db.deinit();
    std.debug.print("   数据库初始化成功: {s}\n\n", .{db_path});

    // 2. 创建和保存Agent状态
    std.debug.print("2. 创建和保存Agent状态...\n");
    const agent_id: u64 = 12345;
    const session_id: u64 = 67890;
    
    // 保存不同类型的状态
    const states = [_]struct { StateType, []const u8 }{
        .{ StateType.context, "用户正在询问关于天气的问题" },
        .{ StateType.working_memory, "当前对话轮次: 3, 主题: 天气查询" },
        .{ StateType.long_term_memory, "用户偏好: 喜欢详细的天气信息" },
        .{ StateType.task_state, "任务: 获取北京天气, 状态: 进行中" },
    };

    for (states) |state_info| {
        db.saveAgentState(agent_id, session_id, state_info[0], state_info[1]) catch |err| {
            std.debug.print("   保存状态失败: {}\n", .{err});
            continue;
        };
        std.debug.print("   保存状态成功: {} - {s}\n", .{ state_info[0], state_info[1] });
    }
    std.debug.print("\n");

    // 3. 加载和显示状态
    std.debug.print("3. 加载Agent状态...\n");
    const loaded_state = db.loadAgentState(agent_id) catch |err| {
        std.debug.print("   加载状态失败: {}\n", .{err});
        return;
    };

    if (loaded_state) |state| {
        defer {
            var mutable_state = state;
            mutable_state.deinit(allocator);
        }
        
        std.debug.print("   Agent ID: {}\n", .{state.agent_id});
        std.debug.print("   Session ID: {}\n", .{state.session_id});
        std.debug.print("   状态类型: {}\n", .{state.state_type});
        std.debug.print("   时间戳: {}\n", .{state.timestamp});
        std.debug.print("   版本: {}\n", .{state.version});
        std.debug.print("   数据: {s}\n", .{state.data});
        std.debug.print("   校验和有效: {}\n", .{state.validateChecksum()});
    } else {
        std.debug.print("   未找到Agent状态\n");
    }
    std.debug.print("\n");

    // 4. 设置和获取元数据
    std.debug.print("4. 设置和获取元数据...\n");
    db.setStateMetadata(agent_id, "priority", "high") catch |err| {
        std.debug.print("   设置元数据失败: {}\n", .{err});
    };
    db.setStateMetadata(agent_id, "category", "weather_query") catch |err| {
        std.debug.print("   设置元数据失败: {}\n", .{err});
    };
    db.setStateMetadata(agent_id, "user_id", "user_001") catch |err| {
        std.debug.print("   设置元数据失败: {}\n", .{err});
    };

    const metadata_keys = [_][]const u8{ "priority", "category", "user_id", "nonexistent" };
    for (metadata_keys) |key| {
        const value = db.getStateMetadata(agent_id, key) catch null;
        if (value) |v| {
            std.debug.print("   {s}: {s}\n", .{ key, v });
        } else {
            std.debug.print("   {s}: (未设置)\n", .{key});
        }
    }
    std.debug.print("\n");

    // 5. 更新状态
    std.debug.print("5. 更新Agent状态...\n");
    const updated_data = "用户询问明天北京的天气，需要包含温度和降雨概率";
    db.updateAgentState(agent_id, updated_data) catch |err| {
        std.debug.print("   更新状态失败: {}\n", .{err});
    };

    const updated_state = db.loadAgentState(agent_id) catch null;
    if (updated_state) |state| {
        defer {
            var mutable_state = state;
            mutable_state.deinit(allocator);
        }
        std.debug.print("   更新后的数据: {s}\n", .{state.data});
        std.debug.print("   新版本号: {}\n", .{state.version});
    }
    std.debug.print("\n");

    // 6. 创建快照
    std.debug.print("6. 创建状态快照...\n");
    db.createStateSnapshot(agent_id, "weather_query_v1") catch |err| {
        std.debug.print("   创建快照失败: {}\n", .{err});
    };
    std.debug.print("   快照 'weather_query_v1' 创建成功\n\n");

    // 7. 再次更新状态
    std.debug.print("7. 再次更新状态...\n");
    const new_data = "用户确认需要北京明天的详细天气预报，包含小时级别数据";
    db.updateAgentState(agent_id, new_data) catch |err| {
        std.debug.print("   更新状态失败: {}\n", .{err});
    };
    std.debug.print("   状态已更新为: {s}\n\n", .{new_data});

    // 8. 恢复到快照
    std.debug.print("8. 恢复到快照...\n");
    db.restoreFromSnapshot(agent_id, "weather_query_v1") catch |err| {
        std.debug.print("   恢复快照失败: {}\n", .{err});
    };

    const restored_state = db.loadAgentState(agent_id) catch null;
    if (restored_state) |state| {
        defer {
            var mutable_state = state;
            mutable_state.deinit(allocator);
        }
        std.debug.print("   恢复后的数据: {s}\n", .{state.data});
    }
    std.debug.print("\n");

    // 9. 获取状态统计信息
    std.debug.print("9. 获取状态统计信息...\n");
    const stats = db.getAgentStats(agent_id) catch |err| {
        std.debug.print("   获取统计信息失败: {}\n", .{err});
        return;
    };
    defer {
        var mutable_stats = stats;
        mutable_stats.deinit();
    }

    std.debug.print("   总状态数: {}\n", .{stats.total_states});
    std.debug.print("   总大小: {} 字节\n", .{stats.total_size});
    std.debug.print("   最早时间戳: {}\n", .{stats.oldest_timestamp});
    std.debug.print("   最新时间戳: {}\n", .{stats.newest_timestamp});

    var type_iterator = stats.state_types.iterator();
    while (type_iterator.next()) |entry| {
        std.debug.print("   状态类型 {}: {} 个\n", .{ entry.key_ptr.*, entry.value_ptr.* });
    }
    std.debug.print("\n");

    // 10. 压缩状态
    std.debug.print("10. 压缩Agent状态...\n");
    db.compressAgentStates(agent_id) catch |err| {
        std.debug.print("   压缩状态失败: {}\n", .{err});
    };
    std.debug.print("   状态压缩完成\n\n");

    // 11. 获取数据库信息
    std.debug.print("11. 获取数据库信息...\n");
    const db_info = db.getDatabaseInfo() catch |err| {
        std.debug.print("   获取数据库信息失败: {}\n", .{err});
        return;
    };

    std.debug.print("   数据库路径: {s}\n", .{db_info.db_path});
    std.debug.print("   版本: {s}\n", .{db_info.version});
    std.debug.print("   创建时间: {}\n", .{db_info.created_at});
    std.debug.print("   最后修改: {}\n", .{db_info.last_modified});
    std.debug.print("   总大小: {} 字节\n", .{db_info.total_size});
    std.debug.print("\n");

    // 12. 备份数据库
    std.debug.print("12. 备份数据库...\n");
    const backup_path = "example_agent_backup.db";
    db.backup(backup_path) catch |err| {
        std.debug.print("   备份失败: {}\n", .{err});
    };
    std.debug.print("   数据库已备份到: {s}\n\n", .{backup_path});

    // 13. 演示多Agent场景
    std.debug.print("13. 演示多Agent场景...\n");
    const agent_ids = [_]u64{ 11111, 22222, 33333 };
    const agent_names = [_][]const u8{ "天气助手", "翻译助手", "计算助手" };

    for (agent_ids, agent_names) |id, name| {
        const data = try std.fmt.allocPrint(allocator, "我是{s}，Agent ID: {}", .{ name, id });
        defer allocator.free(data);
        
        db.saveAgentState(id, session_id, StateType.context, data) catch |err| {
            std.debug.print("   保存Agent {}状态失败: {}\n", .{ id, err });
            continue;
        };
        std.debug.print("   保存Agent {}: {s}\n", .{ id, data });
    }
    std.debug.print("\n");

    // 14. 清理演示
    std.debug.print("14. 清理演示数据...\n");
    
    // 清理旧状态（保留最近7天）
    db.cleanupOldStates(agent_id, 7) catch |err| {
        std.debug.print("   清理旧状态失败: {}\n", .{err});
    };
    std.debug.print("   旧状态清理完成\n");

    // 删除测试文件
    std.fs.cwd().deleteFile(db_path) catch {};
    std.fs.cwd().deleteFile(backup_path) catch {};
    std.debug.print("   测试文件已清理\n\n");

    std.debug.print("=== 示例程序执行完成 ===\n");
}

// 错误处理辅助函数
fn handleError(err: anyerror, operation: []const u8) void {
    std.debug.print("操作 '{s}' 失败: {}\n", .{ operation, err });
}

// 格式化时间戳
fn formatTimestamp(timestamp: i64, allocator: std.mem.Allocator) ![]u8 {
    // 简单的时间戳格式化
    return try std.fmt.allocPrint(allocator, "{}", .{timestamp});
}

// 演示状态类型转换
fn demonstrateStateTypes() void {
    std.debug.print("\n=== 状态类型演示 ===\n");
    
    const state_types = [_]StateType{
        .working_memory,
        .long_term_memory,
        .context,
        .task_state,
        .relationship,
        .embedding,
    };

    for (state_types) |state_type| {
        const type_str = state_type.toString();
        const parsed_type = StateType.fromString(type_str);
        
        std.debug.print("状态类型: {} -> '{s}' -> {?}\n", .{ state_type, type_str, parsed_type });
    }
    std.debug.print("\n");
}
