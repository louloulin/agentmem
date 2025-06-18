// Agent状态数据库性能基准测试
const std = @import("std");
const AgentDB = @import("../src/agent_db.zig").AgentDB;
const StateType = @import("../src/agent_state.zig").StateType;

const BenchmarkResult = struct {
    operation: []const u8,
    total_operations: u32,
    duration_ms: f64,
    ops_per_second: f64,
    avg_latency_us: f64,
};

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("=== Agent状态数据库性能基准测试 ===\n\n");

    // 初始化数据库
    const db_path = "benchmark_agent.db";
    var db = AgentDB.init(db_path, allocator) catch |err| {
        std.debug.print("初始化数据库失败: {}\n", .{err});
        return;
    };
    defer {
        db.deinit();
        std.fs.cwd().deleteFile(db_path) catch {};
    }

    var results = std.ArrayList(BenchmarkResult).init(allocator);
    defer results.deinit();

    // 1. 状态保存性能测试
    try results.append(try benchmarkSaveStates(&db, allocator));

    // 2. 状态加载性能测试
    try results.append(try benchmarkLoadStates(&db, allocator));

    // 3. 状态更新性能测试
    try results.append(try benchmarkUpdateStates(&db, allocator));

    // 4. 元数据操作性能测试
    try results.append(try benchmarkMetadataOperations(&db, allocator));

    // 5. 快照操作性能测试
    try results.append(try benchmarkSnapshotOperations(&db, allocator));

    // 6. 批量操作性能测试
    try results.append(try benchmarkBatchOperations(&db, allocator));

    // 7. 并发操作性能测试（模拟）
    try results.append(try benchmarkConcurrentOperations(&db, allocator));

    // 8. 内存使用测试
    try benchmarkMemoryUsage(&db, allocator);

    // 输出结果
    printBenchmarkResults(results.items);
}

// 状态保存性能测试
fn benchmarkSaveStates(db: *AgentDB, allocator: std.mem.Allocator) !BenchmarkResult {
    const num_operations: u32 = 1000;
    const test_data = "这是一个测试状态数据，用于性能基准测试。包含一些中文字符和数字123456789。";
    
    const start_time = std.time.nanoTimestamp();
    
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);
        const session_id = @as(u64, i * 2);
        
        try db.saveAgentState(agent_id, session_id, StateType.context, test_data);
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const ops_per_second = @as(f64, @floatFromInt(num_operations)) / (duration_ms / 1000.0);
    const avg_latency_us = duration_ms * 1000.0 / @as(f64, @floatFromInt(num_operations));
    
    _ = allocator;
    
    return BenchmarkResult{
        .operation = "状态保存",
        .total_operations = num_operations,
        .duration_ms = duration_ms,
        .ops_per_second = ops_per_second,
        .avg_latency_us = avg_latency_us,
    };
}

// 状态加载性能测试
fn benchmarkLoadStates(db: *AgentDB, allocator: std.mem.Allocator) !BenchmarkResult {
    const num_operations: u32 = 1000;
    
    const start_time = std.time.nanoTimestamp();
    
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);
        
        const state = try db.loadAgentState(agent_id);
        if (state) |s| {
            var mutable_state = s;
            mutable_state.deinit(allocator);
        }
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const ops_per_second = @as(f64, @floatFromInt(num_operations)) / (duration_ms / 1000.0);
    const avg_latency_us = duration_ms * 1000.0 / @as(f64, @floatFromInt(num_operations));
    
    return BenchmarkResult{
        .operation = "状态加载",
        .total_operations = num_operations,
        .duration_ms = duration_ms,
        .ops_per_second = ops_per_second,
        .avg_latency_us = avg_latency_us,
    };
}

// 状态更新性能测试
fn benchmarkUpdateStates(db: *AgentDB, allocator: std.mem.Allocator) !BenchmarkResult {
    const num_operations: u32 = 500;
    const updated_data = "这是更新后的状态数据，包含更多信息用于测试更新操作的性能表现。";
    
    const start_time = std.time.nanoTimestamp();
    
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);
        
        try db.updateAgentState(agent_id, updated_data);
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const ops_per_second = @as(f64, @floatFromInt(num_operations)) / (duration_ms / 1000.0);
    const avg_latency_us = duration_ms * 1000.0 / @as(f64, @floatFromInt(num_operations));
    
    _ = allocator;
    
    return BenchmarkResult{
        .operation = "状态更新",
        .total_operations = num_operations,
        .duration_ms = duration_ms,
        .ops_per_second = ops_per_second,
        .avg_latency_us = avg_latency_us,
    };
}

