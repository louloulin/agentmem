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

    // 创建完整测试（依赖Rust库）
    const tests = b.addTest(.{
        .root_source_file = b.path("src/test_zig_api.zig"),
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

    // 为模块添加include路径
    agent_db_module.addIncludePath(b.path("include"));
    agent_state_module.addIncludePath(b.path("include"));

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
}
