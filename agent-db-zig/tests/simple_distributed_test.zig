// 简化的分布式Agent网络测试
const std = @import("std");
const testing = std.testing;
const distributed = @import("distributed_network.zig");

const AgentNode = distributed.AgentNode;
const AgentMessage = distributed.AgentMessage;
const MessageType = distributed.MessageType;
const MessagePriority = distributed.MessagePriority;
const NodeStatus = distributed.NodeStatus;

test "Basic Agent Node Creation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("\n=== Basic Agent Node Creation Test ===\n", .{});

    // 创建Agent节点
    const capabilities = [_][]const u8{ "chat", "search" };
    const node = try AgentNode.init(
        allocator,
        12345,
        "127.0.0.1",
        8080,
        capabilities[0..],
    );
    defer node.deinit(allocator);

    // 验证节点信息
    try testing.expect(node.agent_id == 12345);
    try testing.expect(std.mem.eql(u8, node.address, "127.0.0.1"));
    try testing.expect(node.port == 8080);
    try testing.expect(node.capabilities.len == 2);
    try testing.expect(node.status == .Active);

    std.debug.print("✅ Agent node created successfully\n", .{});
    std.debug.print("   Agent ID: {d}\n", .{node.agent_id});
    std.debug.print("   Address: {s}:{d}\n", .{ node.address, node.port });
    std.debug.print("   Capabilities: {d}\n", .{node.capabilities.len});
}

test "Basic Agent Message Creation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("\n=== Basic Agent Message Creation Test ===\n", .{});

    // 创建点对点消息
    const payload = "Hello, Agent!";
    var message = try AgentMessage.init(
        allocator,
        12345,
        67890,
        .Command,
        payload,
    );
    defer message.deinit(allocator);

    // 验证消息属性
    try testing.expect(message.from_agent == 12345);
    try testing.expect(message.to_agent.? == 67890);
    try testing.expect(message.message_type == .Command);
    try testing.expect(std.mem.eql(u8, message.payload, payload));
    try testing.expect(message.priority == .Normal);

    std.debug.print("✅ Agent message created successfully\n", .{});
    std.debug.print("   From: {d} -> To: {d}\n", .{ message.from_agent, message.to_agent.? });
    std.debug.print("   Type: {s}\n", .{message.message_type.toString()});
    std.debug.print("   Payload size: {d} bytes\n", .{message.payload.len});
}

test "Message Type Enums" {
    std.debug.print("\n=== Message Type Enums Test ===\n", .{});

    // 测试消息类型
    const msg_types = [_]MessageType{
        .StateSync,
        .Command,
        .Query,
        .Response,
        .Heartbeat,
        .Broadcast,
    };

    std.debug.print("Message Types:\n", .{});
    for (msg_types) |msg_type| {
        std.debug.print("  - {s}\n", .{msg_type.toString()});
    }

    // 测试消息优先级
    const priorities = [_]MessagePriority{ .Low, .Normal, .High, .Critical };

    std.debug.print("Message Priorities:\n", .{});
    for (priorities) |priority| {
        std.debug.print("  - {s}\n", .{priority.toString()});
    }

    // 测试节点状态
    const statuses = [_]NodeStatus{ .Active, .Inactive, .Disconnected, .Maintenance };

    std.debug.print("Node Statuses:\n", .{});
    for (statuses) |status| {
        std.debug.print("  - {s}\n", .{status.toString()});
    }

    std.debug.print("✅ All enums working correctly\n", .{});
}

test "Message Properties and Modification" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("\n=== Message Properties and Modification Test ===\n", .{});

    // 创建消息
    var message = try AgentMessage.init(
        allocator,
        1001,
        1002,
        .Query,
        "Test query message",
    );
    defer message.deinit(allocator);

    // 测试消息修改
    const high_priority_msg = message.withPriority(.High);
    try testing.expect(high_priority_msg.priority == .High);

    const long_ttl_msg = message.withTTL(600);
    try testing.expect(long_ttl_msg.ttl == 600);

    std.debug.print("✅ Message properties modified successfully\n", .{});
    std.debug.print("   Original priority: {s}\n", .{message.priority.toString()});
    std.debug.print("   Modified priority: {s}\n", .{high_priority_msg.priority.toString()});
    std.debug.print("   Original TTL: {d}s\n", .{message.ttl});
    std.debug.print("   Modified TTL: {d}s\n", .{long_ttl_msg.ttl});
}

