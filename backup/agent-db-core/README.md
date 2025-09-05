# AgentDB Core v0.2.0

🚀 **高性能智能体数据库核心库** - 专为 AI 智能体设计的现代化数据存储解决方案

## ✨ 特性

### 🧠 智能体状态管理
- **多类型状态支持**: 工作记忆、长期记忆、上下文、任务状态、关系数据
- **高效存储**: 基于文件系统的简化实现，支持异步操作
- **状态持久化**: 自动保存和加载智能体状态

### 💾 记忆管理系统
- **多种记忆类型**: 情节记忆、语义记忆、程序记忆、工作记忆
- **智能检索**: 基于重要性、内容相似度的记忆搜索
- **自动过期**: 支持记忆过期时间管理

### 📚 RAG (检索增强生成) 引擎
- **文档索引**: 自动文档分块和索引
- **混合搜索**: 结合文本搜索和语义搜索
- **上下文构建**: 智能构建 RAG 上下文

### 🔍 向量搜索引擎
- **多种相似度算法**: 余弦相似度、欧几里得距离、点积
- **高效索引**: 内存中向量索引和搜索
- **灵活配置**: 可配置向量维度和索引参数

### 🔒 安全管理
- **用户认证**: 基于令牌的身份验证
- **权限控制**: 细粒度权限管理
- **会话管理**: 安全的会话令牌管理

### 🌐 C FFI 接口
- **跨语言支持**: 完整的 C 兼容 FFI 接口
- **内存安全**: 安全的内存管理和错误处理
- **易于集成**: 简单的 C API 设计

## 🛠️ 技术栈

- **Rust**: 高性能系统编程语言
- **Arrow**: 高效的列式数据处理
- **LanceDB**: 现代向量数据库 (v0.21.2)
- **Tokio**: 异步运行时
- **Serde**: 序列化/反序列化

## 📦 安装

将以下内容添加到您的 `Cargo.toml`:

```toml
[dependencies]
agent-db-core = "0.2.0"
```

## 🚀 快速开始

```rust
use agent_db_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据库
    let db = create_database("./my_agent_db").await?
        .with_rag_engine().await?;
    
    // 创建智能体状态
    let state = AgentState::new(
        1001, // agent_id
        1,    // session_id
        StateType::WorkingMemory,
        b"Hello, AgentDB!".to_vec(),
    );
    
    // 保存状态
    db.save_agent_state(&state).await?;
    
    // 加载状态
    if let Some(loaded_state) = db.load_agent_state(1001).await? {
        println!("加载成功: {}", String::from_utf8_lossy(&loaded_state.data));
    }
    
    Ok(())
}
```

## 📖 示例

查看 `examples/` 目录中的完整示例：

```bash
cargo run --example basic_usage
```

## 🏗️ 架构

```
AgentDB Core
├── 🧠 Agent State Management
├── 💾 Memory Management  
├── 📚 RAG Engine
├── 🔍 Vector Search
├── 🔒 Security Manager
├── 📊 Performance Monitor
├── 🌐 Distributed Support
├── ⚡ Real-time Streaming
└── 🔌 C FFI Interface
```

## 🔧 配置

```rust
use agent_db_core::*;

let config = DatabaseConfig {
    db_path: "./data".to_string(),
    max_connections: 10,
    cache_size: 1024 * 1024, // 1MB
    enable_wal: true,
    sync_mode: "NORMAL".to_string(),
};

let db = AgentDatabase::new(config).await?;
```

## 🧪 测试

运行所有测试：

```bash
cargo test
```

运行特定测试：

```bash
cargo test --lib
```

## 📈 性能

- **高并发**: 支持多线程异步操作
- **内存优化**: 高效的内存使用和缓存策略
- **快速检索**: 优化的索引和搜索算法
- **可扩展**: 模块化设计，易于扩展

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [文档](https://docs.rs/agent-db-core)
- [示例](./examples/)
- [变更日志](CHANGELOG.md)

## 🆕 版本 0.2.0 更新

- ✅ 完全重构的核心架构
- ✅ 最新的 Arrow 55.2 和 LanceDB 0.21.2 支持
- ✅ 简化但功能完整的实现
- ✅ 完整的 C FFI 接口
- ✅ 改进的错误处理和类型安全
- ✅ 全面的测试覆盖

---

**AgentDB Core** - 为下一代 AI 智能体构建的数据库核心 🤖✨
