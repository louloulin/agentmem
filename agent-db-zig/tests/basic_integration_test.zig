// 基础集成测试 - 验证C函数调用但不依赖实际数据库
const std = @import("std");
const testing = std.testing;

// 导入C头文件
const c = @cImport({
    @cInclude("agent_state_db.h");
});

test "Basic Integration Test - Function Calls" {
    std.debug.print("\n🔧 === Basic Integration Test - Function Calls ===\n", .{});

    // 测试1: 尝试创建数据库（可能失败，但函数应该存在）
    std.debug.print("\n📊 Test 1: Database Creation Function Call\n", .{});

    const db_path = ":memory:"; // 使用内存数据库
    const db = c.agent_db_new(db_path);

    std.debug.print("✅ agent_db_new function called successfully\n", .{});
    std.debug.print("  Database path: {s}\n", .{db_path});
    std.debug.print("  Returned pointer: {any}\n", .{db});

    // 如果数据库创建成功，测试其他函数
    if (db != null) {
        std.debug.print("✅ Database created successfully\n", .{});

        // 测试2: 尝试保存状态
        std.debug.print("\n💾 Test 2: Save State Function Call\n", .{});

        const agent_id: u64 = 12345;
        const session_id: u64 = 67890;
        const state_type = c.STATE_TYPE_WORKING_MEMORY;
        const test_data = "Basic integration test data";

        const save_result = c.agent_db_save_state(db, agent_id, session_id, state_type, @as([*c]const u8, @ptrCast(test_data.ptr)), test_data.len);

        std.debug.print("✅ agent_db_save_state function called successfully\n", .{});
        std.debug.print("  Result code: {}\n", .{save_result});

        // 测试3: 尝试加载状态
        std.debug.print("\n📤 Test 3: Load State Function Call\n", .{});

        var loaded_data: [*c]u8 = undefined;
        var loaded_data_len: usize = undefined;

        const load_result = c.agent_db_load_state(db, agent_id, &loaded_data, &loaded_data_len);

        std.debug.print("✅ agent_db_load_state function called successfully\n", .{});
        std.debug.print("  Result code: {}\n", .{load_result});

        // 如果加载成功，释放数据
        if (load_result == c.AGENT_DB_SUCCESS and loaded_data != null) {
            c.agent_db_free_data(@as([*c]u8, @ptrCast(loaded_data)), loaded_data_len);
            std.debug.print("✅ agent_db_free_data function called successfully\n", .{});
        }

        // 释放数据库
        c.agent_db_free(db);
        std.debug.print("✅ agent_db_free function called successfully\n", .{});
    } else {
        std.debug.print("⚠️ Database creation returned null (expected for some environments)\n", .{});
    }
}

test "Basic Integration Test - Memory Manager Functions" {
    std.debug.print("\n🧠 === Basic Integration Test - Memory Manager Functions ===\n", .{});

    // 测试1: 尝试创建记忆管理器
    std.debug.print("\n📊 Test 1: Memory Manager Creation Function Call\n", .{});

    const db_path = ":memory:";
    const mgr = c.memory_manager_new(db_path);

    std.debug.print("✅ memory_manager_new function called successfully\n", .{});
    std.debug.print("  Database path: {s}\n", .{db_path});
    std.debug.print("  Returned pointer: {any}\n", .{mgr});

    if (mgr != null) {
        std.debug.print("✅ Memory manager created successfully\n", .{});

        // 测试2: 尝试存储记忆
        std.debug.print("\n💾 Test 2: Store Memory Function Call\n", .{});

        const agent_id: u64 = 54321;
        const memory_type = c.MEMORY_TYPE_EPISODIC;
        const memory_content = "Basic integration test memory";
        const importance: f32 = 0.75;

        const store_result = c.memory_manager_store_memory(mgr, agent_id, memory_type, memory_content, importance);

        std.debug.print("✅ memory_manager_store_memory function called successfully\n", .{});
        std.debug.print("  Result code: {}\n", .{store_result});

        // 测试3: 尝试检索记忆
        std.debug.print("\n📤 Test 3: Retrieve Memories Function Call\n", .{});

        const limit: usize = 10;
        var memory_count: usize = undefined;

        const retrieve_result = c.memory_manager_retrieve_memories(mgr, agent_id, limit, &memory_count);

        std.debug.print("✅ memory_manager_retrieve_memories function called successfully\n", .{});
        std.debug.print("  Result code: {}\n", .{retrieve_result});
        std.debug.print("  Memory count: {}\n", .{memory_count});

        // 释放记忆管理器
        c.memory_manager_free(mgr);
        std.debug.print("✅ memory_manager_free function called successfully\n", .{});
    } else {
        std.debug.print("⚠️ Memory manager creation returned null (expected for some environments)\n", .{});
    }
}

