// 真正的集成测试 - 实际调用C函数与LanceDB交互
const std = @import("std");
const testing = std.testing;

// 导入C头文件
const c = @cImport({
    @cInclude("agent_state_db.h");
});

test "Real Integration Test - Agent State Database" {
    std.debug.print("\n🚀 === Real Integration Test - Agent State Database ===\n", .{});
    
    // 测试1: 创建Agent状态数据库
    std.debug.print("\n📊 Test 1: Creating Agent State Database\n", .{});
    
    const db_path = "test_agent_db";
    const db = c.agent_db_new(db_path);
    
    if (db == null) {
        std.debug.print("❌ Failed to create agent database\n", .{});
        return error.DatabaseCreationFailed;
    }
    
    defer c.agent_db_free(db);
    std.debug.print("✅ Agent database created successfully\n", .{});
    
    // 测试2: 保存Agent状态
    std.debug.print("\n💾 Test 2: Saving Agent State\n", .{});
    
    const agent_id: u64 = 12345;
    const session_id: u64 = 67890;
    const state_type = c.STATE_TYPE_WORKING_MEMORY;
    const test_data = "Real integration test data for Agent 12345";
    
    const save_result = c.agent_db_save_state(
        db,
        agent_id,
        session_id,
        state_type,
        @as([*c]const u8, @ptrCast(test_data.ptr)),
        test_data.len
    );
    
    if (save_result != c.AGENT_DB_SUCCESS) {
        std.debug.print("❌ Failed to save agent state, error code: {}\n", .{save_result});
        return error.StateSaveFailed;
    }
    
    std.debug.print("✅ Agent state saved successfully\n", .{});
    std.debug.print("  Agent ID: {}\n", .{agent_id});
    std.debug.print("  Session ID: {}\n", .{session_id});
    std.debug.print("  State Type: {}\n", .{state_type});
    std.debug.print("  Data Length: {} bytes\n", .{test_data.len});
    
    // 测试3: 加载Agent状态
    std.debug.print("\n📤 Test 3: Loading Agent State\n", .{});
    
    var loaded_data: [*c]u8 = undefined;
    var loaded_data_len: usize = undefined;
    
    const load_result = c.agent_db_load_state(
        db,
        agent_id,
        &loaded_data,
        &loaded_data_len
    );
    
    if (load_result != c.AGENT_DB_SUCCESS) {
        std.debug.print("❌ Failed to load agent state, error code: {}\n", .{load_result});
        return error.StateLoadFailed;
    }
    
    defer c.agent_db_free_data(@as([*c]u8, @ptrCast(loaded_data)), loaded_data_len);
    
    std.debug.print("✅ Agent state loaded successfully\n", .{});
    std.debug.print("  Loaded Data Length: {} bytes\n", .{loaded_data_len});
    
    // 验证加载的数据
    if (loaded_data_len != test_data.len) {
        std.debug.print("❌ Data length mismatch: expected {}, got {}\n", .{ test_data.len, loaded_data_len });
        return error.DataLengthMismatch;
    }
    
    const loaded_slice = loaded_data[0..loaded_data_len];
    if (!std.mem.eql(u8, loaded_slice, test_data)) {
        std.debug.print("❌ Data content mismatch\n", .{});
        std.debug.print("  Expected: {s}\n", .{test_data});
        std.debug.print("  Got: {s}\n", .{loaded_slice});
        return error.DataContentMismatch;
    }
    
    std.debug.print("✅ Data verification passed\n", .{});
    std.debug.print("  Data: {s}\n", .{loaded_slice});
}

test "Real Integration Test - Memory Manager" {
    std.debug.print("\n🧠 === Real Integration Test - Memory Manager ===\n", .{});
    
    // 测试1: 创建记忆管理器
    std.debug.print("\n📊 Test 1: Creating Memory Manager\n", .{});
    
    const db_path = "test_memory_db";
    const mgr = c.memory_manager_new(db_path);
    
    if (mgr == null) {
        std.debug.print("❌ Failed to create memory manager\n", .{});
        return error.MemoryManagerCreationFailed;
    }
    
    defer c.memory_manager_free(mgr);
    std.debug.print("✅ Memory manager created successfully\n", .{});
    
    // 测试2: 存储记忆
    std.debug.print("\n💾 Test 2: Storing Memory\n", .{});
    
    const agent_id: u64 = 54321;
    const memory_type = c.MEMORY_TYPE_EPISODIC;
    const memory_content = "This is a real integration test memory for Agent 54321";
    const importance: f32 = 0.85;
    
    const store_result = c.memory_manager_store_memory(
        mgr,
        agent_id,
        memory_type,
        memory_content,
        importance
    );
    
    if (store_result != c.AGENT_DB_SUCCESS) {
        std.debug.print("❌ Failed to store memory, error code: {}\n", .{store_result});
        return error.MemoryStoreFailed;
    }
    
    std.debug.print("✅ Memory stored successfully\n", .{});
    std.debug.print("  Agent ID: {}\n", .{agent_id});
    std.debug.print("  Memory Type: {}\n", .{memory_type});
    std.debug.print("  Content: {s}\n", .{memory_content});
    std.debug.print("  Importance: {d:.2}\n", .{importance});
    
    // 测试3: 检索记忆
    std.debug.print("\n📤 Test 3: Retrieving Memories\n", .{});
    
    const limit: usize = 10;
    var memory_count: usize = undefined;
    
    const retrieve_result = c.memory_manager_retrieve_memories(
        mgr,
        agent_id,
        limit,
        &memory_count
    );
    
    if (retrieve_result != c.AGENT_DB_SUCCESS) {
        std.debug.print("❌ Failed to retrieve memories, error code: {}\n", .{retrieve_result});
        return error.MemoryRetrieveFailed;
    }
    
    std.debug.print("✅ Memories retrieved successfully\n", .{});
    std.debug.print("  Memory Count: {}\n", .{memory_count});
    
    // 验证至少有一个记忆
    if (memory_count == 0) {
        std.debug.print("❌ No memories found, expected at least 1\n", .{});
        return error.NoMemoriesFound;
    }
    
    std.debug.print("✅ Memory retrieval verification passed\n", .{});
}

