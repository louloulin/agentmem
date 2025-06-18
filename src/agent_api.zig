// 完整的Agent状态数据库Zig API
const std = @import("std");
const c = @cImport({
    @cInclude("agent_state_db.h");
});

// 错误类型定义
pub const AgentDbError = error{
    DatabaseCreationFailed,
    StateNotFound,
    SaveFailed,
    LoadFailed,
    InvalidArgument,
    MemoryAllocationFailed,
    OutOfMemory,
    IndexingFailed,
    SearchFailed,
    ContextBuildFailed,
};

// Agent状态类型
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
};

// 记忆类型
pub const MemoryType = enum(c_int) {
    episodic = 0,
    semantic = 1,
    procedural = 2,
    working = 3,

    pub fn toString(self: MemoryType) []const u8 {
        return switch (self) {
            .episodic => "episodic",
            .semantic => "semantic",
            .procedural => "procedural",
            .working => "working",
        };
    }
};

// Agent状态结构
pub const AgentState = struct {
    agent_id: u64,
    session_id: u64,
    state_type: StateType,
    data: []const u8,
    
    pub fn init(agent_id: u64, session_id: u64, state_type: StateType, data: []const u8) AgentState {
        return AgentState{
            .agent_id = agent_id,
            .session_id = session_id,
            .state_type = state_type,
            .data = data,
        };
    }
};

// 记忆结构
pub const Memory = struct {
    agent_id: u64,
    memory_type: MemoryType,
    content: []const u8,
    importance: f32,
    
    pub fn init(agent_id: u64, memory_type: MemoryType, content: []const u8, importance: f32) Memory {
        return Memory{
            .agent_id = agent_id,
            .memory_type = memory_type,
            .content = content,
            .importance = importance,
        };
    }
};

// 文档结构
pub const Document = struct {
    title: []const u8,
    content: []const u8,
    chunk_size: usize,
    overlap: usize,
    
    pub fn init(title: []const u8, content: []const u8, chunk_size: usize, overlap: usize) Document {
        return Document{
            .title = title,
            .content = content,
            .chunk_size = chunk_size,
            .overlap = overlap,
        };
    }
};

// 搜索结果
pub const SearchResults = struct {
    agent_ids: []u64,
    count: usize,
    allocator: std.mem.Allocator,
    
    pub fn deinit(self: *SearchResults) void {
        self.allocator.free(self.agent_ids);
    }
};

