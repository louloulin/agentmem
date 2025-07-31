// 安全的测试文件，避免C FFI调用
const std = @import("std");
const testing = std.testing;

// 模拟Agent状态类型
const StateType = enum(u32) {
    working_memory = 0,
    long_term_memory = 1,
    context = 2,
    task_state = 3,
    relationship = 4,
    embedding = 5,

    pub fn toString(self: StateType) []const u8 {
        return switch (self) {
            .working_memory => "working_memory",
            .long_term_memory => "long_term_memory",
            .context => "context",
            .task_state => "task_state",
            .relationship => "relationship",
            .embedding => "embedding",
        };
    }
};

// 模拟记忆类型
const MemoryType = enum(u32) {
    episodic = 0,
    semantic = 1,
    procedural = 2,
    working = 3,

    pub fn toString(self: MemoryType) []const u8 {
        return switch (self) {
            .episodic => "episodic",
            .semantic => "semantic",
            .procedural => "procedural",
            .working => "working",
        };
    }
};

// 模拟Agent状态结构
const AgentState = struct {
    agent_id: u64,
    session_id: u64,
    state_type: StateType,
    data: []const u8,

    pub fn init(agent_id: u64, session_id: u64, state_type: StateType, data: []const u8) AgentState {
        return AgentState{
            .agent_id = agent_id,
            .session_id = session_id,
            .state_type = state_type,
            .data = data,
        };
    }

    pub fn display(self: AgentState) void {
        std.debug.print("Agent State:\n", .{});
        std.debug.print("  Agent ID: {d}\n", .{self.agent_id});
        std.debug.print("  Session ID: {d}\n", .{self.session_id});
        std.debug.print("  State Type: {s}\n", .{self.state_type.toString()});
        std.debug.print("  Data: {s}\n", .{self.data});
    }
};

// 模拟记忆结构
const Memory = struct {
    agent_id: u64,
    memory_type: MemoryType,
    content: []const u8,
    importance: f32,

    pub fn init(agent_id: u64, memory_type: MemoryType, content: []const u8, importance: f32) Memory {
        return Memory{
            .agent_id = agent_id,
            .memory_type = memory_type,
            .content = content,
            .importance = importance,
        };
    }

    pub fn display(self: Memory) void {
        std.debug.print("Memory:\n", .{});
        std.debug.print("  Agent ID: {d}\n", .{self.agent_id});
        std.debug.print("  Memory Type: {s}\n", .{self.memory_type.toString()});
        std.debug.print("  Content: {s}\n", .{self.content});
        std.debug.print("  Importance: {d:.2}\n", .{self.importance});
    }
};

test "Agent State Creation and Display" {
    std.debug.print("\n=== Agent State Creation and Display Test ===\n", .{});
    
    const state = AgentState.init(12345, 67890, StateType.working_memory, "Test agent state data");
    
    try testing.expect(state.agent_id == 12345);
    try testing.expect(state.session_id == 67890);
    try testing.expect(state.state_type == StateType.working_memory);
    try testing.expectEqualStrings(state.data, "Test agent state data");
    
    state.display();
    std.debug.print("✅ Agent state creation test passed!\n", .{});
}

test "Memory Creation and Display" {
    std.debug.print("\n=== Memory Creation and Display Test ===\n", .{});
    
    const memory = Memory.init(54321, MemoryType.episodic, "Important memory content", 0.85);
    
    try testing.expect(memory.agent_id == 54321);
    try testing.expect(memory.memory_type == MemoryType.episodic);
    try testing.expectEqualStrings(memory.content, "Important memory content");
    try testing.expect(memory.importance == 0.85);
    
    memory.display();
    std.debug.print("✅ Memory creation test passed!\n", .{});
}

test "State Type Enumeration" {
    std.debug.print("\n=== State Type Enumeration Test ===\n", .{});
    
    const state_types = [_]StateType{
        StateType.working_memory,
        StateType.long_term_memory,
        StateType.context,
        StateType.task_state,
        StateType.relationship,
        StateType.embedding,
    };
    
    std.debug.print("State Types:\n", .{});
    for (state_types) |state_type| {
        std.debug.print("  - {s}\n", .{state_type.toString()});
    }
    
    try testing.expect(state_types.len == 6);
    std.debug.print("✅ State type enumeration test passed!\n", .{});
}

test "Memory Type Enumeration" {
    std.debug.print("\n=== Memory Type Enumeration Test ===\n", .{});
    
    const memory_types = [_]MemoryType{
        MemoryType.episodic,
        MemoryType.semantic,
        MemoryType.procedural,
        MemoryType.working,
    };
    
    std.debug.print("Memory Types:\n", .{});
    for (memory_types) |memory_type| {
        std.debug.print("  - {s}\n", .{memory_type.toString()});
    }
    
    try testing.expect(memory_types.len == 4);
    std.debug.print("✅ Memory type enumeration test passed!\n", .{});
}

