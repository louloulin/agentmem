// LanceDB FFI绑定层 - 真正的LanceDB集成
const std = @import("std");

// 导入真正的LanceDB C头文件
const c = @cImport({
    @cInclude("agent_state_db.h");
});

// 错误类型定义
pub const LanceError = error{
    InvalidArgument,
    IOError,
    NotFound,
    AlreadyExists,
    InternalError,
    OutOfMemory,
    NullPointer,
};

// 状态类型定义
pub const StateType = enum(c_int) {
    working_memory = 0,
    long_term_memory = 1,
    context = 2,
    task_state = 3,
    relationship = 4,
    embedding = 5,

    pub fn toString(self: StateType) []const u8 {
        return switch (self) {
            .working_memory => "working_memory",
            .long_term_memory => "long_term_memory",
            .context => "context",
            .task_state => "task_state",
            .relationship => "relationship",
            .embedding => "embedding",
        };
    }

    pub fn fromString(str: []const u8) ?StateType {
        if (std.mem.eql(u8, str, "working_memory")) return .working_memory;
        if (std.mem.eql(u8, str, "long_term_memory")) return .long_term_memory;
        if (std.mem.eql(u8, str, "context")) return .context;
        if (std.mem.eql(u8, str, "task_state")) return .task_state;
        if (std.mem.eql(u8, str, "relationship")) return .relationship;
        if (std.mem.eql(u8, str, "embedding")) return .embedding;
        return null;
    }
};

// 将C返回码转换为Zig错误
fn cResultToZig(result: c_int) LanceError!void {
    switch (result) {
        0 => return,
        -1 => return LanceError.InternalError,
        1 => return LanceError.NotFound,
        else => return LanceError.InternalError,
    }
}

// Agent状态数据库包装器
pub const AgentStateDB = struct {
    handle: *c.CAgentStateDB,
    allocator: std.mem.Allocator,

    pub fn init(path: []const u8, allocator: std.mem.Allocator) !AgentStateDB {
        // 创建以null结尾的C字符串
        const c_path = try allocator.dupeZ(u8, path);
        defer allocator.free(c_path);

        const handle = c.agent_db_new(c_path.ptr);
        if (handle == null) {
            return LanceError.IOError;
        }

        return AgentStateDB{
            .handle = handle.?,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *AgentStateDB) void {
        c.agent_db_free(self.handle);
    }

    pub fn saveState(self: *AgentStateDB, agent_id: u64, session_id: u64, state_type: StateType, data: []const u8) !void {
        const state_type_int = @as(c_int, @intFromEnum(state_type));
        const result = c.agent_db_save_state(self.handle, agent_id, session_id, state_type_int, data.ptr, data.len);
        try cResultToZig(result);
    }

    pub fn loadState(self: *AgentStateDB, agent_id: u64) !?[]u8 {
        var data_ptr: [*]u8 = undefined;
        var data_len: usize = undefined;

        const result = c.agent_db_load_state(self.handle, agent_id, &data_ptr, &data_len);

        switch (result) {
            0 => {
                // 成功，复制数据到Zig管理的内存
                const data = try self.allocator.alloc(u8, data_len);
                @memcpy(data, data_ptr[0..data_len]);

                // 释放C分配的内存
                c.agent_db_free_data(data_ptr, data_len);

                return data;
            },
            1 => return null, // 未找到
            else => return LanceError.InternalError,
        }
    }
};
