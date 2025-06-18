// Agent状态数据库主API
const std = @import("std");
const lance_ffi = @import("lance_ffi.zig");
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;
const AgentStateManager = @import("agent_state_manager.zig").AgentStateManager;
const StateStats = @import("agent_state_manager.zig").StateStats;

// 主要的Agent数据库结构
pub const AgentDB = struct {
    state_manager: AgentStateManager,
    allocator: std.mem.Allocator,
    db_path: []const u8,
    is_initialized: bool,

    const Self = @This();

    // 初始化数据库
    pub fn init(db_path: []const u8, allocator: std.mem.Allocator) !Self {
        const path_copy = try allocator.dupe(u8, db_path);

        const state_manager = try AgentStateManager.init(path_copy, allocator);

        return Self{
            .state_manager = state_manager,
            .allocator = allocator,
            .db_path = path_copy,
            .is_initialized = true,
        };
    }

    // 清理资源
    pub fn deinit(self: *Self) void {
        if (self.is_initialized) {
            self.state_manager.deinit();
            self.allocator.free(self.db_path);
            self.is_initialized = false;
        }
    }

    // 保存Agent状态
    pub fn saveAgentState(self: *Self, agent_id: u64, session_id: u64, state_type: StateType, data: []const u8) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        var state = try AgentState.init(self.allocator, agent_id, session_id, state_type, data);
        defer state.deinit(self.allocator);

        try self.state_manager.saveState(state);
    }

    // 加载Agent状态
    pub fn loadAgentState(self: *Self, agent_id: u64) !?AgentState {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        return try self.state_manager.loadState(agent_id);
    }

    // 更新Agent状态
    pub fn updateAgentState(self: *Self, agent_id: u64, new_data: []const u8) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        var current_state = try self.loadAgentState(agent_id) orelse return error.StateNotFound;
        defer current_state.deinit(self.allocator);

        try current_state.updateData(self.allocator, new_data);
        try self.state_manager.saveState(current_state);
    }

    // 查询Agent状态历史
    pub fn queryAgentHistory(self: *Self, agent_id: u64, from_timestamp: i64, to_timestamp: i64) ![]AgentState {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        return try self.state_manager.queryHistory(agent_id, from_timestamp, to_timestamp);
    }

    // 创建状态快照
    pub fn createStateSnapshot(self: *Self, agent_id: u64, snapshot_name: []const u8) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        try self.state_manager.createSnapshot(agent_id, snapshot_name);
    }

    // 恢复到快照
    pub fn restoreFromSnapshot(self: *Self, agent_id: u64, snapshot_name: []const u8) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        try self.state_manager.restoreFromSnapshot(agent_id, snapshot_name);
    }

    // 设置状态元数据
    pub fn setStateMetadata(self: *Self, agent_id: u64, key: []const u8, value: []const u8) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        var state = try self.loadAgentState(agent_id) orelse return error.StateNotFound;
        defer state.deinit(self.allocator);

        try state.setMetadata(self.allocator, key, value);
        try self.state_manager.saveState(state);
    }

    // 获取状态元数据
    pub fn getStateMetadata(self: *Self, agent_id: u64, key: []const u8) !?[]const u8 {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        const state = try self.loadAgentState(agent_id) orelse return null;
        defer {
            var mutable_state = state;
            mutable_state.deinit(self.allocator);
        }

        return state.getMetadata(key);
    }

    // 获取状态统计信息
    pub fn getAgentStats(self: *Self, agent_id: u64) !StateStats {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        return try self.state_manager.getStateStats(agent_id);
    }

    // 压缩Agent状态
    pub fn compressAgentStates(self: *Self, agent_id: u64) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        try self.state_manager.compressStates(agent_id);
    }

    // 清理旧状态
    pub fn cleanupOldStates(self: *Self, agent_id: u64, keep_days: u32) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        try self.state_manager.cleanupOldStates(agent_id, keep_days);
    }

    // 验证数据库完整性
    pub fn validateIntegrity(self: *Self) !ValidationResult {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        var result = ValidationResult{
            .total_states = 0,
            .valid_states = 0,
            .corrupted_states = 0,
            .errors = std.ArrayList([]const u8).init(self.allocator),
        };

        // 这里应该实现完整性检查逻辑
        // 目前返回一个基本的结果
        result.total_states = 0;
        result.valid_states = 0;
        result.corrupted_states = 0;

        return result;
    }

    // 获取数据库信息
    pub fn getDatabaseInfo(self: *Self) !DatabaseInfo {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        return DatabaseInfo{
            .db_path = self.db_path,
            .version = "1.0.0",
            .created_at = std.time.timestamp(),
            .last_modified = std.time.timestamp(),
            .total_size = 0, // 需要实现实际的大小计算
        };
    }

    // 备份数据库
    pub fn backup(self: *Self, backup_path: []const u8) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        // 简单的文件复制备份
        const source_file = try std.fs.cwd().openFile(self.db_path, .{});
        defer source_file.close();

        const backup_file = try std.fs.cwd().createFile(backup_path, .{});
        defer backup_file.close();

        // 复制文件内容
        var buffer: [4096]u8 = undefined;
        while (true) {
            const bytes_read = try source_file.readAll(&buffer);
            if (bytes_read == 0) break;

            try backup_file.writeAll(buffer[0..bytes_read]);
        }
    }

    // 从备份恢复
    pub fn restore(backup_path: []const u8, target_path: []const u8, allocator: std.mem.Allocator) !void {
        // 简单的文件复制恢复
        const backup_file = try std.fs.cwd().openFile(backup_path, .{});
        defer backup_file.close();

        const target_file = try std.fs.cwd().createFile(target_path, .{});
        defer target_file.close();

        // 复制文件内容
        var buffer: [4096]u8 = undefined;
        while (true) {
            const bytes_read = try backup_file.readAll(&buffer);
            if (bytes_read == 0) break;

            try target_file.writeAll(buffer[0..bytes_read]);
        }

        _ = allocator; // 暂时未使用
    }

    // 获取所有Agent ID
    pub fn getAllAgentIds(self: *Self) ![]u64 {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        // 这里需要实现查询所有唯一Agent ID的逻辑
        // 目前返回空数组
        return try self.allocator.alloc(u64, 0);
    }

    // 删除Agent的所有状态
    pub fn deleteAgentStates(self: *Self, agent_id: u64) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        // 这里需要实现删除逻辑
        // Lance可能不直接支持删除，需要重写表或软删除
        _ = agent_id; // 暂时忽略
    }

    // 批量保存状态
    pub fn batchSaveStates(self: *Self, states: []const AgentState) !void {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        for (states) |state| {
            try self.state_manager.saveState(state);
        }
    }

    // 搜索状态
    pub fn searchStates(self: *Self, query: StateQuery) ![]AgentState {
        if (!self.is_initialized) return error.DatabaseNotInitialized;

        // 构建查询字符串
        var query_parts = std.ArrayList([]const u8).init(self.allocator);
        defer query_parts.deinit();

        if (query.agent_id) |agent_id| {
            const part = try std.fmt.allocPrint(self.allocator, "agent_id = {}", .{agent_id});
            try query_parts.append(part);
        }

        if (query.state_type) |state_type| {
            const part = try std.fmt.allocPrint(self.allocator, "state_type = '{s}'", .{state_type.toString()});
            try query_parts.append(part);
        }

        if (query.from_timestamp) |from| {
            const part = try std.fmt.allocPrint(self.allocator, "timestamp >= {}", .{from});
            try query_parts.append(part);
        }

        if (query.to_timestamp) |to| {
            const part = try std.fmt.allocPrint(self.allocator, "timestamp <= {}", .{to});
            try query_parts.append(part);
        }

        // 组合查询条件
        const query_str = if (query_parts.items.len > 0)
            try std.mem.join(self.allocator, " AND ", query_parts.items)
        else
            try self.allocator.dupe(u8, "1=1"); // 查询所有

        defer self.allocator.free(query_str);
        defer {
            for (query_parts.items) |part| {
                self.allocator.free(part);
            }
        }

        // 执行查询（这里需要实现实际的查询逻辑）
        _ = query_str;
        return try self.allocator.alloc(AgentState, 0); // 暂时返回空结果
    }
};

