// Agent状态数据库测试
const std = @import("std");
const testing = std.testing;
const AgentDB = @import("agent_db.zig").AgentDB;
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;

// 测试基础功能
test "AgentDB basic functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // 创建临时数据库
    const db_path = "test_agent.db";
    var db = try AgentDB.init(db_path, allocator);
    defer db.deinit();

    // 测试保存状态
    const agent_id: u64 = 12345;
    const session_id: u64 = 67890;
    const test_data = "Hello, Agent State!";
    
    try db.saveAgentState(agent_id, session_id, StateType.context, test_data);

    // 测试加载状态
    const loaded_state = try db.loadAgentState(agent_id);
    try testing.expect(loaded_state != null);
    
    if (loaded_state) |state| {
        defer {
            var mutable_state = state;
            mutable_state.deinit(allocator);
        }
        
        try testing.expectEqual(agent_id, state.agent_id);
        try testing.expectEqual(session_id, state.session_id);
        try testing.expectEqual(StateType.context, state.state_type);
        try testing.expect(state.validateChecksum());
    }

    // 清理测试文件
    std.fs.cwd().deleteFile(db_path) catch {};
}

// 测试状态更新
test "AgentState update functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const db_path = "test_update.db";
    var db = try AgentDB.init(db_path, allocator);
    defer db.deinit();

    const agent_id: u64 = 11111;
    const session_id: u64 = 22222;
    const initial_data = "Initial state";
    const updated_data = "Updated state";

    // 保存初始状态
    try db.saveAgentState(agent_id, session_id, StateType.working_memory, initial_data);

    // 更新状态
    try db.updateAgentState(agent_id, updated_data);

    // 验证更新
    const loaded_state = try db.loadAgentState(agent_id);
    try testing.expect(loaded_state != null);
    
    if (loaded_state) |state| {
        defer {
            var mutable_state = state;
            mutable_state.deinit(allocator);
        }
        
        try testing.expect(std.mem.eql(u8, state.data, updated_data));
        try testing.expect(state.version > 1); // 版本应该增加
    }

    std.fs.cwd().deleteFile(db_path) catch {};
}

// 测试元数据功能
test "AgentState metadata functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const db_path = "test_metadata.db";
    var db = try AgentDB.init(db_path, allocator);
    defer db.deinit();

    const agent_id: u64 = 33333;
    const session_id: u64 = 44444;
    const test_data = "Test data with metadata";

    // 保存状态
    try db.saveAgentState(agent_id, session_id, StateType.long_term_memory, test_data);

    // 设置元数据
    try db.setStateMetadata(agent_id, "priority", "high");
    try db.setStateMetadata(agent_id, "category", "important");

    // 获取元数据
    const priority = try db.getStateMetadata(agent_id, "priority");
    const category = try db.getStateMetadata(agent_id, "category");
    const nonexistent = try db.getStateMetadata(agent_id, "nonexistent");

    try testing.expect(priority != null);
    try testing.expect(category != null);
    try testing.expect(nonexistent == null);

    if (priority) |p| {
        try testing.expect(std.mem.eql(u8, p, "high"));
    }

    if (category) |c| {
        try testing.expect(std.mem.eql(u8, c, "important"));
    }

    std.fs.cwd().deleteFile(db_path) catch {};
}

// 测试快照功能
test "AgentState snapshot functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const db_path = "test_snapshot.db";
    var db = try AgentDB.init(db_path, allocator);
    defer db.deinit();

    const agent_id: u64 = 55555;
    const session_id: u64 = 66666;
    const initial_data = "Initial snapshot data";
    const modified_data = "Modified data";

    // 保存初始状态
    try db.saveAgentState(agent_id, session_id, StateType.task_state, initial_data);

    // 创建快照
    try db.createStateSnapshot(agent_id, "backup_v1");

    // 修改状态
    try db.updateAgentState(agent_id, modified_data);

    // 验证状态已修改
    const modified_state = try db.loadAgentState(agent_id);
    try testing.expect(modified_state != null);
    
    if (modified_state) |state| {
        defer {
            var mutable_state = state;
            mutable_state.deinit(allocator);
        }
        try testing.expect(std.mem.eql(u8, state.data, modified_data));
    }

    // 恢复到快照
    try db.restoreFromSnapshot(agent_id, "backup_v1");

    // 验证状态已恢复
    const restored_state = try db.loadAgentState(agent_id);
    try testing.expect(restored_state != null);
    
    if (restored_state) |state| {
        defer {
            var mutable_state = state;
            mutable_state.deinit(allocator);
        }
        try testing.expect(std.mem.eql(u8, state.data, initial_data));
    }

    std.fs.cwd().deleteFile(db_path) catch {};
}