// 元数据操作性能测试
fn benchmarkMetadataOperations(db: *AgentDB, allocator: std.mem.Allocator) !BenchmarkResult {
    const num_operations: u32 = 1000;
    
    const start_time = std.time.nanoTimestamp();
    
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i % 100); // 重复使用前100个agent
        const key = try std.fmt.allocPrint(allocator, "key_{}", .{i});
        defer allocator.free(key);
        const value = try std.fmt.allocPrint(allocator, "value_{}", .{i});
        defer allocator.free(value);
        
        // 设置元数据
        try db.setStateMetadata(agent_id, key, value);
        
        // 获取元数据
        _ = try db.getStateMetadata(agent_id, key);
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const ops_per_second = @as(f64, @floatFromInt(num_operations * 2)) / (duration_ms / 1000.0); // 每次循环2个操作
    const avg_latency_us = duration_ms * 1000.0 / @as(f64, @floatFromInt(num_operations * 2));
    
    return BenchmarkResult{
        .operation = "元数据操作",
        .total_operations = num_operations * 2,
        .duration_ms = duration_ms,
        .ops_per_second = ops_per_second,
        .avg_latency_us = avg_latency_us,
    };
}

// 快照操作性能测试
fn benchmarkSnapshotOperations(db: *AgentDB, allocator: std.mem.Allocator) !BenchmarkResult {
    const num_operations: u32 = 100;
    
    const start_time = std.time.nanoTimestamp();
    
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i);
        const snapshot_name = try std.fmt.allocPrint(allocator, "snapshot_{}", .{i});
        defer allocator.free(snapshot_name);
        
        // 创建快照
        try db.createStateSnapshot(agent_id, snapshot_name);
        
        // 恢复快照
        try db.restoreFromSnapshot(agent_id, snapshot_name);
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const ops_per_second = @as(f64, @floatFromInt(num_operations * 2)) / (duration_ms / 1000.0);
    const avg_latency_us = duration_ms * 1000.0 / @as(f64, @floatFromInt(num_operations * 2));
    
    return BenchmarkResult{
        .operation = "快照操作",
        .total_operations = num_operations * 2,
        .duration_ms = duration_ms,
        .ops_per_second = ops_per_second,
        .avg_latency_us = avg_latency_us,
    };
}

