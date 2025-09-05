// 记忆管理 Zig API
const std = @import("std");
const c = @cImport({
    @cInclude("agent_state_db.h");
});

pub const MemoryType = enum(c_int) {
    episodic = 0,
    semantic = 1,
    procedural = 2,
    working = 3,
};

pub const Memory = struct {
    id: []const u8,
    agent_id: u64,
    memory_type: MemoryType,
    content: []const u8,
    importance: f64,
    timestamp: i64,
    access_count: u32,
    last_accessed: i64,
    embedding: ?[]const f32,
    metadata: std.StringHashMap([]const u8),
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator, agent_id: u64, memory_type: MemoryType, 
                content: []const u8, importance: f64) !Self {
        return Self{
            .id = try std.fmt.allocPrint(allocator, "mem_{d}_{d}", .{agent_id, std.time.timestamp()}),
            .agent_id = agent_id,
            .memory_type = memory_type,
            .content = try allocator.dupe(u8, content),
            .importance = importance,
            .timestamp = std.time.timestamp(),
            .access_count = 0,
            .last_accessed = std.time.timestamp(),
            .embedding = null,
            .metadata = std.StringHashMap([]const u8).init(allocator),
        };
    }
    
    pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.id);
        allocator.free(self.content);
        if (self.embedding) |embedding| {
            allocator.free(embedding);
        }
        
        var iterator = self.metadata.iterator();
        while (iterator.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.metadata.deinit();
    }
    
    pub fn access(self: *Self) void {
        self.access_count += 1;
        self.last_accessed = std.time.timestamp();
    }
    
    pub fn updateImportance(self: *Self, new_importance: f64) void {
        self.importance = new_importance;
    }
    
    pub fn setMetadata(self: *Self, allocator: std.mem.Allocator, key: []const u8, value: []const u8) !void {
        const owned_key = try allocator.dupe(u8, key);
        const owned_value = try allocator.dupe(u8, value);
        try self.metadata.put(owned_key, owned_value);
    }
};

pub const MemoryManager = struct {
    db: ?*c.CAgentStateDB,
    allocator: std.mem.Allocator,
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator, db_path: []const u8) !Self {
        const c_path = try allocator.dupeZ(u8, db_path);
        defer allocator.free(c_path);
        
        const db = c.agent_db_new(c_path.ptr);
        if (db == null) {
            return error.DatabaseInitFailed;
        }
        
        return Self{
            .db = db,
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *Self) void {
        if (self.db) |db| {
            c.agent_db_free(db);
        }
    }
    
    pub fn storeMemory(self: *Self, memory: *const Memory) !void {
        if (self.db == null) return error.DatabaseNotInitialized;
        
        // 序列化记忆数据
        const memory_json = try std.json.stringifyAlloc(self.allocator, memory, .{});
        defer self.allocator.free(memory_json);
        
        const result = c.agent_db_save_state(
            self.db.?,
            memory.agent_id,
            0, // session_id
            @intFromEnum(memory.memory_type),
            memory_json.ptr,
            memory_json.len
        );
        
        if (result != 0) {
            return error.StoreFailed;
        }
    }
    
    pub fn retrieveMemories(self: *Self, agent_id: u64) !std.ArrayList(Memory) {
        _ = agent_id; // 标记为有意未使用
        if (self.db == null) return error.DatabaseNotInitialized;

        const memories = std.ArrayList(Memory).init(self.allocator);
        
        // 简化实现，实际应该调用相应的C函数
        // 这里返回空列表作为占位符
        return memories;
    }
    
    pub fn searchMemories(self: *Self, agent_id: u64, query: []const u8, limit: usize) !std.ArrayList(Memory) {
        _ = agent_id;
        _ = query;
        _ = limit;
        
        if (self.db == null) return error.DatabaseNotInitialized;

        const memories = std.ArrayList(Memory).init(self.allocator);
        
        // 简化实现，实际应该调用相应的C函数
        return memories;
    }
    
    pub fn organizeMemories(self: *Self, agent_id: u64) !void {
        _ = agent_id;
        
        if (self.db == null) return error.DatabaseNotInitialized;
        
        // 简化实现，实际应该调用相应的C函数
    }
};

// 记忆统计信息
pub const MemoryStatistics = struct {
    total_memories: u64,
    memories_by_type: std.HashMap(MemoryType, u64, std.hash_map.AutoContext(MemoryType), std.hash_map.default_max_load_percentage),
    average_importance: f64,
    most_accessed_memory: ?[]const u8,
    oldest_memory: ?i64,
    newest_memory: ?i64,
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator) Self {
        return Self{
            .total_memories = 0,
            .memories_by_type = std.HashMap(MemoryType, u64, std.hash_map.AutoContext(MemoryType), std.hash_map.default_max_load_percentage).init(allocator),
            .average_importance = 0.0,
            .most_accessed_memory = null,
            .oldest_memory = null,
            .newest_memory = null,
        };
    }
    
    pub fn deinit(self: *Self) void {
        self.memories_by_type.deinit();
        if (self.most_accessed_memory) |memory_id| {
            // 注意：这里需要知道分配器来释放内存
            // 在实际实现中应该保存分配器引用
            _ = memory_id;
        }
    }
};

test "Memory creation and management" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var memory = try Memory.init(allocator, 12345, MemoryType.episodic, "Test memory content", 0.8);
    defer memory.deinit(allocator);
    
    try std.testing.expect(memory.agent_id == 12345);
    try std.testing.expect(memory.importance == 0.8);
    try std.testing.expect(std.mem.eql(u8, memory.content, "Test memory content"));
    
    memory.access();
    try std.testing.expect(memory.access_count == 1);
    
    memory.updateImportance(0.9);
    try std.testing.expect(memory.importance == 0.9);
}