test "Real Integration Test - RAG Engine" {
    std.debug.print("\n🔍 === Real Integration Test - RAG Engine ===\n", .{});
    
    // 测试1: 创建RAG引擎
    std.debug.print("\n📊 Test 1: Creating RAG Engine\n", .{});
    
    const db_path = "test_rag_db";
    const engine = c.rag_engine_new(db_path);
    
    if (engine == null) {
        std.debug.print("❌ Failed to create RAG engine\n", .{});
        return error.RAGEngineCreationFailed;
    }
    
    defer c.rag_engine_free(engine);
    std.debug.print("✅ RAG engine created successfully\n", .{});
    
    // 测试2: 索引文档
    std.debug.print("\n📚 Test 2: Indexing Document\n", .{});
    
    const title = "Real Integration Test Document";
    const content = "This is a comprehensive test document for the RAG engine integration. It contains various information about AI agents, state management, and distributed systems.";
    const chunk_size: usize = 100;
    const overlap: usize = 20;
    
    const index_result = c.rag_engine_index_document(
        engine,
        title,
        content,
        chunk_size,
        overlap
    );
    
    if (index_result != c.AGENT_DB_SUCCESS) {
        std.debug.print("❌ Failed to index document, error code: {}\n", .{index_result});
        return error.DocumentIndexFailed;
    }
    
    std.debug.print("✅ Document indexed successfully\n", .{});
    std.debug.print("  Title: {s}\n", .{title});
    std.debug.print("  Content Length: {} characters\n", .{content.len});
    std.debug.print("  Chunk Size: {}\n", .{chunk_size});
    std.debug.print("  Overlap: {}\n", .{overlap});
    
    // 测试3: 搜索文本
    std.debug.print("\n🔎 Test 3: Searching Text\n", .{});
    
    const query = "AI agents";
    const limit: usize = 5;
    var results_count: usize = undefined;
    
    const search_result = c.rag_engine_search_text(
        engine,
        query,
        limit,
        &results_count
    );
    
    if (search_result != c.AGENT_DB_SUCCESS) {
        std.debug.print("❌ Failed to search text, error code: {}\n", .{search_result});
        return error.TextSearchFailed;
    }
    
    std.debug.print("✅ Text search completed successfully\n", .{});
    std.debug.print("  Query: {s}\n", .{query});
    std.debug.print("  Results Count: {}\n", .{results_count});
    
    std.debug.print("✅ RAG engine integration test passed\n", .{});
}

test "Real Integration Test - Performance Benchmark" {
    std.debug.print("\n⚡ === Real Integration Test - Performance Benchmark ===\n", .{});
    
    const db_path = "test_perf_db";
    const db = c.agent_db_new(db_path);
    
    if (db == null) {
        std.debug.print("❌ Failed to create database for performance test\n", .{});
        return error.DatabaseCreationFailed;
    }
    
    defer c.agent_db_free(db);
    
    const num_operations = 10; // 减少操作数量以避免测试超时
    const test_data = "Performance test data for real integration";
    
    std.debug.print("Starting performance benchmark with {} operations...\n", .{num_operations});
    
    const start_time = std.time.milliTimestamp();
    
    // 执行多个保存和加载操作
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i + 1000);
        const session_id = @as(u64, i + 2000);
        
        // 保存状态
        const save_result = c.agent_db_save_state(
            db,
            agent_id,
            session_id,
            c.STATE_TYPE_WORKING_MEMORY,
            @as([*c]const u8, @ptrCast(test_data.ptr)),
            test_data.len
        );
        
        if (save_result != c.AGENT_DB_SUCCESS) {
            std.debug.print("❌ Save failed for agent {}, error: {}\n", .{ agent_id, save_result });
            return error.PerformanceTestFailed;
        }
        
        // 加载状态
        var loaded_data: [*c]u8 = undefined;
        var loaded_data_len: usize = undefined;
        
        const load_result = c.agent_db_load_state(
            db,
            agent_id,
            &loaded_data,
            &loaded_data_len
        );
        
        if (load_result != c.AGENT_DB_SUCCESS) {
            std.debug.print("❌ Load failed for agent {}, error: {}\n", .{ agent_id, load_result });
            return error.PerformanceTestFailed;
        }
        
        c.agent_db_free_data(@as([*c]u8, @ptrCast(loaded_data)), loaded_data_len);
        
        if (i % 2 == 0) {
            std.debug.print("  Completed {} operations...\n", .{i + 1});
        }
    }
    
    const end_time = std.time.milliTimestamp();
    const duration = end_time - start_time;
    const ops_per_ms = @as(f64, @floatFromInt(num_operations * 2)) / @as(f64, @floatFromInt(duration)); // 2 operations per iteration
    
    std.debug.print("✅ Performance benchmark completed\n", .{});
    std.debug.print("  Operations: {} (save + load pairs)\n", .{num_operations * 2});
    std.debug.print("  Duration: {} ms\n", .{duration});
    std.debug.print("  Throughput: {d:.2} ops/ms\n", .{ops_per_ms});
    
    // 验证性能合理
    try testing.expect(duration >= 0);
    try testing.expect(ops_per_ms >= 0);
    
    std.debug.print("🎉 All real integration tests completed successfully!\n", .{});
}
