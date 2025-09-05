// 最小化测试文件，用于诊断问题
const std = @import("std");
const testing = std.testing;

test "Basic Zig Test" {
    std.debug.print("Basic Zig test is running...\n", .{});
    try testing.expect(true);
    std.debug.print("Basic Zig test passed!\n", .{});
}

test "Memory Allocation Test" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    std.debug.print("Memory allocation test is running...\n", .{});
    
    const data = try allocator.alloc(u8, 100);
    defer allocator.free(data);
    
    try testing.expect(data.len == 100);
    std.debug.print("Memory allocation test passed!\n", .{});
}

test "String Operations Test" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    std.debug.print("String operations test is running...\n", .{});
    
    const test_string = "Hello, Zig!";
    const copied_string = try allocator.dupe(u8, test_string);
    defer allocator.free(copied_string);
    
    try testing.expectEqualStrings(test_string, copied_string);
    std.debug.print("String operations test passed!\n", .{});
}
