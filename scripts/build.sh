#!/bin/bash

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found. Please install Rust."
        exit 1
    fi

    if ! command -v zig &> /dev/null; then
        log_error "Zig not found. Please install Zig 0.14.0 or later."
        exit 1
    fi

    log_success "All dependencies found"
}

# 构建 Rust 核心模块
build_rust_core() {
    log_info "Building Rust core module..."

    cd agent-db-core

    # 清理之前的构建
    cargo clean

    # 构建发布版本
    if cargo build --release; then
        log_success "Rust core module built successfully"
    else
        log_error "Failed to build Rust core module"
        exit 1
    fi

    # 生成 C 头文件
    log_info "Generating C headers..."
    if cargo run --bin generate_bindings; then
        log_success "C headers generated successfully"
    else
        log_warning "Failed to generate C headers, using existing ones"
    fi

    cd ..
}

# 构建 Zig API 模块
build_zig_api() {
    log_info "Building Zig API module..."

    cd agent-db-zig

    # 清理之前的构建
    zig build clean

    # 设置 Rust 库路径
    export RUST_LIB_PATH="../agent-db-core/target/release"

    # 构建 Zig 模块
    if zig build --rust-lib-path "$RUST_LIB_PATH"; then
        log_success "Zig API module built successfully"
    else
        log_error "Failed to build Zig API module"
        exit 1
    fi

    cd ..
}

# 运行测试
run_tests() {
    log_info "Running tests..."

    # Rust 测试
    log_info "Running Rust tests..."
    cd agent-db-core
    if cargo test; then
        log_success "Rust tests passed"
    else
        log_error "Rust tests failed"
        exit 1
    fi
    cd ..

    # Zig 测试
    log_info "Running Zig tests..."
    cd agent-db-zig
    if zig build test; then
        log_success "Zig tests passed"
    else
        log_error "Zig tests failed"
        exit 1
    fi
    cd ..

    log_success "All tests passed"
}

# 运行示例
run_examples() {
    log_info "Running examples..."

    cd agent-db-zig
    if zig build example; then
        log_success "Examples ran successfully"
    else
        log_warning "Examples failed to run"
    fi
    cd ..
}

# 生成文档
generate_docs() {
    log_info "Generating documentation..."

    # Rust 文档
    cd agent-db-core
    cargo doc --no-deps
    cd ..

    # 复制文档到统一位置
    mkdir -p docs/rust
    cp -r agent-db-core/target/doc/* docs/rust/

    log_success "Documentation generated"
}

# 主函数
main() {
    log_info "Starting AgentDB modular build process..."

    # 解析命令行参数
    SKIP_TESTS=false
    SKIP_EXAMPLES=false
    GENERATE_DOCS=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --skip-examples)
                SKIP_EXAMPLES=true
                shift
                ;;
            --docs)
                GENERATE_DOCS=true
                shift
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --skip-tests     Skip running tests"
                echo "  --skip-examples  Skip running examples"
                echo "  --docs          Generate documentation"
                echo "  -h, --help      Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    # 执行构建步骤
    check_dependencies
    build_rust_core
    build_zig_api

    if [ "$SKIP_TESTS" = false ]; then
        run_tests
    fi

    if [ "$SKIP_EXAMPLES" = false ]; then
        run_examples
    fi

    if [ "$GENERATE_DOCS" = true ]; then
        generate_docs
    fi

    log_success "Build process completed successfully!"
    log_info "Built artifacts:"
    log_info "  - Rust library: agent-db-core/target/release/libagent_db_core.so"
    log_info "  - C headers: agent-db-core/include/agent_state_db.h"
    log_info "  - Zig library: agent-db-zig/zig-out/lib/libagent_db_zig.a"
    log_info "  - Examples: agent-db-zig/zig-out/bin/"
}

# 错误处理
trap 'log_error "Build process failed at line $LINENO"' ERR

# 运行主函数
main "$@"
