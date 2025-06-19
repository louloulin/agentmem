// 简单的FFI测试 - 只测试头文件导入
const std = @import("std");
const testing = std.testing;

// 尝试导入C头文件
const c = @cImport({
    @cInclude("agent_state_db.h");
});

test "Simple FFI Header Constants Test" {
    std.debug.print("\n=== Simple FFI Header Constants Test ===\n", .{});

    // 测试常量是否正确导入
    std.debug.print("Testing constants...\n", .{});

    try testing.expect(c.AGENT_DB_SUCCESS == 0);
    try testing.expect(c.AGENT_DB_ERROR == -1);
    try testing.expect(c.AGENT_DB_NOT_FOUND == 1);

    std.debug.print("✅ Constants test passed!\n", .{});
    std.debug.print("  AGENT_DB_SUCCESS = {}\n", .{c.AGENT_DB_SUCCESS});
    std.debug.print("  AGENT_DB_ERROR = {}\n", .{c.AGENT_DB_ERROR});
    std.debug.print("  AGENT_DB_NOT_FOUND = {}\n", .{c.AGENT_DB_NOT_FOUND});
}

test "Simple FFI State Type Constants Test" {
    std.debug.print("\n=== Simple FFI State Type Constants Test ===\n", .{});

    // 测试状态类型常量
    std.debug.print("Testing state type constants...\n", .{});

    try testing.expect(c.STATE_TYPE_WORKING_MEMORY == 0);
    try testing.expect(c.STATE_TYPE_LONG_TERM_MEMORY == 1);
    try testing.expect(c.STATE_TYPE_CONTEXT == 2);
    try testing.expect(c.STATE_TYPE_TASK_STATE == 3);
    try testing.expect(c.STATE_TYPE_RELATIONSHIP == 4);
    try testing.expect(c.STATE_TYPE_EMBEDDING == 5);

    std.debug.print("✅ State type constants test passed!\n", .{});
    std.debug.print("  STATE_TYPE_WORKING_MEMORY = {}\n", .{c.STATE_TYPE_WORKING_MEMORY});
    std.debug.print("  STATE_TYPE_LONG_TERM_MEMORY = {}\n", .{c.STATE_TYPE_LONG_TERM_MEMORY});
    std.debug.print("  STATE_TYPE_CONTEXT = {}\n", .{c.STATE_TYPE_CONTEXT});
    std.debug.print("  STATE_TYPE_TASK_STATE = {}\n", .{c.STATE_TYPE_TASK_STATE});
    std.debug.print("  STATE_TYPE_RELATIONSHIP = {}\n", .{c.STATE_TYPE_RELATIONSHIP});
    std.debug.print("  STATE_TYPE_EMBEDDING = {}\n", .{c.STATE_TYPE_EMBEDDING});
}

test "Simple FFI Memory Type Constants Test" {
    std.debug.print("\n=== Simple FFI Memory Type Constants Test ===\n", .{});

    // 测试记忆类型常量
    std.debug.print("Testing memory type constants...\n", .{});

    try testing.expect(c.MEMORY_TYPE_EPISODIC == 0);
    try testing.expect(c.MEMORY_TYPE_SEMANTIC == 1);
    try testing.expect(c.MEMORY_TYPE_PROCEDURAL == 2);
    try testing.expect(c.MEMORY_TYPE_WORKING == 3);

    std.debug.print("✅ Memory type constants test passed!\n", .{});
    std.debug.print("  MEMORY_TYPE_EPISODIC = {}\n", .{c.MEMORY_TYPE_EPISODIC});
    std.debug.print("  MEMORY_TYPE_SEMANTIC = {}\n", .{c.MEMORY_TYPE_SEMANTIC});
    std.debug.print("  MEMORY_TYPE_PROCEDURAL = {}\n", .{c.MEMORY_TYPE_PROCEDURAL});
    std.debug.print("  MEMORY_TYPE_WORKING = {}\n", .{c.MEMORY_TYPE_WORKING});
}

