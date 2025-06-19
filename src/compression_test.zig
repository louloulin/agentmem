// 数据压缩系统测试
const std = @import("std");
const testing = std.testing;
const print = std.debug.print;

// 压缩类型枚举
const CompressionType = enum {
    None,
    LZ4,
    Zstd,
    Gzip,
    Snappy,
};

// 压缩指标结构体
const CompressionMetrics = struct {
    original_size: usize,
    compressed_size: usize,
    compression_ratio: f64,
    compression_time_ms: f64,
    decompression_time_ms: f64,
    algorithm: CompressionType,
};

// 模拟数据压缩器
const MockDataCompressor = struct {
    compression_type: CompressionType,
    compression_level: i32,
    min_size_threshold: usize,

    const Self = @This();

    pub fn init(compression_type: CompressionType, compression_level: i32, min_size_threshold: usize) Self {
        return Self{
            .compression_type = compression_type,
            .compression_level = compression_level,
            .min_size_threshold = min_size_threshold,
        };
    }

    pub fn compress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) !struct { []u8, CompressionMetrics } {
        if (data.len < self.min_size_threshold) {
            // 数据太小，不压缩
            const result = try allocator.dupe(u8, data);
            const metrics = CompressionMetrics{
                .original_size = data.len,
                .compressed_size = data.len,
                .compression_ratio = 1.0,
                .compression_time_ms = 0.0,
                .decompression_time_ms = 0.0,
                .algorithm = CompressionType.None,
            };
            return .{ result, metrics };
        }

        const start_time = std.time.milliTimestamp();

        const compressed_data = switch (self.compression_type) {
            .None => try allocator.dupe(u8, data),
            .LZ4 => try self.lz4Compress(allocator, data),
            .Zstd => try self.zstdCompress(allocator, data),
            .Gzip => try self.gzipCompress(allocator, data),
            .Snappy => try self.snappyCompress(allocator, data),
        };

        const compression_time = @as(f64, @floatFromInt(std.time.milliTimestamp() - start_time));
        const compression_ratio = @as(f64, @floatFromInt(data.len)) / @as(f64, @floatFromInt(compressed_data.len));

        const metrics = CompressionMetrics{
            .original_size = data.len,
            .compressed_size = compressed_data.len,
            .compression_ratio = compression_ratio,
            .compression_time_ms = compression_time,
            .decompression_time_ms = 0.0,
            .algorithm = self.compression_type,
        };

        return .{ compressed_data, metrics };
    }

    pub fn decompress(self: *const Self, allocator: std.mem.Allocator, compressed_data: []const u8, algorithm: CompressionType) ![]u8 {
        return switch (algorithm) {
            .None => try allocator.dupe(u8, compressed_data),
            .LZ4 => try self.lz4Decompress(allocator, compressed_data),
            .Zstd => try self.zstdDecompress(allocator, compressed_data),
            .Gzip => try self.gzipDecompress(allocator, compressed_data),
            .Snappy => try self.snappyDecompress(allocator, compressed_data),
        };
    }

    // 简化的LZ4压缩实现（RLE）
    fn lz4Compress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        var compressed = std.ArrayList(u8).init(allocator);
        defer compressed.deinit();

        var i: usize = 0;
        while (i < data.len) {
            const current_byte = data[i];
            var count: u8 = 1;

            // 计算连续相同字节的数量
            while (i + count < data.len and data[i + count] == current_byte and count < 255) {
                count += 1;
            }

            if (count > 3) {
                // 使用RLE编码
                try compressed.append(0xFF); // 标记字节
                try compressed.append(count);
                try compressed.append(current_byte);
            } else {
                // 直接存储
                var j: u8 = 0;
                while (j < count) : (j += 1) {
                    try compressed.append(current_byte);
                }
            }

            i += count;
        }

        return compressed.toOwnedSlice();
    }

    fn lz4Decompress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        var decompressed = std.ArrayList(u8).init(allocator);
        defer decompressed.deinit();

        var i: usize = 0;
        while (i < data.len) {
            if (data[i] == 0xFF and i + 2 < data.len) {
                // RLE解码
                const count = data[i + 1];
                const byte_value = data[i + 2];
                var j: u8 = 0;
                while (j < count) : (j += 1) {
                    try decompressed.append(byte_value);
                }
                i += 3;
            } else {
                try decompressed.append(data[i]);
                i += 1;
            }
        }

        return decompressed.toOwnedSlice();
    }

    // 简化的Zstd压缩实现
    fn zstdCompress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        // 简化的字典压缩
        var compressed = std.ArrayList(u8).init(allocator);
        defer compressed.deinit();

        var dictionary = std.HashMap([]const u8, u16, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator);
        defer dictionary.deinit();

        var dict_index: u16 = 0;
        var i: usize = 0;

        while (i < data.len) {
            var best_match_len: usize = 0;
            var best_match_index: u16 = 0;

            // 寻找最长匹配
            var len: usize = 1;
            while (len <= 8 and len <= data.len - i) : (len += 1) {
                const pattern = data[i .. i + len];
                if (dictionary.get(pattern)) |index| {
                    best_match_len = len;
                    best_match_index = index;
                }
            }

            if (best_match_len > 2) {
                // 使用字典引用
                try compressed.append(0xFE); // 字典标记
                try compressed.appendSlice(std.mem.asBytes(&best_match_index));
                try compressed.append(@intCast(best_match_len));
                i += best_match_len;
            } else {
                // 直接存储并添加到字典
                try compressed.append(data[i]);
                if (i + 1 < data.len) {
                    const pattern = try allocator.dupe(u8, data[i .. i + 2]);
                    try dictionary.put(pattern, dict_index);
                    dict_index += 1;
                }
                i += 1;
            }
        }

        return compressed.toOwnedSlice();
    }

    fn zstdDecompress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        var decompressed = std.ArrayList(u8).init(allocator);
        defer decompressed.deinit();

        var dictionary = std.ArrayList([]u8).init(allocator);
        defer {
            for (dictionary.items) |item| {
                allocator.free(item);
            }
            dictionary.deinit();
        }

        var i: usize = 0;
        while (i < data.len) {
            if (data[i] == 0xFE and i + 3 < data.len) {
                // 字典引用
                const index = std.mem.readInt(u16, data[i + 1 .. i + 3][0..2], .little);
                const length = data[i + 3];

                if (index < dictionary.items.len and dictionary.items[index].len >= length) {
                    const pattern = dictionary.items[index][0..length];
                    try decompressed.appendSlice(pattern);
                }
                i += 4;
            } else {
                try decompressed.append(data[i]);

                // 更新字典
                if (decompressed.items.len >= 2) {
                    const start = decompressed.items.len - 2;
                    const pattern = try allocator.dupe(u8, decompressed.items[start..]);
                    try dictionary.append(pattern);
                }
                i += 1;
            }
        }

        return decompressed.toOwnedSlice();
    }

    // 简化的Gzip和Snappy实现（使用基本的变长编码）
    fn gzipCompress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        var compressed = std.ArrayList(u8).init(allocator);
        defer compressed.deinit();

        // 添加标识符和长度
        try compressed.appendSlice("GZIP");
        try compressed.appendSlice(std.mem.asBytes(&@as(u32, @intCast(data.len))));

        // 简化的压缩：高频字节使用短编码
        var frequency = [_]u32{0} ** 256;
        for (data) |byte| {
            frequency[byte] += 1;
        }

        for (data) |byte| {
            if (frequency[byte] > data.len / 10) {
                // 高频字节使用短编码
                try compressed.append(0x80 | (byte >> 1));
            } else {
                // 低频字节使用原始编码
                try compressed.append(byte);
            }
        }

        return compressed.toOwnedSlice();
    }

    fn gzipDecompress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        if (data.len < 8 or !std.mem.eql(u8, data[0..4], "GZIP")) {
            return error.InvalidGzipData;
        }

        const original_len = std.mem.readInt(u32, data[4..8][0..4], .little);
        var decompressed = std.ArrayList(u8).init(allocator);
        defer decompressed.deinit();

        try decompressed.ensureTotalCapacity(original_len);

        for (data[8..]) |byte| {
            if (byte & 0x80 != 0) {
                // 解码高频字节
                try decompressed.append((byte & 0x7F) << 1);
            } else {
                try decompressed.append(byte);
            }
        }

        return decompressed.toOwnedSlice();
    }

    fn snappyCompress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        return try allocator.dupe(u8, data); // 简化实现，直接返回原数据
    }

    fn snappyDecompress(self: *const Self, allocator: std.mem.Allocator, data: []const u8) ![]u8 {
        _ = self;
        return try allocator.dupe(u8, data); // 简化实现，直接返回原数据
    }

    pub fn chooseBestAlgorithm(self: *const Self, data: []const u8) CompressionType {
        _ = self;
        const entropy = calculateEntropy(data);
        const repetition_ratio = calculateRepetitionRatio(data);

        if (entropy < 3.0) {
            // 低熵数据，适合RLE类压缩
            return CompressionType.LZ4;
        } else if (repetition_ratio > 0.3) {
            // 高重复率，适合字典压缩
            return CompressionType.Zstd;
        } else if (data.len > 1024) {
            // 大数据，使用通用压缩
            return CompressionType.Gzip;
        } else {
            // 小数据，使用快速压缩
            return CompressionType.Snappy;
        }
    }
};

