// RAG 引擎模块 - 简化但功能完整的实现
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::core::{AgentDbError, Document, SearchResult};

// RAG 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_results: usize,
    pub similarity_threshold: f32,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 200,
            max_results: 10,
            similarity_threshold: 0.7,
        }
    }
}

// 文档存储数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentData {
    pub document: Document,
    pub indexed_at: i64,
}

// RAG 引擎 - 简化实现
pub struct RAGEngine {
    db_path: String,
    config: RAGConfig,
    documents: Arc<RwLock<HashMap<String, DocumentData>>>,
}

impl RAGEngine {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let engine = Self {
            db_path: db_path.to_string(),
            config: RAGConfig::default(),
            documents: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // 加载现有文档
        engine.load_from_disk().await?;
        
        Ok(engine)
    }

    pub async fn new_with_config(db_path: &str, config: RAGConfig) -> Result<Self, AgentDbError> {
        let engine = Self {
            db_path: db_path.to_string(),
            config,
            documents: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // 加载现有文档
        engine.load_from_disk().await?;
        
        Ok(engine)
    }

    async fn load_from_disk(&self) -> Result<(), AgentDbError> {
        let documents_file = format!("{}/documents.json", self.db_path);
        
        if Path::new(&documents_file).exists() {
            let content = fs::read_to_string(&documents_file).map_err(|e| AgentDbError::Io(e))?;
            if !content.trim().is_empty() {
                let loaded_documents: HashMap<String, DocumentData> = serde_json::from_str(&content)
                    .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
                
                let mut documents = self.documents.write().unwrap();
                *documents = loaded_documents;
            }
        }
        
        Ok(())
    }
    
    async fn save_to_disk(&self) -> Result<(), AgentDbError> {
        // 确保目录存在
        if let Some(parent) = Path::new(&self.db_path).parent() {
            fs::create_dir_all(parent).map_err(|e| AgentDbError::Io(e))?;
        }
        
        let documents_file = format!("{}/documents.json", self.db_path);
        
        let documents = self.documents.read().unwrap();
        let content = serde_json::to_string_pretty(&*documents)
            .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
        fs::write(&documents_file, content).map_err(|e| AgentDbError::Io(e))?;
        
        Ok(())
    }

    pub async fn index_document(&self, document: &Document) -> Result<String, AgentDbError> {
        let document_data = DocumentData {
            document: document.clone(),
            indexed_at: chrono::Utc::now().timestamp(),
        };
        
        {
            let mut documents = self.documents.write().unwrap();
            documents.insert(document.doc_id.clone(), document_data);
        }

        self.save_to_disk().await?;
        Ok(document.doc_id.clone())
    }

    pub async fn search_by_text(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        let documents = self.documents.read().unwrap();
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<SearchResult> = Vec::new();
        
        for document_data in documents.values() {
            let document = &document_data.document;
            
            // 搜索文档标题和内容
            let title_score = self.calculate_text_score(&query_lower, &document.title.to_lowercase());
            let content_score = self.calculate_text_score(&query_lower, &document.content.to_lowercase());
            
            if title_score > 0.0 || content_score > 0.0 {
                let total_score = title_score * 2.0 + content_score; // 标题权重更高
                
                results.push(SearchResult {
                    document_id: document.doc_id.clone(),
                    chunk_id: None,
                    doc_id: document.doc_id.clone(),
                    score: total_score,
                    content: if title_score > content_score {
                        document.title.clone()
                    } else {
                        self.extract_relevant_content(&document.content, query, 200)
                    },
                    metadata: document.metadata.clone(),
                });
            }
            
            // 搜索文档块
            for chunk in &document.chunks {
                let chunk_score = self.calculate_text_score(&query_lower, &chunk.content.to_lowercase());
                
                if chunk_score > 0.0 {
                    results.push(SearchResult {
                        document_id: document.doc_id.clone(),
                        chunk_id: Some(chunk.chunk_id.clone()),
                        doc_id: document.doc_id.clone(),
                        score: chunk_score,
                        content: self.extract_relevant_content(&chunk.content, query, 200),
                        metadata: document.metadata.clone(),
                    });
                }
            }
        }
        
        // 按分数排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // 限制结果数量
        results.truncate(limit);
        
        Ok(results)
    }

    pub async fn semantic_search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        let documents = self.documents.read().unwrap();
        let mut results: Vec<SearchResult> = Vec::new();
        
        for document_data in documents.values() {
            let document = &document_data.document;
            
            // 搜索有嵌入向量的文档块
            for chunk in &document.chunks {
                if let Some(ref chunk_embedding) = chunk.embedding {
                    let similarity = self.calculate_cosine_similarity(&query_embedding, chunk_embedding);

                    if similarity >= self.config.similarity_threshold {
                        results.push(SearchResult {
                            document_id: document.doc_id.clone(),
                            chunk_id: Some(chunk.chunk_id.clone()),
                            doc_id: document.doc_id.clone(),
                            score: similarity,
                            content: chunk.content.clone(),
                            metadata: document.metadata.clone(),
                        });
                    }
                }
            }
        }
        
        // 按相似度排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // 限制结果数量
        results.truncate(limit);
        
        Ok(results)
    }