test "Simple FFI Opaque Pointer Types Test" {
    std.debug.print("\n=== Simple FFI Opaque Pointer Types Test ===\n", .{});

    // 测试不透明指针类型是否正确导入
    std.debug.print("Testing opaque pointer type availability...\n", .{});

    // 检查指针类型大小
    const agent_db_ptr_size = @sizeOf(?*c.CAgentStateDB);
    const memory_mgr_ptr_size = @sizeOf(?*c.CMemoryManager);
    const rag_engine_ptr_size = @sizeOf(?*c.CRAGEngine);
    const network_mgr_ptr_size = @sizeOf(?*c.CAgentNetworkManager);
    const stream_processor_ptr_size = @sizeOf(?*c.CRealTimeStreamProcessor);

    std.debug.print("✅ Opaque pointer types test passed!\n", .{});
    std.debug.print("  CAgentStateDB* size: {} bytes\n", .{agent_db_ptr_size});
    std.debug.print("  CMemoryManager* size: {} bytes\n", .{memory_mgr_ptr_size});
    std.debug.print("  CRAGEngine* size: {} bytes\n", .{rag_engine_ptr_size});
    std.debug.print("  CAgentNetworkManager* size: {} bytes\n", .{network_mgr_ptr_size});
    std.debug.print("  CRealTimeStreamProcessor* size: {} bytes\n", .{stream_processor_ptr_size});

    // 基本合理性检查 - 所有指针应该是8字节（64位系统）
    try testing.expect(agent_db_ptr_size == 8);
    try testing.expect(memory_mgr_ptr_size == 8);
    try testing.expect(rag_engine_ptr_size == 8);
    try testing.expect(network_mgr_ptr_size == 8);
    try testing.expect(stream_processor_ptr_size == 8);
}

test "Simple FFI Struct Definition Test" {
    std.debug.print("\n=== Simple FFI Struct Definition Test ===\n", .{});

    // 测试StreamProcessingStats结构体（这是头文件中完整定义的结构体）
    std.debug.print("Testing StreamProcessingStats struct...\n", .{});

    const stats_size = @sizeOf(c.StreamProcessingStats);

    // 创建一个结构体实例来测试字段访问
    var stats: c.StreamProcessingStats = undefined;
    stats.items_received = 100;
    stats.items_processed = 95;
    stats.items_dropped = 5;
    stats.batches_processed = 10;
    stats.avg_latency_ms = 1.5;
    stats.max_latency_ms = 10;
    stats.throughput_per_sec = 63.33;
    stats.buffer_utilization = 0.75;
    stats.error_count = 2;
    stats.last_update = 1234567890;

    std.debug.print("✅ StreamProcessingStats struct test passed!\n", .{});
    std.debug.print("  Struct size: {} bytes\n", .{stats_size});
    std.debug.print("  items_received: {}\n", .{stats.items_received});
    std.debug.print("  items_processed: {}\n", .{stats.items_processed});
    std.debug.print("  items_dropped: {}\n", .{stats.items_dropped});
    std.debug.print("  avg_latency_ms: {d:.2}\n", .{stats.avg_latency_ms});
    std.debug.print("  throughput_per_sec: {d:.2}\n", .{stats.throughput_per_sec});

    // 验证字段值
    try testing.expect(stats.items_received == 100);
    try testing.expect(stats.items_processed == 95);
    try testing.expect(stats.items_dropped == 5);
    try testing.expect(stats.batches_processed == 10);
    try testing.expect(stats.error_count == 2);
    try testing.expect(stats.last_update == 1234567890);
    try testing.expect(stats_size > 0);
}

// 不实际调用C函数的测试
test "Simple FFI Type Sizes Test" {
    std.debug.print("\n=== Simple FFI Type Sizes Test ===\n", .{});

    // 测试C类型大小
    std.debug.print("Testing C type sizes...\n", .{});

    const ptr_size = @sizeOf(?*c.CAgentStateDB);
    const int_size = @sizeOf(c_int);
    const uint64_size = @sizeOf(u64);
    const size_t_size = @sizeOf(usize);

    std.debug.print("✅ Type sizes test passed!\n", .{});
    std.debug.print("  Pointer size: {} bytes\n", .{ptr_size});
    std.debug.print("  c_int size: {} bytes\n", .{int_size});
    std.debug.print("  uint64_t size: {} bytes\n", .{uint64_size});
    std.debug.print("  size_t size: {} bytes\n", .{size_t_size});

    // 基本的合理性检查
    try testing.expect(ptr_size == 8); // 64位系统
    try testing.expect(int_size == 4);
    try testing.expect(uint64_size == 8);
    try testing.expect(size_t_size == 8);
}
