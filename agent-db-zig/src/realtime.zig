// 实时数据流处理系统 - Zig API层
const std = @import("std");
const c = @cImport({
    @cInclude("agent_state_db.h");
});

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

    pub fn init(agent_id: u64, data_type: StreamDataType, payload: []const u8) StreamDataItem {
        return StreamDataItem{
            .agent_id = agent_id,
            .data_type = data_type,
            .payload = payload,
            .priority = 128, // 默认中等优先级
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

    pub fn display(self: StreamDataItem) void {
        std.debug.print("Stream Data Item:\n", .{});
        std.debug.print("  Agent ID: {d}\n", .{self.agent_id});
        std.debug.print("  Data Type: {s}\n", .{self.data_type.toString()});
        std.debug.print("  Payload Size: {d} bytes\n", .{self.payload.len});
        std.debug.print("  Priority: {d}\n", .{self.priority});
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
};

// 实时流处理器
pub const RealTimeStreamProcessor = struct {
    processor: *c.CRealTimeStreamProcessor,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) !RealTimeStreamProcessor {
        const processor = c.stream_processor_new();
        if (processor == null) {
            return error.InitializationFailed;
        }

        return RealTimeStreamProcessor{
            .processor = processor.?,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *RealTimeStreamProcessor) void {
        c.stream_processor_free(self.processor);
    }

    pub fn start(self: *RealTimeStreamProcessor) !void {
        const result = c.stream_processor_start(self.processor);
        if (result != 0) {
            return error.StartFailed;
        }
    }

    pub fn stop(self: *RealTimeStreamProcessor) !void {
        const result = c.stream_processor_stop(self.processor);
        if (result != 0) {
            return error.StopFailed;
        }
    }

    pub fn submitData(self: *RealTimeStreamProcessor, item: StreamDataItem) !void {
        const result = c.stream_processor_submit_data(
            self.processor,
            item.agent_id,
            @intFromEnum(item.data_type),
            item.payload.ptr,
            item.payload.len,
            item.priority,
        );

        if (result != 0) {
            return error.SubmitDataFailed;
        }
    }

    pub fn getStats(self: *RealTimeStreamProcessor) !StreamProcessingStats {
        var stats: c.StreamProcessingStats = undefined;
        const result = c.stream_processor_get_stats(self.processor, &stats);
        if (result != 0) {
            return error.GetStatsFailed;
        }

        return StreamProcessingStats{
            .items_received = stats.items_received,
            .items_processed = stats.items_processed,
            .items_dropped = stats.items_dropped,
            .batches_processed = stats.batches_processed,
            .avg_latency_ms = stats.avg_latency_ms,
            .max_latency_ms = stats.max_latency_ms,
            .throughput_per_sec = stats.throughput_per_sec,
            .buffer_utilization = stats.buffer_utilization,
            .error_count = stats.error_count,
            .last_update = stats.last_update,
        };
    }

    pub fn getBufferSize(self: *RealTimeStreamProcessor) !u32 {
        const result = c.stream_processor_get_buffer_size(self.processor);
        if (result < 0) {
            return error.GetBufferSizeFailed;
        }
        return @intCast(result);
    }
};

// 流查询处理器
pub const StreamQueryProcessor = struct {
    processor: *c.CStreamQueryProcessor,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) !StreamQueryProcessor {
        const processor = c.stream_query_processor_new();
        if (processor == null) {
            return error.InitializationFailed;
        }

        return StreamQueryProcessor{
            .processor = processor.?,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *StreamQueryProcessor) void {
        c.stream_query_processor_free(self.processor);
    }

    pub fn registerQuery(
        self: *StreamQueryProcessor,
        query_id: []const u8,
        query_type: StreamQueryType,
        callback: []const u8,
    ) !void {
        const c_query_id = try self.allocator.dupeZ(u8, query_id);
        defer self.allocator.free(c_query_id);

        const c_callback = try self.allocator.dupeZ(u8, callback);
        defer self.allocator.free(c_callback);

        const result = c.stream_query_register(
            self.processor,
            c_query_id.ptr,
            @intFromEnum(query_type),
            c_callback.ptr,
        );

        if (result != 0) {
            return error.RegisterQueryFailed;
        }
    }

    pub fn unregisterQuery(self: *StreamQueryProcessor, query_id: []const u8) !void {
        const c_query_id = try self.allocator.dupeZ(u8, query_id);
        defer self.allocator.free(c_query_id);

        const result = c.stream_query_unregister(self.processor, c_query_id.ptr);
        if (result != 0) {
            return error.UnregisterQueryFailed;
        }
    }

    pub fn getActiveQueryCount(self: *StreamQueryProcessor) !u32 {
        const result = c.stream_query_get_active_count(self.processor);
        if (result < 0) {
            return error.GetActiveCountFailed;
        }
        return @intCast(result);
    }
};

// 流数据生成器（用于测试）
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