fn calculateEntropy(data: []const u8) f64 {
    var frequency = [_]u32{0} ** 256;
    for (data) |byte| {
        frequency[byte] += 1;
    }

    const len = @as(f64, @floatFromInt(data.len));
    var entropy: f64 = 0.0;

    for (frequency) |freq| {
        if (freq > 0) {
            const p = @as(f64, @floatFromInt(freq)) / len;
            entropy -= p * std.math.log2(p);
        }
    }

    return entropy;
}

fn calculateRepetitionRatio(data: []const u8) f64 {
    if (data.len < 2) return 0.0;

    var repeated_bytes: usize = 0;
    for (data[1..], 1..) |byte, i| {
        if (byte == data[i - 1]) {
            repeated_bytes += 1;
        }
    }

    return @as(f64, @floatFromInt(repeated_bytes)) / @as(f64, @floatFromInt(data.len - 1));
}

// 测试基础压缩功能
test "Basic compression functionality" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const compressor = MockDataCompressor.init(CompressionType.LZ4, 1, 10);

    // 测试重复数据压缩
    const test_data = "AAAAAAAAAAAABBBBBBBBBBBBCCCCCCCCCCCC";
    const result = try compressor.compress(allocator, test_data);
    defer allocator.free(result[0]);

    const compressed_data = result[0];
    const metrics = result[1];

    try testing.expect(metrics.original_size == test_data.len);
    try testing.expect(metrics.compressed_size <= test_data.len);
    try testing.expect(metrics.compression_ratio >= 1.0);

    // 测试解压缩
    const decompressed = try compressor.decompress(allocator, compressed_data, CompressionType.LZ4);
    defer allocator.free(decompressed);

    try testing.expect(std.mem.eql(u8, test_data, decompressed));

    print("✓ 基础压缩功能测试通过\n", .{});
}

