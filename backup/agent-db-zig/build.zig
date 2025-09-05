const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // 检查 Rust 核心库路径
    const rust_lib_path = b.option([]const u8, "rust-lib-path", "Path to Rust core library") orelse "../agent-db-core/target/release";

    // 构建Rust核心库
    const cargo_build = b.addSystemCommand(&[_][]const u8{ "cargo", "build", "--release" });
    cargo_build.setCwd(b.path("../agent-db-core"));

    // 创建 Zig API 库
    const agent_db_lib = b.addStaticLibrary(.{
        .name = "agent_db_zig",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // 添加 C 头文件路径
    agent_db_lib.addIncludePath(b.path("../agent-db-core/include"));

    // 链接 Rust 核心库
    agent_db_lib.addLibraryPath(b.path(rust_lib_path));
    agent_db_lib.linkSystemLibrary("agent_db_core");
    agent_db_lib.linkLibC();

    // 平台特定链接
    if (target.result.os.tag == .windows) {
        agent_db_lib.linkSystemLibrary("ws2_32");
        agent_db_lib.linkSystemLibrary("advapi32");
        agent_db_lib.linkSystemLibrary("userenv");
        agent_db_lib.linkSystemLibrary("ntdll");
        agent_db_lib.linkSystemLibrary("bcrypt");
    }

    // 确保Rust库先构建
    agent_db_lib.step.dependOn(&cargo_build.step);

    b.installArtifact(agent_db_lib);

    // 创建测试
    const tests = b.addTest(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    tests.addIncludePath(b.path("../agent-db-core/include"));
    tests.addLibraryPath(b.path(rust_lib_path));
    tests.linkSystemLibrary("agent_db_core");
    tests.linkLibC();

    // 平台特定链接 (测试)
    if (target.result.os.tag == .windows) {
        tests.linkSystemLibrary("ws2_32");
        tests.linkSystemLibrary("advapi32");
        tests.linkSystemLibrary("userenv");
        tests.linkSystemLibrary("ntdll");
        tests.linkSystemLibrary("bcrypt");
    }

    tests.step.dependOn(&cargo_build.step);

    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_tests.step);

    // 创建示例
    const example = b.addExecutable(.{
        .name = "agent_db_example",
        .root_source_file = b.path("examples/basic_usage.zig"),
        .target = target,
        .optimize = optimize,
    });

    // 添加模块依赖
    const agent_api_module = b.createModule(.{
        .root_source_file = b.path("src/agent_api.zig"),
    });
    agent_api_module.addIncludePath(b.path("../agent-db-core/include"));
    example.root_module.addImport("agent_api", agent_api_module);

    example.addIncludePath(b.path("../agent-db-core/include"));
    example.addLibraryPath(b.path(rust_lib_path));
    example.linkSystemLibrary("agent_db_core");
    example.linkLibC();

    // 平台特定链接 (示例)
    if (target.result.os.tag == .windows) {
        example.linkSystemLibrary("ws2_32");
        example.linkSystemLibrary("advapi32");
        example.linkSystemLibrary("userenv");
        example.linkSystemLibrary("ntdll");
        example.linkSystemLibrary("bcrypt");
    }

    example.step.dependOn(&cargo_build.step);

    b.installArtifact(example);

    const run_example = b.addRunArtifact(example);
    const example_step = b.step("example", "Run example");
    example_step.dependOn(&run_example.step);

    // 添加清理步骤
    const clean_step = b.step("clean", "Clean build artifacts");
    const clean_cmd = b.addSystemCommand(&[_][]const u8{ "rm", "-rf", "zig-out", ".zig-cache" });
    clean_step.dependOn(&clean_cmd.step);
}
