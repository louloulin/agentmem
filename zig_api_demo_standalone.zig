const std = @import("std");

// 简化的Zig API演示（独立版本）
pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("🚀 Zig Agent Database API Demo\n", .{});
    std.debug.print("==============================\n\n", .{});

    // 1. 基本数据结构演示
    std.debug.print("1. Basic Data Structures Demo\n", .{});

    const StateType = enum {
        working_memory,
        long_term_memory,
        context,
        task_state,
        relationship,
        embedding,

        pub fn toString(self: @This()) []const u8 {
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

    const MemoryType = enum {
        episodic,
        semantic,
        procedural,
        working,

        pub fn toString(self: @This()) []const u8 {
            return switch (self) {
                .episodic => "episodic",
                .semantic => "semantic",
                .procedural => "procedural",
                .working => "working",
            };
        }
    };

    // 演示状态类型
    const state_types = [_]StateType{ .working_memory, .long_term_memory, .context, .task_state, .relationship, .embedding };

    for (state_types) |state_type| {
        std.debug.print("✅ State type: {s}\n", .{state_type.toString()});
    }

    // 演示记忆类型
    const memory_types = [_]MemoryType{ .episodic, .semantic, .procedural, .working };

    for (memory_types) |memory_type| {
        std.debug.print("✅ Memory type: {s}\n", .{memory_type.toString()});
    }
    std.debug.print("\n", .{});

    // 2. Agent状态结构演示
    std.debug.print("2. Agent State Structure Demo\n", .{});

    const AgentState = struct {
        agent_id: u64,
        session_id: u64,
        state_type: StateType,
        data: []const u8,

        pub fn init(agent_id: u64, session_id: u64, state_type: StateType, data: []const u8) @This() {
            return @This(){
                .agent_id = agent_id,
                .session_id = session_id,
                .state_type = state_type,
                .data = data,
            };
        }

        pub fn display(self: @This()) void {
            std.debug.print("   Agent ID: {}, Session: {}, Type: {s}, Data: {s}\n", .{ self.agent_id, self.session_id, self.state_type.toString(), self.data });
        }
    };

    const sample_states = [_]AgentState{
        AgentState.init(12345, 1, .working_memory, "Processing user query about AI"),
        AgentState.init(12345, 1, .context, "User is learning about neural networks"),
        AgentState.init(12345, 1, .task_state, "Explaining backpropagation algorithm"),
    };

    for (sample_states) |state| {
        state.display();
    }
    std.debug.print("\n", .{});

    // 3. 记忆结构演示
    std.debug.print("3. Memory Structure Demo\n", .{});

    const Memory = struct {
        agent_id: u64,
        memory_type: MemoryType,
        content: []const u8,
        importance: f32,

        pub fn init(agent_id: u64, memory_type: MemoryType, content: []const u8, importance: f32) @This() {
            return @This(){
                .agent_id = agent_id,
                .memory_type = memory_type,
                .content = content,
                .importance = importance,
            };
        }

        pub fn display(self: @This()) void {
            std.debug.print("   Agent: {}, Type: {s}, Importance: {d:.1}, Content: {s}\n", .{ self.agent_id, self.memory_type.toString(), self.importance, self.content });
        }
    };

    const sample_memories = [_]Memory{
        Memory.init(12345, .episodic, "User asked about neural networks at 14:30", 0.9),
        Memory.init(12345, .semantic, "Neural networks are inspired by biological neurons", 0.8),
        Memory.init(12345, .procedural, "To explain AI: start simple, use analogies, provide examples", 0.7),
        Memory.init(12345, .working, "Currently explaining: gradient descent optimization", 0.6),
    };

    for (sample_memories) |memory| {
        memory.display();
    }
    std.debug.print("\n", .{});

    // 4. 文档结构演示
    std.debug.print("4. Document Structure Demo\n", .{});

    const Document = struct {
        title: []const u8,
        content: []const u8,
        chunk_size: usize,
        overlap: usize,

        pub fn init(title: []const u8, content: []const u8, chunk_size: usize, overlap: usize) @This() {
            return @This(){
                .title = title,
                .content = content,
                .chunk_size = chunk_size,
                .overlap = overlap,
            };
        }

        pub fn display(self: @This()) void {
            const preview_len = @min(80, self.content.len);
            std.debug.print("   Title: {s}\n", .{self.title});
            std.debug.print("   Content: {s}...\n", .{self.content[0..preview_len]});
            std.debug.print("   Chunk size: {}, Overlap: {}\n", .{ self.chunk_size, self.overlap });
        }
    };

    const sample_documents = [_]Document{
        Document.init("Neural Networks Basics", "Neural networks are computing systems inspired by biological neural networks that constitute animal brains. They learn by adjusting weights between neurons.", 150, 30),
        Document.init("Machine Learning Overview", "Machine learning is a method of data analysis that automates analytical model building using algorithms that iteratively learn from data.", 150, 30),
    };

    for (sample_documents) |doc| {
        doc.display();
        std.debug.print("\n", .{});
    }

    // 5. 简单的内存数据库演示
    std.debug.print("5. Simple In-Memory Database Demo\n", .{});

    var agent_states = std.HashMap(u64, []const u8, std.hash_map.AutoContext(u64), std.hash_map.default_max_load_percentage).init(allocator);
    defer agent_states.deinit();

    var memories = std.ArrayList(Memory).init(allocator);
    defer memories.deinit();

    var documents = std.ArrayList(Document).init(allocator);
    defer documents.deinit();

    // 存储一些数据
    try agent_states.put(12345, "AI Assistant Agent - Active");
    try agent_states.put(67890, "Learning Agent - Training Mode");

    try memories.append(Memory.init(12345, .semantic, "Knowledge about machine learning", 0.9));
    try memories.append(Memory.init(67890, .episodic, "Completed training session #42", 0.7));

    try documents.append(Document.init("AI Guide", "Comprehensive guide to artificial intelligence", 200, 40));

    std.debug.print("✅ Stored {} agent states\n", .{agent_states.count()});
    std.debug.print("✅ Stored {} memories\n", .{memories.items.len});
    std.debug.print("✅ Stored {} documents\n", .{documents.items.len});
    std.debug.print("\n", .{});

    // 6. 搜索功能演示
    std.debug.print("6. Search Functionality Demo\n", .{});

    // 简单的文本搜索
    const search_query = "machine learning";
    var found_docs: usize = 0;

    for (documents.items) |doc| {
        if (std.mem.indexOf(u8, doc.content, search_query) != null) {
            found_docs += 1;
            std.debug.print("🔍 Found in document: {s}\n", .{doc.title});
        }
    }

    std.debug.print("✅ Search for '{s}' found {} documents\n", .{ search_query, found_docs });
    std.debug.print("\n", .{});

    // 7. 性能测试
    std.debug.print("7. Performance Test\n", .{});

    const start_time = std.time.milliTimestamp();

    // 批量操作
    for (0..100) |i| {
        const agent_id = @as(u64, @intCast(20000 + i));
        const data = try std.fmt.allocPrint(allocator, "Batch agent {}", .{i});
        defer allocator.free(data);

        // 这里只是演示，实际中会存储到数据库
        _ = agent_id;
    }

    const end_time = std.time.milliTimestamp();
    std.debug.print("✅ Batch operation (100 agents) completed in {} ms\n", .{end_time - start_time});
    std.debug.print("\n", .{});

    // 8. 总结
    std.debug.print("🎉 Zig API Demo Completed Successfully!\n", .{});
    std.debug.print("📊 Demonstrated Features:\n", .{});
    std.debug.print("   ✅ Type-safe enums for states and memories\n", .{});
    std.debug.print("   ✅ Structured data types for agents, memories, documents\n", .{});
    std.debug.print("   ✅ In-memory storage with HashMap and ArrayList\n", .{});
    std.debug.print("   ✅ Basic search functionality\n", .{});
    std.debug.print("   ✅ Performance testing capabilities\n", .{});
    std.debug.print("   ✅ Memory-safe operations with allocators\n", .{});
    std.debug.print("\n🚀 Ready for integration with Rust backend!\n", .{});
}