// 测试不同压缩算法
test "Different compression algorithms" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const test_data = "Hello World! This is a test string for compression. Hello World!";
    const algorithms = [_]CompressionType{ CompressionType.LZ4, CompressionType.Zstd, CompressionType.Gzip };

    for (algorithms) |algorithm| {
        const compressor = MockDataCompressor.init(algorithm, 1, 10);

        const result = try compressor.compress(allocator, test_data);
        defer allocator.free(result[0]);

        const compressed_data = result[0];
        const metrics = result[1];

        try testing.expect(metrics.algorithm == algorithm);

        const decompressed = try compressor.decompress(allocator, compressed_data, algorithm);
        defer allocator.free(decompressed);

        try testing.expect(std.mem.eql(u8, test_data, decompressed));
    }

    print("✓ 不同压缩算法测试通过\n", .{});
}

// 测试小数据不压缩
test "Small data threshold" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const compressor = MockDataCompressor.init(CompressionType.LZ4, 1, 20); // 20字节阈值

    const small_data = "Hello"; // 5字节，小于阈值
    const result = try compressor.compress(allocator, small_data);
    defer allocator.free(result[0]);

    const metrics = result[1];

    try testing.expect(metrics.algorithm == CompressionType.None);
    try testing.expect(metrics.compression_ratio == 1.0);
    try testing.expect(std.mem.eql(u8, small_data, result[0]));

    print("✓ 小数据阈值测试通过\n", .{});
}

// 测试算法选择
test "Algorithm selection" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const compressor = MockDataCompressor.init(CompressionType.LZ4, 1, 10);

    // 高重复数据
    const repetitive_data = "AAAAAAAAAAAABBBBBBBBBBBBCCCCCCCCCCCC";
    var algorithm = compressor.chooseBestAlgorithm(repetitive_data);
    try testing.expect(algorithm == CompressionType.LZ4);

    // 大数据
    const large_data = try allocator.alloc(u8, 2048);
    defer allocator.free(large_data);
    for (large_data, 0..) |*byte, i| {
        byte.* = @intCast(i % 256);
    }
    algorithm = compressor.chooseBestAlgorithm(large_data);
    try testing.expect(algorithm == CompressionType.Gzip);

    print("✓ 算法选择测试通过\n", .{});
}

// 测试压缩性能
test "Compression performance" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const compressor = MockDataCompressor.init(CompressionType.LZ4, 1, 10);

    // 创建测试数据
    const test_data = try allocator.alloc(u8, 1024);
    defer allocator.free(test_data);

    for (test_data, 0..) |*byte, i| {
        byte.* = @intCast((i / 4) % 256); // 创建一些重复模式
    }

    const start_time = std.time.milliTimestamp();

    const result = try compressor.compress(allocator, test_data);
    defer allocator.free(result[0]);

    const end_time = std.time.milliTimestamp();
    const duration = end_time - start_time;

    const metrics = result[1];

    try testing.expect(metrics.compression_time_ms >= 0);
    try testing.expect(duration < 100); // 应该在100ms内完成

    print("✓ 压缩性能测试通过 ({}ms)\n", .{duration});
}
