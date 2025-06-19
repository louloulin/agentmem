// 单个测试文件，用于诊断问题
const std = @import("std");
const testing = std.testing;
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;

test "Single AgentState basic functionality" {
    std.debug.print("\n=== Single Basic Test Starting ===\n", .{});
    
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("Creating agent state...\n", .{});
    
    // 创建Agent状态
    const test_data = "Hello, Agent State!";
    var state = try AgentState.init(allocator, 12345, 67890, StateType.context, test_data);
    defer state.deinit(allocator);

    std.debug.print("Verifying agent state properties...\n", .{});
    
    // 验证基础属性
    try testing.expectEqual(@as(u64, 12345), state.agent_id);
    try testing.expectEqual(@as(u64, 67890), state.session_id);
    try testing.expectEqual(StateType.context, state.state_type);
    try testing.expect(std.mem.eql(u8, state.data, test_data));
    try testing.expect(state.validateChecksum());
    try testing.expectEqual(@as(u32, 1), state.version);
    
    std.debug.print("✅ Single basic test completed successfully!\n", .{});
}
