// 简单的Agent状态测试（不依赖LanceDB）
const std = @import("std");
const testing = std.testing;
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;

// 测试Agent状态基础功能
test "AgentState basic functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // 创建Agent状态
    const test_data = "Hello, Agent State!";
    var state = try AgentState.init(allocator, 12345, 67890, StateType.context, test_data);
    defer state.deinit(allocator);

    // 验证基础属性
    try testing.expectEqual(@as(u64, 12345), state.agent_id);
    try testing.expectEqual(@as(u64, 67890), state.session_id);
    try testing.expectEqual(StateType.context, state.state_type);
    try testing.expect(std.mem.eql(u8, state.data, test_data));
    try testing.expect(state.validateChecksum());
    try testing.expectEqual(@as(u32, 1), state.version);
}

// 测试状态更新
test "AgentState update functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const initial_data = "Initial state";
    const updated_data = "Updated state";

    var state = try AgentState.init(allocator, 11111, 22222, StateType.working_memory, initial_data);
    defer state.deinit(allocator);

    // 验证初始状态
    try testing.expect(std.mem.eql(u8, state.data, initial_data));
    try testing.expectEqual(@as(u32, 1), state.version);
    const initial_timestamp = state.timestamp;

    // 等待一点时间确保时间戳不同
    std.time.sleep(100000000); // 100ms

    // 更新状态
    try state.updateData(allocator, updated_data);

    // 验证更新后的状态
    try testing.expect(std.mem.eql(u8, state.data, updated_data));
    try testing.expectEqual(@as(u32, 2), state.version);
    // 时间戳应该被更新（允许一些误差）
    try testing.expect(state.timestamp >= initial_timestamp);
    try testing.expect(state.validateChecksum());
}

// 测试元数据功能
test "AgentState metadata functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const test_data = "Test data with metadata";
    var state = try AgentState.init(allocator, 33333, 44444, StateType.long_term_memory, test_data);
    defer state.deinit(allocator);

    // 设置元数据
    try state.setMetadata(allocator, "priority", "high");
    try state.setMetadata(allocator, "category", "important");

    // 获取元数据
    const priority = state.getMetadata("priority");
    const category = state.getMetadata("category");
    const nonexistent = state.getMetadata("nonexistent");

    try testing.expect(priority != null);
    try testing.expect(category != null);
    try testing.expect(nonexistent == null);

    if (priority) |p| {
        try testing.expect(std.mem.eql(u8, p, "high"));
    }

    if (category) |c| {
        try testing.expect(std.mem.eql(u8, c, "important"));
    }
}

// 测试状态快照
test "AgentState snapshot functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const initial_data = "Initial snapshot data";
    var state = try AgentState.init(allocator, 55555, 66666, StateType.task_state, initial_data);
    defer state.deinit(allocator);

    // 创建快照
    var snapshot = try state.createSnapshot(allocator, "backup_v1");
    defer snapshot.deinit(allocator);

    // 验证快照
    try testing.expectEqual(state.agent_id, snapshot.agent_id);
    try testing.expectEqual(state.session_id, snapshot.session_id);
    try testing.expectEqual(state.state_type, snapshot.state_type);
    try testing.expect(std.mem.eql(u8, state.data, snapshot.data));
    try testing.expect(snapshot.version > state.version);

    // 验证快照元数据
    const snapshot_name = snapshot.getMetadata("snapshot_name");
    const is_snapshot = snapshot.getMetadata("is_snapshot");

    try testing.expect(snapshot_name != null);
    try testing.expect(is_snapshot != null);

    if (snapshot_name) |name| {
        try testing.expect(std.mem.eql(u8, name, "backup_v1"));
    }

    if (is_snapshot) |flag| {
        try testing.expect(std.mem.eql(u8, flag, "true"));
    }
}

// 测试状态压缩
test "AgentState compression functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // 创建包含重复数据的状态
    const repetitive_data = "AAAAAABBBBBBCCCCCCDDDDDD";
    var state = try AgentState.init(allocator, 77777, 88888, StateType.context, repetitive_data);
    defer state.deinit(allocator);

    const original_size = state.data.len;
    const original_data = try allocator.dupe(u8, state.data);
    defer allocator.free(original_data);

    // 压缩状态
    try state.compress(allocator);

    // 检查是否有压缩标记
    const is_compressed = state.getMetadata("compressed");
    if (is_compressed) |compressed| {
        try testing.expect(std.mem.eql(u8, compressed, "true"));

        // 解压缩
        try state.decompress(allocator);

        // 验证数据完整性
        try testing.expect(std.mem.eql(u8, state.data, original_data));
        try testing.expectEqual(original_size, state.data.len);
        try testing.expect(state.validateChecksum());

        // 压缩标记应该被移除
        const still_compressed = state.getMetadata("compressed");
        try testing.expect(still_compressed == null);
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

// 测试状态类型转换
test "StateType conversion functionality" {
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

        try testing.expect(parsed_type != null);
        if (parsed_type) |parsed| {
            try testing.expectEqual(state_type, parsed);
        }
    }

    // 测试无效字符串
    const invalid_type = StateType.fromString("invalid_type");
    try testing.expect(invalid_type == null);
}

// 测试JSON序列化
test "AgentState JSON serialization" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var state = try AgentState.init(allocator, 12345, 67890, StateType.context, "Test JSON data");
    defer state.deinit(allocator);

    // 序列化为JSON
    const json_str = try state.toJson(allocator);
    defer allocator.free(json_str);

    // 验证JSON包含必要字段
    try testing.expect(std.mem.indexOf(u8, json_str, "\"agent_id\":12345") != null);
    try testing.expect(std.mem.indexOf(u8, json_str, "\"session_id\":67890") != null);
    try testing.expect(std.mem.indexOf(u8, json_str, "\"state_type\":\"context\"") != null);
}

// 性能测试
test "AgentState performance test" {
    std.debug.print("\n=== Starting Performance Test ===\n", .{});

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const num_operations = 100; // 减少操作数量
    const test_data = "Performance test data";

    std.debug.print("Starting {} operations...\n", .{num_operations});
    const start_time = std.time.nanoTimestamp();

    // 创建和销毁大量状态
    for (0..num_operations) |i| {
        var state = try AgentState.init(allocator, @as(u64, i), @as(u64, i * 2), StateType.working_memory, test_data);
        defer state.deinit(allocator);

        // 执行一些操作
        try state.setMetadata(allocator, "test_key", "test_value");
        _ = state.getMetadata("test_key");
        try state.updateData(allocator, "Updated test data");
        _ = state.validateChecksum();

        // 每10个操作打印一次进度
        if (i % 10 == 0) {
            std.debug.print("Completed {} operations...\n", .{i});
        }
    }

    const end_time = std.time.nanoTimestamp();
    const duration_ms = @as(f64, @floatFromInt(end_time - start_time)) / 1_000_000.0;

    std.debug.print("Performance test: {} operations in {d:.2} ms\n", .{ num_operations, duration_ms });

    // 验证性能（应该在合理时间内完成）
    try testing.expect(duration_ms < 5000.0); // 5秒内完成

    std.debug.print("✅ Performance test completed successfully!\n", .{});
}
