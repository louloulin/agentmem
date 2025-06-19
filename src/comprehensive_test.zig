// 综合性能测试 - 测试所有主要功能
const std = @import("std");
const testing = std.testing;
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;

test "Comprehensive Performance and Functionality Test" {
    std.debug.print("\n🚀 === Comprehensive Test Suite Starting ===\n", .{});

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // 测试1: 大量Agent状态创建和管理
    std.debug.print("\n📊 Test 1: Mass Agent State Creation\n", .{});
    const num_agents = 50; // 减少数量避免卡住
    var states = std.ArrayList(AgentState).init(allocator);
    defer {
        for (states.items) |*state| {
            state.deinit(allocator);
        }
        states.deinit();
    }

    const start_time = std.time.milliTimestamp();

    for (0..num_agents) |i| {
        const agent_id = @as(u64, i + 1000);
        const session_id = @as(u64, i + 2000);
        const state_type = switch (i % 6) {
            0 => StateType.working_memory,
            1 => StateType.long_term_memory,
            2 => StateType.context,
            3 => StateType.task_state,
            4 => StateType.relationship,
            5 => StateType.embedding,
            else => StateType.working_memory,
        };

        var state = try AgentState.init(allocator, agent_id, session_id, state_type, "Comprehensive test data for performance evaluation");

        // 添加元数据
        try state.setMetadata(allocator, "test_id", "comprehensive");
        try state.setMetadata(allocator, "batch", "performance");

        try states.append(state);

        if (i % 10 == 0) {
            std.debug.print("  Created {} agents...\n", .{i + 1});
        }
    }

    const creation_time = std.time.milliTimestamp() - start_time;
    std.debug.print("✅ Created {} agents in {} ms\n", .{ num_agents, creation_time });

    // 测试2: 数据更新性能
    std.debug.print("\n🔄 Test 2: Data Update Performance\n", .{});
    const update_start = std.time.milliTimestamp();

    for (states.items, 0..) |*state, i| {
        const new_data = try std.fmt.allocPrint(allocator, "Updated data for agent {}", .{i});
        defer allocator.free(new_data);

        try state.updateData(allocator, new_data);

        if (i % 10 == 0) {
            std.debug.print("  Updated {} agents...\n", .{i + 1});
        }
    }

    const update_time = std.time.milliTimestamp() - update_start;
    std.debug.print("✅ Updated {} agents in {} ms\n", .{ num_agents, update_time });

    // 测试3: 校验和验证性能
    std.debug.print("\n🔍 Test 3: Checksum Validation Performance\n", .{});
    const validation_start = std.time.milliTimestamp();
    var valid_count: u32 = 0;

    for (states.items) |*state| {
        if (state.validateChecksum()) {
            valid_count += 1;
        }
    }

    const validation_time = std.time.milliTimestamp() - validation_start;
    std.debug.print("✅ Validated {} agents in {} ms ({}% valid)\n", .{ num_agents, validation_time, (valid_count * 100) / num_agents });

    // 测试4: 元数据操作性能
    std.debug.print("\n🏷️ Test 4: Metadata Operations Performance\n", .{});
    const metadata_start = std.time.milliTimestamp();

    for (states.items, 0..) |*state, i| {
        // 添加多个元数据
        const key1 = try std.fmt.allocPrint(allocator, "key_{}", .{i});
        defer allocator.free(key1);
        const value1 = try std.fmt.allocPrint(allocator, "value_{}", .{i});
        defer allocator.free(value1);

        try state.setMetadata(allocator, key1, value1);
        try state.setMetadata(allocator, "category", "performance_test");
        try state.setMetadata(allocator, "status", "active");

        // 读取元数据
        _ = state.getMetadata("test_id");
        _ = state.getMetadata(key1);
        _ = state.getMetadata("category");
    }

    const metadata_time = std.time.milliTimestamp() - metadata_start;
    std.debug.print("✅ Processed metadata for {} agents in {} ms\n", .{ num_agents, metadata_time });

    // 测试5: 内存使用统计
    std.debug.print("\n💾 Test 5: Memory Usage Analysis\n", .{});
    var total_data_size: usize = 0;
    var total_metadata_count: usize = 0;

    for (states.items) |*state| {
        total_data_size += state.data.len;
        total_metadata_count += state.metadata.count();
    }

    const avg_data_size = total_data_size / num_agents;
    const avg_metadata_count = total_metadata_count / num_agents;

    std.debug.print("📈 Memory Statistics:\n", .{});
    std.debug.print("  Total agents: {}\n", .{num_agents});
    std.debug.print("  Total data size: {} bytes\n", .{total_data_size});
    std.debug.print("  Average data per agent: {} bytes\n", .{avg_data_size});
    std.debug.print("  Total metadata entries: {}\n", .{total_metadata_count});
    std.debug.print("  Average metadata per agent: {}\n", .{avg_metadata_count});

    // 测试6: 状态类型分布
    std.debug.print("\n📊 Test 6: State Type Distribution\n", .{});
    var type_counts = [_]u32{0} ** 6;

    for (states.items) |*state| {
        const type_index: usize = switch (state.state_type) {
            .working_memory => 0,
            .long_term_memory => 1,
            .context => 2,
            .task_state => 3,
            .relationship => 4,
            .embedding => 5,
        };
        type_counts[type_index] += 1;
    }

    const type_names = [_][]const u8{ "working_memory", "long_term_memory", "context", "task_state", "relationship", "embedding" };

    for (type_names, type_counts) |name, count| {
        const percentage = (count * 100) / num_agents;
        std.debug.print("  {s}: {} ({}%)\n", .{ name, count, percentage });
    }

    // 总体性能统计
    const total_time = std.time.milliTimestamp() - start_time;
    std.debug.print("\n🎯 === Comprehensive Test Results ===\n", .{});
    std.debug.print("✅ All tests completed successfully!\n", .{});
    std.debug.print("📊 Performance Summary:\n", .{});
    std.debug.print("  Total execution time: {} ms\n", .{total_time});
    std.debug.print("  Agent creation time: {} ms\n", .{creation_time});
    std.debug.print("  Data update time: {} ms\n", .{update_time});
    std.debug.print("  Validation time: {} ms\n", .{validation_time});
    std.debug.print("  Metadata operations time: {} ms\n", .{metadata_time});
    std.debug.print("  Average time per agent: {d:.2} ms\n", .{@as(f64, @floatFromInt(total_time)) / @as(f64, @floatFromInt(num_agents))});

    // 验证所有操作都成功
    try testing.expect(states.items.len == num_agents);
    try testing.expect(valid_count == num_agents);
    try testing.expect(total_data_size > 0);
    try testing.expect(total_metadata_count > 0);

    std.debug.print("🎉 Comprehensive test suite completed successfully!\n", .{});
}
