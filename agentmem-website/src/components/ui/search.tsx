"use client";

import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Search, X, FileText, Code, Book } from "lucide-react";
import { cn } from "@/lib/utils";
import Link from "next/link";

interface SearchResult {
  id: string;
  title: string;
  content: string;
  url: string;
  type: 'doc' | 'api' | 'example';
}

/**
 * 搜索组件 - 支持文档、API和示例搜索
 */
export function SearchDialog() {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // 模拟搜索数据
  const searchData: SearchResult[] = [
    {
      id: '1',
      title: '快速开始',
      content: '使用 Cargo 安装 AgentMem 核心库，配置环境变量',
      url: '/docs#quick-start',
      type: 'doc'
    },
    {
      id: '2',
      title: 'MemoryEngine API',
      content: 'MemoryEngine::new() 创建记忆引擎实例',
      url: '/docs#api-reference',
      type: 'api'
    },
    {
      id: '3',
      title: '智能推理引擎',
      content: 'DeepSeek 驱动的事实提取和记忆决策示例',
      url: '/demo#intelligent-reasoning',
      type: 'example'
    },
    {
      id: '4',
      title: 'Mem0 兼容性',
      content: '100% API 兼容，支持无缝迁移',
      url: '/demo#mem0-compat',
      type: 'example'
    },
    {
      id: '5',
      title: '架构设计',
      content: '13个模块化 Crate，分层架构设计',
      url: '/#architecture',
      type: 'doc'
    }
  ];

  /**
   * 执行搜索
   */
  const performSearch = (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([]);
      return;
    }

    setIsLoading(true);
    
    // 模拟搜索延迟
    setTimeout(() => {
      const filtered = searchData.filter(item => 
        item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.content.toLowerCase().includes(searchQuery.toLowerCase())
      );
      setResults(filtered);
      setIsLoading(false);
    }, 300);
  };

  /**
   * 处理搜索输入
   */
  const handleSearch = (value: string) => {
    setQuery(value);
    performSearch(value);
  };

  /**
   * 打开搜索对话框
   */
  const openSearch = () => {
    setIsOpen(true);
    setTimeout(() => inputRef.current?.focus(), 100);
  };

  /**
   * 关闭搜索对话框
   */
  const closeSearch = () => {
    setIsOpen(false);
    setQuery('');
    setResults([]);
  };

  /**
   * 键盘快捷键
   */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        openSearch();
      }
      if (e.key === 'Escape') {
        closeSearch();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  /**
   * 获取结果类型图标
   */
  const getTypeIcon = (type: SearchResult['type']) => {
    switch (type) {
      case 'doc':
        return <Book className="h-4 w-4 text-blue-400" />;
      case 'api':
        return <Code className="h-4 w-4 text-green-400" />;
      case 'example':
        return <FileText className="h-4 w-4 text-purple-400" />;
    }
  };

  return (
    <>
      {/* 搜索触发按钮 */}
      <Button
        variant="outline"
        className="border-slate-600 text-slate-300 hover:bg-slate-800 w-64 justify-start"
        onClick={openSearch}
      >
        <Search className="h-4 w-4 mr-2" />
        <span className="text-sm">搜索文档...</span>
        <kbd className="ml-auto text-xs bg-slate-700 px-2 py-1 rounded">
          ⌘K
        </kbd>
      </Button>

      {/* 搜索对话框 */}
      {isOpen && (
        <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={closeSearch}>
          <div className="fixed top-20 left-1/2 transform -translate-x-1/2 w-full max-w-2xl mx-4">
            <Card className="bg-slate-800 border-slate-700" onClick={e => e.stopPropagation()}>
              <div className="flex items-center border-b border-slate-700 p-4">
                <Search className="h-5 w-5 text-slate-400 mr-3" />
                <input
                  ref={inputRef}
                  type="text"
                  placeholder="搜索文档、API 和示例..."
                  value={query}
                  onChange={(e) => handleSearch(e.target.value)}
                  className="flex-1 bg-transparent text-white placeholder-slate-400 outline-none"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={closeSearch}
                  className="text-slate-400 hover:text-white"
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
              
              <CardContent className="p-0 max-h-96 overflow-y-auto">
                {isLoading ? (
                  <div className="p-4 text-center text-slate-400">
                    搜索中...
                  </div>
                ) : results.length > 0 ? (
                  <div className="py-2">
                    {results.map((result) => (
                      <Link
                        key={result.id}
                        href={result.url}
                        onClick={closeSearch}
                        className="block px-4 py-3 hover:bg-slate-700 transition-colors"
                      >
                        <div className="flex items-start space-x-3">
                          {getTypeIcon(result.type)}
                          <div className="flex-1 min-w-0">
                            <div className="text-white font-medium">
                              {result.title}
                            </div>
                            <div className="text-slate-400 text-sm mt-1 line-clamp-2">
                              {result.content}
                            </div>
                          </div>
                        </div>
                      </Link>
                    ))}
                  </div>
                ) : query ? (
                  <div className="p-4 text-center text-slate-400">
                    未找到相关结果
                  </div>
                ) : (
                  <div className="p-4 text-center text-slate-400">
                    输入关键词开始搜索
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      )}
    </>
  );
}