// 批量操作性能测试
fn benchmarkBatchOperations(db: *AgentDB, allocator: std.mem.Allocator) !BenchmarkResult {
    const num_batches: u32 = 10;
    const batch_size: u32 = 100;
    const total_operations = num_batches * batch_size;
    
    const start_time = std.time.nanoTimestamp();
    
    for (0..num_batches) |batch| {
        // 模拟批量操作
        for (0..batch_size) |i| {
            const agent_id = @as(u64, batch * batch_size + i + 10000); // 避免与之前的ID冲突
            const session_id = @as(u64, batch);
            const data = try std.fmt.allocPrint(allocator, "批量数据 batch:{} item:{}", .{ batch, i });
            defer allocator.free(data);
            
            try db.saveAgentState(agent_id, session_id, StateType.working_memory, data);
        }
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const ops_per_second = @as(f64, @floatFromInt(total_operations)) / (duration_ms / 1000.0);
    const avg_latency_us = duration_ms * 1000.0 / @as(f64, @floatFromInt(total_operations));
    
    return BenchmarkResult{
        .operation = "批量操作",
        .total_operations = total_operations,
        .duration_ms = duration_ms,
        .ops_per_second = ops_per_second,
        .avg_latency_us = avg_latency_us,
    };
}

// 并发操作性能测试（模拟）
fn benchmarkConcurrentOperations(db: *AgentDB, allocator: std.mem.Allocator) !BenchmarkResult {
    const num_operations: u32 = 500;
    const test_data = "并发测试数据";
    
    const start_time = std.time.nanoTimestamp();
    
    // 模拟并发操作（实际上是顺序执行，但模拟并发场景）
    for (0..num_operations) |i| {
        const agent_id = @as(u64, i + 20000); // 避免ID冲突
        const session_id = @as(u64, i);
        
        // 混合操作：保存、加载、更新
        try db.saveAgentState(agent_id, session_id, StateType.context, test_data);
        
        const state = try db.loadAgentState(agent_id);
        if (state) |s| {
            var mutable_state = s;
            mutable_state.deinit(allocator);
        }
        
        if (i % 2 == 0) {
            try db.updateAgentState(agent_id, "更新的并发数据");
        }
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ns = end_time - start_time;
    const duration_ms = @as(f64, @floatFromInt(duration_ns)) / 1_000_000.0;
    const total_ops = num_operations * 3; // 每次循环3个操作
    const ops_per_second = @as(f64, @floatFromInt(total_ops)) / (duration_ms / 1000.0);
    const avg_latency_us = duration_ms * 1000.0 / @as(f64, @floatFromInt(total_ops));
    
    return BenchmarkResult{
        .operation = "并发操作",
        .total_operations = total_ops,
        .duration_ms = duration_ms,
        .ops_per_second = ops_per_second,
        .avg_latency_us = avg_latency_us,
    };
}

// 内存使用测试
fn benchmarkMemoryUsage(db: *AgentDB, allocator: std.mem.Allocator) !void {
    std.debug.print("=== 内存使用测试 ===\n");
    
    const num_states: u32 = 1000;
    const large_data_size = 1024; // 1KB per state
    
    // 创建大数据
    var large_data = try allocator.alloc(u8, large_data_size);
    defer allocator.free(large_data);
    
    for (large_data, 0..) |*byte, i| {
        byte.* = @as(u8, @truncate(i % 256));
    }
    
    const start_time = std.time.nanoTimestamp();
    
    // 保存大量状态
    for (0..num_states) |i| {
        const agent_id = @as(u64, i + 30000);
        const session_id = @as(u64, i);
        
        try db.saveAgentState(agent_id, session_id, StateType.long_term_memory, large_data);
    }
    
    const end_time = std.time.nanoTimestamp();
    const duration_ms = @as(f64, @floatFromInt(end_time - start_time)) / 1_000_000.0;
    
    const total_data_size = num_states * large_data_size;
    const throughput_mb_s = (@as(f64, @floatFromInt(total_data_size)) / (1024.0 * 1024.0)) / (duration_ms / 1000.0);
    
    std.debug.print("保存 {} 个状态，每个 {} 字节\n", .{ num_states, large_data_size });
    std.debug.print("总数据量: {d:.2} MB\n", .{@as(f64, @floatFromInt(total_data_size)) / (1024.0 * 1024.0)});
    std.debug.print("耗时: {d:.2} ms\n", .{duration_ms});
    std.debug.print("吞吐量: {d:.2} MB/s\n", .{throughput_mb_s});
    std.debug.print("\n");
}

// 输出基准测试结果
fn printBenchmarkResults(results: []const BenchmarkResult) void {
    std.debug.print("=== 性能基准测试结果 ===\n\n");
    std.debug.print("{s:<15} {s:>10} {s:>12} {s:>15} {s:>15}\n", .{ "操作类型", "操作数", "耗时(ms)", "QPS", "平均延迟(μs)" });
    std.debug.print("{s}\n", .{"-" ** 70});
    
    for (results) |result| {
        std.debug.print("{s:<15} {d:>10} {d:>12.2} {d:>15.0} {d:>15.2}\n", .{
            result.operation,
            result.total_operations,
            result.duration_ms,
            result.ops_per_second,
            result.avg_latency_us,
        });
    }
    
    std.debug.print("\n");
    
    // 计算总体统计
    var total_operations: u32 = 0;
    var total_duration: f64 = 0;
    
    for (results) |result| {
        total_operations += result.total_operations;
        total_duration += result.duration_ms;
    }
    
    const overall_qps = @as(f64, @floatFromInt(total_operations)) / (total_duration / 1000.0);
    
    std.debug.print("=== 总体统计 ===\n");
    std.debug.print("总操作数: {}\n", .{total_operations});
    std.debug.print("总耗时: {d:.2} ms\n", .{total_duration});
    std.debug.print("平均QPS: {d:.0}\n", .{overall_qps});
    std.debug.print("\n");
    
    // 性能评估
    std.debug.print("=== 性能评估 ===\n");
    if (overall_qps > 10000) {
        std.debug.print("✅ 性能优秀 (QPS > 10,000)\n");
    } else if (overall_qps > 5000) {
        std.debug.print("✅ 性能良好 (QPS > 5,000)\n");
    } else if (overall_qps > 1000) {
        std.debug.print("⚠️  性能一般 (QPS > 1,000)\n");
    } else {
        std.debug.print("❌ 性能需要优化 (QPS < 1,000)\n");
    }
    
    // 延迟评估
    var total_latency: f64 = 0;
    for (results) |result| {
        total_latency += result.avg_latency_us;
    }
    const avg_latency = total_latency / @as(f64, @floatFromInt(results.len));
    
    if (avg_latency < 100) {
        std.debug.print("✅ 延迟优秀 (< 100μs)\n");
    } else if (avg_latency < 1000) {
        std.debug.print("✅ 延迟良好 (< 1ms)\n");
    } else if (avg_latency < 10000) {
        std.debug.print("⚠️  延迟一般 (< 10ms)\n");
    } else {
        std.debug.print("❌ 延迟需要优化 (> 10ms)\n");
    }
    
    std.debug.print("\n");
}
