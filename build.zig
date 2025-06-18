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

    // 创建完整测试（依赖Rust库）
    const tests = b.addTest(.{
        .root_source_file = b.path("src/test_zig_api.zig"),
        .target = target,
        .optimize = optimize,
    });

    tests.linkLibrary(agent_db_lib);

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

    example.linkLibrary(agent_db_lib);
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

    benchmark.linkLibrary(agent_db_lib);
    b.installArtifact(benchmark);

    const run_benchmark = b.addRunArtifact(benchmark);
    const benchmark_step = b.step("benchmark", "Run performance benchmarks");
    benchmark_step.dependOn(&run_benchmark.step);
}