// 查询条件结构
pub const StateQuery = struct {
    agent_id: ?u64 = null,
    state_type: ?StateType = null,
    from_timestamp: ?i64 = null,
    to_timestamp: ?i64 = null,
    limit: ?u32 = null,
    offset: ?u32 = null,
};

// 验证结果
pub const ValidationResult = struct {
    total_states: u32,
    valid_states: u32,
    corrupted_states: u32,
    errors: std.ArrayList([]const u8),

    pub fn deinit(self: *ValidationResult) void {
        for (self.errors.items) |error_msg| {
            // 注意：这里假设错误消息是用allocator分配的
            // 实际使用中需要确保正确的内存管理
            _ = error_msg;
        }
        self.errors.deinit();
    }
};

// 数据库信息
pub const DatabaseInfo = struct {
    db_path: []const u8,
    version: []const u8,
    created_at: i64,
    last_modified: i64,
    total_size: u64,
};

// C FFI导出接口
export fn agent_db_init(db_path: [*:0]const u8) ?*AgentDB {
    const allocator = std.heap.c_allocator;
    const path = std.mem.span(db_path);

    const db = allocator.create(AgentDB) catch return null;
    db.* = AgentDB.init(path, allocator) catch {
        allocator.destroy(db);
        return null;
    };

    return db;
}

export fn agent_db_deinit(db: ?*AgentDB) void {
    if (db) |database| {
        database.deinit();
        std.heap.c_allocator.destroy(database);
    }
}

export fn agent_db_save_state(db: ?*AgentDB, agent_id: u64, session_id: u64, state_type: u32, data: [*]const u8, data_len: usize) c_int {
    const database = db orelse return -1;
    const state_type_enum = @as(StateType, @enumFromInt(state_type));
    const data_slice = data[0..data_len];

    database.saveAgentState(agent_id, session_id, state_type_enum, data_slice) catch return -1;
    return 0;
}

export fn agent_db_load_state(db: ?*AgentDB, agent_id: u64, data_out: *[*]u8, data_len_out: *usize) c_int {
    const database = db orelse return -1;

    const state = database.loadAgentState(agent_id) catch return -1;
    if (state) |s| {
        defer {
            var mutable_state = s;
            mutable_state.deinit(database.allocator);
        }

        // 分配内存并复制数据
        const data_copy = database.allocator.dupe(u8, s.data) catch return -1;
        data_out.* = data_copy.ptr;
        data_len_out.* = data_copy.len;
        return 0;
    }

    return -1; // 未找到状态
}

export fn agent_db_free_data(data: [*]u8, data_len: usize) void {
    const slice = data[0..data_len];
    std.heap.c_allocator.free(slice);
}
