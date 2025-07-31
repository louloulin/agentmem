// 简单的基准测试
const std = @import("std");
const c = @cImport({
    @cInclude("agent_state_db.h");
});

pub fn main() !void {
    std.debug.print("=== Agent状态数据库简单基准测试 ===\n\n", .{});

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // 初始化数据库
    const db_path = "simple_benchmark.lance";
    const c_path = try allocator.dupeZ(u8, db_path);
    defer allocator.free(c_path);

    const db_handle = c.agent_db_new(c_path.ptr);
    if (db_handle == null) {
        std.debug.print("❌ 数据库初始化失败\n", .{});
        return;
    }
    defer c.agent_db_free(db_handle);

    std.debug.print("✅ 数据库初始化成功\n", .{});

    // 基准测试参数
    const num_operations: u32 = 1000;
    const test_data = "这是一个测试状态数据，用于性能基准测试。包含一些中文字符和数字123456789。";

    // 1. 状态保存性能测试
    std.debug.print("\n1. 状态保存性能测试...\n", .{});
    const start_time = std.time.nanoTimestamp();

    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);
        const session_id = @as(u64, i * 2);

        const result = c.agent_db_save_state(
            db_handle,
            agent_id,
            session_id,
            2, // StateType.context
            test_data.ptr,
            test_data.len,
        );

        if (result != 0) {
            std.debug.print("❌ 保存状态失败: agent_id={}\n", .{agent_id});
            return;
        }
    }

    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const ops_per_second = @as(f64, @floatFromInt(num_operations)) / (duration_ms / 1000.0);

    std.debug.print("✅ 保存 {} 个状态\n", .{num_operations});
    std.debug.print("   耗时: {d:.2} ms\n", .{duration_ms});
    std.debug.print("   QPS: {d:.0}\n", .{ops_per_second});

    // 2. 状态加载性能测试
    std.debug.print("\n2. 状态加载性能测试...\n", .{});
    const load_start_time = std.time.nanoTimestamp();

    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);

        var data_ptr: [*c]u8 = undefined;
        var data_len: usize = undefined;

        const result = c.agent_db_load_state(db_handle, agent_id, &data_ptr, &data_len);

        if (result == 0) {
            // 成功加载，释放数据
            c.agent_db_free_data(data_ptr, data_len);
        } else if (result == 1) {
            // 未找到，这是正常的
        } else {
            std.debug.print("❌ 加载状态失败: agent_id={}\n", .{agent_id});
            return;
        }
    }

    const load_end_time = std.time.nanoTimestamp();
    const load_duration_ns = load_end_time - load_start_time;
    const load_duration_ms = @as(f64, @floatFromInt(load_duration_ns)) / 1_000_000.0;
    const load_ops_per_second = @as(f64, @floatFromInt(num_operations)) / (load_duration_ms / 1000.0);

    std.debug.print("✅ 加载 {} 个状态\n", .{num_operations});
    std.debug.print("   耗时: {d:.2} ms\n", .{load_duration_ms});
    std.debug.print("   QPS: {d:.0}\n", .{load_ops_per_second});

    // 3. 记忆系统测试
    std.debug.print("\n3. 记忆系统性能测试...\n", .{});
    
    const memory_handle = c.memory_manager_new(c_path.ptr);
    if (memory_handle == null) {
        std.debug.print("❌ 记忆管理器初始化失败\n", .{});
        return;
    }
    defer c.memory_manager_free(memory_handle);

    const memory_start_time = std.time.nanoTimestamp();
    const memory_operations: u32 = 500;

    for (0..memory_operations) |i| {
        const agent_id = @as(u64, i + 10000);
        const memory_content = try std.fmt.allocPrintZ(allocator, "记忆内容 {}", .{i});
        defer allocator.free(memory_content);

        const result = c.memory_manager_store_memory(
            memory_handle,
            agent_id,
            0, // MemoryType.episodic
            memory_content.ptr,
            0.8,
        );

        if (result != 0) {
            std.debug.print("❌ 存储记忆失败: agent_id={}\n", .{agent_id});
            return;
        }
    }

    const memory_end_time = std.time.nanoTimestamp();
    const memory_duration_ns = memory_end_time - memory_start_time;
    const memory_duration_ms = @as(f64, @floatFromInt(memory_duration_ns)) / 1_000_000.0;
    const memory_ops_per_second = @as(f64, @floatFromInt(memory_operations)) / (memory_duration_ms / 1000.0);

    std.debug.print("✅ 存储 {} 个记忆\n", .{memory_operations});
    std.debug.print("   耗时: {d:.2} ms\n", .{memory_duration_ms});
    std.debug.print("   QPS: {d:.0}\n", .{memory_ops_per_second});

    // 4. RAG系统测试
    std.debug.print("\n4. RAG系统性能测试...\n", .{});
    
    const rag_handle = c.rag_engine_new(c_path.ptr);
    if (rag_handle == null) {
        std.debug.print("❌ RAG引擎初始化失败\n", .{});
        return;
    }
    defer c.rag_engine_free(rag_handle);

    const rag_start_time = std.time.nanoTimestamp();
    const rag_operations: u32 = 100;

    for (0..rag_operations) |i| {
        const title = try std.fmt.allocPrintZ(allocator, "文档 {}", .{i});
        defer allocator.free(title);
        
        const content = try std.fmt.allocPrintZ(allocator, "这是文档 {} 的内容，用于测试RAG系统的索引和搜索性能。", .{i});
        defer allocator.free(content);

        const result = c.rag_engine_index_document(
            rag_handle,
            title.ptr,
            content.ptr,
            200,
            50,
        );

        if (result != 0) {
            std.debug.print("❌ 索引文档失败: {}\n", .{i});
            return;
        }
    }

    const rag_end_time = std.time.nanoTimestamp();
    const rag_duration_ns = rag_end_time - rag_start_time;
    const rag_duration_ms = @as(f64, @floatFromInt(rag_duration_ns)) / 1_000_000.0;
    const rag_ops_per_second = @as(f64, @floatFromInt(rag_operations)) / (rag_duration_ms / 1000.0);

    std.debug.print("✅ 索引 {} 个文档\n", .{rag_operations});
    std.debug.print("   耗时: {d:.2} ms\n", .{rag_duration_ms});
    std.debug.print("   QPS: {d:.0}\n", .{rag_ops_per_second});

    // 总结
    std.debug.print("\n=== 基准测试总结 ===\n", .{});
    const total_operations = num_operations * 2 + memory_operations + rag_operations;
    const total_duration = duration_ms + load_duration_ms + memory_duration_ms + rag_duration_ms;
    const overall_qps = @as(f64, @floatFromInt(total_operations)) / (total_duration / 1000.0);

    std.debug.print("总操作数: {}\n", .{total_operations});
    std.debug.print("总耗时: {d:.2} ms\n", .{total_duration});
    std.debug.print("平均QPS: {d:.0}\n", .{overall_qps});

    if (overall_qps > 5000) {
        std.debug.print("✅ 性能优秀 (QPS > 5,000)\n", .{});
    } else if (overall_qps > 1000) {
        std.debug.print("✅ 性能良好 (QPS > 1,000)\n", .{});
    } else {
        std.debug.print("⚠️  性能一般 (QPS < 1,000)\n", .{});
    }

    std.debug.print("\n🎉 基准测试完成！\n", .{});

    // 清理测试文件
    std.fs.cwd().deleteFile(db_path) catch {};
}