test "Multiple Agent States" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    std.debug.print("\n=== Multiple Agent States Test ===\n", .{});
    
    var states = std.ArrayList(AgentState).init(allocator);
    defer states.deinit();
    
    const agent_configs = [_]struct {
        agent_id: u64,
        session_id: u64,
        state_type: StateType,
        data: []const u8,
    }{
        .{ .agent_id = 1001, .session_id = 1, .state_type = StateType.working_memory, .data = "Agent 1001 working memory" },
        .{ .agent_id = 1002, .session_id = 2, .state_type = StateType.long_term_memory, .data = "Agent 1002 long term memory" },
        .{ .agent_id = 1003, .session_id = 3, .state_type = StateType.context, .data = "Agent 1003 context data" },
    };
    
    for (agent_configs) |config| {
        const state = AgentState.init(config.agent_id, config.session_id, config.state_type, config.data);
        try states.append(state);
    }
    
    try testing.expect(states.items.len == 3);
    
    std.debug.print("Created {} agent states:\n", .{states.items.len});
    for (states.items, 0..) |state, i| {
        std.debug.print("State {}:\n", .{i + 1});
        state.display();
        std.debug.print("\n", .{});
    }
    
    std.debug.print("✅ Multiple agent states test passed!\n", .{});
}

test "Multiple Memories" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    std.debug.print("\n=== Multiple Memories Test ===\n", .{});
    
    var memories = std.ArrayList(Memory).init(allocator);
    defer memories.deinit();
    
    const memory_configs = [_]struct {
        agent_id: u64,
        memory_type: MemoryType,
        content: []const u8,
        importance: f32,
    }{
        .{ .agent_id = 2001, .memory_type = MemoryType.episodic, .content = "First meeting with user", .importance = 0.9 },
        .{ .agent_id = 2001, .memory_type = MemoryType.semantic, .content = "User prefers coffee over tea", .importance = 0.7 },
        .{ .agent_id = 2002, .memory_type = MemoryType.procedural, .content = "How to process user requests", .importance = 0.8 },
        .{ .agent_id = 2002, .memory_type = MemoryType.working, .content = "Current task: analyze data", .importance = 0.6 },
    };
    
    for (memory_configs) |config| {
        const memory = Memory.init(config.agent_id, config.memory_type, config.content, config.importance);
        try memories.append(memory);
    }
    
    try testing.expect(memories.items.len == 4);
    
    std.debug.print("Created {} memories:\n", .{memories.items.len});
    for (memories.items, 0..) |memory, i| {
        std.debug.print("Memory {}:\n", .{i + 1});
        memory.display();
        std.debug.print("\n", .{});
    }
    
    std.debug.print("✅ Multiple memories test passed!\n", .{});
}

test "Performance Test - Data Structure Operations" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    std.debug.print("\n=== Performance Test - Data Structure Operations ===\n", .{});
    
    const start_time = std.time.milliTimestamp();
    
    // 创建大量状态和记忆
    const batch_size = 1000;
    var states = std.ArrayList(AgentState).init(allocator);
    defer states.deinit();
    
    var memories = std.ArrayList(Memory).init(allocator);
    defer memories.deinit();
    
    for (0..batch_size) |i| {
        const state = AgentState.init(
            @intCast(i),
            0,
            StateType.working_memory,
            "Batch test data",
        );
        try states.append(state);
        
        const memory = Memory.init(
            @intCast(i),
            MemoryType.working,
            "Batch test memory",
            0.5,
        );
        try memories.append(memory);
    }
    
    const end_time = std.time.milliTimestamp();
    const duration = end_time - start_time;
    
    try testing.expect(states.items.len == batch_size);
    try testing.expect(memories.items.len == batch_size);
    
    std.debug.print("Created {} states and {} memories in {} ms\n", .{ batch_size, batch_size, duration });
    std.debug.print("Average time per operation: {d:.2} ms\n", .{@as(f64, @floatFromInt(duration)) / @as(f64, @floatFromInt(batch_size * 2))});
    
    // 性能应该在合理范围内
    try testing.expect(duration < 5000); // 5秒上限
    
    std.debug.print("✅ Performance test passed!\n", .{});
}

// 运行所有测试的主函数
pub fn runAllTests() !void {
    std.debug.print("🚀 Starting Safe Zig API Tests\n", .{});
    std.debug.print("=" ** 50 ++ "\n", .{});

    // 运行所有测试
    try testing.refAllDecls(@This());

    std.debug.print("=" ** 50 ++ "\n", .{});
    std.debug.print("🎉 All Safe Zig API Tests Completed!\n", .{});
}
