import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { Brain, Book, Code, Terminal, Zap, Database, Settings, ArrowRight } from "lucide-react";
import Link from "next/link";

/**
 * 文档页面组件 - 展示API文档、快速开始指南和使用示例
 */
export default function DocsPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <Link href="/" className="flex items-center">
                <Brain className="h-8 w-8 text-purple-400" />
                <span className="ml-2 text-xl font-bold text-white">AgentMem</span>
              </Link>
            </div>
            <div className="flex items-center space-x-8">
              <Link href="/" className="text-slate-300 hover:text-white transition-colors">
                首页
              </Link>
              <Link href="/docs" className="text-white font-semibold">
                文档
              </Link>
              <Link href="/demo" className="text-slate-300 hover:text-white transition-colors">
                演示
              </Link>
              <Link href="/about" className="text-slate-300 hover:text-white transition-colors">
                关于
              </Link>
              <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                GitHub
              </Button>
            </div>
          </div>
        </div>
      </nav>

      {/* 文档内容 */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          {/* 侧边栏导航 */}
          <div className="lg:col-span-1">
            <div className="sticky top-8">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white flex items-center">
                    <Book className="h-5 w-5 mr-2" />
                    文档导航
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <nav className="space-y-2">
                    <Link href="#quick-start" className="block text-slate-300 hover:text-purple-400 transition-colors py-2">
                      快速开始
                    </Link>
                    <Link href="#installation" className="block text-slate-300 hover:text-purple-400 transition-colors py-2">
                      安装指南
                    </Link>
                    <Link href="#api-reference" className="block text-slate-300 hover:text-purple-400 transition-colors py-2">
                      API 参考
                    </Link>
                    <Link href="#examples" className="block text-slate-300 hover:text-purple-400 transition-colors py-2">
                      代码示例
                    </Link>
                    <Link href="#architecture" className="block text-slate-300 hover:text-purple-400 transition-colors py-2">
                      架构设计
                    </Link>
                    <Link href="#deployment" className="block text-slate-300 hover:text-purple-400 transition-colors py-2">
                      部署指南
                    </Link>
                  </nav>
                </CardContent>
              </Card>
            </div>
          </div>

          {/* 主要内容 */}
          <div className="lg:col-span-3">
            {/* 页面标题 */}
            <div className="mb-12">
              <h1 className="text-4xl font-bold text-white mb-4">AgentMem 文档</h1>
              <p className="text-xl text-slate-300">
                完整的 API 文档、快速开始指南和最佳实践
              </p>
            </div>

            {/* 快速开始 */}
            <section id="quick-start" className="mb-16">
              <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                <Zap className="h-8 w-8 text-yellow-400 mr-3" />
                快速开始
              </h2>
              
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <div className="w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center mb-4">
                      <span className="text-2xl font-bold text-purple-400">1</span>
                    </div>
                    <CardTitle className="text-white">安装 AgentMem</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p>使用 Cargo 安装 AgentMem 核心库</p>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <div className="w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center mb-4">
                      <span className="text-2xl font-bold text-blue-400">2</span>
                    </div>
                    <CardTitle className="text-white">配置环境</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p>设置 API 密钥和存储后端配置</p>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                      <span className="text-2xl font-bold text-green-400">3</span>
                    </div>
                    <CardTitle className="text-white">开始编码</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p>创建您的第一个智能记忆应用</p>
                  </CardContent>
                </Card>
              </div>
            </section>

            {/* 安装指南 */}
            <section id="installation" className="mb-16">
              <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                <Terminal className="h-8 w-8 text-green-400 mr-3" />
                安装指南
              </h2>
              
              <Tabs defaultValue="cargo" className="w-full">
                <TabsList className="grid w-full grid-cols-3 bg-slate-800/50">
                  <TabsTrigger value="cargo" className="data-[state=active]:bg-purple-600">Cargo</TabsTrigger>
                  <TabsTrigger value="mem0" className="data-[state=active]:bg-purple-600">Mem0 兼容</TabsTrigger>
                  <TabsTrigger value="docker" className="data-[state=active]:bg-purple-600">Docker</TabsTrigger>
                </TabsList>
                
                <TabsContent value="cargo" className="mt-6">
                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <CardTitle className="text-white">使用 Cargo 安装</CardTitle>
                      <CardDescription className="text-slate-300">
                        将 AgentMem 添加到您的 Rust 项目
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="bg-slate-900 p-4 rounded-lg mb-4">
                        <code className="text-green-400">
                          {`# 添加到 Cargo.toml
[dependencies]
agent-mem-core = "2.0"
agent-mem-intelligence = "2.0"
agent-mem-compat = "2.0"`}
                        </code>
                      </div>
                      <div className="bg-slate-900 p-4 rounded-lg">
                        <code className="text-green-400">
                          {`# 或使用 cargo add
cargo add agent-mem-core agent-mem-intelligence`}
                        </code>
                      </div>
                    </CardContent>
                  </Card>
                </TabsContent>
                
                <TabsContent value="mem0" className="mt-6">
                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <CardTitle className="text-white">Mem0 兼容模式</CardTitle>
                      <CardDescription className="text-slate-300">
                        无缝替换现有的 Mem0 实现
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="bg-slate-900 p-4 rounded-lg mb-4">
                        <code className="text-green-400">
                          {`use agent_mem_compat::Mem0Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Mem0Client::new().await?;
    
    // 使用与 Mem0 相同的 API
    let memory_id = client.add(
        "user123", 
        "我喜欢喝咖啡", 
        None
    ).await?;
    
    Ok(())
}`}
                        </code>
                      </div>
                    </CardContent>
                  </Card>
                </TabsContent>
                
                <TabsContent value="docker" className="mt-6">
                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <CardTitle className="text-white">Docker 部署</CardTitle>
                      <CardDescription className="text-slate-300">
                        使用 Docker 快速部署 AgentMem 服务
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="bg-slate-900 p-4 rounded-lg mb-4">
                        <code className="text-green-400">
                          {`# 拉取镜像
docker pull agentmem/server:latest

# 运行服务
docker run -p 8080:8080 \
  -e DEEPSEEK_API_KEY=your_key \
  agentmem/server:latest`}
                        </code>
                      </div>
                      <div className="bg-slate-900 p-4 rounded-lg">
                        <code className="text-green-400">
                          {`# 使用 docker-compose
version: '3.8'
services:
  agentmem:
    image: agentmem/server:latest
    ports:
      - "8080:8080"
    environment:
      - DEEPSEEK_API_KEY=your_key`}
                        </code>
                      </div>
                    </CardContent>
                  </Card>
                </TabsContent>
              </Tabs>
            </section>

            {/* API 参考 */}
            <section id="api-reference" className="mb-16">
              <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                <Code className="h-8 w-8 text-blue-400 mr-3" />
                API 参考
              </h2>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">核心 API</CardTitle>
                    <CardDescription className="text-slate-300">
                      记忆管理的核心功能
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <div className="space-y-3">
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">add_memory()</code>
                        <Badge variant="outline" className="text-xs">POST</Badge>
                      </div>
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">search_memories()</code>
                        <Badge variant="outline" className="text-xs">GET</Badge>
                      </div>
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">update_memory()</code>
                        <Badge variant="outline" className="text-xs">PUT</Badge>
                      </div>
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">delete_memory()</code>
                        <Badge variant="outline" className="text-xs">DELETE</Badge>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">智能推理 API</CardTitle>
                    <CardDescription className="text-slate-300">
                      AI 驱动的智能处理功能
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <div className="space-y-3">
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">extract_facts()</code>
                        <Badge variant="outline" className="text-xs">POST</Badge>
                      </div>
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">make_decisions()</code>
                        <Badge variant="outline" className="text-xs">POST</Badge>
                      </div>
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">analyze_health()</code>
                        <Badge variant="outline" className="text-xs">GET</Badge>
                      </div>
                      <div className="flex justify-between items-center">
                        <code className="text-purple-400">process_messages()</code>
                        <Badge variant="outline" className="text-xs">POST</Badge>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </section>

            {/* 代码示例 */}
            <section id="examples" className="mb-16">
              <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                <Code className="h-8 w-8 text-purple-400 mr-3" />
                代码示例
              </h2>
              
              <div className="space-y-8">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">基础记忆操作</CardTitle>
                    <CardDescription className="text-slate-300">
                      创建、搜索和管理记忆的基本示例
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-900 p-4 rounded-lg overflow-x-auto">
                      <code className="text-sm text-green-400 whitespace-pre">
{`use agent_mem_core::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::{MemoryType, Session};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建记忆引擎
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config);
    
    // 添加记忆
    let memory_id = engine.add_memory(
        "user123",
        "我喜欢在周末喝咖啡和阅读",
        Some(MemoryType::Episodic)
    ).await?;
    
    // 搜索记忆
    let results = engine.search_memories(
        "咖啡", 
        "user123", 
        10
    ).await?;
    
    println!("找到 {} 条相关记忆", results.len());
    Ok(())
}`}
                      </code>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">智能推理示例</CardTitle>
                    <CardDescription className="text-slate-300">
                      使用 DeepSeek 驱动的智能推理引擎
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-900 p-4 rounded-lg overflow-x-auto">
                      <code className="text-sm text-green-400 whitespace-pre">
{`use agent_mem_intelligence::{IntelligentMemoryProcessor, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建智能处理器
    let processor = IntelligentMemoryProcessor::new(
        "your-deepseek-api-key".to_string()
    )?;
    
    // 准备对话消息
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "我是 Alice，来自北京，喜欢编程".to_string(),
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            message_id: Some("msg1".to_string()),
        }
    ];
    
    // 智能处理消息
    let result = processor.process_messages(&messages, &[]).await?;
    
    println!("提取了 {} 个事实", result.extracted_facts.len());
    println!("生成了 {} 个记忆决策", result.memory_decisions.len());
    
    Ok(())
}`}
                      </code>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </section>

            {/* 架构设计 */}
            <section id="architecture" className="mb-16">
              <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                <Database className="h-8 w-8 text-green-400 mr-3" />
                架构设计
              </h2>
              
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white">模块化架构</CardTitle>
                  <CardDescription className="text-slate-300">
                    AgentMem 采用分层模块化设计，支持灵活扩展
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                      <h4 className="text-white font-semibold mb-3">核心模块</h4>
                      <ul className="space-y-2 text-slate-300">
                        <li>• <code className="text-purple-400">agent-mem-traits</code> - 核心抽象</li>
                        <li>• <code className="text-purple-400">agent-mem-core</code> - 记忆引擎</li>
                        <li>• <code className="text-purple-400">agent-mem-intelligence</code> - 智能处理</li>
                        <li>• <code className="text-purple-400">agent-mem-storage</code> - 存储抽象</li>
                      </ul>
                    </div>
                    <div>
                      <h4 className="text-white font-semibold mb-3">扩展模块</h4>
                      <ul className="space-y-2 text-slate-300">
                        <li>• <code className="text-purple-400">agent-mem-llm</code> - LLM 集成</li>
                        <li>• <code className="text-purple-400">agent-mem-server</code> - HTTP 服务</li>
                        <li>• <code className="text-purple-400">agent-mem-compat</code> - Mem0 兼容</li>
                        <li>• <code className="text-purple-400">agent-mem-distributed</code> - 分布式</li>
                      </ul>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </section>

            {/* 部署指南 */}
            <section id="deployment" className="mb-16">
              <h2 className="text-3xl font-bold text-white mb-6 flex items-center">
                <Settings className="h-8 w-8 text-yellow-400 mr-3" />
                部署指南
              </h2>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">生产环境</CardTitle>
                    <CardDescription className="text-slate-300">
                      企业级生产部署配置
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <ul className="space-y-2">
                      <li>• Kubernetes 集群部署</li>
                      <li>• 负载均衡和自动扩缩容</li>
                      <li>• 监控和日志收集</li>
                      <li>• 备份和灾难恢复</li>
                    </ul>
                    <Button className="mt-4 bg-purple-600 hover:bg-purple-700">
                      查看部署指南
                      <ArrowRight className="ml-2 h-4 w-4" />
                    </Button>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">开发环境</CardTitle>
                    <CardDescription className="text-slate-300">
                      本地开发和测试环境配置
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <ul className="space-y-2">
                      <li>• Docker Compose 快速启动</li>
                      <li>• 本地存储后端配置</li>
                      <li>• 开发工具和调试</li>
                      <li>• 测试数据和示例</li>
                    </ul>
                    <Button variant="outline" className="mt-4 border-slate-600 text-slate-300 hover:bg-slate-800">
                      快速开始
                      <ArrowRight className="ml-2 h-4 w-4" />
                    </Button>
                  </CardContent>
                </Card>
              </div>
            </section>

            {/* 获取帮助 */}
            <section className="mb-16">
              <Card className="bg-gradient-to-r from-purple-600/20 to-pink-600/20 border-purple-500/30">
                <CardHeader>
                  <CardTitle className="text-white text-2xl">需要帮助？</CardTitle>
                  <CardDescription className="text-slate-300">
                    我们提供多种方式来帮助您快速上手 AgentMem
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="flex flex-col sm:flex-row gap-4">
                    <Button className="bg-purple-600 hover:bg-purple-700">
                      加入社区
                    </Button>
                    <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                      联系支持
                    </Button>
                    <Button variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                      查看示例
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </section>
          </div>
        </div>
      </div>
    </div>
  );
}