// 分布式Agent网络支持 - Zig API层
const std = @import("std");
const c = @cImport({
    @cInclude("agent_state_db.h");
});

// 节点状态枚举
pub const NodeStatus = enum(u32) {
    Active = 0,
    Inactive = 1,
    Disconnected = 2,
    Maintenance = 3,

    pub fn toString(self: NodeStatus) []const u8 {
        return switch (self) {
            .Active => "Active",
            .Inactive => "Inactive",
            .Disconnected => "Disconnected",
            .Maintenance => "Maintenance",
        };
    }
};

// 消息类型枚举
pub const MessageType = enum(u32) {
    StateSync = 0,
    Command = 1,
    Query = 2,
    Response = 3,
    Heartbeat = 4,
    Broadcast = 5,
    Registration = 6,
    Deregistration = 7,

    pub fn toString(self: MessageType) []const u8 {
        return switch (self) {
            .StateSync => "StateSync",
            .Command => "Command",
            .Query => "Query",
            .Response => "Response",
            .Heartbeat => "Heartbeat",
            .Broadcast => "Broadcast",
            .Registration => "Registration",
            .Deregistration => "Deregistration",
        };
    }
};

// 消息优先级枚举
pub const MessagePriority = enum(u32) {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,

    pub fn toString(self: MessagePriority) []const u8 {
        return switch (self) {
            .Low => "Low",
            .Normal => "Normal",
            .High => "High",
            .Critical => "Critical",
        };
    }
};

// Agent节点信息结构
pub const AgentNode = struct {
    node_id: []const u8,
    agent_id: u64,
    address: []const u8,
    port: u16,
    capabilities: [][]const u8,
    status: NodeStatus,
    last_heartbeat: i64,
    join_time: i64,
    version: []const u8,

    pub fn init(
        allocator: std.mem.Allocator,
        agent_id: u64,
        address: []const u8,
        port: u16,
        capabilities: []const []const u8,
    ) !AgentNode {
        const now = std.time.timestamp();

        return AgentNode{
            .node_id = try std.fmt.allocPrint(allocator, "node_{d}_{d}", .{ agent_id, now }),
            .agent_id = agent_id,
            .address = try allocator.dupe(u8, address),
            .port = port,
            .capabilities = try allocator.dupe([]const u8, capabilities),
            .status = .Active,
            .last_heartbeat = now,
            .join_time = now,
            .version = "1.0.0",
        };
    }

    pub fn deinit(self: *AgentNode, allocator: std.mem.Allocator) void {
        allocator.free(self.node_id);
        allocator.free(self.address);
        for (self.capabilities) |cap| {
            allocator.free(cap);
        }
        allocator.free(self.capabilities);
    }

    pub fn display(self: AgentNode) void {
        std.debug.print("Agent Node:\n", .{});
        std.debug.print("  ID: {s}\n", .{self.node_id});
        std.debug.print("  Agent ID: {d}\n", .{self.agent_id});
        std.debug.print("  Address: {s}:{d}\n", .{ self.address, self.port });
        std.debug.print("  Status: {s}\n", .{self.status.toString()});
        std.debug.print("  Capabilities: ", .{});
        for (self.capabilities, 0..) |cap, i| {
            if (i > 0) std.debug.print(", ", .{});
            std.debug.print("{s}", .{cap});
        }
        std.debug.print("\n", .{});
        std.debug.print("  Last Heartbeat: {d}\n", .{self.last_heartbeat});
        std.debug.print("  Version: {s}\n", .{self.version});
    }
};

// Agent消息结构
pub const AgentMessage = struct {
    message_id: []const u8,
    from_agent: u64,
    to_agent: ?u64,
    message_type: MessageType,
    payload: []const u8,
    timestamp: i64,
    ttl: u32,
    priority: MessagePriority,

    pub fn init(
        allocator: std.mem.Allocator,
        from_agent: u64,
        to_agent: ?u64,
        message_type: MessageType,
        payload: []const u8,
    ) !AgentMessage {
        const now = std.time.timestamp();
        const message_id = try std.fmt.allocPrint(allocator, "msg_{d}_{d}", .{ from_agent, now });

        return AgentMessage{
            .message_id = message_id,
            .from_agent = from_agent,
            .to_agent = to_agent,
            .message_type = message_type,
            .payload = try allocator.dupe(u8, payload),
            .timestamp = now,
            .ttl = 300, // 5 minutes default
            .priority = .Normal,
        };
    }

    pub fn deinit(self: *AgentMessage, allocator: std.mem.Allocator) void {
        allocator.free(self.message_id);
        allocator.free(self.payload);
    }

    pub fn withPriority(self: AgentMessage, priority: MessagePriority) AgentMessage {
        var msg = self;
        msg.priority = priority;
        return msg;
    }

    pub fn withTTL(self: AgentMessage, ttl: u32) AgentMessage {
        var msg = self;
        msg.ttl = ttl;
        return msg;
    }

    pub fn isExpired(self: AgentMessage) bool {
        const now = std.time.timestamp();
        return now - self.timestamp > self.ttl;
    }

    pub fn display(self: AgentMessage) void {
        std.debug.print("Agent Message:\n", .{});
        std.debug.print("  ID: {s}\n", .{self.message_id});
        std.debug.print("  From: {d}\n", .{self.from_agent});
        if (self.to_agent) |to| {
            std.debug.print("  To: {d}\n", .{to});
        } else {
            std.debug.print("  To: Broadcast\n", .{});
        }
        std.debug.print("  Type: {s}\n", .{self.message_type.toString()});
        std.debug.print("  Priority: {s}\n", .{self.priority.toString()});
        std.debug.print("  Payload Size: {d} bytes\n", .{self.payload.len});
        std.debug.print("  TTL: {d} seconds\n", .{self.ttl});
        std.debug.print("  Timestamp: {d}\n", .{self.timestamp});
    }
};

