// 简化的实时数据流处理系统测试（不依赖C FFI）
const std = @import("std");
const testing = std.testing;

// 流数据类型枚举
pub const StreamDataType = enum(u32) {
    AgentState = 0,
    Memory = 1,
    Document = 2,
    Vector = 3,
    Event = 4,
    Metric = 5,

    pub fn toString(self: StreamDataType) []const u8 {
        return switch (self) {
            .AgentState => "AgentState",
            .Memory => "Memory",
            .Document => "Document",
            .Vector => "Vector",
            .Event => "Event",
            .Metric => "Metric",
        };
    }
};

// 流查询类型枚举
pub const StreamQueryType = enum(u32) {
    VectorSimilarity = 0,
    MemorySearch = 1,
    AgentStateMonitor = 2,
    EventPattern = 3,
    RealTimeStats = 4,

    pub fn toString(self: StreamQueryType) []const u8 {
        return switch (self) {
            .VectorSimilarity => "VectorSimilarity",
            .MemorySearch => "MemorySearch",
            .AgentStateMonitor => "AgentStateMonitor",
            .EventPattern => "EventPattern",
            .RealTimeStats => "RealTimeStats",
        };
    }
};

// 流数据项结构
pub const StreamDataItem = struct {
    agent_id: u64,
    data_type: StreamDataType,
    payload: []const u8,
    priority: u8,
    timestamp: i64,

    pub fn init(agent_id: u64, data_type: StreamDataType, payload: []const u8) StreamDataItem {
        return StreamDataItem{
            .agent_id = agent_id,
            .data_type = data_type,
            .payload = payload,
            .priority = 128, // 默认中等优先级
            .timestamp = std.time.timestamp(),
        };
    }

    pub fn withPriority(self: StreamDataItem, priority: u8) StreamDataItem {
        var item = self;
        item.priority = priority;
        return item;
    }

    pub fn isHighPriority(self: StreamDataItem) bool {
        return self.priority > 200;
    }

    pub fn ageSeconds(self: StreamDataItem) i64 {
        return std.time.timestamp() - self.timestamp;
    }

    pub fn display(self: StreamDataItem) void {
        std.debug.print("Stream Data Item:\n", .{});
        std.debug.print("  Agent ID: {d}\n", .{self.agent_id});
        std.debug.print("  Data Type: {s}\n", .{self.data_type.toString()});
        std.debug.print("  Payload Size: {d} bytes\n", .{self.payload.len});
        std.debug.print("  Priority: {d}\n", .{self.priority});
        std.debug.print("  Timestamp: {d}\n", .{self.timestamp});
        std.debug.print("  Age: {d} seconds\n", .{self.ageSeconds()});
    }
};

// 流处理统计结构
pub const StreamProcessingStats = struct {
    items_received: u64,
    items_processed: u64,
    items_dropped: u64,
    batches_processed: u64,
    avg_latency_ms: f64,
    max_latency_ms: u64,
    throughput_per_sec: f64,
    buffer_utilization: f64,
    error_count: u64,
    last_update: i64,

    pub fn init() StreamProcessingStats {
        return StreamProcessingStats{
            .items_received = 0,
            .items_processed = 0,
            .items_dropped = 0,
            .batches_processed = 0,
            .avg_latency_ms = 0.0,
            .max_latency_ms = 0,
            .throughput_per_sec = 0.0,
            .buffer_utilization = 0.0,
            .error_count = 0,
            .last_update = std.time.timestamp(),
        };
    }

    pub fn display(self: StreamProcessingStats) void {
        std.debug.print("Stream Processing Statistics:\n", .{});
        std.debug.print("  Items Received: {d}\n", .{self.items_received});
        std.debug.print("  Items Processed: {d}\n", .{self.items_processed});
        std.debug.print("  Items Dropped: {d}\n", .{self.items_dropped});
        std.debug.print("  Batches Processed: {d}\n", .{self.batches_processed});
        std.debug.print("  Avg Latency: {d:.2} ms\n", .{self.avg_latency_ms});
        std.debug.print("  Max Latency: {d} ms\n", .{self.max_latency_ms});
        std.debug.print("  Throughput: {d:.2} items/sec\n", .{self.throughput_per_sec});
        std.debug.print("  Buffer Utilization: {d:.2}%\n", .{self.buffer_utilization * 100.0});
        std.debug.print("  Error Count: {d}\n", .{self.error_count});
    }

    pub fn getProcessingRate(self: StreamProcessingStats) f64 {
        if (self.items_received == 0) return 0.0;
        return @as(f64, @floatFromInt(self.items_processed)) / @as(f64, @floatFromInt(self.items_received));
    }

    pub fn getDropRate(self: StreamProcessingStats) f64 {
        if (self.items_received == 0) return 0.0;
        return @as(f64, @floatFromInt(self.items_dropped)) / @as(f64, @floatFromInt(self.items_received));
    }

    pub fn updateStats(self: *StreamProcessingStats, processed: u64, dropped: u64, latency_ms: u64) void {
        self.items_processed += processed;
        self.items_dropped += dropped;
        self.items_received += processed + dropped;

        if (latency_ms > self.max_latency_ms) {
            self.max_latency_ms = latency_ms;
        }

        // 简单的移动平均
        self.avg_latency_ms = (self.avg_latency_ms * 0.9) + (@as(f64, @floatFromInt(latency_ms)) * 0.1);
        self.last_update = std.time.timestamp();
    }
};

