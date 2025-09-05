// RAG (检索增强生成) Zig API
const std = @import("std");
const c = @cImport({
    @cInclude("agent_state_db.h");
});

pub const Document = struct {
    id: []const u8,
    title: []const u8,
    content: []const u8,
    metadata: std.StringHashMap([]const u8),
    embedding: ?[]const f32,
    chunks: std.ArrayList(DocumentChunk),
    indexed_at: i64,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, title: []const u8, content: []const u8, chunk_size: usize, overlap: usize) !Self {
        var doc = Self{
            .id = try std.fmt.allocPrint(allocator, "doc_{d}", .{std.time.timestamp()}),
            .title = try allocator.dupe(u8, title),
            .content = try allocator.dupe(u8, content),
            .metadata = std.StringHashMap([]const u8).init(allocator),
            .embedding = null,
            .chunks = std.ArrayList(DocumentChunk).init(allocator),
            .indexed_at = std.time.timestamp(),
        };

        try doc.createChunks(allocator, chunk_size, overlap);
        return doc;
    }

    pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.id);
        allocator.free(self.title);
        allocator.free(self.content);

        if (self.embedding) |embedding| {
            allocator.free(embedding);
        }

        for (self.chunks.items) |*chunk| {
            chunk.deinit(allocator);
        }
        self.chunks.deinit();

        var iterator = self.metadata.iterator();
        while (iterator.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.metadata.deinit();
    }

    fn createChunks(self: *Self, allocator: std.mem.Allocator, chunk_size: usize, overlap: usize) !void {
        var start: usize = 0;
        var chunk_id: usize = 0;

        while (start < self.content.len) {
            const end = @min(start + chunk_size, self.content.len);
            const chunk_content = self.content[start..end];

            const chunk = try DocumentChunk.init(allocator, chunk_id, chunk_content, start, end);
            try self.chunks.append(chunk);

            chunk_id += 1;
            if (end >= self.content.len) break;
            start = end - overlap;
        }
    }

    pub fn setMetadata(self: *Self, allocator: std.mem.Allocator, key: []const u8, value: []const u8) !void {
        const owned_key = try allocator.dupe(u8, key);
        const owned_value = try allocator.dupe(u8, value);
        try self.metadata.put(owned_key, owned_value);
    }
};

pub const DocumentChunk = struct {
    id: []const u8,
    content: []const u8,
    start_pos: usize,
    end_pos: usize,
    embedding: ?[]const f32,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, chunk_id: usize, content: []const u8, start_pos: usize, end_pos: usize) !Self {
        return Self{
            .id = try std.fmt.allocPrint(allocator, "chunk_{d}", .{chunk_id}),
            .content = try allocator.dupe(u8, content),
            .start_pos = start_pos,
            .end_pos = end_pos,
            .embedding = null,
        };
    }

    pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.id);
        allocator.free(self.content);
        if (self.embedding) |embedding| {
            allocator.free(embedding);
        }
    }
};

pub const SearchResult = struct {
    document_id: []const u8,
    chunk_id: ?[]const u8,
    score: f32,
    content: []const u8,
    metadata: std.StringHashMap([]const u8),

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, document_id: []const u8, content: []const u8, score: f32) !Self {
        return Self{
            .document_id = try allocator.dupe(u8, document_id),
            .chunk_id = null,
            .score = score,
            .content = try allocator.dupe(u8, content),
            .metadata = std.StringHashMap([]const u8).init(allocator),
        };
    }

    pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.document_id);
        if (self.chunk_id) |chunk_id| {
            allocator.free(chunk_id);
        }
        allocator.free(self.content);

        var iterator = self.metadata.iterator();
        while (iterator.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.metadata.deinit();
    }
};

pub const SearchResults = struct {
    results: std.ArrayList(SearchResult),
    total_count: usize,
    query_time_ms: f64,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator) Self {
        return Self{
            .results = std.ArrayList(SearchResult).init(allocator),
            .total_count = 0,
            .query_time_ms = 0.0,
        };
    }

    pub fn deinit(self: *Self) void {
        for (self.results.items) |*result| {
            // 注意：这里需要分配器来释放内存
            // 在实际实现中应该保存分配器引用
            _ = result;
        }
        self.results.deinit();
    }

    pub fn addResult(self: *Self, result: SearchResult) !void {
        try self.results.append(result);
        self.total_count += 1;
    }
};

