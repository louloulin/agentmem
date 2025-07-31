const std = @import("std");
const testing = std.testing;
const AgentDatabase = @import("agent_api.zig").AgentDatabase;
const AgentState = @import("agent_api.zig").AgentState;
const StateType = @import("agent_api.zig").StateType;
const Memory = @import("agent_api.zig").Memory;
const MemoryType = @import("agent_api.zig").MemoryType;
const Document = @import("agent_api.zig").Document;

test "Agent Database Creation and Cleanup" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_api.lance");
    defer db.deinit();
    
    // 如果能创建和清理，测试通过
    try testing.expect(db.db_handle != null);
    try testing.expect(db.memory_handle != null);
    try testing.expect(db.rag_handle != null);
}

test "Agent State Save and Load" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_state.lance");
    defer db.deinit();
    
    // 创建测试状态
    const test_data = "Hello from Zig API!";
    const state = AgentState.init(12345, 67890, StateType.working_memory, test_data);
    
    // 保存状态
    try db.saveState(state);
    
    // 加载状态
    const loaded_data = try db.loadState(12345);
    defer if (loaded_data) |data| allocator.free(data);
    
    try testing.expect(loaded_data != null);
    if (loaded_data) |data| {
        try testing.expectEqualStrings(test_data, data);
    }
}

test "Vector State Operations" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_vector.lance");
    defer db.deinit();
    
    // 创建测试向量状态
    const test_data = "Vector test data";
    const state = AgentState.init(54321, 98765, StateType.embedding, test_data);
    
    // 创建测试向量
    const embedding = [_]f32{ 0.1, 0.2, 0.3, 0.4, 0.5 };
    
    // 保存向量状态
    try db.saveVectorState(state, &embedding);
    
    // 向量搜索
    var search_results = try db.vectorSearch(&embedding, 5);
    defer search_results.deinit();
    
    try testing.expect(search_results.count > 0);
}

test "Memory Management" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_memory.lance");
    defer db.deinit();
    
    // 创建测试记忆
    const memory = Memory.init(12345, MemoryType.episodic, "Test memory from Zig", 0.8);
    
    // 存储记忆
    try db.storeMemory(memory);
    
    // 检索记忆
    const memory_count = try db.retrieveMemories(12345, 10);
    try testing.expect(memory_count > 0);
}

test "Document Indexing and Search" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_rag.lance");
    defer db.deinit();
    
    // 创建测试文档
    const document = Document.init(
        "Zig Programming Language",
        "Zig is a general-purpose programming language and toolchain for maintaining robust, optimal, and reusable software. It focuses on compile-time code execution and has no hidden control flow.",
        100,
        20
    );
    
    // 索引文档
    try db.indexDocument(document);
    
    // 搜索文档
    const results_count = try db.searchText("programming language", 5);
    try testing.expect(results_count > 0);
}

test "RAG Context Building" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_context.lance");
    defer db.deinit();
    
    // 先索引一些文档
    const doc1 = Document.init(
        "Machine Learning Basics",
        "Machine learning is a method of data analysis that automates analytical model building. It is a branch of artificial intelligence based on the idea that systems can learn from data.",
        150,
        30
    );
    
    const doc2 = Document.init(
        "Deep Learning Introduction",
        "Deep learning is part of a broader family of machine learning methods based on artificial neural networks with representation learning.",
        150,
        30
    );
    
    try db.indexDocument(doc1);
    try db.indexDocument(doc2);
    
    // 构建上下文
    const context = try db.buildContext("What is machine learning?", 500);
    defer allocator.free(context);
    
    try testing.expect(context.len > 0);
    // 验证上下文包含相关内容
    try testing.expect(std.mem.indexOf(u8, context, "machine learning") != null);
}

test "Convenience Methods" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_convenience.lance");
    defer db.deinit();
    
    // 测试便利方法
    try db.createAgent(99999, "Initial agent data");
    try db.updateAgent(99999, "Updated agent data");
    try db.addMemory(99999, "Important memory", MemoryType.semantic, 0.9);
    try db.addDocument("Test Document", "This is a test document for convenience methods.");
    
    // 查询知识
    const knowledge = try db.queryKnowledge("test document");
    defer allocator.free(knowledge);
    
    try testing.expect(knowledge.len > 0);
}

test "Multiple State Types" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_types.lance");
    defer db.deinit();
    
    const agent_id = 77777;
    
    // 测试不同的状态类型
    const state_types = [_]StateType{
        StateType.working_memory,
        StateType.long_term_memory,
        StateType.context,
        StateType.task_state,
        StateType.relationship,
        StateType.embedding,
    };
    
    for (state_types, 0..) |state_type, i| {
        const data = try std.fmt.allocPrint(allocator, "Data for state type {}", .{i});
        defer allocator.free(data);
        
        const state = AgentState.init(agent_id, @intCast(i), state_type, data);
        try db.saveState(state);
    }
    
    // 验证可以加载状态（这里简化为只检查最后一个）
    const loaded_data = try db.loadState(agent_id);
    defer if (loaded_data) |data| allocator.free(data);
    
    try testing.expect(loaded_data != null);
}

test "Multiple Memory Types" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_memory_types.lance");
    defer db.deinit();
    
    const agent_id = 88888;
    
    // 测试不同的记忆类型
    const memory_types = [_]MemoryType{
        MemoryType.episodic,
        MemoryType.semantic,
        MemoryType.procedural,
        MemoryType.working,
    };
    
    for (memory_types, 0..) |memory_type, i| {
        const content = try std.fmt.allocPrint(allocator, "Memory content for type {s}", .{memory_type.toString()});
        defer allocator.free(content);
        
        const memory = Memory.init(agent_id, memory_type, content, 0.5 + @as(f32, @floatFromInt(i)) * 0.1);
        try db.storeMemory(memory);
    }
    
    // 检索所有记忆
    const memory_count = try db.retrieveMemories(agent_id, 10);
    try testing.expect(memory_count >= memory_types.len);
}

// 性能测试
test "Performance Test - Batch Operations" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var db = try AgentDatabase.init(allocator, "test_zig_performance.lance");
    defer db.deinit();
    
    const start_time = std.time.milliTimestamp();
    
    // 批量保存状态
    const batch_size = 100;
    for (0..batch_size) |i| {
        const data = try std.fmt.allocPrint(allocator, "Batch data {}", .{i});
        defer allocator.free(data);
        
        const state = AgentState.init(@intCast(i), 0, StateType.working_memory, data);
        try db.saveState(state);
    }
    
    const end_time = std.time.milliTimestamp();
    const duration = end_time - start_time;
    
    std.debug.print("Batch save of {} states took {} ms\n", .{ batch_size, duration });
    
    // 性能应该在合理范围内（这里设置为10秒上限）
    try testing.expect(duration < 10000);
}