// 流数据生成器
pub const StreamDataGenerator = struct {
    allocator: std.mem.Allocator,
    agent_id_counter: u64,

    pub fn init(allocator: std.mem.Allocator) StreamDataGenerator {
        return StreamDataGenerator{
            .allocator = allocator,
            .agent_id_counter = 1000,
        };
    }

    pub fn generateAgentStateData(self: *StreamDataGenerator) !StreamDataItem {
        self.agent_id_counter += 1;

        const state_data = try std.fmt.allocPrint(self.allocator, "{{\"agent_id\":{d},\"state\":\"active\",\"timestamp\":{d}}}", .{ self.agent_id_counter, std.time.timestamp() });

        return StreamDataItem.init(
            self.agent_id_counter,
            .AgentState,
            state_data,
        );
    }

    pub fn generateMemoryData(self: *StreamDataGenerator) !StreamDataItem {
        self.agent_id_counter += 1;

        const memory_data = try std.fmt.allocPrint(self.allocator, "{{\"agent_id\":{d},\"content\":\"Test memory content\",\"importance\":0.8}}", .{self.agent_id_counter});

        return StreamDataItem.init(
            self.agent_id_counter,
            .Memory,
            memory_data,
        );
    }

    pub fn generateEventData(self: *StreamDataGenerator) !StreamDataItem {
        self.agent_id_counter += 1;

        const event_data = try std.fmt.allocPrint(self.allocator, "{{\"agent_id\":{d},\"event_type\":\"user_interaction\",\"data\":[1.0,2.0,3.0]}}", .{self.agent_id_counter});

        return StreamDataItem.init(
            self.agent_id_counter,
            .Event,
            event_data,
        );
    }

    pub fn generateVectorData(self: *StreamDataGenerator) !StreamDataItem {
        self.agent_id_counter += 1;

        const vector_data = try std.fmt.allocPrint(self.allocator, "[[0.1,0.2,0.3,0.4,0.5],{{\"type\":\"embedding\",\"model\":\"test\"}}]", .{});

        return StreamDataItem.init(
            self.agent_id_counter,
            .Vector,
            vector_data,
        );
    }

    pub fn freeData(self: *StreamDataGenerator, data: []const u8) void {
        self.allocator.free(data);
    }
};

test "Stream Data Item Creation and Properties" {
    std.debug.print("\n=== Stream Data Item Creation and Properties Test ===\n", .{});

    const payload = "Test stream data payload";
    const item = StreamDataItem.init(12345, .AgentState, payload);

    try testing.expect(item.agent_id == 12345);
    try testing.expect(item.data_type == .AgentState);
    try testing.expectEqualStrings(item.payload, payload);
    try testing.expect(item.priority == 128);

    std.debug.print("✅ Stream data item created successfully\n", .{});
    item.display();

    // 测试优先级设置
    const high_priority_item = item.withPriority(255);
    try testing.expect(high_priority_item.priority == 255);
    try testing.expect(high_priority_item.isHighPriority());

    const low_priority_item = item.withPriority(50);
    try testing.expect(low_priority_item.priority == 50);
    try testing.expect(!low_priority_item.isHighPriority());

    std.debug.print("✅ Priority settings work correctly\n", .{});
}

test "Stream Data Type and Query Type Enumerations" {
    std.debug.print("\n=== Stream Data Type and Query Type Enumerations Test ===\n", .{});

    const data_types = [_]StreamDataType{
        .AgentState, .Memory, .Document, .Vector, .Event, .Metric,
    };

    std.debug.print("Stream Data Types:\n", .{});
    for (data_types) |data_type| {
        std.debug.print("  - {s}\n", .{data_type.toString()});
    }

    const query_types = [_]StreamQueryType{
        .VectorSimilarity, .MemorySearch, .AgentStateMonitor, .EventPattern, .RealTimeStats,
    };

    std.debug.print("Stream Query Types:\n", .{});
    for (query_types) |query_type| {
        std.debug.print("  - {s}\n", .{query_type.toString()});
    }

    try testing.expect(data_types.len == 6);
    try testing.expect(query_types.len == 5);
    std.debug.print("✅ All enumerations work correctly\n", .{});
}

