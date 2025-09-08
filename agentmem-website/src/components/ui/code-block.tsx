"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Copy, Check } from "lucide-react";
import { cn } from "@/lib/utils";

interface CodeBlockProps {
  code: string;
  language?: string;
  className?: string;
  showCopy?: boolean;
}

/**
 * 代码块组件 - 支持语法高亮和复制功能
 */
export function CodeBlock({ 
  code, 
  language = "rust", 
  className, 
  showCopy = true 
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  /**
   * 复制代码到剪贴板
   */
  const copyCode = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  };

  return (
    <div className={cn("relative group", className)}>
      <pre className="bg-slate-900 text-slate-100 p-4 rounded-lg overflow-x-auto text-sm">
        <code className={`language-${language}`}>
          {code}
        </code>
      </pre>
      {showCopy && (
        <Button
          size="sm"
          variant="outline"
          className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity border-slate-600 bg-slate-800 hover:bg-slate-700"
          onClick={copyCode}
        >
          {copied ? (
            <Check className="h-4 w-4 text-green-400" />
          ) : (
            <Copy className="h-4 w-4" />
          )}
        </Button>
      )}
    </div>
  );
}

/**
 * 内联代码组件
 */
export function InlineCode({ 
  children, 
  className 
}: { 
  children: React.ReactNode; 
  className?: string; 
}) {
  return (
    <code className={cn(
      "bg-slate-800 text-purple-300 px-2 py-1 rounded text-sm font-mono",
      className
    )}>
      {children}
    </code>
  );
}