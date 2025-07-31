.PHONY: all clean test rust-core zig-api install docs

# 默认目标
all: rust-core zig-api

# 构建 Rust 核心模块
rust-core:
	@echo "Building Rust core module..."
	cd agent-db-core && cargo build --release
	@echo "Generating C headers..."
	cd agent-db-core && cargo run --bin generate_bindings

# 构建 Zig API 模块
zig-api: rust-core
	@echo "Building Zig API module..."
	cd agent-db-zig && zig build

# 运行所有测试
test: test-rust test-zig test-integration

test-rust:
	@echo "Running Rust tests..."
	cd agent-db-core && cargo test

test-zig:
	@echo "Running Zig tests..."
	cd agent-db-zig && zig build test

test-integration:
	@echo "Running integration tests..."
	cd agent-db-zig && zig build example

# 安装到系统
install: all
	@echo "Installing libraries..."
	sudo cp agent-db-core/target/release/libagent_db_core.so /usr/local/lib/
	sudo cp agent-db-core/include/agent_state_db.h /usr/local/include/
	sudo ldconfig

# 生成文档
docs:
	@echo "Generating documentation..."
	cd agent-db-core && cargo doc --no-deps
	cd agent-db-zig && zig build docs

# 清理构建产物
clean:
	@echo "Cleaning build artifacts..."
	cd agent-db-core && cargo clean
	cd agent-db-zig && zig build clean

# 发布准备
release: clean all test docs
	@echo "Preparing release..."
	@echo "All modules built and tested successfully!"

# 开发环境设置
dev-setup:
	@echo "Setting up development environment..."
	rustup update
	# 安装 Zig 如果需要
	@echo "Development environment ready!"
