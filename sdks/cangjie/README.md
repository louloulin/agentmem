# AgentMem 仓颉 SDK

AgentMem 仓颉 SDK 是一个企业级智能记忆管理平台的仓颉语言接口，提供类型安全、高性能的API，使仓颉开发者能够充分利用AgentMem的强大功能。

## 🚀 特性

- **类型安全**：利用仓颉强类型系统确保API安全性
- **高性能**：优化的FFI绑定，最小化性能开销
- **易用性**：符合仓颉语言习惯的API设计
- **完整功能**：支持AgentMem所有核心功能
- **企业级**：支持生产环境部署和大规模应用

## 📦 安装

将以下内容添加到您的 `cjpm.toml` 文件中：

```toml
[dependencies]
agentmem-cangjie-sdk = "1.0.0"
```

## 🔧 快速开始

```cangjie
import agentmem.api.{AgentMemClient, ClientConfig}
import agentmem.core.{Memory, MemoryType}

main() {
    // 创建客户端配置
    let config = ClientConfig("http://localhost:8080")
    config.apiKey = Some("your-api-key")
    
    // 创建客户端
    let client = AgentMemClient(config)
    
    try {
        // 初始化客户端
        client.initialize().getOrThrow()
        
        // 创建记忆
        let memory = Memory(
            "memory-1",
            "agent-123", 
            "我喜欢喝咖啡",
            MemoryType.Semantic
        )
        
        // 添加记忆
        let memoryId = client.addMemory(memory).getOrThrow()
        println("添加记忆成功，ID: ${memoryId}")
        
        // 搜索记忆
        let searchResults = client.searchMemories("咖啡", 5).getOrThrow()
        println("搜索到 ${searchResults.size} 条记忆")
        
        // 关闭客户端
        client.close()
        
    } catch (e: Exception) {
        println("错误: ${e}")
    }
}
```

## 📚 文档

- [API 参考](docs/api_reference.md)
- [用户指南](docs/user_guide.md)
- [示例代码](docs/examples.md)

## 🏗️ 架构

```
仓颉应用层
    ↓
AgentMem 仓颉 SDK (高级API)
    ↓
FFI 绑定层 (类型安全封装)
    ↓
AgentMem C FFI 接口
    ↓
AgentMem Rust 核心引擎
```

## 🧪 测试

```bash
# 运行所有测试
cjpm test

# 运行单元测试
cjpm test --unit

# 运行集成测试
cjpm test --integration
```

## 📄 许可证

MIT License

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

## 📞 支持

如有问题或建议，请提交 Issue 或联系我们。
