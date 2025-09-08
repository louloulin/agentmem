import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { LanguageProvider } from "@/components/ui/language-provider";
import { ThemeProvider } from "next-themes";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "AgentMem - 智能记忆管理平台",
  description: "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎",
  keywords: "AgentMem, 智能记忆, AI, Rust, DeepSeek, 记忆管理, 向量数据库",
  authors: [{ name: "AgentMem Team" }],
  robots: "index, follow",
  openGraph: {
    title: "AgentMem - 智能记忆管理平台",
    description: "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎",
    type: "website",
    locale: "zh_CN",
    alternateLocale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "AgentMem - 智能记忆管理平台",
    description: "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎",
  },
};

export const viewport = {
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="zh" suppressHydrationWarning>
      <body className={inter.className}>
        <ThemeProvider
          attribute="class"
          defaultTheme="dark"
          enableSystem
          disableTransitionOnChange
        >
          <LanguageProvider>
            {children}
          </LanguageProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