// 分布式Agent网络管理器
pub const AgentNetworkManager = struct {
    manager: *c.CAgentNetworkManager,
    allocator: std.mem.Allocator,
    local_agent_id: u64,

    pub fn init(
        allocator: std.mem.Allocator,
        agent_id: u64,
        address: []const u8,
        port: u16,
        capabilities: []const []const u8,
    ) !AgentNetworkManager {
        // 准备C字符串
        const c_address = try allocator.dupeZ(u8, address);
        defer allocator.free(c_address);

        // 准备capabilities数组
        var c_capabilities = try allocator.alloc([*c]const u8, capabilities.len);
        defer allocator.free(c_capabilities);

        var capability_strings = try allocator.alloc([]u8, capabilities.len);
        defer {
            for (capability_strings) |cap_str| {
                allocator.free(cap_str);
            }
            allocator.free(capability_strings);
        }

        for (capabilities, 0..) |cap, i| {
            capability_strings[i] = try allocator.dupeZ(u8, cap);
            c_capabilities[i] = capability_strings[i].ptr;
        }

        const manager = c.agent_network_manager_new(
            agent_id,
            c_address.ptr,
            port,
            c_capabilities.ptr,
            capabilities.len,
        );

        if (manager == null) {
            return error.InitializationFailed;
        }

        return AgentNetworkManager{
            .manager = manager.?,
            .allocator = allocator,
            .local_agent_id = agent_id,
        };
    }

    pub fn deinit(self: *AgentNetworkManager) void {
        c.agent_network_manager_free(self.manager);
    }

    pub fn joinNetwork(self: *AgentNetworkManager, bootstrap_nodes: [][]const u8) !void {
        // 准备bootstrap节点数组
        var c_bootstrap = try self.allocator.alloc([*c]const u8, bootstrap_nodes.len);
        defer self.allocator.free(c_bootstrap);

        var bootstrap_strings = try self.allocator.alloc([]u8, bootstrap_nodes.len);
        defer {
            for (bootstrap_strings) |node_str| {
                self.allocator.free(node_str);
            }
            self.allocator.free(bootstrap_strings);
        }

        for (bootstrap_nodes, 0..) |node, i| {
            bootstrap_strings[i] = try self.allocator.dupeZ(u8, node);
            c_bootstrap[i] = bootstrap_strings[i].ptr;
        }

        const result = c.agent_network_join_network(
            self.manager,
            c_bootstrap.ptr,
            bootstrap_nodes.len,
        );

        if (result != 0) {
            return error.JoinNetworkFailed;
        }
    }

    pub fn sendMessage(
        self: *AgentNetworkManager,
        to_agent: u64,
        message_type: MessageType,
        payload: []const u8,
    ) !void {
        const result = c.agent_network_send_message(
            self.manager,
            self.local_agent_id,
            to_agent,
            @intFromEnum(message_type),
            payload.ptr,
            payload.len,
        );

        if (result != 0) {
            return error.SendMessageFailed;
        }
    }

    pub fn broadcastMessage(self: *AgentNetworkManager, payload: []const u8) !void {
        const result = c.agent_network_broadcast_message(
            self.manager,
            payload.ptr,
            payload.len,
        );

        if (result != 0) {
            return error.BroadcastFailed;
        }
    }

    pub fn leaveNetwork(self: *AgentNetworkManager) !void {
        const result = c.agent_network_leave_network(self.manager);

        if (result != 0) {
            return error.LeaveNetworkFailed;
        }
    }

    pub fn getActiveNodesCount(self: *AgentNetworkManager) !u32 {
        const count = c.agent_network_get_active_nodes_count(self.manager);
        if (count < 0) {
            return error.GetActiveNodesCountFailed;
        }
        return @intCast(count);
    }

    pub fn findNodesByCapability(
        self: *AgentNetworkManager,
        capability: []const u8,
    ) ![]u64 {
        const c_capability = try self.allocator.dupeZ(u8, capability);
        defer self.allocator.free(c_capability);

        var nodes_ptr: [*]u64 = undefined;
        var nodes_count: usize = undefined;

        const result = c.agent_network_find_nodes_by_capability(
            self.manager,
            c_capability.ptr,
            @ptrCast(&nodes_ptr),
            &nodes_count,
        );

        if (result != 0) {
            return error.FindNodesFailed;
        }

        if (nodes_count == 0) {
            return &[_]u64{};
        }

        // 复制结果到Zig管理的内存
        const nodes = try self.allocator.alloc(u64, nodes_count);
        @memcpy(nodes, nodes_ptr[0..nodes_count]);

        // 释放C分配的内存
        c.agent_db_free_data(@ptrCast(nodes_ptr), nodes_count * @sizeOf(u64));

        return nodes;
    }
};
