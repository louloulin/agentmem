.PHONY: all clean test rust-core zig-api install docs help bench

# 默认目标
all: rust-core zig-api

# 帮助信息
help:
	@echo "AgentDB 模块化构建系统"
	@echo ""
	@echo "可用目标:"
	@echo "  all          - 构建所有模块 (默认)"
	@echo "  rust-core    - 构建 Rust 核心模块"
	@echo "  zig-api      - 构建 Zig API 模块"
	@echo "  test         - 运行所有测试"
	@echo "  test-rust    - 运行 Rust 测试"
	@echo "  test-zig     - 运行 Zig 测试"
	@echo "  bench        - 运行性能基准测试"
	@echo "  clean        - 清理构建产物"
	@echo "  install      - 安装到系统"
	@echo "  docs         - 生成文档"
	@echo "  help         - 显示此帮助信息"

# 构建 Rust 核心模块
rust-core:
	@echo "🦀 构建 Rust 核心模块..."
	cd agent-db-core && cargo build --release
	@echo "✅ Rust 核心模块构建完成"

# 构建 Zig API 模块
zig-api: rust-core
	@echo "⚡ 构建 Zig API 模块..."
	cd agent-db-zig && zig build
	@echo "✅ Zig API 模块构建完成"

# 运行所有测试
test: test-rust test-zig test-integration

test-rust:
	@echo "🧪 运行 Rust 测试..."
	cd agent-db-core && cargo test

test-zig:
	@echo "🧪 运行 Zig 测试..."
	cd agent-db-zig && zig build test

test-integration:
	@echo "🧪 运行集成测试..."
	cd agent-db-zig && zig build example

# 性能基准测试
bench:
	@echo "📊 运行性能基准测试..."
	cd agent-db-core && cargo bench

# 安装到系统
install: all
	@echo "📦 安装库文件..."
	sudo cp agent-db-core/target/release/libagent_db_core.* /usr/local/lib/ 2>/dev/null || true
	sudo cp agent-db-core/include/agent_db_core.h /usr/local/include/ 2>/dev/null || true
	sudo ldconfig 2>/dev/null || true
	@echo "✅ 安装完成"

# 生成文档
docs:
	@echo "📚 生成文档..."
	cd agent-db-core && cargo doc --no-deps
	mkdir -p docs/rust
	cp -r agent-db-core/target/doc/* docs/rust/ 2>/dev/null || true
	@echo "✅ 文档生成完成"

# 清理构建产物
clean:
	@echo "🧹 清理构建产物..."
	cd agent-db-core && cargo clean
	cd agent-db-zig && zig build clean
	rm -rf docs/rust
	@echo "✅ 清理完成"

# 发布准备
release: clean all test docs
	@echo "🚀 准备发布..."
	@echo "✅ 所有模块构建和测试成功!"

# 开发环境设置
dev-setup:
	@echo "🛠️ 设置开发环境..."
	rustup update
	@echo "✅ 开发环境就绪!"
