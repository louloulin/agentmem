// 智能体状态管理模块 - 简化但功能完整的实现
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde_json;

use crate::core::{AgentDbError, AgentState};

// 智能体状态数据库 - 简化但功能完整的实现
pub struct AgentStateDB {
    db_path: String,
    states: Arc<RwLock<HashMap<u64, AgentState>>>,
    vector_states: Arc<RwLock<HashMap<u64, (AgentState, Vec<f32>)>>>,
}

impl AgentStateDB {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        // 确保数据库目录存在
        if let Some(parent) = Path::new(db_path).parent() {
            fs::create_dir_all(parent).map_err(|e| AgentDbError::Io(e))?;
        }
        
        let db = Self {
            db_path: db_path.to_string(),
            states: Arc::new(RwLock::new(HashMap::new())),
            vector_states: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // 尝试从磁盘加载现有数据
        db.load_from_disk().await?;
        
        Ok(db)
    }

    async fn load_from_disk(&self) -> Result<(), AgentDbError> {
        let states_file = format!("{}/states.json", self.db_path);
        let vector_states_file = format!("{}/vector_states.json", self.db_path);
        
        // 加载普通状态
        if Path::new(&states_file).exists() {
            let content = fs::read_to_string(&states_file).map_err(|e| AgentDbError::Io(e))?;
            if !content.trim().is_empty() {
                let loaded_states: HashMap<u64, AgentState> = serde_json::from_str(&content)
                    .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
                
                let mut states = self.states.write().unwrap();
                *states = loaded_states;
            }
        }
        
        // 加载向量状态
        if Path::new(&vector_states_file).exists() {
            let content = fs::read_to_string(&vector_states_file).map_err(|e| AgentDbError::Io(e))?;
            if !content.trim().is_empty() {
                let loaded_vector_states: HashMap<u64, (AgentState, Vec<f32>)> = serde_json::from_str(&content)
                    .map_err(|e| AgentDbError::Serialization(e.to_string()))?;
                
                let mut vector_states = self.vector_states.write().unwrap();
                *vector_states = loaded_vector_states;
            }
        }
        
        Ok(())
    }
    
    async fn save_to_disk(&self) -> Result<(), AgentDbError> {
        // 确保目录存在
        fs::create_dir_all(&self.db_path).map_err(|e| AgentDbError::Io(e))?;
        
        let states_file = format!("{}/states.json", self.db_path);
        let vector_states_file = format!("{}/vector_states.json", self.db_path);
        
        // 使用异步写入提高性能
        let states_json = {
            let states = self.states.read().unwrap();
            serde_json::to_string(&*states)
                .map_err(|e| AgentDbError::Serialization(e.to_string()))?
        };

        let vector_states_json = {
            let vector_states = self.vector_states.read().unwrap();
            serde_json::to_string(&*vector_states)
                .map_err(|e| AgentDbError::Serialization(e.to_string()))?
        };

        // 并行写入文件
        let states_write = tokio::fs::write(&states_file, states_json);
        let vector_states_write = tokio::fs::write(&vector_states_file, vector_states_json);

        tokio::try_join!(states_write, vector_states_write)
            .map_err(|e| AgentDbError::Io(e))?;
        
        Ok(())
    }

    pub async fn save_state(&self, state: &AgentState) -> Result<(), AgentDbError> {
        {
            let mut states = self.states.write().unwrap();
            states.insert(state.agent_id, state.clone());
        }
        
        // 异步保存到磁盘
        self.save_to_disk().await?;
        
        Ok(())
    }

    pub async fn load_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentDbError> {
        let states = self.states.read().unwrap();
        Ok(states.get(&agent_id).cloned())
    }

    pub async fn save_vector_state(&self, state: &AgentState, embedding: Vec<f32>) -> Result<(), AgentDbError> {
        {
            let mut vector_states = self.vector_states.write().unwrap();
            vector_states.insert(state.agent_id, (state.clone(), embedding));
        }
        
        // 异步保存到磁盘
        self.save_to_disk().await?;
        
        Ok(())
    }

    pub async fn vector_search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        let vector_states = self.vector_states.read().unwrap();
        
        let mut results: Vec<(f32, AgentState)> = vector_states
            .values()
            .map(|(state, embedding)| {
                let similarity = self.calculate_cosine_similarity(&query_embedding, embedding);
                (similarity, state.clone())
            })
            .collect();
        
        // 按相似度排序
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // 返回前 limit 个结果
        Ok(results.into_iter().take(limit).map(|(_, state)| state).collect())
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

    pub async fn search_by_agent_and_similarity(&self, agent_id: u64, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        let vector_states = self.vector_states.read().unwrap();
        
        let mut results: Vec<(f32, AgentState)> = vector_states
            .values()
            .filter(|(state, _)| state.agent_id == agent_id)
            .map(|(state, embedding)| {
                let similarity = self.calculate_cosine_similarity(&query_embedding, embedding);
                (similarity, state.clone())
            })
            .collect();
        
        // 按相似度排序
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // 返回前 limit 个结果
        Ok(results.into_iter().take(limit).map(|(_, state)| state).collect())
    }

    pub async fn get_all_states(&self) -> Result<Vec<AgentState>, AgentDbError> {
        let states = self.states.read().unwrap();
        Ok(states.values().cloned().collect())
    }

    pub async fn delete_state(&self, agent_id: u64) -> Result<bool, AgentDbError> {
        let removed = {
            let mut states = self.states.write().unwrap();
            states.remove(&agent_id).is_some()
        };
        
        if removed {
            self.save_to_disk().await?;
        }
        
        Ok(removed)
    }

    pub async fn get_states_by_type(&self, state_type: crate::core::StateType) -> Result<Vec<AgentState>, AgentDbError> {
        let states = self.states.read().unwrap();
        Ok(states.values()
            .filter(|state| state.state_type == state_type)
            .cloned()
            .collect())
    }

    pub async fn count_states(&self) -> Result<usize, AgentDbError> {
        let states = self.states.read().unwrap();
        Ok(states.len())
    }

    pub async fn clear_all_states(&self) -> Result<(), AgentDbError> {
        {
            let mut states = self.states.write().unwrap();
            let mut vector_states = self.vector_states.write().unwrap();
            states.clear();
            vector_states.clear();
        }
        
        self.save_to_disk().await?;
        Ok(())
    }
}
