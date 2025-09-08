import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Brain, Zap, Shield, Database, Cpu, Network, Code, Rocket } from "lucide-react";
import Link from "next/link";

/**
 * 主页组件 - 展示AgentMem的核心特性和优势
 */
export default function HomePage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <Brain className="h-8 w-8 text-purple-400" />
              <span className="ml-2 text-xl font-bold text-white">AgentMem</span>
            </div>
            <div className="hidden md:flex items-center space-x-8">
              <Link href="#features" className="text-slate-300 hover:text-white transition-colors">
                特性
              </Link>
              <Link href="#architecture" className="text-slate-300 hover:text-white transition-colors">
                架构
              </Link>
              <Link href="/demo" className="text-slate-300 hover:text-white transition-colors">
                演示
              </Link>
              <Link href="/docs" className="text-slate-300 hover:text-white transition-colors">
                文档
              </Link>
              <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                开始使用
              </Button>
            </div>
            {/* 移动端菜单按钮 */}
            <div className="md:hidden flex items-center">
              <Button variant="outline" size="sm" className="border-purple-400 text-purple-400">
                菜单
              </Button>
            </div>
          </div>
        </div>
      </nav>

      {/* 英雄区域 */}
      <section className="relative py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto text-center">
          <div className="mb-8">
            <Badge className="bg-purple-500/20 text-purple-300 border-purple-500/30 mb-4">
              🚀 v2.0 - 智能推理引擎已发布
            </Badge>
          </div>
          <h1 className="text-5xl md:text-7xl font-bold text-white mb-6">
            智能记忆管理
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-pink-400">
              平台
            </span>
          </h1>
          <p className="text-xl text-slate-300 mb-8 max-w-3xl mx-auto">
            为 AI 代理提供先进的记忆处理能力。基于 Rust 的模块化架构，集成 DeepSeek 智能推理引擎，
            支持多存储后端，100% Mem0 兼容。
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Button size="lg" className="bg-purple-600 hover:bg-purple-700 text-white">
              <Rocket className="mr-2 h-5 w-5" />
              快速开始
            </Button>
            <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
              <Code className="mr-2 h-5 w-5" />
              查看代码
            </Button>
          </div>
        </div>
      </section>

      {/* 核心特性 */}
      <section id="features" className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-white mb-4">核心特性</h2>
            <p className="text-xl text-slate-300">企业级智能记忆管理解决方案</p>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {/* 智能推理引擎 */}
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-colors">
              <CardHeader>
                <Brain className="h-12 w-12 text-purple-400 mb-4" />
                <CardTitle className="text-white">智能推理引擎</CardTitle>
                <CardDescription className="text-slate-300">
                  DeepSeek 驱动的事实提取和记忆决策
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 自动事实提取</li>
                  <li>• 智能冲突解决</li>
                  <li>• 上下文感知搜索</li>
                  <li>• 动态重要性评估</li>
                </ul>
              </CardContent>
            </Card>

            {/* 模块化架构 */}
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-colors">
              <CardHeader>
                <Cpu className="h-12 w-12 text-blue-400 mb-4" />
                <CardTitle className="text-white">模块化架构</CardTitle>
                <CardDescription className="text-slate-300">
                  13个专业化 Crate，职责清晰分离
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 核心记忆引擎</li>
                  <li>• 智能处理模块</li>
                  <li>• 多存储后端</li>
                  <li>• LLM 集成层</li>
                </ul>
              </CardContent>
            </Card>

            {/* 高性能架构 */}
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-colors">
              <CardHeader>
                <Zap className="h-12 w-12 text-yellow-400 mb-4" />
                <CardTitle className="text-white">高性能架构</CardTitle>
                <CardDescription className="text-slate-300">
                  基于 Tokio 的异步优先设计
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 多级缓存系统</li>
                  <li>• 批量处理优化</li>
                  <li>• 实时性能监控</li>
                  <li>• 自适应优化</li>
                </ul>
              </CardContent>
            </Card>

            {/* 多存储后端 */}
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-colors">
              <CardHeader>
                <Database className="h-12 w-12 text-green-400 mb-4" />
                <CardTitle className="text-white">多存储后端</CardTitle>
                <CardDescription className="text-slate-300">
                  支持8+种向量数据库和图数据库
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• Pinecone, Qdrant, Chroma</li>
                  <li>• PostgreSQL, Redis</li>
                  <li>• Neo4j, Memgraph</li>
                  <li>• 内存存储优化</li>
                </ul>
              </CardContent>
            </Card>

            {/* 企业级特性 */}
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-colors">
              <CardHeader>
                <Shield className="h-12 w-12 text-red-400 mb-4" />
                <CardTitle className="text-white">企业级特性</CardTitle>
                <CardDescription className="text-slate-300">
                  生产就绪的安全和可靠性保障
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 类型安全保证</li>
                  <li>• 完整测试覆盖</li>
                  <li>• 分布式支持</li>
                  <li>• 监控和遥测</li>
                </ul>
              </CardContent>
            </Card>

            {/* Mem0 兼容 */}
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-colors">
              <CardHeader>
                <Network className="h-12 w-12 text-indigo-400 mb-4" />
                <CardTitle className="text-white">Mem0 兼容</CardTitle>
                <CardDescription className="text-slate-300">
                  100% API 兼容，支持无缝迁移
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 完整 API 兼容</li>
                  <li>• 零代码迁移</li>
                  <li>• 性能提升</li>
                  <li>• 扩展功能</li>
                </ul>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* 技术架构 */}
      <section id="architecture" className="py-20 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-white mb-4">技术架构</h2>
            <p className="text-xl text-slate-300">分层模块化设计，支持大规模部署</p>
          </div>
          
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
            <div>
              <h3 className="text-2xl font-bold text-white mb-6">分层架构设计</h3>
              <div className="space-y-4">
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-purple-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">应用层</h4>
                    <p className="text-slate-300 text-sm">HTTP服务器、客户端、兼容层</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-blue-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">业务逻辑层</h4>
                    <p className="text-slate-300 text-sm">智能处理、性能监控、核心引擎</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-green-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">服务层</h4>
                    <p className="text-slate-300 text-sm">LLM集成、嵌入模型、分布式支持</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-yellow-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">数据层</h4>
                    <p className="text-slate-300 text-sm">存储抽象、配置管理</p>
                  </div>
                </div>
                <div className="flex items-center p-4 bg-slate-700/50 rounded-lg">
                  <div className="w-4 h-4 bg-red-400 rounded-full mr-4"></div>
                  <div>
                    <h4 className="text-white font-semibold">基础设施层</h4>
                    <p className="text-slate-300 text-sm">核心抽象、工具库</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="bg-slate-800/50 p-8 rounded-lg border border-slate-700">
              <h3 className="text-2xl font-bold text-white mb-6">性能指标</h3>
              <div className="grid grid-cols-2 gap-6">
                <div className="text-center">
                  <div className="text-3xl font-bold text-purple-400 mb-2">13</div>
                  <div className="text-slate-300">核心 Crate</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-blue-400 mb-2">100%</div>
                  <div className="text-slate-300">Mem0 兼容</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-green-400 mb-2">8+</div>
                  <div className="text-slate-300">存储后端</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-yellow-400 mb-2">15+</div>
                  <div className="text-slate-300">LLM 提供商</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA 区域 */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl font-bold text-white mb-6">
            准备开始使用 AgentMem？
          </h2>
          <p className="text-xl text-slate-300 mb-8">
            立即体验下一代智能记忆管理平台，为您的 AI 应用提供强大的记忆能力。
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Button size="lg" className="bg-purple-600 hover:bg-purple-700 text-white">
              开始免费试用
            </Button>
            <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
              联系销售团队
            </Button>
          </div>
        </div>
      </section>

      {/* 页脚 */}
      <footer className="border-t border-slate-800 bg-slate-900/50 py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div>
              <div className="flex items-center mb-4">
                <Brain className="h-6 w-6 text-purple-400" />
                <span className="ml-2 text-lg font-bold text-white">AgentMem</span>
              </div>
              <p className="text-slate-400">
                智能记忆管理平台，为 AI 代理提供先进的记忆处理能力。
              </p>
            </div>
            <div>
              <h3 className="text-white font-semibold mb-4">产品</h3>
              <ul className="space-y-2 text-slate-400">
                <li><Link href="#" className="hover:text-white transition-colors">核心引擎</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">智能推理</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">企业版</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">云服务</Link></li>
              </ul>
            </div>
            <div>
              <h3 className="text-white font-semibold mb-4">开发者</h3>
              <ul className="space-y-2 text-slate-400">
                <li><Link href="#" className="hover:text-white transition-colors">API 文档</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">快速开始</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">示例代码</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">GitHub</Link></li>
              </ul>
            </div>
            <div>
              <h3 className="text-white font-semibold mb-4">公司</h3>
              <ul className="space-y-2 text-slate-400">
                <li><Link href="#" className="hover:text-white transition-colors">关于我们</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">博客</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">职业机会</Link></li>
                <li><Link href="#" className="hover:text-white transition-colors">联系我们</Link></li>
              </ul>
            </div>
          </div>
          <Separator className="my-8 bg-slate-800" />
          <div className="flex flex-col md:flex-row justify-between items-center">
            <p className="text-slate-400">
              © 2024 AgentMem. All rights reserved.
            </p>
            <div className="flex space-x-6 mt-4 md:mt-0">
              <Link href="#" className="text-slate-400 hover:text-white transition-colors">
                隐私政策
              </Link>
              <Link href="#" className="text-slate-400 hover:text-white transition-colors">
                服务条款
              </Link>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
