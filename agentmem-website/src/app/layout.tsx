import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

/**
 * 网站元数据配置
 */
export const metadata: Metadata = {
  title: {
    default: "AgentMem - 智能记忆管理平台",
    template: "%s | AgentMem"
  },
  description: "为 AI 代理提供先进的记忆处理能力。基于 Rust 的模块化架构，集成 DeepSeek 智能推理引擎，支持多存储后端，100% Mem0 兼容。",
  keywords: ["AI", "记忆管理", "智能代理", "Rust", "DeepSeek", "Mem0", "向量数据库", "机器学习"],
  authors: [{ name: "AgentMem Team" }],
  creator: "AgentMem Team",
  publisher: "AgentMem",
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
  openGraph: {
    type: "website",
    locale: "zh_CN",
    url: "https://agentmem.com",
    title: "AgentMem - 智能记忆管理平台",
    description: "为 AI 代理提供先进的记忆处理能力。基于 Rust 的模块化架构，集成 DeepSeek 智能推理引擎。",
    siteName: "AgentMem",
  },
  twitter: {
    card: "summary_large_image",
    title: "AgentMem - 智能记忆管理平台",
    description: "为 AI 代理提供先进的记忆处理能力。基于 Rust 的模块化架构，集成 DeepSeek 智能推理引擎。",
    creator: "@agentmem",
  },
};

/**
 * 视口配置
 */
export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 1,
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "white" },
    { media: "(prefers-color-scheme: dark)", color: "#8b5cf6" },
  ],
};

/**
 * 根布局组件
 * @param children - 子组件
 */
export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="zh-CN" className="dark">
      <head>
        <link rel="icon" href="/favicon.ico" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
        <meta name="theme-color" content="#8b5cf6" />
      </head>
      <body className={`${inter.className} antialiased`}>
        <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
          {children}
        </div>
      </body>
    </html>
  );
}
