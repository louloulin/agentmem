const std = @import("std");
const testing = std.testing;

test "Minimal Test - No FFI" {
    std.debug.print("Running minimal test without FFI...\n", .{});
    
    // 只测试基本的Zig功能
    const x = 42;
    const y = 24;
    const result = x + y;
    
    std.debug.print("Basic arithmetic: {} + {} = {}\n", .{ x, y, result });
    
    try testing.expect(result == 66);
    std.debug.print("Minimal test passed!\n", .{});
}

test "Memory Allocation Test" {
    std.debug.print("Testing memory allocation...\n", .{});
    
    var allocator = std.testing.allocator;
    const data = try allocator.alloc(u8, 100);
    defer allocator.free(data);
    
    // 填充数据
    for (data, 0..) |*byte, i| {
        byte.* = @intCast(i % 256);
    }
    
    std.debug.print("Memory allocation test passed!\n", .{});
}

test "String Operations Test" {
    std.debug.print("Testing string operations...\n", .{});
    
    const test_string = "Hello, Zig!";
    const expected_len = 11;
    
    try testing.expect(test_string.len == expected_len);
    std.debug.print("String test passed: '{}' has length {}\n", .{ test_string, test_string.len });
}