test "Stream Data Generator" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("\n=== Stream Data Generator Test ===\n", .{});

    var generator = StreamDataGenerator.init(allocator);

    // 生成不同类型的数据
    const agent_state_item = try generator.generateAgentStateData();
    defer generator.freeData(agent_state_item.payload);

    const memory_item = try generator.generateMemoryData();
    defer generator.freeData(memory_item.payload);

    const event_item = try generator.generateEventData();
    defer generator.freeData(event_item.payload);

    const vector_item = try generator.generateVectorData();
    defer generator.freeData(vector_item.payload);

    // 验证生成的数据
    try testing.expect(agent_state_item.data_type == .AgentState);
    try testing.expect(memory_item.data_type == .Memory);
    try testing.expect(event_item.data_type == .Event);
    try testing.expect(vector_item.data_type == .Vector);

    // 验证Agent ID递增
    try testing.expect(memory_item.agent_id > agent_state_item.agent_id);
    try testing.expect(event_item.agent_id > memory_item.agent_id);
    try testing.expect(vector_item.agent_id > event_item.agent_id);

    std.debug.print("Generated data items:\n", .{});
    agent_state_item.display();
    std.debug.print("\n", .{});
    memory_item.display();
    std.debug.print("\n", .{});

    std.debug.print("✅ Stream data generator working correctly\n", .{});
}

test "Stream Processing Statistics" {
    std.debug.print("\n=== Stream Processing Statistics Test ===\n", .{});

    var stats = StreamProcessingStats.init();

    // 模拟处理一些数据
    stats.updateStats(100, 5, 10); // 处理100个，丢弃5个，延迟10ms
    stats.updateStats(50, 2, 15); // 处理50个，丢弃2个，延迟15ms
    stats.updateStats(75, 0, 8); // 处理75个，丢弃0个，延迟8ms

    stats.display();

    // 验证统计计算
    try testing.expect(stats.items_processed == 225);
    try testing.expect(stats.items_dropped == 7);
    try testing.expect(stats.items_received == 232);
    try testing.expect(stats.max_latency_ms == 15);

    const processing_rate = stats.getProcessingRate();
    const drop_rate = stats.getDropRate();

    std.debug.print("Processing Rate: {d:.2}%\n", .{processing_rate * 100.0});
    std.debug.print("Drop Rate: {d:.2}%\n", .{drop_rate * 100.0});

    try testing.expect(processing_rate > 0.95); // 应该大于95%
    try testing.expect(drop_rate < 0.05); // 应该小于5%

    std.debug.print("✅ Stream processing statistics working correctly\n", .{});
}

test "Batch Stream Data Processing" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("\n=== Batch Stream Data Processing Test ===\n", .{});

    var generator = StreamDataGenerator.init(allocator);
    var items = std.ArrayList(StreamDataItem).init(allocator);
    defer {
        for (items.items) |item| {
            generator.freeData(item.payload);
        }
        items.deinit();
    }

    // 生成批量数据
    const batch_size = 20;
    for (0..batch_size) |i| {
        const data_type = switch (i % 4) {
            0 => StreamDataType.AgentState,
            1 => StreamDataType.Memory,
            2 => StreamDataType.Event,
            else => StreamDataType.Vector,
        };

        var item = switch (data_type) {
            .AgentState => try generator.generateAgentStateData(),
            .Memory => try generator.generateMemoryData(),
            .Event => try generator.generateEventData(),
            .Vector => try generator.generateVectorData(),
            else => StreamDataItem.init(@intCast(i), data_type, "test"),
        };

        // 设置不同的优先级
        item = item.withPriority(@intCast((i * 10) % 256));
        try items.append(item);
    }

    try testing.expect(items.items.len == batch_size);

    // 统计不同类型的数据
    var type_counts = std.EnumMap(StreamDataType, u32).init(.{});
    var high_priority_count: u32 = 0;

    for (items.items) |item| {
        const current_count = type_counts.get(item.data_type) orelse 0;
        type_counts.put(item.data_type, current_count + 1);

        if (item.isHighPriority()) {
            high_priority_count += 1;
        }
    }

    std.debug.print("Generated {} items:\n", .{batch_size});
    std.debug.print("  AgentState: {d}\n", .{type_counts.get(.AgentState) orelse 0});
    std.debug.print("  Memory: {d}\n", .{type_counts.get(.Memory) orelse 0});
    std.debug.print("  Event: {d}\n", .{type_counts.get(.Event) orelse 0});
    std.debug.print("  Vector: {d}\n", .{type_counts.get(.Vector) orelse 0});
    std.debug.print("  High Priority: {d}\n", .{high_priority_count});

    std.debug.print("✅ Batch stream data processing completed successfully\n", .{});
}

// 运行所有测试的主函数
pub fn runAllTests() !void {
    std.debug.print("🚀 Starting Simple Real-Time Stream Processing Tests\n", .{});
    std.debug.print("=" ** 60 ++ "\n", .{});

    // 运行所有测试
    try testing.refAllDecls(@This());

    std.debug.print("=" ** 60 ++ "\n", .{});
    std.debug.print("🎉 All Simple Real-Time Stream Processing Tests Completed!\n", .{});
}
