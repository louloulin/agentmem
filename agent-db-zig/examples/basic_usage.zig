// Agent状态数据库基础使用示例
const std = @import("std");
const agent_api = @import("agent_api");
const AgentDB = agent_api.AgentDatabase;
const StateType = agent_api.StateType;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("=== Agent状态数据库基础使用示例 ===\n\n", .{});

    // 1. 初始化数据库
    std.debug.print("1. 初始化数据库...\n", .{});
    const db_path = "example_agent.db";
    var db = AgentDB.init(allocator, db_path) catch |err| {
        std.debug.print("初始化数据库失败: {}\n", .{err});
        return;
    };
    defer db.deinit();
    std.debug.print("   数据库初始化成功: {s}\n\n", .{db_path});

    // 2. 创建和保存Agent状态
    std.debug.print("2. 创建和保存Agent状态...\n", .{});
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
        const state = agent_api.AgentState.init(agent_id, session_id, state_info[0], state_info[1]);
        db.saveState(state) catch |err| {
            std.debug.print("   保存状态失败: {}\n", .{err});
            continue;
        };
        std.debug.print("   保存状态成功: {} - {s}\n", .{ state_info[0], state_info[1] });
    }
    std.debug.print("\n", .{});

    // 3. 加载和显示状态
    std.debug.print("3. 加载Agent状态...\n", .{});
    const loaded_state = db.loadState(agent_id) catch |err| {
        std.debug.print("   加载状态失败: {}\n", .{err});
        return;
    };

    if (loaded_state) |state_data| {
        defer allocator.free(state_data);
        std.debug.print("   加载的状态数据: {s}\n", .{state_data});
    } else {
        std.debug.print("   未找到Agent状态\n", .{});
    }
    std.debug.print("\n", .{});

    // 4. 测试记忆功能
    std.debug.print("4. 测试记忆功能...\n", .{});
    const memory = agent_api.Memory.init(agent_id, agent_api.MemoryType.episodic, "这是一个测试记忆", 0.8);
    db.storeMemory(memory) catch |err| {
        std.debug.print("   存储记忆失败: {}\n", .{err});
    };
    std.debug.print("   记忆存储完成\n", .{});

    const memory_count = db.retrieveMemories(agent_id, 10) catch 0;
    std.debug.print("   检索到 {} 个记忆\n", .{memory_count});
    std.debug.print("\n", .{});

    // 5. 测试 RAG 功能
    std.debug.print("5. 测试 RAG 功能...\n", .{});
    const document = agent_api.Document.init("测试文档", "这是一个测试文档的内容，用于演示 RAG 功能。", 512, 50);
    db.indexDocument(document) catch |err| {
        std.debug.print("   索引文档失败: {}\n", .{err});
    };
    std.debug.print("   文档索引完成\n", .{});

    const search_results = db.searchText("测试", 5) catch 0;
    std.debug.print("   搜索到 {} 个结果\n", .{search_results});

    const context = db.buildContext("测试查询", 1000) catch |err| blk: {
        std.debug.print("   构建上下文失败: {}\n", .{err});
        break :blk try allocator.dupe(u8, "");
    };
    defer allocator.free(context);
    std.debug.print("   构建的上下文长度: {}\n", .{context.len});

    std.debug.print("\n", .{});

    // 6. 测试完成
    std.debug.print("6. 测试完成\n", .{});
    std.debug.print("✅ AgentDB Zig API 基础功能测试完成！\n", .{});

    // 清理测试文件
    std.fs.cwd().deleteFile(db_path) catch {};
    std.debug.print("测试文件已清理\n", .{});
}