pub const RAGContext = struct {
    query: []const u8,
    context: []const u8,
    sources: std.ArrayList(SearchResult),
    token_count: usize,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, query: []const u8) !Self {
        return Self{
            .query = try allocator.dupe(u8, query),
            .context = try allocator.alloc(u8, 0),
            .sources = std.ArrayList(SearchResult).init(allocator),
            .token_count = 0,
        };
    }

    pub fn deinit(self: *Self, allocator: std.mem.Allocator) void {
        allocator.free(self.query);
        allocator.free(self.context);

        for (self.sources.items) |*source| {
            source.deinit(allocator);
        }
        self.sources.deinit();
    }

    pub fn buildContext(self: *Self, allocator: std.mem.Allocator, search_results: []const SearchResult, max_tokens: usize) !void {
        var context_builder = std.ArrayList(u8).init(allocator);
        defer context_builder.deinit();

        var current_tokens: usize = 0;

        for (search_results) |result| {
            // 简化的token计算：假设每个字符约等于0.25个token
            const estimated_tokens = result.content.len / 4;

            if (current_tokens + estimated_tokens > max_tokens) {
                break;
            }

            try context_builder.appendSlice(result.content);
            try context_builder.appendSlice("\n\n");

            current_tokens += estimated_tokens;
            try self.sources.append(result);
        }

        allocator.free(self.context);
        self.context = try context_builder.toOwnedSlice();
        self.token_count = current_tokens;
    }
};

pub const RAGEngine = struct {
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

    pub fn indexDocument(self: *Self, document: *const Document) !void {
        if (self.db == null) return error.DatabaseNotInitialized;

        // 序列化文档数据
        const doc_json = try std.json.stringifyAlloc(self.allocator, document, .{});
        defer self.allocator.free(doc_json);

        // 简化实现，实际应该调用相应的C函数
        // 使用 doc_json 进行实际的索引操作
        std.log.debug("Indexing document: {s}", .{doc_json});
    }

    pub fn searchByText(self: *Self, query: []const u8, limit: usize) !SearchResults {
        if (self.db == null) return error.DatabaseNotInitialized;

        _ = query;
        _ = limit;

        const results = SearchResults.init(self.allocator);

        // 简化实现，实际应该调用相应的C函数进行文本搜索
        return results;
    }

    pub fn semanticSearch(self: *Self, query_embedding: []const f32, limit: usize) !SearchResults {
        if (self.db == null) return error.DatabaseNotInitialized;

        _ = query_embedding;
        _ = limit;

        const results = SearchResults.init(self.allocator);

        // 简化实现，实际应该调用相应的C函数进行语义搜索
        return results;
    }

    pub fn hybridSearch(self: *Self, text_query: []const u8, query_embedding: []const f32, alpha: f32, limit: usize) !SearchResults {
        if (self.db == null) return error.DatabaseNotInitialized;

        _ = text_query;
        _ = query_embedding;
        _ = alpha;
        _ = limit;

        const results = SearchResults.init(self.allocator);

        // 简化实现，实际应该结合文本搜索和语义搜索
        return results;
    }

    pub fn buildContext(self: *Self, query: []const u8, search_results: []const SearchResult, max_tokens: usize) !RAGContext {
        var context = try RAGContext.init(self.allocator, query);
        try context.buildContext(self.allocator, search_results, max_tokens);

        return context;
    }

    pub fn getDocument(self: *Self, doc_id: []const u8) !?Document {
        if (self.db == null) return error.DatabaseNotInitialized;

        _ = doc_id;

        // 简化实现，实际应该调用相应的C函数
        return null;
    }
};

test "Document creation and chunking" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var doc = try Document.init(allocator, "Test Document", "This is a test document with some content for chunking.", 20, 5);
    defer doc.deinit(allocator);

    try std.testing.expect(std.mem.eql(u8, doc.title, "Test Document"));
    try std.testing.expect(doc.chunks.items.len > 0);

    // 测试元数据设置
    try doc.setMetadata(allocator, "author", "Test Author");
    try std.testing.expect(doc.metadata.contains("author"));
}

test "SearchResult creation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var result = try SearchResult.init(allocator, "doc_123", "Test content", 0.95);
    defer result.deinit(allocator);

    try std.testing.expect(std.mem.eql(u8, result.document_id, "doc_123"));
    try std.testing.expect(result.score == 0.95);
    try std.testing.expect(std.mem.eql(u8, result.content, "Test content"));
}
