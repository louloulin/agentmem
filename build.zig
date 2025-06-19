const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // 构建Rust库
    const cargo_build = b.addSystemCommand(&[_][]const u8{ "cargo", "build", "--release" });

    // 生成C头文件
    const generate_bindings = b.addSystemCommand(&[_][]const u8{ "cargo", "run", "--bin", "generate_bindings" });
    generate_bindings.step.dependOn(&cargo_build.step);

    // 创建Agent状态数据库库
    const agent_db_lib = b.addStaticLibrary(.{
        .name = "agent_db_zig",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // 添加C头文件路径
    agent_db_lib.addIncludePath(b.path("include"));

    // 链接Rust库
    agent_db_lib.addLibraryPath(b.path("target/release"));
    agent_db_lib.linkSystemLibrary("agent_state_db_rust");
    agent_db_lib.linkLibC();

    // 在Windows上需要链接额外的系统库
    if (target.result.os.tag == .windows) {
        agent_db_lib.linkSystemLibrary("ws2_32");
        agent_db_lib.linkSystemLibrary("advapi32");
        agent_db_lib.linkSystemLibrary("userenv");
        agent_db_lib.linkSystemLibrary("ntdll");
        agent_db_lib.linkSystemLibrary("bcrypt");
    }

    // 确保Rust库先构建
    agent_db_lib.step.dependOn(&generate_bindings.step);

    b.installArtifact(agent_db_lib);

    // 创建简单测试（不依赖Rust库）
    const simple_tests = b.addTest(.{
        .root_source_file = b.path("src/simple_test.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_simple_tests = b.addRunArtifact(simple_tests);
    const simple_test_step = b.step("test-simple", "Run simple unit tests");
    simple_test_step.dependOn(&run_simple_tests.step);

    // 创建最小测试（基础Zig功能）
    const minimal_tests = b.addTest(.{
        .root_source_file = b.path("src/minimal_test.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_minimal_tests = b.addRunArtifact(minimal_tests);
    const minimal_test_step = b.step("test-minimal", "Run minimal unit tests");
    minimal_test_step.dependOn(&run_minimal_tests.step);

    // 创建单个测试（诊断用）
    const single_tests = b.addTest(.{
        .root_source_file = b.path("src/single_test.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_single_tests = b.addRunArtifact(single_tests);
    const single_test_step = b.step("test-single", "Run single diagnostic test");
    single_test_step.dependOn(&run_single_tests.step);

    // 创建完整测试（使用安全的Zig测试）
    const tests = b.addTest(.{
        .root_source_file = b.path("src/safe_test.zig"),
        .target = target,
        .optimize = optimize,
    });

    // 添加头文件路径
    tests.addIncludePath(b.path("include"));

    // 链接Rust库
    tests.addLibraryPath(b.path("target/release"));
    tests.linkSystemLibrary("agent_state_db_rust");
    tests.linkLibC();

    // 在Windows上需要链接额外的系统库
    if (target.result.os.tag == .windows) {
        tests.linkSystemLibrary("ws2_32");
        tests.linkSystemLibrary("advapi32");
        tests.linkSystemLibrary("userenv");
        tests.linkSystemLibrary("ntdll");
        tests.linkSystemLibrary("bcrypt");
    }

    // 确保Rust库先构建
    tests.step.dependOn(&generate_bindings.step);

    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_tests.step);

    // 创建示例程序
    const example = b.addExecutable(.{
        .name = "agent_db_example",
        .root_source_file = b.path("examples/zig_api_demo.zig"),
        .target = target,
        .optimize = optimize,
    });

    // 添加模块依赖
    const agent_db_module = b.addModule("agent_db", .{
        .root_source_file = b.path("src/main.zig"),
    });

    const agent_state_module = b.addModule("agent_state", .{
        .root_source_file = b.path("src/agent_state.zig"),
    });

    const distributed_network_module = b.addModule("distributed_network", .{
        .root_source_file = b.path("src/distributed_network.zig"),
    });

    const realtime_stream_module = b.addModule("realtime_stream", .{
        .root_source_file = b.path("src/realtime_stream.zig"),
    });

    // 为模块添加include路径
    agent_db_module.addIncludePath(b.path("include"));
    agent_state_module.addIncludePath(b.path("include"));
    distributed_network_module.addIncludePath(b.path("include"));
    realtime_stream_module.addIncludePath(b.path("include"));

    example.root_module.addImport("agent_db", agent_db_module);

    // 添加头文件路径
    example.addIncludePath(b.path("include"));

    // 链接Rust库
    example.addLibraryPath(b.path("target/release"));
    example.linkSystemLibrary("agent_state_db_rust");
    example.linkLibC();

    // 在Windows上需要链接额外的系统库
    if (target.result.os.tag == .windows) {
        example.linkSystemLibrary("ws2_32");
        example.linkSystemLibrary("advapi32");
        example.linkSystemLibrary("userenv");
        example.linkSystemLibrary("ntdll");
        example.linkSystemLibrary("bcrypt");
    }

    // 确保Rust库先构建
    example.step.dependOn(&generate_bindings.step);
    b.installArtifact(example);

    const run_example = b.addRunArtifact(example);
    const example_step = b.step("example", "Run example");
    example_step.dependOn(&run_example.step);

    // 创建基准测试
    const benchmark = b.addExecutable(.{
        .name = "agent_db_benchmark",
        .root_source_file = b.path("benchmarks/performance.zig"),
        .target = target,
        .optimize = optimize,
    });

    // 添加模块依赖
    benchmark.root_module.addImport("agent_db.zig", agent_db_module);
    benchmark.root_module.addImport("agent_state.zig", agent_state_module);
    benchmark.root_module.addImport("distributed_network.zig", distributed_network_module);

    // 添加头文件路径
    benchmark.addIncludePath(b.path("include"));

    // 链接Rust库
    benchmark.addLibraryPath(b.path("target/release"));
    benchmark.linkSystemLibrary("agent_state_db_rust");
    benchmark.linkLibC();

    // 在Windows上需要链接额外的系统库
    if (target.result.os.tag == .windows) {
        benchmark.linkSystemLibrary("ws2_32");
        benchmark.linkSystemLibrary("advapi32");
        benchmark.linkSystemLibrary("userenv");
        benchmark.linkSystemLibrary("ntdll");
        benchmark.linkSystemLibrary("bcrypt");
    }

    // 确保Rust库先构建
    benchmark.step.dependOn(&generate_bindings.step);
    b.installArtifact(benchmark);

    const run_benchmark = b.addRunArtifact(benchmark);
    const benchmark_step = b.step("benchmark", "Run performance benchmarks");
    benchmark_step.dependOn(&run_benchmark.step);

    // 创建分布式网络测试
    const distributed_test = b.addTest(.{
        .name = "distributed_network_test",
        .root_source_file = b.path("src/simple_distributed_test.zig"),
        .target = target,
        .optimize = optimize,
    });

    distributed_test.addIncludePath(b.path("include"));
    distributed_test.addLibraryPath(b.path("target/release"));
    distributed_test.linkSystemLibrary("agent_state_db_rust");
    distributed_test.linkLibC();

    // 在Windows上需要链接额外的系统库
    if (target.result.os.tag == .windows) {
        distributed_test.linkSystemLibrary("ws2_32");
        distributed_test.linkSystemLibrary("advapi32");
        distributed_test.linkSystemLibrary("userenv");
        distributed_test.linkSystemLibrary("ntdll");
        distributed_test.linkSystemLibrary("bcrypt");
    }

    // 确保Rust库先构建
    distributed_test.step.dependOn(&generate_bindings.step);

    const run_distributed_test = b.addRunArtifact(distributed_test);
    const distributed_test_step = b.step("test-distributed", "Run distributed network tests");
    distributed_test_step.dependOn(&run_distributed_test.step);

    // 创建实时数据流处理测试
    const realtime_test = b.addTest(.{
        .name = "realtime_stream_test",
        .root_source_file = b.path("src/simple_realtime_test.zig"),
        .target = target,
        .optimize = optimize,
    });

    realtime_test.addIncludePath(b.path("include"));
    realtime_test.addLibraryPath(b.path("target/release"));
    realtime_test.linkSystemLibrary("agent_state_db_rust");
    realtime_test.linkLibC();

    // 在Windows上需要链接额外的系统库
    if (target.result.os.tag == .windows) {
        realtime_test.linkSystemLibrary("ws2_32");
        realtime_test.linkSystemLibrary("advapi32");
        realtime_test.linkSystemLibrary("userenv");
        realtime_test.linkSystemLibrary("ntdll");
        realtime_test.linkSystemLibrary("bcrypt");
    }

    // 确保Rust库先构建
    realtime_test.step.dependOn(&generate_bindings.step);

    const run_realtime_test = b.addRunArtifact(realtime_test);
    const realtime_test_step = b.step("test-realtime", "Run real-time stream processing tests");
    realtime_test_step.dependOn(&run_realtime_test.step);

    // 创建综合性能测试
    const comprehensive_test = b.addTest(.{
        .root_source_file = b.path("src/comprehensive_test.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_comprehensive_test = b.addRunArtifact(comprehensive_test);
    const comprehensive_test_step = b.step("test-comprehensive", "Run comprehensive performance tests");
    comprehensive_test_step.dependOn(&run_comprehensive_test.step);

    // 创建所有测试的总目标
    const all_tests_step = b.step("test-all", "Run all test suites");
    all_tests_step.dependOn(minimal_test_step);
    all_tests_step.dependOn(single_test_step);
    all_tests_step.dependOn(test_step);
    all_tests_step.dependOn(distributed_test_step);
    all_tests_step.dependOn(realtime_test_step);
    all_tests_step.dependOn(comprehensive_test_step);
}
