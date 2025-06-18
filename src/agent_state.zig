// Agent状态数据模型
const std = @import("std");
const lance_ffi = @import("lance_ffi.zig");

// 重新导出StateType
pub const StateType = lance_ffi.StateType;

// Agent状态结构
pub const AgentState = struct {
    agent_id: u64,
    session_id: u64,
    timestamp: i64,
    state_type: StateType,
    data: []u8,
    metadata: std.StringHashMap([]const u8),
    version: u32,
    checksum: u32,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, agent_id: u64, session_id: u64, state_type: StateType, data: []const u8) !Self {
        const timestamp = std.time.timestamp();
        const data_copy = try allocator.dupe(u8, data);

        const metadata = std.StringHashMap([]const u8).init(allocator);

        // 计算简单的校验和
        var checksum: u32 = 0;
        for (data) |byte| {
            checksum = checksum +% byte;
        }

        return Self{
            .agent_id = agent_id,
            .session_id = session_id,
            .timestamp = timestamp,
            .state_type = state_type,
            .data = data_copy,
            .metadata = metadata,
            .version = 1,
            .checksum = checksum,
        };
    }

    pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.data);

        // 清理metadata
        var iterator = self.metadata.iterator();
        while (iterator.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.metadata.deinit();
    }

    pub fn setMetadata(self: *Self, allocator: std.mem.Allocator, key: []const u8, value: []const u8) !void {
        // 检查是否已存在该键，如果存在则先释放旧的内存
        if (self.metadata.fetchRemove(key)) |old_entry| {
            allocator.free(old_entry.key);
            allocator.free(old_entry.value);
        }

        const key_copy = try allocator.dupe(u8, key);
        const value_copy = try allocator.dupe(u8, value);
        try self.metadata.put(key_copy, value_copy);
    }

    pub fn getMetadata(self: *Self, key: []const u8) ?[]const u8 {
        return self.metadata.get(key);
    }

    // 序列化为JSON字符串（用于存储到LanceDB）
    pub fn toJson(self: *const Self, allocator: std.mem.Allocator) ![]u8 {
        // 简化的JSON序列化，不包含复杂的metadata
        return try std.fmt.allocPrint(allocator, "{{\"agent_id\":{},\"session_id\":{},\"timestamp\":{},\"state_type\":\"{s}\",\"version\":{},\"checksum\":{}}}", .{ self.agent_id, self.session_id, self.timestamp, self.state_type.toString(), self.version, self.checksum });
    }

    // 从JSON字符串反序列化（简化版本）
    pub fn fromJson(json_str: []const u8, data: []const u8, allocator: std.mem.Allocator) !Self {
        // 这里应该实现真正的JSON解析
        // 目前返回一个基本的状态
        _ = json_str;

        return Self.init(allocator, 12345, 67890, StateType.context, data);
    }

    // 验证校验和
    pub fn validateChecksum(self: *const Self) bool {
        var calculated_checksum: u32 = 0;
        for (self.data) |byte| {
            calculated_checksum = calculated_checksum +% byte;
        }
        return calculated_checksum == self.checksum;
    }

    // 更新数据并重新计算校验和
    pub fn updateData(self: *Self, allocator: std.mem.Allocator, new_data: []const u8) !void {
        allocator.free(self.data);
        self.data = try allocator.dupe(u8, new_data);

        // 重新计算校验和
        var checksum: u32 = 0;
        for (self.data) |byte| {
            checksum = checksum +% byte;
        }
        self.checksum = checksum;

        // 更新版本和时间戳
        self.version += 1;
        self.timestamp = std.time.timestamp();
    }

    // 创建快照
    pub fn createSnapshot(self: *const Self, allocator: std.mem.Allocator, snapshot_name: []const u8) !Self {
        var snapshot = Self{
            .agent_id = self.agent_id,
            .session_id = self.session_id,
            .timestamp = std.time.timestamp(),
            .state_type = self.state_type,
            .data = try allocator.dupe(u8, self.data),
            .metadata = std.StringHashMap([]const u8).init(allocator),
            .version = self.version + 1,
            .checksum = self.checksum,
        };

        // 复制metadata
        var iterator = self.metadata.iterator();
        while (iterator.next()) |entry| {
            const key_copy = try allocator.dupe(u8, entry.key_ptr.*);
            const value_copy = try allocator.dupe(u8, entry.value_ptr.*);
            try snapshot.metadata.put(key_copy, value_copy);
        }

        // 添加快照标记
        try snapshot.setMetadata(allocator, "snapshot_name", snapshot_name);
        try snapshot.setMetadata(allocator, "is_snapshot", "true");

        return snapshot;
    }

    // 比较两个状态
    pub fn equals(self: *const Self, other: *const Self) bool {
        return self.agent_id == other.agent_id and
            self.session_id == other.session_id and
            self.state_type == other.state_type and
            std.mem.eql(u8, self.data, other.data) and
            self.checksum == other.checksum;
    }

    // 获取状态大小（用于内存管理）
    pub fn getSize(self: *const Self) usize {
        var size = @sizeOf(Self) + self.data.len;

        var iterator = self.metadata.iterator();
        while (iterator.next()) |entry| {
            size += entry.key_ptr.len + entry.value_ptr.len;
        }

        return size;
    }

    // 压缩状态数据（简单的RLE压缩）
    pub fn compress(self: *Self, allocator: std.mem.Allocator) !void {
        if (self.data.len < 4) return; // 太小的数据不压缩

        var compressed = std.ArrayList(u8).init(allocator);
        defer compressed.deinit();

        var i: usize = 0;
        while (i < self.data.len) {
            const current_byte = self.data[i];
            var count: u8 = 1;

            // 计算连续相同字节的数量
            while (i + count < self.data.len and
                self.data[i + count] == current_byte and
                count < 255)
            {
                count += 1;
            }

            // 如果连续字节数量大于3，使用RLE编码
            if (count >= 3) {
                try compressed.append(0xFF); // 特殊标记
                try compressed.append(count);
                try compressed.append(current_byte);
                i += count;
            } else {
                // 否则直接存储
                for (0..count) |_| {
                    try compressed.append(current_byte);
                }
                i += count;
            }
        }

        // 只有在压缩后更小时才替换
        if (compressed.items.len < self.data.len) {
            allocator.free(self.data);
            self.data = try compressed.toOwnedSlice();

            // 重新计算校验和
            var checksum: u32 = 0;
            for (self.data) |byte| {
                checksum = checksum +% byte;
            }
            self.checksum = checksum;

            try self.setMetadata(allocator, "compressed", "true");
        }
    }

    // 解压缩状态数据
    pub fn decompress(self: *Self, allocator: std.mem.Allocator) !void {
        const is_compressed = self.getMetadata("compressed");
        if (is_compressed == null or !std.mem.eql(u8, is_compressed.?, "true")) {
            return; // 未压缩
        }

        var decompressed = std.ArrayList(u8).init(allocator);
        defer decompressed.deinit();

        var i: usize = 0;
        while (i < self.data.len) {
            if (self.data[i] == 0xFF and i + 2 < self.data.len) {
                // RLE解码
                const count = self.data[i + 1];
                const byte_value = self.data[i + 2];

                for (0..count) |_| {
                    try decompressed.append(byte_value);
                }
                i += 3;
            } else {
                try decompressed.append(self.data[i]);
                i += 1;
            }
        }

        allocator.free(self.data);
        self.data = try decompressed.toOwnedSlice();

        // 重新计算校验和
        var checksum: u32 = 0;
        for (self.data) |byte| {
            checksum = checksum +% byte;
        }
        self.checksum = checksum;

        // 移除压缩标记
        if (self.metadata.fetchRemove("compressed")) |removed| {
            // 释放分配的内存
            allocator.free(removed.key);
            allocator.free(removed.value);
        }
    }
};
