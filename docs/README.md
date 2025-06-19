# AgentDB Documentation

## 🌍 Language Selection / 语言选择

Choose your preferred language for documentation:

选择您偏好的文档语言：

### 📖 Available Languages / 可用语言

| Language | 语言 | Documentation | 文档链接 |
|----------|------|---------------|----------|
| **English** | **英文** | [English Documentation](en/README.md) | [英文文档](en/README.md) |
| **中文** | **Chinese** | [中文文档](zh/README.md) | [Chinese Documentation](zh/README.md) |

## 🚀 Quick Navigation / 快速导航

### English Documentation
- **[Getting Started](en/getting-started.md)** - Quick start guide
- **[Architecture](en/architecture.md)** - System architecture design
- **[API Reference](en/api.md)** - Complete API documentation

### 中文文档
- **[快速开始](zh/getting-started.md)** - 快速入门指南
- **[架构设计](zh/architecture.md)** - 系统架构设计
- **[API 参考](zh/api.md)** - 完整 API 文档

## 📊 Documentation Overview / 文档概览

### 🎯 What is AgentDB? / 什么是 AgentDB？

**English**: AgentDB is a high-performance AI Agent state database built on a hybrid Rust+Zig+LanceDB architecture, designed for large-scale AI Agent deployments with millisecond response times and enterprise-grade reliability.

**中文**: AgentDB 是一个基于 Rust+Zig+LanceDB 混合架构的高性能 AI Agent 状态数据库，专为大规模 AI Agent 部署而设计，具有毫秒级响应时间和企业级可靠性。

### 🔧 Core Features / 核心功能

| Feature | 功能 | Description | 描述 |
|---------|------|-------------|------|
| **Agent State Management** | **Agent 状态管理** | Multi-type state persistence with version control | 多种状态类型持久化，支持版本控制 |
| **Intelligent Memory** | **智能记忆系统** | Hierarchical memory with smart retrieval | 分层记忆架构，智能检索机制 |
| **Vector Search** | **向量搜索** | High-dimensional vector storage and similarity search | 高维向量存储和相似性搜索 |
| **RAG Engine** | **RAG 引擎** | Document indexing and semantic search | 文档索引和语义搜索 |
| **Distributed Network** | **分布式网络** | Multi-node topology and load balancing | 多节点拓扑和负载均衡 |
| **Security** | **安全管理** | RBAC, encryption, and audit logging | 基于角色的访问控制、加密和审计日志 |

### 📈 Performance Metrics / 性能指标

| Operation | 操作 | Performance | 性能 | Target | 目标 |
|-----------|------|-------------|------|--------|------|
| **Vector Search** | **向量搜索** | 22.09ms | 22.09ms | < 100ms | < 100ms |
| **Document Search** | **文档搜索** | 22.63ms | 22.63ms | < 50ms | < 50ms |
| **Semantic Search** | **语义搜索** | 16.93ms | 16.93ms | < 50ms | < 50ms |
| **Memory Retrieval** | **记忆检索** | 166.17ms | 166.17ms | < 200ms | < 200ms |
| **Integrated Workflow** | **集成工作流** | 265.19ms | 265.19ms | < 300ms | < 300ms |

### 🏗️ Architecture Highlights / 架构亮点

**English**:
- **Hybrid Architecture**: Rust (performance) + Zig (zero-cost abstractions) + LanceDB (vector storage)
- **Multi-language Support**: C FFI interface enabling Python, JavaScript, Go bindings
- **Production Ready**: 100% test coverage, enterprise-grade error handling
- **Scalable Design**: Distributed architecture supporting horizontal scaling

**中文**:
- **混合架构**: Rust (性能) + Zig (零成本抽象) + LanceDB (向量存储)
- **多语言支持**: C FFI 接口支持 Python、JavaScript、Go 绑定
- **生产就绪**: 100% 测试覆盖率，企业级错误处理
- **可扩展设计**: 分布式架构支持水平扩展

## 🛠️ Development Status / 开发状态

### ✅ Completed Features / 已完成功能

- **Core Database Engine** / **核心数据库引擎** ✅
- **Agent State Management** / **Agent 状态管理** ✅
- **Memory System** / **记忆系统** ✅
- **Vector Operations** / **向量操作** ✅
- **RAG Engine** / **RAG 引擎** ✅
- **Distributed Network** / **分布式网络** ✅
- **Security Framework** / **安全框架** ✅
- **Performance Optimization** / **性能优化** ✅
- **C FFI Interface** / **C FFI 接口** ✅
- **Comprehensive Testing** / **全面测试** ✅

### 🔄 In Progress / 进行中

- **Python Bindings** / **Python 绑定** 🚧
- **JavaScript Bindings** / **JavaScript 绑定** 🚧
- **Cloud-native Features** / **云原生功能** 🚧
- **Advanced Monitoring** / **高级监控** 🚧

### 📅 Planned Features / 计划功能

- **Go Bindings** / **Go 绑定** 📋
- **Kubernetes Operator** / **Kubernetes 操作器** 📋
- **Web Management UI** / **Web 管理界面** 📋
- **Enterprise Features** / **企业级功能** 📋

## 🎯 Getting Started / 开始使用

### Quick Links / 快速链接

**For English speakers**:
1. [Installation Guide](en/getting-started.md#installation-guide)
2. [Your First Program](en/getting-started.md#your-first-program)
3. [API Reference](en/api.md)

**中文用户**:
1. [安装指南](zh/getting-started.md#安装指南)
2. [第一个程序](zh/getting-started.md#第一个程序)
3. [API 参考](zh/api.md)

### System Requirements / 系统要求

| Component | 组件 | Minimum | 最低要求 | Recommended | 推荐配置 |
|-----------|------|---------|----------|-------------|----------|
| **OS** | **操作系统** | Windows 10+, Linux, macOS 10.15+ | Windows 10+, Linux, macOS 10.15+ | Windows 11, Ubuntu 22.04+, macOS 12+ | Windows 11, Ubuntu 22.04+, macOS 12+ |
| **Memory** | **内存** | 4GB RAM | 4GB RAM | 8GB+ RAM | 8GB+ RAM |
| **Storage** | **存储** | 1GB available | 1GB 可用空间 | 10GB+ SSD | 10GB+ SSD |
| **CPU** | **处理器** | 2 cores | 2 核心 | 4+ cores | 4+ 核心 |

## 🤝 Community / 社区

### Contributing / 贡献

**English**: We welcome contributions! Please read our [Contributing Guide](../CONTRIBUTING.md) for details on how to submit pull requests, report issues, and contribute to the project.

**中文**: 我们欢迎贡献！请阅读我们的[贡献指南](../CONTRIBUTING.md)了解如何提交拉取请求、报告问题和为项目做贡献的详细信息。

### Support / 支持

- **GitHub Issues**: [Report bugs and request features](https://github.com/louloulin/AgentDB/issues)
- **GitHub Discussions**: [Community discussions and Q&A](https://github.com/louloulin/AgentDB/discussions)
- **Documentation**: [Complete documentation](../README.md)

### License / 许可证

AgentDB is licensed under the MIT License. See [LICENSE](../LICENSE) for details.

AgentDB 采用 MIT 许可证。详见 [LICENSE](../LICENSE) 文件。

---

**Documentation Version** / **文档版本**: v1.0  
**Last Updated** / **最后更新**: June 19, 2025 / 2025年6月19日  
**Maintainer** / **维护者**: AgentDB Development Team / AgentDB 开发团队