    fn calculate_text_score(&self, query: &str, content: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let content_words: Vec<&str> = content.split_whitespace().collect();
        
        let mut score = 0.0;
        
        // 精确匹配加分
        if content.contains(query) {
            score += 3.0;
        }
        
        // 单词匹配加分
        for query_word in &query_words {
            for content_word in &content_words {
                if content_word.contains(query_word) {
                    score += 1.0;
                } else if query_word.contains(content_word) {
                    score += 0.5;
                }
            }
        }
        
        // 根据匹配词数量调整分数
        let match_ratio = score / (query_words.len() as f32);
        
        // 根据内容长度调整分数（避免过长内容得分过高）
        let length_penalty = 1.0 / (1.0 + (content_words.len() as f32 / 100.0));
        
        match_ratio * length_penalty
    }
    
    fn calculate_cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }

    fn extract_relevant_content(&self, content: &str, query: &str, max_length: usize) -> String {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        // 找到查询词在内容中的位置
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(max_length / 2);
            let end = (pos + query.len() + max_length / 2).min(content.len());
            
            let mut result = content[start..end].to_string();
            
            // 添加省略号
            if start > 0 {
                result = format!("...{}", result);
            }
            if end < content.len() {
                result = format!("{}...", result);
            }
            
            result
        } else {
            // 如果没有找到精确匹配，返回开头部分
            if content.len() <= max_length {
                content.to_string()
            } else {
                format!("{}...", &content[..max_length])
            }
        }
    }

    pub async fn get_document(&self, document_id: &str) -> Result<Option<Document>, AgentDbError> {
        let documents = self.documents.read().unwrap();
        Ok(documents.get(document_id).map(|data| data.document.clone()))
    }

    pub async fn delete_document(&self, document_id: &str) -> Result<bool, AgentDbError> {
        let removed = {
            let mut documents = self.documents.write().unwrap();
            documents.remove(document_id).is_some()
        };
        
        if removed {
            self.save_to_disk().await?;
        }
        
        Ok(removed)
    }

    pub async fn list_documents(&self) -> Result<Vec<String>, AgentDbError> {
        let documents = self.documents.read().unwrap();
        Ok(documents.keys().cloned().collect())
    }

    pub async fn count_documents(&self) -> Result<usize, AgentDbError> {
        let documents = self.documents.read().unwrap();
        Ok(documents.len())
    }

    pub async fn clear_all_documents(&self) -> Result<(), AgentDbError> {
        {
            let mut documents = self.documents.write().unwrap();
            documents.clear();
        }
        
        self.save_to_disk().await?;
        Ok(())
    }

    pub fn get_config(&self) -> &RAGConfig {
        &self.config
    }

    pub async fn update_config(&mut self, config: RAGConfig) -> Result<(), AgentDbError> {
        self.config = config;
        Ok(())
    }
}
