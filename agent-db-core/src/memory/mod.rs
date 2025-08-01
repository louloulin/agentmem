// 记忆管理模块 - 简化但功能完整的实现
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::fs;
use serde_json;

use crate::core::{AgentDbError, Memory, MemoryType};

// 记忆管理器 - 简化实现
pub struct MemoryManager {
    db_path: String,
    memories: Arc<RwLock<HashMap<String, Memory>>>,
}

impl MemoryManager {
    pub fn new(db_path: &str) -> Self {
        Self { 
            db_path: db_path.to_string(),
            memories: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn init(&mut self) -> Result<(), AgentDbError> {
        // 确保目录存在
        if let Some(parent) = Path::new(&self.db_path).parent() {
            fs::create_dir_all(parent).map_err(|e| AgentDbError::Io(e))?;
        }
        
        // 加载现有记忆
        self.load_from_disk().await?;
        
        Ok(())
    }

    async fn load_from_disk(&self) -> Result<(), AgentDbError> {
        let memories_file = format!("{}/memories.json", self.db_path);
        
        if Path::new(&memories_file).exists() {
            let content = fs::read_to_string(&memories_file).map_err(|e| AgentDbError::Io(e))?;
            if !content.trim().is_empty() {
                let loaded_memories: HashMap<String, Memory> = serde_json::from_str(&content)
                    .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
                
                let mut memories = self.memories.write().unwrap();
                *memories = loaded_memories;
            }
        }
        
        Ok(())
    }
    
    async fn save_to_disk(&self) -> Result<(), AgentDbError> {
        // 确保目录存在
        fs::create_dir_all(&self.db_path).map_err(|e| AgentDbError::Io(e))?;
        
        let memories_file = format!("{}/memories.json", self.db_path);
        
        let memories = self.memories.read().unwrap();
        let content = serde_json::to_string_pretty(&*memories)
            .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
        fs::write(&memories_file, content).map_err(|e| AgentDbError::Io(e))?;
        
        Ok(())
    }

    pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentDbError> {
        {
            let mut memories = self.memories.write().unwrap();
            memories.insert(memory.memory_id.clone(), memory.clone());
        }
        
        self.save_to_disk().await?;
        Ok(())
    }

    pub async fn get_memories_by_agent(&self, agent_id: u64) -> Result<Vec<Memory>, AgentDbError> {
        let memories = self.memories.read().unwrap();
        
        let mut agent_memories: Vec<Memory> = memories
            .values()
            .filter(|memory| memory.agent_id == agent_id)
            .cloned()
            .collect();
        
        // 按创建时间排序
        agent_memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(agent_memories)
    }
    
    pub async fn search_memories_by_importance(&self, agent_id: u64, min_importance: f32, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let memories = self.memories.read().unwrap();
        
        let mut filtered_memories: Vec<Memory> = memories
            .values()
            .filter(|memory| memory.agent_id == agent_id && memory.importance >= min_importance)
            .cloned()
            .collect();
        
        // 按重要性排序
        filtered_memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        
        // 限制结果数量
        filtered_memories.truncate(limit);
        
        Ok(filtered_memories)
    }

    pub async fn search_memories_by_content(&self, agent_id: u64, query: &str, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let memories = self.memories.read().unwrap();
        let query_lower = query.to_lowercase();
        
        let mut matching_memories: Vec<(f32, Memory)> = memories
            .values()
            .filter(|memory| memory.agent_id == agent_id)
            .filter_map(|memory| {
                let content_lower = memory.content.to_lowercase();
                if content_lower.contains(&query_lower) {
                    // 简单的文本匹配评分
                    let score = self.calculate_text_score(&query_lower, &content_lower);
                    Some((score, memory.clone()))
                } else {
                    None
                }
            })
            .collect();
        
        // 按评分排序
        matching_memories.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // 限制结果数量并返回
        Ok(matching_memories.into_iter().take(limit).map(|(_, memory)| memory).collect())
    }

    fn calculate_text_score(&self, query: &str, content: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let content_words: Vec<&str> = content.split_whitespace().collect();
        
        let mut score = 0.0;
        
        // 精确匹配加分
        if content.contains(query) {
            score += 2.0;
        }
        
        // 单词匹配加分
        for query_word in &query_words {
            for content_word in &content_words {
                if content_word.contains(query_word) {
                    score += 1.0;
                }
            }
        }
        
        // 根据内容长度调整分数
        score / (content_words.len() as f32).max(1.0)
    }

    pub async fn update_memory_access(&self, memory_id: &str) -> Result<(), AgentDbError> {
        {
            let mut memories = self.memories.write().unwrap();
            if let Some(memory) = memories.get_mut(memory_id) {
                memory.access_count += 1;
                memory.last_access = chrono::Utc::now().timestamp();
            }
        }
        
        self.save_to_disk().await?;
        Ok(())
    }

    pub async fn delete_memory(&self, memory_id: &str) -> Result<bool, AgentDbError> {
        let removed = {
            let mut memories = self.memories.write().unwrap();
            memories.remove(memory_id).is_some()
        };
        
        if removed {
            self.save_to_disk().await?;
        }
        
        Ok(removed)
    }

    pub async fn get_memory_by_id(&self, memory_id: &str) -> Result<Option<Memory>, AgentDbError> {
        let memories = self.memories.read().unwrap();
        Ok(memories.get(memory_id).cloned())
    }

    pub async fn get_memories_by_type(&self, agent_id: u64, memory_type: MemoryType) -> Result<Vec<Memory>, AgentDbError> {
        let memories = self.memories.read().unwrap();
        
        let filtered_memories: Vec<Memory> = memories
            .values()
            .filter(|memory| memory.agent_id == agent_id && memory.memory_type == memory_type)
            .cloned()
            .collect();
        
        Ok(filtered_memories)
    }

    pub async fn cleanup_expired_memories(&self) -> Result<usize, AgentDbError> {
        let current_time = chrono::Utc::now().timestamp();
        let mut removed_count = 0;
        
        {
            let mut memories = self.memories.write().unwrap();
            let initial_count = memories.len();
            
            memories.retain(|_, memory| {
                if let Some(expires_at) = memory.expires_at {
                    expires_at > current_time
                } else {
                    true
                }
            });
            
            removed_count = initial_count - memories.len();
        }
        
        if removed_count > 0 {
            self.save_to_disk().await?;
        }
        
        Ok(removed_count)
    }

    pub async fn count_memories(&self, agent_id: Option<u64>) -> Result<usize, AgentDbError> {
        let memories = self.memories.read().unwrap();
        
        if let Some(agent_id) = agent_id {
            Ok(memories.values().filter(|memory| memory.agent_id == agent_id).count())
        } else {
            Ok(memories.len())
        }
    }

    pub async fn clear_all_memories(&self) -> Result<(), AgentDbError> {
        {
            let mut memories = self.memories.write().unwrap();
            memories.clear();
        }
        
        self.save_to_disk().await?;
        Ok(())
    }
}
