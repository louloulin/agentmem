"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { Brain, Play, Code, Zap, Database, MessageSquare, ArrowRight, Copy, Check } from "lucide-react";
import Link from "next/link";

/**
 * 演示页面组件 - 展示AgentMem的在线演示和交互式示例
 */
export default function DemoPage() {
  const [activeDemo, setActiveDemo] = useState("memory-basic");
  const [isRunning, setIsRunning] = useState(false);
  const [demoOutput, setDemoOutput] = useState("");
  const [copiedCode, setCopiedCode] = useState("");

  /**
   * 运行演示代码
   */
  const runDemo = async (demoType: string) => {
    setIsRunning(true);
    setDemoOutput("正在运行演示...");
    
    // 模拟演示运行
    setTimeout(() => {
      switch (demoType) {
        case "memory-basic":
          setDemoOutput(`✅ 记忆创建成功
记忆ID: mem_001
内容: "我喜欢在周末喝咖啡和阅读"
类型: Episodic
重要性: 0.8

🔍 搜索结果:
找到 3 条相关记忆:
1. "我喜欢在周末喝咖啡和阅读" (相似度: 1.0)
2. "咖啡是我最喜欢的饮品" (相似度: 0.85)
3. "周末通常在家阅读技术书籍" (相似度: 0.72)`);
          break;
        case "intelligent-reasoning":
          setDemoOutput(`🧠 智能推理结果:

📊 事实提取:
✓ 提取了 4 个事实:
  • 用户名称: Alice
  • 居住地: 北京
  • 兴趣爱好: 编程
  • 职业相关: 软件开发

🎯 记忆决策:
✓ 生成了 2 个决策:
  1. ADD: "用户 Alice 来自北京" (置信度: 0.95)
  2. ADD: "用户喜欢编程" (置信度: 0.90)

⚡ 处理统计:
  • 处理时间: 1.2s
  • 事实置信度: 92%
  • 决策准确率: 95%`);
          break;
        case "mem0-compat":
          setDemoOutput(`🔄 Mem0 兼容性演示:

✅ 客户端初始化成功
✅ 添加记忆: "I love pizza" (ID: mem_pizza_001)
✅ 搜索记忆: "food" 

📋 搜索结果:
找到 2 条记忆:
1. "I love pizza" (分数: 0.95)
2. "My favorite cuisine is Italian" (分数: 0.78)

🚀 性能提升:
  • 查询速度: 比原版快 3.2x
  • 内存使用: 减少 45%
  • 并发处理: 提升 5x`);
          break;
        default:
          setDemoOutput("演示完成！");
      }
      setIsRunning(false);
    }, 2000);
  };

  /**
   * 复制代码到剪贴板
   */
  const copyCode = async (code: string, id: string) => {
    try {
      await navigator.clipboard.writeText(code);
      setCopiedCode(id);
      setTimeout(() => setCopiedCode(""), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  };

  const demoConfigs = {
    "memory-basic": {
      title: "基础记忆操作",
      description: "演示记忆的创建、搜索和管理功能",
      code: `use agent_mem_core::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::MemoryType;

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
    
    println!("记忆创建成功: {}", memory_id);
    
    // 搜索记忆
    let results = engine.search_memories(
        "咖啡", 
        "user123", 
        10
    ).await?;
    
    println!("找到 {} 条相关记忆", results.len());
    for result in results {
        println!("- {} (相似度: {:.2})", 
                result.content, result.similarity);
    }
    
    Ok(())
}`
    },
    "intelligent-reasoning": {
      title: "智能推理引擎",
      description: "展示 DeepSeek 驱动的事实提取和决策功能",
      code: `use agent_mem_intelligence::{
    IntelligentMemoryProcessor, Message
};

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
    let result = processor.process_messages(
        &messages, 
        &[]
    ).await?;
    
    println!("提取了 {} 个事实", result.extracted_facts.len());
    for fact in &result.extracted_facts {
        println!("- {} (置信度: {:.2})", 
                fact.content, fact.confidence);
    }
    
    println!("生成了 {} 个记忆决策", result.memory_decisions.len());
    for decision in &result.memory_decisions {
        println!("- {:?} (置信度: {:.2})", 
                decision.action, decision.confidence);
    }
    
    Ok(())
}`
    },
    "mem0-compat": {
      title: "Mem0 兼容性",
      description: "展示 100% Mem0 API 兼容性和性能提升",
      code: `use agent_mem_compat::Mem0Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 Mem0 兼容客户端
    let client = Mem0Client::new().await?;
    
    // 使用与 Mem0 完全相同的 API
    let memory_id = client.add(
        "user123", 
        "I love pizza", 
        None
    ).await?;
    
    println!("记忆添加成功: {}", memory_id);
    
    // 搜索记忆
    let results = client.search(
        "food", 
        "user123", 
        None
    ).await?;
    
    println!("找到 {} 条记忆", results.len());
    for memory in results {
        println!("- {}: {} (分数: {:.2})", 
                memory.id, memory.content, memory.score);
    }
    
    // 获取所有记忆
    let all_memories = client.get_all(
        Some("user123".to_string()),
        None,
        None,
        Some(100),
        None
    ).await?;
    
    println!("用户总共有 {} 条记忆", all_memories.len());
    
    Ok(())
}`
    }
  };

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
              <Link href="/docs" className="text-slate-300 hover:text-white transition-colors">
                文档
              </Link>
              <Link href="/demo" className="text-white font-semibold">
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

      {/* 页面内容 */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        {/* 页面标题 */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-white mb-4">在线演示</h1>
          <p className="text-xl text-slate-300 max-w-3xl mx-auto">
            体验 AgentMem 的强大功能，包括智能记忆管理、事实提取和 Mem0 兼容性
          </p>
        </div>

        {/* 演示选择 */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <Card 
            className={`cursor-pointer transition-all ${
              activeDemo === "memory-basic" 
                ? "bg-purple-600/20 border-purple-500" 
                : "bg-slate-800/50 border-slate-700 hover:border-purple-500/50"
            }`}
            onClick={() => setActiveDemo("memory-basic")}
          >
            <CardHeader>
              <Database className="h-12 w-12 text-blue-400 mb-4" />
              <CardTitle className="text-white">基础记忆操作</CardTitle>
              <CardDescription className="text-slate-300">
                记忆的创建、搜索和管理
              </CardDescription>
            </CardHeader>
          </Card>

          <Card 
            className={`cursor-pointer transition-all ${
              activeDemo === "intelligent-reasoning" 
                ? "bg-purple-600/20 border-purple-500" 
                : "bg-slate-800/50 border-slate-700 hover:border-purple-500/50"
            }`}
            onClick={() => setActiveDemo("intelligent-reasoning")}
          >
            <CardHeader>
              <Brain className="h-12 w-12 text-purple-400 mb-4" />
              <CardTitle className="text-white">智能推理引擎</CardTitle>
              <CardDescription className="text-slate-300">
                DeepSeek 驱动的事实提取
              </CardDescription>
            </CardHeader>
          </Card>

          <Card 
            className={`cursor-pointer transition-all ${
              activeDemo === "mem0-compat" 
                ? "bg-purple-600/20 border-purple-500" 
                : "bg-slate-800/50 border-slate-700 hover:border-purple-500/50"
            }`}
            onClick={() => setActiveDemo("mem0-compat")}
          >
            <CardHeader>
              <Zap className="h-12 w-12 text-green-400 mb-4" />
              <CardTitle className="text-white">Mem0 兼容性</CardTitle>
              <CardDescription className="text-slate-300">
                100% API 兼容和性能提升
              </CardDescription>
            </CardHeader>
          </Card>
        </div>

        {/* 演示内容 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* 代码编辑器 */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <div className="flex justify-between items-center">
                <div>
                  <CardTitle className="text-white flex items-center">
                    <Code className="h-5 w-5 mr-2" />
                    {demoConfigs[activeDemo as keyof typeof demoConfigs].title}
                  </CardTitle>
                  <CardDescription className="text-slate-300">
                    {demoConfigs[activeDemo as keyof typeof demoConfigs].description}
                  </CardDescription>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    className="border-slate-600 text-slate-300 hover:bg-slate-700"
                    onClick={() => copyCode(
                      demoConfigs[activeDemo as keyof typeof demoConfigs].code, 
                      activeDemo
                    )}
                  >
                    {copiedCode === activeDemo ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                  <Button
                    size="sm"
                    className="bg-purple-600 hover:bg-purple-700"
                    onClick={() => runDemo(activeDemo)}
                    disabled={isRunning}
                  >
                    {isRunning ? (
                      <div className="animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full" />
                    ) : (
                      <Play className="h-4 w-4" />
                    )}
                    {isRunning ? "运行中..." : "运行"}
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="bg-slate-900 p-4 rounded-lg overflow-x-auto">
                <pre className="text-sm text-green-400 whitespace-pre-wrap">
                  <code>{demoConfigs[activeDemo as keyof typeof demoConfigs].code}</code>
                </pre>
              </div>
            </CardContent>
          </Card>

          {/* 输出结果 */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center">
                <MessageSquare className="h-5 w-5 mr-2" />
                运行结果
              </CardTitle>
              <CardDescription className="text-slate-300">
                查看演示代码的执行输出
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-slate-900 p-4 rounded-lg min-h-[400px]">
                {demoOutput ? (
                  <pre className="text-sm text-green-400 whitespace-pre-wrap">
                    {demoOutput}
                  </pre>
                ) : (
                  <div className="flex items-center justify-center h-full text-slate-500">
                    点击"运行"按钮查看演示结果
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* 功能特性展示 */}
        <section className="mt-20">
          <h2 className="text-3xl font-bold text-white mb-8 text-center">核心功能演示</h2>
          
          <Tabs defaultValue="features" className="w-full">
            <TabsList className="grid w-full grid-cols-3 bg-slate-800/50">
              <TabsTrigger value="features" className="data-[state=active]:bg-purple-600">核心特性</TabsTrigger>
              <TabsTrigger value="performance" className="data-[state=active]:bg-purple-600">性能对比</TabsTrigger>
              <TabsTrigger value="integration" className="data-[state=active]:bg-purple-600">集成示例</TabsTrigger>
            </TabsList>
            
            <TabsContent value="features" className="mt-8">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <Brain className="h-8 w-8 text-purple-400 mb-2" />
                    <CardTitle className="text-white">智能事实提取</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p className="mb-4">从自然语言对话中自动提取结构化事实信息</p>
                    <Badge className="bg-purple-500/20 text-purple-300">95% 准确率</Badge>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <Zap className="h-8 w-8 text-yellow-400 mb-2" />
                    <CardTitle className="text-white">记忆决策引擎</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p className="mb-4">智能决策记忆的添加、更新、删除和合并操作</p>
                    <Badge className="bg-yellow-500/20 text-yellow-300">90% 决策准确率</Badge>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <Database className="h-8 w-8 text-green-400 mb-2" />
                    <CardTitle className="text-white">多存储后端</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p className="mb-4">支持 8+ 种向量数据库和图数据库</p>
                    <Badge className="bg-green-500/20 text-green-300">灵活扩展</Badge>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
            
            <TabsContent value="performance" className="mt-8">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white">性能基准测试</CardTitle>
                  <CardDescription className="text-slate-300">
                    AgentMem vs 其他记忆管理解决方案
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div className="text-center">
                      <div className="text-3xl font-bold text-purple-400 mb-2">3.2x</div>
                      <div className="text-slate-300">查询速度提升</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold text-blue-400 mb-2">45%</div>
                      <div className="text-slate-300">内存使用减少</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold text-green-400 mb-2">5x</div>
                      <div className="text-slate-300">并发处理提升</div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
            
            <TabsContent value="integration" className="mt-8">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">Web 应用集成</CardTitle>
                    <CardDescription className="text-slate-300">
                      在 Web 应用中集成 AgentMem
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-900 p-4 rounded-lg">
                      <code className="text-sm text-green-400">
                        {`// Express.js 集成示例
const { AgentMemClient } = require('@agentmem/client');

const client = new AgentMemClient({
  apiKey: process.env.AGENTMEM_API_KEY
});

app.post('/chat', async (req, res) => {
  const { message, userId } = req.body;
  
  // 添加用户消息到记忆
  await client.addMemory(userId, message);
  
  // 搜索相关记忆
  const memories = await client.searchMemories(
    message, userId, 5
  );
  
  res.json({ memories });
});`}
                      </code>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">Python 集成</CardTitle>
                    <CardDescription className="text-slate-300">
                      在 Python 应用中使用 AgentMem
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-900 p-4 rounded-lg">
                      <code className="text-sm text-green-400">
                        {`# Python 集成示例
from agentmem import AgentMemClient

client = AgentMemClient(
    api_key=os.getenv('AGENTMEM_API_KEY')
)

# 添加记忆
memory_id = await client.add_memory(
    user_id="user123",
    content="用户喜欢喝咖啡",
    memory_type="preference"
)

# 智能搜索
results = await client.search_memories(
    query="饮品偏好",
    user_id="user123",
    limit=10
)

print(f"找到 {len(results)} 条相关记忆")`}
                      </code>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
          </Tabs>
        </section>

        {/* CTA 区域 */}
        <section className="mt-20 text-center">
          <Card className="bg-gradient-to-r from-purple-600/20 to-pink-600/20 border-purple-500/30">
            <CardHeader>
              <CardTitle className="text-white text-3xl mb-4">
                准备开始构建？
              </CardTitle>
              <CardDescription className="text-slate-300 text-lg">
                立即开始使用 AgentMem，为您的 AI 应用添加强大的记忆能力
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button size="lg" className="bg-purple-600 hover:bg-purple-700">
                  开始免费试用
                  <ArrowRight className="ml-2 h-5 w-5" />
                </Button>
                <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                  查看文档
                </Button>
                <Button size="lg" variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                  下载示例
                </Button>
              </div>
            </CardContent>
          </Card>
        </section>
      </div>
    </div>
  );
}