// 测试状态压缩
test "AgentState compression functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // 创建一个包含重复数据的状态
    var state = try AgentState.init(allocator, 77777, 88888, StateType.context, "AAAAAABBBBBBCCCCCC");
    defer state.deinit(allocator);

    const original_size = state.data.len;
    
    // 压缩状态
    try state.compress(allocator);
    
    // 检查是否有压缩标记
    const is_compressed = state.getMetadata("compressed");
    if (is_compressed) |compressed| {
        try testing.expect(std.mem.eql(u8, compressed, "true"));
        
        // 解压缩
        try state.decompress(allocator);
        
        // 验证数据完整性
        try testing.expect(std.mem.eql(u8, state.data, "AAAAAABBBBBBCCCCCC"));
        try testing.expectEqual(original_size, state.data.len);
    }
}

// 测试状态验证
test "AgentState validation functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var state = try AgentState.init(allocator, 99999, 11111, StateType.embedding, "Test validation data");
    defer state.deinit(allocator);

    // 验证初始状态
    try testing.expect(state.validateChecksum());

    // 手动破坏校验和
    state.checksum = 0;
    try testing.expect(!state.validateChecksum());

    // 重新计算校验和
    var checksum: u32 = 0;
    for (state.data) |byte| {
        checksum = checksum +% byte;
    }
    state.checksum = checksum;
    
    try testing.expect(state.validateChecksum());
}

// 测试状态比较
test "AgentState comparison functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var state1 = try AgentState.init(allocator, 12345, 67890, StateType.context, "Same data");
    defer state1.deinit(allocator);

    var state2 = try AgentState.init(allocator, 12345, 67890, StateType.context, "Same data");
    defer state2.deinit(allocator);

    var state3 = try AgentState.init(allocator, 12345, 67890, StateType.context, "Different data");
    defer state3.deinit(allocator);

    // 相同状态应该相等
    try testing.expect(state1.equals(&state2));

    // 不同数据的状态应该不相等
    try testing.expect(!state1.equals(&state3));
}

// 测试C FFI接口
test "C FFI interface" {
    const c_db_path = "test_c_ffi.db";
    
    // 测试初始化
    const db = @import("agent_db.zig").agent_db_init(c_db_path);
    try testing.expect(db != null);
    
    if (db) |database| {
        defer @import("agent_db.zig").agent_db_deinit(database);
        
        // 测试保存状态
        const test_data = "C FFI test data";
        const result = @import("agent_db.zig").agent_db_save_state(
            database, 
            12345, 
            67890, 
            @intFromEnum(StateType.context), 
            test_data.ptr, 
            test_data.len
        );
        try testing.expectEqual(@as(c_int, 0), result);
        
        // 测试加载状态
        var data_ptr: [*]u8 = undefined;
        var data_len: usize = undefined;
        const load_result = @import("agent_db.zig").agent_db_load_state(database, 12345, &data_ptr, &data_len);
        try testing.expectEqual(@as(c_int, 0), load_result);
        
        // 清理数据
        @import("agent_db.zig").agent_db_free_data(data_ptr, data_len);
    }
    
    std.fs.cwd().deleteFile(c_db_path) catch {};
}

// 性能测试
test "Performance test - batch operations" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const db_path = "test_performance.db";
    var db = try AgentDB.init(db_path, allocator);
    defer db.deinit();

    const num_operations = 100;
    const start_time = std.time.nanoTimestamp();

    // 批量保存状态
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);
        const session_id = @as(u64, i * 2);
        const data = try std.fmt.allocPrint(allocator, "Test data for agent {}", .{i});
        defer allocator.free(data);
        
        try db.saveAgentState(agent_id, session_id, StateType.working_memory, data);
    }

    // 批量加载状态
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);
        const loaded_state = try db.loadAgentState(agent_id);
        
        if (loaded_state) |state| {
            defer {
                var mutable_state = state;
                mutable_state.deinit(allocator);
            }
            try testing.expectEqual(agent_id, state.agent_id);
        }
    }

    const end_time = std.time.nanoTimestamp();
    const duration_ms = @as(f64, @floatFromInt(end_time - start_time)) / 1_000_000.0;
    
    std.debug.print("Performance test: {} operations in {d:.2} ms\n", .{ num_operations * 2, duration_ms });
    
    // 验证性能（应该在合理时间内完成）
    try testing.expect(duration_ms < 10000.0); // 10秒内完成

    std.fs.cwd().deleteFile(db_path) catch {};
}
