// 向量处理 Zig API
const std = @import("std");
const c = @cImport({
    @cInclude("agent_state_db.h");
});

pub const SimilarityAlgorithm = enum(c_int) {
    cosine = 0,
    euclidean = 1,
    dot_product = 2,
    manhattan = 3,
};

pub const VectorSearchResult = struct {
    id: u64,
    vector: []f32,
    metadata: std.StringHashMap([]const u8),
    similarity: f32,
    distance: f32,
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator, id: u64, vector: []const f32, similarity: f32) !Self {
        return Self{
            .id = id,
            .vector = try allocator.dupe(f32, vector),
            .metadata = std.StringHashMap([]const u8).init(allocator),
            .similarity = similarity,
            .distance = 1.0 - similarity,
        };
    }
    
    pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.vector);
        
        var iterator = self.metadata.iterator();
        while (iterator.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.metadata.deinit();
    }
    
    pub fn setMetadata(self: *Self, allocator: std.mem.Allocator, key: []const u8, value: []const u8) !void {
        const owned_key = try allocator.dupe(u8, key);
        const owned_value = try allocator.dupe(u8, value);
        try self.metadata.put(owned_key, owned_value);
    }
};

pub const VectorEngine = struct {
    db: ?*c.CAgentStateDB,
    allocator: std.mem.Allocator,
    dimension: usize,
    algorithm: SimilarityAlgorithm,
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator, db_path: []const u8, dimension: usize, algorithm: SimilarityAlgorithm) !Self {
        const c_path = try allocator.dupeZ(u8, db_path);
        defer allocator.free(c_path);
        
        const db = c.agent_db_new(c_path.ptr);
        if (db == null) {
            return error.DatabaseInitFailed;
        }
        
        return Self{
            .db = db,
            .allocator = allocator,
            .dimension = dimension,
            .algorithm = algorithm,
        };
    }
    
    pub fn deinit(self: *Self) void {
        if (self.db) |db| {
            c.agent_db_free(db);
        }
    }
    
    pub fn addVector(self: *Self, id: u64, vector: []const f32, metadata: ?std.StringHashMap([]const u8)) !void {
        if (self.db == null) return error.DatabaseNotInitialized;
        
        if (vector.len != self.dimension) {
            return error.InvalidVectorDimension;
        }
        
        // 序列化向量和元数据
        const vector_bytes = std.mem.sliceAsBytes(vector);
        
        const result = c.agent_db_save_state(
            self.db.?,
            id,
            0, // session_id
            5, // embeddings type
            vector_bytes.ptr,
            vector_bytes.len
        );
        
        if (result != 0) {
            return error.AddVectorFailed;
        }
    }
    
    pub fn searchVectors(self: *Self, query_vector: []const f32, limit: usize) !std.ArrayList(VectorSearchResult) {
        if (self.db == null) return error.DatabaseNotInitialized;
        
        if (query_vector.len != self.dimension) {
            return error.InvalidVectorDimension;
        }
        
        const results = std.ArrayList(VectorSearchResult).init(self.allocator);
        
        // 简化实现，实际应该调用相应的C函数进行向量搜索
        // 这里返回空结果作为占位符
        _ = limit;
        
        return results;
    }
    
    pub fn getVector(self: *Self, id: u64) !?VectorSearchResult {
        if (self.db == null) return error.DatabaseNotInitialized;
        
        // 简化实现，实际应该调用相应的C函数
        _ = id;
        return null;
    }
    
    pub fn updateVector(self: *Self, id: u64, vector: []const f32, metadata: ?std.StringHashMap([]const u8)) !void {
        if (self.db == null) return error.DatabaseNotInitialized;
        
        if (vector.len != self.dimension) {
            return error.InvalidVectorDimension;
        }
        
        // 简化实现，实际应该先删除再添加
        try self.addVector(id, vector, metadata);
    }
    
    pub fn deleteVector(self: *Self, id: u64) !void {
        if (self.db == null) return error.DatabaseNotInitialized;
        
        // 简化实现，实际应该调用相应的C函数
        _ = id;
    }
    
    pub fn batchAddVectors(self: *Self, vectors: []const struct { id: u64, vector: []const f32, metadata: ?std.StringHashMap([]const u8) }) !void {
        for (vectors) |vec_data| {
            try self.addVector(vec_data.id, vec_data.vector, vec_data.metadata);
        }
    }
    
    pub fn getVectorCount(self: *Self) !usize {
        if (self.db == null) return error.DatabaseNotInitialized;
        
        // 简化实现，实际应该调用相应的C函数
        return 0;
    }
};

// 相似度计算函数
pub fn cosineSimilarity(a: []const f32, b: []const f32) f32 {
    if (a.len != b.len) return 0.0;
    
    var dot_product: f32 = 0.0;
    var norm_a: f32 = 0.0;
    var norm_b: f32 = 0.0;
    
    for (a, b) |x, y| {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    
    norm_a = @sqrt(norm_a);
    norm_b = @sqrt(norm_b);
    
    if (norm_a == 0.0 or norm_b == 0.0) {
        return 0.0;
    }
    
    return dot_product / (norm_a * norm_b);
}

pub fn euclideanDistance(a: []const f32, b: []const f32) f32 {
    if (a.len != b.len) return std.math.inf(f32);
    
    var sum: f32 = 0.0;
    for (a, b) |x, y| {
        const diff = x - y;
        sum += diff * diff;
    }
    
    return @sqrt(sum);
}

pub fn dotProduct(a: []const f32, b: []const f32) f32 {
    if (a.len != b.len) return 0.0;
    
    var result: f32 = 0.0;
    for (a, b) |x, y| {
        result += x * y;
    }
    
    return result;
}

pub fn manhattanDistance(a: []const f32, b: []const f32) f32 {
    if (a.len != b.len) return std.math.inf(f32);
    
    var sum: f32 = 0.0;
    for (a, b) |x, y| {
        sum += @abs(x - y);
    }
    
    return sum;
}

test "Vector similarity calculations" {
    const a = [_]f32{ 1.0, 2.0, 3.0 };
    const b = [_]f32{ 4.0, 5.0, 6.0 };
    
    const cosine_sim = cosineSimilarity(&a, &b);
    try std.testing.expect(cosine_sim > 0.9); // 应该很相似
    
    const euclidean_dist = euclideanDistance(&a, &b);
    try std.testing.expect(euclidean_dist > 0.0);
    
    const dot_prod = dotProduct(&a, &b);
    try std.testing.expect(dot_prod == 32.0); // 1*4 + 2*5 + 3*6 = 32
    
    const manhattan_dist = manhattanDistance(&a, &b);
    try std.testing.expect(manhattan_dist == 9.0); // |1-4| + |2-5| + |3-6| = 9
}

test "VectorSearchResult creation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const vector = [_]f32{ 1.0, 2.0, 3.0 };
    var result = try VectorSearchResult.init(allocator, 123, &vector, 0.95);
    defer result.deinit(allocator);
    
    try std.testing.expect(result.id == 123);
    try std.testing.expect(result.similarity == 0.95);
    try std.testing.expect(result.distance == 0.05);
    try std.testing.expect(result.vector.len == 3);
}