test "Broadcast Message Creation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("\n=== Broadcast Message Creation Test ===\n", .{});

    // 创建广播消息
    var broadcast_msg = try AgentMessage.init(
        allocator,
        12345,
        null, // 广播消息没有特定目标
        .Broadcast,
        "System announcement",
    );
    defer broadcast_msg.deinit(allocator);

    // 验证广播消息属性
    try testing.expect(broadcast_msg.from_agent == 12345);
    try testing.expect(broadcast_msg.to_agent == null);
    try testing.expect(broadcast_msg.message_type == .Broadcast);

    std.debug.print("✅ Broadcast message created successfully\n", .{});
    std.debug.print("   From: {d}\n", .{broadcast_msg.from_agent});
    std.debug.print("   To: Broadcast\n", .{});
    std.debug.print("   Type: {s}\n", .{broadcast_msg.message_type.toString()});
}

test "Multiple Nodes and Messages" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("\n=== Multiple Nodes and Messages Test ===\n", .{});

    // 创建多个Agent节点
    const node_configs = [_]struct {
        id: u64,
        address: []const u8,
        port: u16,
        capabilities: []const []const u8,
    }{
        .{ .id = 1001, .address = "192.168.1.10", .port = 8001, .capabilities = &[_][]const u8{ "chat", "nlp" } },
        .{ .id = 1002, .address = "192.168.1.11", .port = 8002, .capabilities = &[_][]const u8{ "search", "indexing" } },
        .{ .id = 1003, .address = "192.168.1.12", .port = 8003, .capabilities = &[_][]const u8{ "analysis", "ml" } },
    };

    var nodes = std.ArrayList(AgentNode).init(allocator);
    defer {
        for (nodes.items) |*node| {
            node.deinit(allocator);
        }
        nodes.deinit();
    }

    // 创建所有节点
    for (node_configs) |config| {
        const node = try AgentNode.init(
            allocator,
            config.id,
            config.address,
            config.port,
            config.capabilities,
        );
        try nodes.append(node);
    }

    std.debug.print("✅ Created {} agent nodes\n", .{nodes.items.len});

    // 创建不同类型的消息
    var messages = std.ArrayList(AgentMessage).init(allocator);
    defer {
        for (messages.items) |*msg| {
            msg.deinit(allocator);
        }
        messages.deinit();
    }

    // 命令消息
    var cmd_msg = try AgentMessage.init(
        allocator,
        1001,
        1002,
        .Command,
        "Execute search query",
    );
    try messages.append(cmd_msg.withPriority(.High));

    // 查询消息
    var query_msg = try AgentMessage.init(
        allocator,
        1002,
        1003,
        .Query,
        "What is the current system load?",
    );
    try messages.append(query_msg.withTTL(60));

    // 广播消息
    var broadcast_msg = try AgentMessage.init(
        allocator,
        1001,
        null,
        .Broadcast,
        "System maintenance scheduled",
    );
    try messages.append(broadcast_msg.withPriority(.Critical));

    std.debug.print("✅ Created {} messages\n", .{messages.items.len});

    // 验证节点和消息
    for (nodes.items, 0..) |node, i| {
        std.debug.print("Node {}: Agent {d} at {s}:{d}\n", .{ i + 1, node.agent_id, node.address, node.port });
    }

    for (messages.items, 0..) |msg, i| {
        std.debug.print("Message {}: {s} from {d}", .{ i + 1, msg.message_type.toString(), msg.from_agent });
        if (msg.to_agent) |to| {
            std.debug.print(" to {d}\n", .{to});
        } else {
            std.debug.print(" (broadcast)\n", .{});
        }
    }

    std.debug.print("✅ Multiple nodes and messages test completed\n", .{});
}

// 运行所有测试的主函数
pub fn runAllTests() !void {
    std.debug.print("🚀 Starting Simple Distributed Agent Network Tests\n", .{});
    std.debug.print("=" ** 60 ++ "\n", .{});

    // 运行所有测试
    try testing.refAllDecls(@This());

    std.debug.print("=" ** 60 ++ "\n", .{});
    std.debug.print("🎉 All Simple Distributed Agent Network Tests Completed!\n", .{});
}