// 统一的Agent数据库接口
pub const AgentDatabase = struct {
    db_handle: ?*c.CAgentStateDB,
    memory_handle: ?*c.CMemoryManager,
    rag_handle: ?*c.CRAGEngine,
    allocator: std.mem.Allocator,
    db_path: []u8,
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator, db_path: []const u8) !Self {
        // 复制路径
        const path_copy = try allocator.dupe(u8, db_path);
        errdefer allocator.free(path_copy);
        
        // 创建以null结尾的C字符串
        const c_path = try allocator.dupeZ(u8, db_path);
        defer allocator.free(c_path);
        
        // 初始化各个组件
        const db_handle = c.agent_db_new(c_path.ptr);
        if (db_handle == null) {
            return AgentDbError.DatabaseCreationFailed;
        }
        
        const memory_handle = c.memory_manager_new(c_path.ptr);
        if (memory_handle == null) {
            c.agent_db_free(db_handle);
            return AgentDbError.DatabaseCreationFailed;
        }
        
        const rag_handle = c.rag_engine_new(c_path.ptr);
        if (rag_handle == null) {
            c.agent_db_free(db_handle);
            c.memory_manager_free(memory_handle);
            return AgentDbError.DatabaseCreationFailed;
        }
        
        return Self{
            .db_handle = db_handle,
            .memory_handle = memory_handle,
            .rag_handle = rag_handle,
            .allocator = allocator,
            .db_path = path_copy,
        };
    }
    
    pub fn deinit(self: *Self) void {
        if (self.db_handle) |handle| {
            c.agent_db_free(handle);
            self.db_handle = null;
        }
        
        if (self.memory_handle) |handle| {
            c.memory_manager_free(handle);
            self.memory_handle = null;
        }
        
        if (self.rag_handle) |handle| {
            c.rag_engine_free(handle);
            self.rag_handle = null;
        }
        
        self.allocator.free(self.db_path);
    }
    
    // Agent状态管理
    pub fn saveState(self: *Self, state: AgentState) !void {
        const handle = self.db_handle orelse return AgentDbError.InvalidArgument;
        
        const result = c.agent_db_save_state(
            handle,
            state.agent_id,
            state.session_id,
            @intFromEnum(state.state_type),
            state.data.ptr,
            state.data.len,
        );
        
        if (result != 0) {
            return AgentDbError.SaveFailed;
        }
    }
    
    pub fn loadState(self: *Self, agent_id: u64) !?[]u8 {
        const handle = self.db_handle orelse return AgentDbError.InvalidArgument;
        
        var data_ptr: [*c]u8 = undefined;
        var data_len: usize = undefined;
        
        const result = c.agent_db_load_state(handle, agent_id, &data_ptr, &data_len);
        
        switch (result) {
            0 => {
                const data = try self.allocator.alloc(u8, data_len);
                @memcpy(data, data_ptr[0..data_len]);
                c.agent_db_free_data(data_ptr, data_len);
                return data;
            },
            1 => return null,
            else => return AgentDbError.LoadFailed,
        }
    }
    
    pub fn saveVectorState(self: *Self, state: AgentState, embedding: []const f32) !void {
        const handle = self.db_handle orelse return AgentDbError.InvalidArgument;
        
        const result = c.agent_db_save_vector_state(
            handle,
            state.agent_id,
            state.session_id,
            @intFromEnum(state.state_type),
            state.data.ptr,
            state.data.len,
            embedding.ptr,
            embedding.len,
        );
        
        if (result != 0) {
            return AgentDbError.SaveFailed;
        }
    }
    
    pub fn vectorSearch(self: *Self, query_embedding: []const f32, limit: usize) !SearchResults {
        const handle = self.db_handle orelse return AgentDbError.InvalidArgument;
        
        var results_ptr: [*c]u64 = undefined;
        var results_count: usize = undefined;
        
        const result = c.agent_db_vector_search(
            handle,
            query_embedding.ptr,
            query_embedding.len,
            limit,
            &results_ptr,
            &results_count,
        );
        
        if (result != 0) {
            return AgentDbError.SearchFailed;
        }
        
        const agent_ids = try self.allocator.alloc(u64, results_count);
        @memcpy(agent_ids, results_ptr[0..results_count]);
        std.c.free(results_ptr);
        
        return SearchResults{
            .agent_ids = agent_ids,
            .count = results_count,
            .allocator = self.allocator,
        };
    }
    
    // 记忆管理
    pub fn storeMemory(self: *Self, memory: Memory) !void {
        const handle = self.memory_handle orelse return AgentDbError.InvalidArgument;
        
        const c_content = try self.allocator.dupeZ(u8, memory.content);
        defer self.allocator.free(c_content);
        
        const result = c.memory_manager_store_memory(
            handle,
            memory.agent_id,
            @intFromEnum(memory.memory_type),
            c_content.ptr,
            memory.importance,
        );
        
        if (result != 0) {
            return AgentDbError.SaveFailed;
        }
    }
    
    pub fn retrieveMemories(self: *Self, agent_id: u64, limit: usize) !usize {
        const handle = self.memory_handle orelse return AgentDbError.InvalidArgument;
        
        var memory_count: usize = undefined;
        
        const result = c.memory_manager_retrieve_memories(handle, agent_id, limit, &memory_count);
        
        if (result != 0) {
            return AgentDbError.LoadFailed;
        }
        
        return memory_count;
    }
    
    // RAG功能
    pub fn indexDocument(self: *Self, document: Document) !void {
        const handle = self.rag_handle orelse return AgentDbError.InvalidArgument;
        
        const c_title = try self.allocator.dupeZ(u8, document.title);
        defer self.allocator.free(c_title);
        
        const c_content = try self.allocator.dupeZ(u8, document.content);
        defer self.allocator.free(c_content);
        
        const result = c.rag_engine_index_document(
            handle,
            c_title.ptr,
            c_content.ptr,
            document.chunk_size,
            document.overlap,
        );
        
        if (result != 0) {
            return AgentDbError.IndexingFailed;
        }
    }
    
    pub fn searchText(self: *Self, query: []const u8, limit: usize) !usize {
        const handle = self.rag_handle orelse return AgentDbError.InvalidArgument;
        
        const c_query = try self.allocator.dupeZ(u8, query);
        defer self.allocator.free(c_query);
        
        var results_count: usize = undefined;
        
        const result = c.rag_engine_search_text(handle, c_query.ptr, limit, &results_count);
        
        if (result != 0) {
            return AgentDbError.SearchFailed;
        }
        
        return results_count;
    }
    
    pub fn buildContext(self: *Self, query: []const u8, max_tokens: usize) ![]u8 {
        const handle = self.rag_handle orelse return AgentDbError.InvalidArgument;
        
        const c_query = try self.allocator.dupeZ(u8, query);
        defer self.allocator.free(c_query);
        
        var context_ptr: [*c]u8 = undefined;
        var context_len: usize = undefined;
        
        const result = c.rag_engine_build_context(
            handle,
            c_query.ptr,
            max_tokens,
            &context_ptr,
            &context_len,
        );
        
        if (result != 0) {
            return AgentDbError.ContextBuildFailed;
        }
        
        const context = try self.allocator.alloc(u8, context_len);
        @memcpy(context, context_ptr[0..context_len]);
        c.rag_engine_free_context(context_ptr);
        
        return context;
    }
    
    // 便利方法
    pub fn createAgent(self: *Self, agent_id: u64, initial_data: []const u8) !void {
        const state = AgentState.init(agent_id, 0, StateType.working_memory, initial_data);
        try self.saveState(state);
    }
    
    pub fn updateAgent(self: *Self, agent_id: u64, new_data: []const u8) !void {
        const state = AgentState.init(agent_id, 0, StateType.working_memory, new_data);
        try self.saveState(state);
    }
    
    pub fn addMemory(self: *Self, agent_id: u64, content: []const u8, memory_type: MemoryType, importance: f32) !void {
        const memory = Memory.init(agent_id, memory_type, content, importance);
        try self.storeMemory(memory);
    }
    
    pub fn addDocument(self: *Self, title: []const u8, content: []const u8) !void {
        const document = Document.init(title, content, 200, 50);
        try self.indexDocument(document);
    }
    
    pub fn queryKnowledge(self: *Self, query: []const u8) ![]u8 {
        return try self.buildContext(query, 1000);
    }
};
