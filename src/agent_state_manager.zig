// Agent状态管理器
const std = @import("std");
const lance_ffi = @import("lance_ffi.zig");
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;

pub const AgentStateManager = struct {
    db: lance_ffi.AgentStateDB,
    allocator: std.mem.Allocator,
    cache: std.HashMap(u64, AgentState),
    cache_size_limit: usize,

    const Self = @This();

    pub fn init(db_path: []const u8, allocator: std.mem.Allocator) !Self {
        var db = try lance_ffi.AgentStateDB.init(db_path, allocator);
        var cache = std.HashMap(u64, AgentState).init(allocator);

        return Self{
            .db = db,
            .allocator = allocator,
            .cache = cache,
            .cache_size_limit = 1000, // 默认缓存1000个状态
        };
    }

    pub fn deinit(self: *Self) void {
        // 清理缓存
        var iterator = self.cache.iterator();
        while (iterator.next()) |entry| {
            var state = entry.value_ptr;
            state.deinit(self.allocator);
        }
        self.cache.deinit();

        // 关闭数据库连接
        self.db.deinit();
    }

    // 保存Agent状态
    pub fn saveState(self: *Self, state: AgentState) !void {
        // 验证状态
        if (!state.validateChecksum()) {
            return error.InvalidChecksum;
        }

        // 使用新的LanceDB接口保存状态
        try self.db.saveState(state.agent_id, state.session_id, state.state_type, state.data);

        // 更新缓存
        try self.updateCache(state);
    }

    // 加载Agent状态
    pub fn loadState(self: *Self, agent_id: u64) !?AgentState {
        // 首先检查缓存
        if (self.cache.get(agent_id)) |cached_state| {
            return cached_state;
        }

        // 从数据库加载
        const data = try self.db.loadState(agent_id);
        if (data) |state_data| {
            defer self.allocator.free(state_data);

            // 创建一个基本的AgentState（简化版本）
            var state = try AgentState.init(self.allocator, agent_id, 0, StateType.context, state_data);

            // 添加到缓存
            try self.updateCache(state);

            return state;
        }

        return null;
    }

    // 查询状态历史
    pub fn queryHistory(self: *Self, agent_id: u64, from_timestamp: i64, to_timestamp: i64) ![]AgentState {
        const query = try std.fmt.allocPrint(self.allocator, "agent_id = {} AND timestamp >= {} AND timestamp <= {}", .{ agent_id, from_timestamp, to_timestamp });
        defer self.allocator.free(query);

        const results = try self.table.search(query);
        defer self.allocator.free(results);

        var states = try self.allocator.alloc(AgentState, results.len);
        var valid_count: usize = 0;

        for (results) |result| {
            if (result.record) |record| {
                states[valid_count] = try AgentState.fromLanceRecord(record, self.allocator);
                valid_count += 1;
            }
        }

        // 调整数组大小
        if (valid_count < states.len) {
            const resized_states = try self.allocator.realloc(states, valid_count);
            return resized_states;
        }

        return states;
    }

    // 创建状态快照
    pub fn createSnapshot(self: *Self, agent_id: u64, snapshot_name: []const u8) !void {
        const current_state = try self.loadState(agent_id) orelse return error.StateNotFound;

        const snapshot = try current_state.createSnapshot(self.allocator, snapshot_name);
        defer {
            var mutable_snapshot = snapshot;
            mutable_snapshot.deinit(self.allocator);
        }

        try self.saveState(snapshot);
    }

    // 恢复到快照
    pub fn restoreFromSnapshot(self: *Self, agent_id: u64, snapshot_name: []const u8) !void {
        const query = try std.fmt.allocPrint(self.allocator, "agent_id = {} AND metadata.snapshot_name = '{s}'", .{ agent_id, snapshot_name });
        defer self.allocator.free(query);

        const results = try self.table.search(query);
        defer self.allocator.free(results);

        if (results.len == 0) {
            return error.SnapshotNotFound;
        }

        if (results[0].record) |record| {
            var snapshot_state = try AgentState.fromLanceRecord(record, self.allocator);
            defer snapshot_state.deinit(self.allocator);

            // 创建新的当前状态
            var new_state = try AgentState.init(self.allocator, agent_id, snapshot_state.session_id, snapshot_state.state_type, snapshot_state.data);
            defer new_state.deinit(self.allocator);

            try self.saveState(new_state);
        }
    }

    // 删除旧状态（清理历史）
    pub fn cleanupOldStates(self: *Self, agent_id: u64, keep_days: u32) !void {
        const cutoff_timestamp = std.time.timestamp() - (@as(i64, keep_days) * 24 * 3600);

        const query = try std.fmt.allocPrint(self.allocator, "agent_id = {} AND timestamp < {} AND metadata.is_snapshot != 'true'", .{ agent_id, cutoff_timestamp });
        defer self.allocator.free(query);

        // 注意：这里需要实现删除功能，Lance可能不直接支持
        // 实际实现中可能需要重写表或使用软删除
        _ = query; // 临时忽略
    }

    // 获取Agent状态统计信息
    pub fn getStateStats(self: *Self, agent_id: u64) !StateStats {
        const query = try std.fmt.allocPrint(self.allocator, "agent_id = {}", .{agent_id});
        defer self.allocator.free(query);

        const results = try self.table.search(query);
        defer self.allocator.free(results);

        var stats = StateStats{
            .total_states = results.len,
            .state_types = std.HashMap(StateType, u32).init(self.allocator),
            .total_size = 0,
            .oldest_timestamp = std.math.maxInt(i64),
            .newest_timestamp = 0,
        };

        for (results) |result| {
            if (result.record) |record| {
                const state = try AgentState.fromLanceRecord(record, self.allocator);
                defer {
                    var mutable_state = state;
                    mutable_state.deinit(self.allocator);
                }

                // 统计状态类型
                const current_count = stats.state_types.get(state.state_type) orelse 0;
                try stats.state_types.put(state.state_type, current_count + 1);

                // 统计大小
                stats.total_size += state.getSize();

                // 统计时间范围
                if (state.timestamp < stats.oldest_timestamp) {
                    stats.oldest_timestamp = state.timestamp;
                }
                if (state.timestamp > stats.newest_timestamp) {
                    stats.newest_timestamp = state.timestamp;
                }
            }
        }

        return stats;
    }

    // 压缩Agent状态
    pub fn compressStates(self: *Self, agent_id: u64) !void {
        const query = try std.fmt.allocPrint(self.allocator, "agent_id = {}", .{agent_id});
        defer self.allocator.free(query);

        const results = try self.table.search(query);
        defer self.allocator.free(results);

        for (results) |result| {
            if (result.record) |record| {
                var state = try AgentState.fromLanceRecord(record, self.allocator);
                defer state.deinit(self.allocator);

                try state.compress(self.allocator);
                try self.saveState(state);
            }
        }
    }

    // 私有方法：更新缓存
    fn updateCache(self: *Self, state: AgentState) !void {
        // 如果缓存已满，移除最旧的条目
        if (self.cache.count() >= self.cache_size_limit) {
            try self.evictOldestFromCache();
        }

        // 复制状态到缓存
        const state_copy = try self.copyState(state);
        try self.cache.put(state.agent_id, state_copy);
    }

    // 私有方法：从缓存中移除最旧的条目
    fn evictOldestFromCache(self: *Self) !void {
        var oldest_timestamp: i64 = std.math.maxInt(i64);
        var oldest_agent_id: u64 = 0;

        var iterator = self.cache.iterator();
        while (iterator.next()) |entry| {
            if (entry.value_ptr.timestamp < oldest_timestamp) {
                oldest_timestamp = entry.value_ptr.timestamp;
                oldest_agent_id = entry.key_ptr.*;
            }
        }

        if (oldest_agent_id != 0) {
            if (self.cache.fetchRemove(oldest_agent_id)) |removed| {
                var state = removed.value;
                state.deinit(self.allocator);
            }
        }
    }

    // 私有方法：复制状态
    fn copyState(self: *Self, state: AgentState) !AgentState {
        var copy = try AgentState.init(self.allocator, state.agent_id, state.session_id, state.state_type, state.data);

        copy.timestamp = state.timestamp;
        copy.version = state.version;
        copy.checksum = state.checksum;

        // 复制metadata
        var iterator = state.metadata.iterator();
        while (iterator.next()) |entry| {
            try copy.setMetadata(self.allocator, entry.key_ptr.*, entry.value_ptr.*);
        }

        return copy;
    }

    // 私有方法：序列化记录（简化实现）
    fn serializeRecord(self: *Self, record: *lance_ffi.Record) ![]u8 {
        // 这里应该实现真正的序列化逻辑
        // 目前返回一个简单的占位符
        _ = record;
        return try self.allocator.dupe(u8, "serialized_record_placeholder");
    }
};

// 状态统计信息
pub const StateStats = struct {
    total_states: usize,
    state_types: std.HashMap(StateType, u32),
    total_size: usize,
    oldest_timestamp: i64,
    newest_timestamp: i64,

    pub fn deinit(self: *StateStats) void {
        self.state_types.deinit();
    }
};
