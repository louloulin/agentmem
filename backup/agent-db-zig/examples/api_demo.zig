const std = @import("std");
const agent_db = @import("agent_db");
const AgentDatabase = agent_db.AgentDatabase;
const AgentState = agent_db.AgentState;
const StateType = agent_db.StateType;
const Memory = agent_db.Memory;
const MemoryType = agent_db.MemoryType;
const Document = agent_db.Document;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("🚀 Zig Agent Database API Demo\n", .{});
    std.debug.print("================================\n\n", .{});

    // 1. 初始化数据库
    std.debug.print("1. Initializing Agent Database...\n", .{});
    var db = AgentDatabase.init(allocator, "demo_zig_api.lance") catch |err| {
        std.debug.print("❌ Failed to initialize database: {}\n", .{err});
        return;
    };
    defer db.deinit();
    std.debug.print("✅ Database initialized successfully\n\n", .{});

    // 2. 创建和管理Agent状态
    std.debug.print("2. Agent State Management\n", .{});
    const agent_id = 12345;

    // 创建Agent
    try db.createAgent(agent_id, "Initial agent state data");
    std.debug.print("✅ Created agent {}\n", .{agent_id});

    // 更新Agent状态
    try db.updateAgent(agent_id, "Updated agent state with new information");
    std.debug.print("✅ Updated agent state\n", .{});

    // 加载Agent状态
    const loaded_state = try db.loadState(agent_id);
    defer if (loaded_state) |data| allocator.free(data);

    if (loaded_state) |data| {
        std.debug.print("✅ Loaded agent state: {s}\n", .{data});
    }
    std.debug.print("\n", .{});

    // 3. 向量状态管理
    std.debug.print("3. Vector State Management\n", .{});
    const vector_agent_id = 54321;
    const test_embedding = [_]f32{ 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 };

    // 保存向量状态
    const vector_state = AgentState.init(vector_agent_id, 1, StateType.embedding, "Vector state data");
    try db.saveVectorState(vector_state, &test_embedding);
    std.debug.print("✅ Saved vector state for agent {}\n", .{vector_agent_id});

    // 向量搜索
    var search_results = try db.vectorSearch(&test_embedding, 5);
    defer search_results.deinit();
    std.debug.print("✅ Vector search found {} results\n", .{search_results.count});

    for (search_results.agent_ids, 0..) |found_agent_id, i| {
        std.debug.print("   Result {}: Agent ID {}\n", .{ i + 1, found_agent_id });
    }
    std.debug.print("\n", .{});

    // 4. 记忆管理
    std.debug.print("4. Memory Management\n", .{});

    // 添加不同类型的记忆
    const memory_types = [_]struct { MemoryType, []const u8, f32 }{
        .{ MemoryType.episodic, "I remember meeting John at the conference last week", 0.8 },
        .{ MemoryType.semantic, "The capital of France is Paris", 0.9 },
        .{ MemoryType.procedural, "To make coffee: 1) Heat water 2) Add coffee 3) Brew", 0.7 },
        .{ MemoryType.working, "Current task: Analyze quarterly sales data", 0.6 },
    };

    for (memory_types) |memory_info| {
        try db.addMemory(agent_id, memory_info[1], memory_info[0], memory_info[2]);
        std.debug.print("✅ Added {} memory: {s}\n", .{ memory_info[0], memory_info[1] });
    }

    // 检索记忆
    const memory_count = try db.retrieveMemories(agent_id, 10);
    std.debug.print("✅ Retrieved {} memories for agent {}\n\n", .{ memory_count, agent_id });

    // 5. 文档索引和RAG
    std.debug.print("5. Document Indexing and RAG\n", .{});

    // 索引多个文档
    const documents = [_]struct { []const u8, []const u8 }{
        .{ "Artificial Intelligence Overview", "Artificial Intelligence (AI) is the simulation of human intelligence in machines that are programmed to think and learn like humans. AI systems can perform tasks that typically require human intelligence, such as visual perception, speech recognition, decision-making, and language translation." },
        .{ "Machine Learning Fundamentals", "Machine Learning is a subset of AI that provides systems the ability to automatically learn and improve from experience without being explicitly programmed. ML focuses on the development of computer programs that can access data and use it to learn for themselves." },
        .{ "Deep Learning Introduction", "Deep Learning is a subset of machine learning that uses neural networks with multiple layers (deep neural networks) to model and understand complex patterns in data. It has been particularly successful in areas like image recognition, natural language processing, and speech recognition." },
        .{ "Natural Language Processing", "Natural Language Processing (NLP) is a branch of AI that helps computers understand, interpret and manipulate human language. NLP draws from many disciplines, including computer science and computational linguistics, in its pursuit to fill the gap between human communication and computer understanding." },
    };

    for (documents) |doc| {
        try db.addDocument(doc[0], doc[1]);
        std.debug.print("✅ Indexed document: {s}\n", .{doc[0]});
    }

    // 执行文本搜索
    const search_queries = [_][]const u8{
        "machine learning",
        "neural networks",
        "artificial intelligence",
        "natural language",
    };

    std.debug.print("\n📊 Search Results:\n", .{});
    for (search_queries) |query| {
        const results_count = try db.searchText(query, 3);
        std.debug.print("   Query '{s}': {} results\n", .{ query, results_count });
    }

    // 6. RAG上下文构建
    std.debug.print("\n6. RAG Context Building\n", .{});

    const rag_queries = [_][]const u8{
        "What is artificial intelligence?",
        "How does machine learning work?",
        "What are neural networks?",
    };

    for (rag_queries) |query| {
        const context = try db.buildContext(query, 300);
        defer allocator.free(context);

        std.debug.print("🤖 Query: {s}\n", .{query});
        std.debug.print("📝 Context (first 150 chars): {s}...\n", .{context[0..@min(150, context.len)]});
        std.debug.print("📏 Full context length: {} characters\n\n", .{context.len});
    }

    // 7. 高级功能演示
    std.debug.print("7. Advanced Features Demo\n", .{});

    // 批量操作
    std.debug.print("Performing batch operations...\n", .{});
    const batch_start = std.time.milliTimestamp();

    for (0..50) |i| {
        const data = try std.fmt.allocPrint(allocator, "Batch agent {} data", .{i});
        defer allocator.free(data);

        try db.createAgent(@intCast(10000 + i), data);

        const memory_content = try std.fmt.allocPrint(allocator, "Batch memory {} content", .{i});
        defer allocator.free(memory_content);

        try db.addMemory(@intCast(10000 + i), memory_content, MemoryType.working, 0.5);
    }

    const batch_end = std.time.milliTimestamp();
    std.debug.print("✅ Batch operations completed in {} ms\n", .{batch_end - batch_start});

    // 8. 状态类型演示
    std.debug.print("\n8. State Types Demo\n", .{});
    const demo_agent_id = 99999;

    const state_demos = [_]struct { StateType, []const u8 }{
        .{ StateType.working_memory, "Current working memory content" },
        .{ StateType.long_term_memory, "Long-term stored information" },
        .{ StateType.context, "Current conversation context" },
        .{ StateType.task_state, "Active task information" },
        .{ StateType.relationship, "Relationship mapping data" },
        .{ StateType.embedding, "Vector embedding representation" },
    };

    for (state_demos) |demo| {
        const state = AgentState.init(demo_agent_id, 0, demo[0], demo[1]);
        try db.saveState(state);
        std.debug.print("✅ Saved {} state: {s}\n", .{ demo[0], demo[1] });
    }

    std.debug.print("\n🎉 Demo completed successfully!\n", .{});
    std.debug.print("📊 Summary:\n", .{});
    std.debug.print("   - Created and managed multiple agents\n", .{});
    std.debug.print("   - Demonstrated vector operations\n", .{});
    std.debug.print("   - Stored and retrieved memories\n", .{});
    std.debug.print("   - Indexed and searched documents\n", .{});
    std.debug.print("   - Built RAG contexts\n", .{});
    std.debug.print("   - Performed batch operations\n", .{});
    std.debug.print("   - Showcased all state types\n", .{});
}
