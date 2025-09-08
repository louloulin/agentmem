import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Brain, Users, Target, Rocket, Globe, Award, TrendingUp, Building, Mail, Github, Twitter, Linkedin } from "lucide-react";
import Link from "next/link";

/**
 * 关于页面组件 - 展示团队介绍、商业化方向和未来规划
 */
export default function AboutPage() {
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
              <Link href="/demo" className="text-slate-300 hover:text-white transition-colors">
                演示
              </Link>
              <Link href="/about" className="text-white font-semibold">
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
        {/* 公司介绍 */}
        <section className="mb-20">
          <div className="text-center mb-16">
            <h1 className="text-5xl font-bold text-white mb-6">
              重新定义
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-pink-400">
                AI 记忆
              </span>
            </h1>
            <p className="text-xl text-slate-300 max-w-4xl mx-auto">
              AgentMem 致力于为 AI 代理提供最先进的记忆管理能力，让人工智能拥有真正的记忆和学习能力。
              我们相信，记忆是智能的基础，而智能记忆管理将推动 AI 应用的下一次革命。
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Target className="h-12 w-12 text-purple-400 mx-auto mb-4" />
                <CardTitle className="text-white">我们的使命</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>
                  让每个 AI 代理都拥有智能的记忆能力，
                  推动人工智能向更高层次的智能发展。
                </p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Globe className="h-12 w-12 text-blue-400 mx-auto mb-4" />
                <CardTitle className="text-white">我们的愿景</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>
                  成为全球领先的智能记忆管理平台，
                  为 AI 时代的到来奠定坚实基础。
                </p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Award className="h-12 w-12 text-yellow-400 mx-auto mb-4" />
                <CardTitle className="text-white">我们的价值观</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>
                  创新、开放、可靠、高效。
                  我们致力于构建最优秀的技术产品。
                </p>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* 团队介绍 */}
        <section className="mb-20">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <Users className="h-10 w-10 text-purple-400 mr-4" />
              核心团队
            </h2>
            <p className="text-xl text-slate-300">
              由经验丰富的 AI 研究者和工程师组成的世界级团队
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-purple-400 to-pink-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">LZ</span>
                </div>
                <CardTitle className="text-white">张磊</CardTitle>
                <CardDescription className="text-slate-300">创始人 & CEO</CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4">
                  前 Google AI 研究员，专注于大语言模型和记忆系统研究 8 年，
                  发表顶级会议论文 20+ 篇。
                </p>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-blue-400 to-cyan-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">WY</span>
                </div>
                <CardTitle className="text-white">王宇</CardTitle>
                <CardDescription className="text-slate-300">CTO & 联合创始人</CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4">
                  前 OpenAI 高级工程师，Rust 生态核心贡献者，
                  在分布式系统和高性能计算领域有丰富经验。
                </p>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-green-400 to-emerald-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">LM</span>
                </div>
                <CardTitle className="text-white">李明</CardTitle>
                <CardDescription className="text-slate-300">首席科学家</CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4">
                  斯坦福大学 AI 博士，专注于认知科学和记忆模型研究，
                  在 Nature、Science 等顶级期刊发表论文 15+ 篇。
                </p>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* 商业化方向 */}
        <section className="mb-20">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <Building className="h-10 w-10 text-blue-400 mr-4" />
              商业化方向
            </h2>
            <p className="text-xl text-slate-300">
              多元化的商业模式，服务不同规模的客户需求
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">企业级解决方案</CardTitle>
                <CardDescription className="text-slate-300">
                  为大型企业提供定制化的智能记忆管理解决方案
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-purple-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">私有化部署</h4>
                      <p className="text-sm">支持本地部署，确保数据安全和合规性</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-purple-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">定制开发</h4>
                      <p className="text-sm">根据企业需求定制特定功能和集成方案</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-purple-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">技术支持</h4>
                      <p className="text-sm">7x24 小时技术支持和专业咨询服务</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-purple-500/20 text-purple-300 mr-2">企业级</Badge>
                  <Badge className="bg-blue-500/20 text-blue-300">定制化</Badge>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">云服务平台</CardTitle>
                <CardDescription className="text-slate-300">
                  基于云的 SaaS 服务，快速集成和部署
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-blue-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">API 服务</h4>
                      <p className="text-sm">RESTful API 和 SDK，支持多种编程语言</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-blue-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">弹性扩展</h4>
                      <p className="text-sm">根据使用量自动扩展，按需付费</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-blue-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">全球部署</h4>
                      <p className="text-sm">多地域部署，确保低延迟和高可用性</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-blue-500/20 text-blue-300 mr-2">云原生</Badge>
                  <Badge className="bg-green-500/20 text-green-300">按需付费</Badge>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">开发者生态</CardTitle>
                <CardDescription className="text-slate-300">
                  构建开放的开发者生态系统和社区
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">开源项目</h4>
                      <p className="text-sm">核心组件开源，促进社区发展和创新</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">开发者工具</h4>
                      <p className="text-sm">提供完整的开发工具链和调试工具</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">社区支持</h4>
                      <p className="text-sm">活跃的开发者社区和技术交流平台</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-green-500/20 text-green-300 mr-2">开源</Badge>
                  <Badge className="bg-yellow-500/20 text-yellow-300">社区驱动</Badge>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">行业解决方案</CardTitle>
                <CardDescription className="text-slate-300">
                  针对特定行业的垂直解决方案
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-yellow-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">金融科技</h4>
                      <p className="text-sm">智能客服、风险评估、投资顾问等应用</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-yellow-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">医疗健康</h4>
                      <p className="text-sm">医疗问答、病历管理、诊断辅助等场景</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-yellow-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">教育培训</h4>
                      <p className="text-sm">个性化学习、知识管理、智能辅导等应用</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-yellow-500/20 text-yellow-300 mr-2">垂直行业</Badge>
                  <Badge className="bg-red-500/20 text-red-300">专业化</Badge>
                </div>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* 未来规划 */}
        <section className="mb-20">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <TrendingUp className="h-10 w-10 text-green-400 mr-4" />
              未来规划
            </h2>
            <p className="text-xl text-slate-300">
              持续创新，引领智能记忆管理技术的发展
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <div className="w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center mb-4">
                  <span className="text-2xl font-bold text-purple-400">2024</span>
                </div>
                <CardTitle className="text-white">技术突破年</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 发布 AgentMem 3.0</li>
                  <li>• 多模态记忆支持</li>
                  <li>• 实时学习能力</li>
                  <li>• 联邦学习框架</li>
                  <li>• 边缘计算支持</li>
                </ul>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <div className="w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center mb-4">
                  <span className="text-2xl font-bold text-blue-400">2025</span>
                </div>
                <CardTitle className="text-white">生态扩展年</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 全球市场拓展</li>
                  <li>• 合作伙伴生态</li>
                  <li>• 行业标准制定</li>
                  <li>• 开发者大会</li>
                  <li>• 认证培训体系</li>
                </ul>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                  <span className="text-2xl font-bold text-green-400">2026</span>
                </div>
                <CardTitle className="text-white">智能革命年</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• AGI 记忆系统</li>
                  <li>• 量子计算集成</li>
                  <li>• 脑机接口研究</li>
                  <li>• 通用智能平台</li>
                  <li>• 下一代 AI 基础设施</li>
                </ul>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* 投资与合作 */}
        <section className="mb-20">
          <Card className="bg-gradient-to-r from-purple-600/20 to-pink-600/20 border-purple-500/30">
            <CardHeader className="text-center">
              <Rocket className="h-16 w-16 text-purple-400 mx-auto mb-4" />
              <CardTitle className="text-white text-3xl mb-4">
                加入我们的旅程
              </CardTitle>
              <CardDescription className="text-slate-300 text-lg">
                我们正在寻找志同道合的投资者、合作伙伴和人才
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
                <div className="text-center">
                  <div className="text-3xl font-bold text-purple-400 mb-2">A 轮</div>
                  <div className="text-slate-300">融资进行中</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-blue-400 mb-2">50+</div>
                  <div className="text-slate-300">合作伙伴</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-green-400 mb-2">100+</div>
                  <div className="text-slate-300">团队成员</div>
                </div>
              </div>
              
              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button size="lg" className="bg-purple-600 hover:bg-purple-700">
                  投资合作
                </Button>
                <Button size="lg" variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                  商务合作
                </Button>
                <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                  加入团队
                </Button>
              </div>
            </CardContent>
          </Card>
        </section>

        {/* 联系我们 */}
        <section>
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <Mail className="h-10 w-10 text-blue-400 mr-4" />
              联系我们
            </h2>
            <p className="text-xl text-slate-300">
              我们期待与您的交流和合作
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Mail className="h-8 w-8 text-blue-400 mx-auto mb-2" />
                <CardTitle className="text-white">商务合作</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>business@agentmem.com</p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Users className="h-8 w-8 text-green-400 mx-auto mb-2" />
                <CardTitle className="text-white">技术支持</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>support@agentmem.com</p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Building className="h-8 w-8 text-purple-400 mx-auto mb-2" />
                <CardTitle className="text-white">媒体合作</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>media@agentmem.com</p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Rocket className="h-8 w-8 text-yellow-400 mx-auto mb-2" />
                <CardTitle className="text-white">投资合作</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>invest@agentmem.com</p>
              </CardContent>
            </Card>
          </div>
        </section>
      </div>
    </div>
  );
}