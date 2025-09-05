const std = @import("std");
const testing = std.testing;

// 简化的Zig API结构，不依赖C FFI
pub const StateType = enum(u32) {
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

pub const MemoryType = enum(u32) {
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

pub const AgentState = struct {
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
};

pub const Memory = struct {
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
};

pub const Document = struct {
    title: []const u8,
    content: []const u8,
    chunk_size: usize,
    overlap: usize,

    pub fn init(title: []const u8, content: []const u8, chunk_size: usize, overlap: usize) Document {
        return Document{
            .title = title,
            .content = content,
            .chunk_size = chunk_size,
            .overlap = overlap,
        };
    }
};

// 模拟的Agent数据库（内存版本）
pub const MockAgentDatabase = struct {
    states: std.HashMap(u64, []u8, std.hash_map.AutoContext(u64), std.hash_map.default_max_load_percentage),
    memories: std.ArrayList(Memory),
    documents: std.ArrayList(Document),
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) MockAgentDatabase {
        return MockAgentDatabase{
            .states = std.HashMap(u64, []u8, std.hash_map.AutoContext(u64), std.hash_map.default_max_load_percentage).init(allocator),
            .memories = std.ArrayList(Memory).init(allocator),
            .documents = std.ArrayList(Document).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *MockAgentDatabase) void {
        // 清理状态数据
        var state_iter = self.states.iterator();
        while (state_iter.next()) |entry| {
            self.allocator.free(entry.value_ptr.*);
        }
        self.states.deinit();

        self.memories.deinit();
        self.documents.deinit();
    }

    pub fn saveState(self: *MockAgentDatabase, state: AgentState) !void {
        const data_copy = try self.allocator.dupe(u8, state.data);
        try self.states.put(state.agent_id, data_copy);
    }

    pub fn loadState(self: *MockAgentDatabase, agent_id: u64) ?[]const u8 {
        return self.states.get(agent_id);
    }

    pub fn storeMemory(self: *MockAgentDatabase, memory: Memory) !void {
        try self.memories.append(memory);
    }

    pub fn retrieveMemories(self: *MockAgentDatabase, agent_id: u64, limit: usize) usize {
        var count: usize = 0;
        for (self.memories.items) |memory| {
            if (memory.agent_id == agent_id) {
                count += 1;
                if (count >= limit) break;
            }
        }
        return count;
    }

    pub fn indexDocument(self: *MockAgentDatabase, document: Document) !void {
        try self.documents.append(document);
    }

    pub fn searchText(self: *MockAgentDatabase, query: []const u8, limit: usize) usize {
        var count: usize = 0;
        for (self.documents.items) |doc| {
            if (std.mem.indexOf(u8, doc.content, query) != null) {
                count += 1;
                if (count >= limit) break;
            }
        }
        return count;
    }

    pub fn buildContext(self: *MockAgentDatabase, query: []const u8, max_tokens: usize, allocator: std.mem.Allocator) ![]u8 {
        _ = max_tokens;
        var context = std.ArrayList(u8).init(allocator);
        defer context.deinit();

        try context.appendSlice("Context for query: ");
        try context.appendSlice(query);
        try context.appendSlice("\n");

        for (self.documents.items) |doc| {
            if (std.mem.indexOf(u8, doc.content, query) != null) {
                try context.appendSlice("Document: ");
                try context.appendSlice(doc.title);
                try context.appendSlice("\nContent: ");
                try context.appendSlice(doc.content);
                try context.appendSlice("\n\n");
            }
        }

        return try context.toOwnedSlice();
    }
};

// 测试用例
test "Zig API Basic Types" {
    const state_type = StateType.working_memory;
    try testing.expectEqualStrings("working_memory", state_type.toString());

    const memory_type = MemoryType.episodic;
    try testing.expectEqualStrings("episodic", memory_type.toString());
}

test "Zig API Agent State" {
    const state = AgentState.init(123, 456, StateType.context, "test data");
    try testing.expectEqual(@as(u64, 123), state.agent_id);
    try testing.expectEqual(@as(u64, 456), state.session_id);
    try testing.expectEqual(StateType.context, state.state_type);
    try testing.expectEqualStrings("test data", state.data);
}

test "Zig API Memory" {
    const memory = Memory.init(789, MemoryType.semantic, "test memory", 0.8);
    try testing.expectEqual(@as(u64, 789), memory.agent_id);
    try testing.expectEqual(MemoryType.semantic, memory.memory_type);
    try testing.expectEqualStrings("test memory", memory.content);
    try testing.expectEqual(@as(f32, 0.8), memory.importance);
}

test "Zig API Document" {
    const doc = Document.init("Test Title", "Test content", 100, 20);
    try testing.expectEqualStrings("Test Title", doc.title);
    try testing.expectEqualStrings("Test content", doc.content);
    try testing.expectEqual(@as(usize, 100), doc.chunk_size);
    try testing.expectEqual(@as(usize, 20), doc.overlap);
}

test "Mock Agent Database Operations" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var db = MockAgentDatabase.init(allocator);
    defer db.deinit();

    // 测试状态操作
    const state = AgentState.init(123, 456, StateType.working_memory, "test state data");
    try db.saveState(state);

    const loaded_state = db.loadState(123);
    try testing.expect(loaded_state != null);
    if (loaded_state) |data| {
        try testing.expectEqualStrings("test state data", data);
    }

    // 测试记忆操作
    const memory = Memory.init(123, MemoryType.episodic, "test memory", 0.9);
    try db.storeMemory(memory);

    const memory_count = db.retrieveMemories(123, 10);
    try testing.expectEqual(@as(usize, 1), memory_count);

    // 测试文档操作
    const doc = Document.init("Test Doc", "This is a test document with some content", 50, 10);
    try db.indexDocument(doc);

    const search_results = db.searchText("test document", 5);
    try testing.expectEqual(@as(usize, 1), search_results);

    // 测试上下文构建
    const context = try db.buildContext("test", 200, allocator);
    defer allocator.free(context);

    try testing.expect(context.len > 0);
    try testing.expect(std.mem.indexOf(u8, context, "test") != null);
}

test "Multiple State Types" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var db = MockAgentDatabase.init(allocator);
    defer db.deinit();

    const state_types = [_]StateType{
        StateType.working_memory,
        StateType.long_term_memory,
        StateType.context,
        StateType.task_state,
        StateType.relationship,
        StateType.embedding,
    };

    for (state_types, 0..) |state_type, i| {
        const agent_id = @as(u64, @intCast(i + 1000));
        const state = AgentState.init(agent_id, 0, state_type, "test data");
        try db.saveState(state);

        const loaded = db.loadState(agent_id);
        try testing.expect(loaded != null);
    }
}

