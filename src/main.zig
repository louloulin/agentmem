const std = @import("std");
const agent_api = @import("agent_api.zig");
const testing = std.testing;

// 导出主要的API
pub const AgentDatabase = agent_api.AgentDatabase;
pub const AgentState = agent_api.AgentState;
pub const StateType = agent_api.StateType;
pub const Memory = agent_api.Memory;
pub const MemoryType = agent_api.MemoryType;
pub const Document = agent_api.Document;
pub const SearchResults = agent_api.SearchResults;
pub const AgentDbError = agent_api.AgentDbError;

// 简单的测试主函数
pub fn main() !void {
    std.debug.print("🚀 Agent State Database - Zig API\n", .{});
    std.debug.print("==================================\n\n", .{});

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // 创建数据库实例
    std.debug.print("1. Initializing database...\n", .{});
    var db = AgentDatabase.init(allocator, "test_main.lance") catch |err| {
        std.debug.print("❌ Failed to initialize database: {}\n", .{err});
        return;
    };
    defer db.deinit();
    std.debug.print("✅ Database initialized\n\n", .{});

    // 测试基本Agent操作
    std.debug.print("2. Testing Agent operations...\n", .{});
    const agent_id = 12345;

    try db.createAgent(agent_id, "Hello from Zig API!");
    std.debug.print("✅ Created agent {}\n", .{agent_id});

    const loaded_data = try db.loadState(agent_id);
    defer if (loaded_data) |data| allocator.free(data);

    if (loaded_data) |data| {
        std.debug.print("✅ Loaded state: {s}\n", .{data});
    }

    // 测试记忆功能
    std.debug.print("\n3. Testing Memory operations...\n", .{});
    try db.addMemory(agent_id, "This is a test memory", MemoryType.episodic, 0.8);
    std.debug.print("✅ Added memory\n", .{});

    const memory_count = try db.retrieveMemories(agent_id, 10);
    std.debug.print("✅ Retrieved {} memories\n", .{memory_count});

    // 测试文档索引
    std.debug.print("\n4. Testing Document operations...\n", .{});
    try db.addDocument("Test Document", "This is a test document for the Zig API demonstration.");
    std.debug.print("✅ Indexed document\n", .{});

    const search_count = try db.searchText("test document", 5);
    std.debug.print("✅ Search found {} results\n", .{search_count});

    // 测试RAG功能
    std.debug.print("\n5. Testing RAG operations...\n", .{});
    const context = try db.queryKnowledge("What is this test about?");
    defer allocator.free(context);
    std.debug.print("✅ Built context ({} chars): {s}...\n", .{ context.len, context[0..@min(50, context.len)] });

    std.debug.print("\n🎉 All tests completed successfully!\n", .{});
    std.debug.print("📊 The Zig API is working correctly.\n", .{});
}

// 单元测试
test "Zig API Basic Functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var db = try AgentDatabase.init(allocator, "test_unit.lance");
    defer db.deinit();

    // 基本状态操作
    const state = AgentState.init(123, 456, StateType.working_memory, "test data");
    try db.saveState(state);

    const loaded = try db.loadState(123);
    defer if (loaded) |data| allocator.free(data);

    try testing.expect(loaded != null);
    if (loaded) |data| {
        try testing.expectEqualStrings("test data", data);
    }
}

test "Zig API Memory Operations" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var db = try AgentDatabase.init(allocator, "test_memory_unit.lance");
    defer db.deinit();

    const memory = Memory.init(789, MemoryType.semantic, "test memory", 0.9);
    try db.storeMemory(memory);

    const count = try db.retrieveMemories(789, 5);
    try testing.expect(count > 0);
}

test "Zig API Document Operations" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var db = try AgentDatabase.init(allocator, "test_doc_unit.lance");
    defer db.deinit();

    const doc = Document.init("Test", "Test content for unit test", 50, 10);
    try db.indexDocument(doc);

    const results = try db.searchText("test content", 3);
    try testing.expect(results > 0);
}