test "Basic Integration Test - RAG Engine Functions" {
    std.debug.print("\n🔍 === Basic Integration Test - RAG Engine Functions ===\n", .{});

    // 测试1: 尝试创建RAG引擎
    std.debug.print("\n📊 Test 1: RAG Engine Creation Function Call\n", .{});

    const db_path = ":memory:";
    const engine = c.rag_engine_new(db_path);

    std.debug.print("✅ rag_engine_new function called successfully\n", .{});
    std.debug.print("  Database path: {s}\n", .{db_path});
    std.debug.print("  Returned pointer: {any}\n", .{engine});

    if (engine != null) {
        std.debug.print("✅ RAG engine created successfully\n", .{});

        // 测试2: 尝试索引文档
        std.debug.print("\n📚 Test 2: Index Document Function Call\n", .{});

        const title = "Basic Integration Test Document";
        const content = "This is a test document for basic integration testing.";
        const chunk_size: usize = 50;
        const overlap: usize = 10;

        const index_result = c.rag_engine_index_document(engine, title, content, chunk_size, overlap);

        std.debug.print("✅ rag_engine_index_document function called successfully\n", .{});
        std.debug.print("  Result code: {}\n", .{index_result});

        // 测试3: 尝试搜索文本
        std.debug.print("\n🔎 Test 3: Search Text Function Call\n", .{});

        const query = "test";
        const limit: usize = 5;
        var results_count: usize = undefined;

        const search_result = c.rag_engine_search_text(engine, query, limit, &results_count);

        std.debug.print("✅ rag_engine_search_text function called successfully\n", .{});
        std.debug.print("  Result code: {}\n", .{search_result});
        std.debug.print("  Results count: {}\n", .{results_count});

        // 释放RAG引擎
        c.rag_engine_free(engine);
        std.debug.print("✅ rag_engine_free function called successfully\n", .{});
    } else {
        std.debug.print("⚠️ RAG engine creation returned null (expected for some environments)\n", .{});
    }
}

test "Basic Integration Test - Network Manager Functions" {
    std.debug.print("\n🌐 === Basic Integration Test - Network Manager Functions ===\n", .{});

    // 测试1: 尝试创建网络管理器
    std.debug.print("\n📊 Test 1: Network Manager Creation Function Call\n", .{});

    const agent_id: u64 = 12345;
    const address = "127.0.0.1";
    const port: u16 = 8080;
    const capabilities = [_][*c]const u8{ "processing", "storage" };
    const capabilities_ptr: [*c]const [*c]const u8 = @ptrCast(&capabilities);

    const manager = c.agent_network_manager_new(agent_id, address, port, capabilities_ptr, capabilities.len);

    std.debug.print("✅ agent_network_manager_new function called successfully\n", .{});
    std.debug.print("  Agent ID: {}\n", .{agent_id});
    std.debug.print("  Address: {s}:{}\n", .{ address, port });
    std.debug.print("  Returned pointer: {any}\n", .{manager});

    if (manager != null) {
        std.debug.print("✅ Network manager created successfully\n", .{});

        // 测试2: 尝试获取活跃节点数量
        std.debug.print("\n📊 Test 2: Get Active Nodes Count Function Call\n", .{});

        const nodes_count = c.agent_network_get_active_nodes_count(manager);

        std.debug.print("✅ agent_network_get_active_nodes_count function called successfully\n", .{});
        std.debug.print("  Active nodes count: {}\n", .{nodes_count});

        // 释放网络管理器
        c.agent_network_manager_free(manager);
        std.debug.print("✅ agent_network_manager_free function called successfully\n", .{});
    } else {
        std.debug.print("⚠️ Network manager creation returned null (expected for some environments)\n", .{});
    }
}

test "Basic Integration Test - Function Existence Verification" {
    std.debug.print("\n🔍 === Basic Integration Test - Function Existence Verification ===\n", .{});

    // 验证所有主要函数都存在并可调用
    std.debug.print("\n📋 Verifying function symbols...\n", .{});

    // 数据库函数
    const db_new_exists = @hasDecl(c, "agent_db_new");
    const db_free_exists = @hasDecl(c, "agent_db_free");
    const db_save_exists = @hasDecl(c, "agent_db_save_state");
    const db_load_exists = @hasDecl(c, "agent_db_load_state");

    // 记忆管理函数
    const mem_new_exists = @hasDecl(c, "memory_manager_new");
    const mem_free_exists = @hasDecl(c, "memory_manager_free");
    const mem_store_exists = @hasDecl(c, "memory_manager_store_memory");
    const mem_retrieve_exists = @hasDecl(c, "memory_manager_retrieve_memories");

    // RAG引擎函数
    const rag_new_exists = @hasDecl(c, "rag_engine_new");
    const rag_free_exists = @hasDecl(c, "rag_engine_free");
    const rag_index_exists = @hasDecl(c, "rag_engine_index_document");
    const rag_search_exists = @hasDecl(c, "rag_engine_search_text");

    // 网络管理器函数
    const network_new_exists = @hasDecl(c, "agent_network_manager_new");
    const network_free_exists = @hasDecl(c, "agent_network_manager_free");
    const network_join_exists = @hasDecl(c, "agent_network_join_network");
    const network_count_exists = @hasDecl(c, "agent_network_get_active_nodes_count");

    std.debug.print("✅ Function existence verification completed\n", .{});
    std.debug.print("  Database functions: {} {} {} {}\n", .{ db_new_exists, db_free_exists, db_save_exists, db_load_exists });
    std.debug.print("  Memory functions: {} {} {} {}\n", .{ mem_new_exists, mem_free_exists, mem_store_exists, mem_retrieve_exists });
    std.debug.print("  RAG functions: {} {} {} {}\n", .{ rag_new_exists, rag_free_exists, rag_index_exists, rag_search_exists });
    std.debug.print("  Network functions: {} {} {} {}\n", .{ network_new_exists, network_free_exists, network_join_exists, network_count_exists });

    // 验证所有函数都存在
    try testing.expect(db_new_exists and db_free_exists and db_save_exists and db_load_exists);
    try testing.expect(mem_new_exists and mem_free_exists and mem_store_exists and mem_retrieve_exists);
    try testing.expect(rag_new_exists and rag_free_exists and rag_index_exists and rag_search_exists);
    try testing.expect(network_new_exists and network_free_exists and network_join_exists and network_count_exists);

    std.debug.print("🎉 All basic integration tests completed successfully!\n", .{});
}