test "Multiple Memory Types" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var db = MockAgentDatabase.init(allocator);
    defer db.deinit();

    const memory_types = [_]MemoryType{
        MemoryType.episodic,
        MemoryType.semantic,
        MemoryType.procedural,
        MemoryType.working,
    };

    const agent_id = 2000;
    for (memory_types) |memory_type| {
        const memory = Memory.init(agent_id, memory_type, "test memory content", 0.7);
        try db.storeMemory(memory);
    }

    const memory_count = db.retrieveMemories(agent_id, 10);
    try testing.expectEqual(@as(usize, 4), memory_count);
}

test "Document Search Functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var db = MockAgentDatabase.init(allocator);
    defer db.deinit();

    const documents = [_]Document{
        Document.init("AI Basics", "Artificial intelligence is the simulation of human intelligence", 100, 20),
        Document.init("ML Guide", "Machine learning is a subset of artificial intelligence", 100, 20),
        Document.init("DL Tutorial", "Deep learning uses neural networks with multiple layers", 100, 20),
        Document.init("NLP Overview", "Natural language processing deals with human language", 100, 20),
    };

    for (documents) |doc| {
        try db.indexDocument(doc);
    }

    // 搜索不同的查询
    const ai_results = db.searchText("artificial", 10);
    try testing.expect(ai_results >= 1); // AI Basics 和 ML Guide

    const learning_results = db.searchText("learning", 10);
    try testing.expect(learning_results >= 1); // ML Guide 和 DL Tutorial

    const neural_results = db.searchText("neural", 10);
    try testing.expect(neural_results >= 1); // DL Tutorial
}
