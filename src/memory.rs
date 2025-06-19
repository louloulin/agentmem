// 记忆管理模块
use std::collections::HashMap;
use std::sync::Arc;
<<<<<<< HEAD
use arrow::array::{Array, Float32Array, Int64Array, StringArray, UInt32Array, UInt64Array, RecordBatchIterator};
=======
use arrow::array::{Array, BinaryArray, Float64Array, Int64Array, StringArray, UInt32Array, UInt64Array, RecordBatchIterator};
>>>>>>> origin/feature-module
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

<<<<<<< HEAD
use crate::types::{AgentDbError, Memory, MemoryType};

pub struct MemoryManager {
    connection: Arc<Connection>,
}

impl MemoryManager {
    pub fn new(connection: Arc<Connection>) -> Self {
=======
use crate::core::{AgentDbError, Memory, MemoryType};

// 记忆管理器
pub struct MemoryManager {
    connection: Connection,
}

impl MemoryManager {
    pub fn new(connection: Connection) -> Self {
>>>>>>> origin/feature-module
        Self { connection }
    }

    pub async fn ensure_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("memories").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                let schema = Schema::new(vec![
<<<<<<< HEAD
                    Field::new("memory_id", DataType::Utf8, false),
                    Field::new("agent_id", DataType::UInt64, false),
                    Field::new("memory_type", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("importance", DataType::Float32, false),
                    Field::new("created_at", DataType::Int64, false),
                    Field::new("access_count", DataType::UInt32, false),
                    Field::new("last_access", DataType::Int64, false),
                    Field::new("expires_at", DataType::Int64, true),
=======
                    Field::new("id", DataType::Utf8, false),
                    Field::new("agent_id", DataType::UInt64, false),
                    Field::new("memory_type", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("importance", DataType::Float64, false),
                    Field::new("timestamp", DataType::Int64, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("embedding", DataType::Binary, true),
                    Field::new("access_count", DataType::UInt32, false),
                    Field::new("last_accessed", DataType::Int64, false),
                    Field::new("expiry_time", DataType::Int64, true),
>>>>>>> origin/feature-module
                ]);

                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("memories", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;
<<<<<<< HEAD
        let schema = table.schema().await?;

        let expires_at_value = memory.expires_at.unwrap_or(-1);
=======

        let metadata_json = serde_json::to_string(&memory.metadata)?;
        let embedding_bytes = memory.embedding.as_ref()
            .map(|emb| serde_json::to_vec(emb).unwrap())
            .unwrap_or_default();

        let schema = table.schema().await?;
>>>>>>> origin/feature-module

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
<<<<<<< HEAD
                Arc::new(StringArray::from(vec![memory.memory_id.clone()])),
                Arc::new(UInt64Array::from(vec![memory.agent_id])),
                Arc::new(StringArray::from(vec![memory.memory_type.to_string()])),
                Arc::new(StringArray::from(vec![memory.content.clone()])),
                Arc::new(Float32Array::from(vec![memory.importance])),
                Arc::new(Int64Array::from(vec![memory.created_at])),
                Arc::new(UInt32Array::from(vec![memory.access_count])),
                Arc::new(Int64Array::from(vec![memory.last_access])),
                Arc::new(Int64Array::from(vec![expires_at_value])),
=======
                Arc::new(StringArray::from(vec![memory.id.clone()])),
                Arc::new(UInt64Array::from(vec![memory.agent_id])),
                Arc::new(StringArray::from(vec![memory.memory_type.to_string()])),
                Arc::new(StringArray::from(vec![memory.content.clone()])),
                Arc::new(Float64Array::from(vec![memory.importance])),
                Arc::new(Int64Array::from(vec![memory.timestamp])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(BinaryArray::from(vec![embedding_bytes.as_slice()])),
                Arc::new(UInt32Array::from(vec![memory.access_count])),
                Arc::new(Int64Array::from(vec![memory.last_accessed])),
                Arc::new(Int64Array::from(vec![memory.expiry_time.unwrap_or(-1)])),
>>>>>>> origin/feature-module
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    pub async fn retrieve_memory(&self, memory_id: &str) -> Result<Option<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
<<<<<<< HEAD
            .only_if(&format!("memory_id = '{}'", memory_id))
=======
            .only_if(&format!("id = '{}'", memory_id))
>>>>>>> origin/feature-module
            .limit(1)
            .execute()
            .await?;

        let batch = match results.try_next().await? {
            Some(batch) => batch,
            None => return Ok(None),
        };

        if batch.num_rows() == 0 {
            return Ok(None);
        }

<<<<<<< HEAD
        let memory = self.batch_to_memory(&batch, 0)?;
        Ok(Some(memory))
    }

    pub async fn get_agent_memories(
        &self,
        agent_id: u64,
        memory_type: Option<MemoryType>,
        limit: usize,
    ) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        let filter = if let Some(mem_type) = memory_type {
            format!("agent_id = {} AND memory_type = '{}'", agent_id, mem_type.to_string())
        } else {
            format!("agent_id = {}", agent_id)
        };

        let mut results = table
            .query()
            .only_if(&filter)
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.batch_to_memory(&batch, row)?;
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    pub async fn update_memory_access(&self, memory_id: &str) -> Result<(), AgentDbError> {
        if let Some(mut memory) = self.retrieve_memory(memory_id).await? {
            memory.access();
            
            // 删除旧记录
            let table = self.ensure_table().await?;
            table.delete(&format!("memory_id = '{}'", memory_id)).await?;
            
            // 插入更新后的记录
            self.store_memory(&memory).await?;
        }
        Ok(())
    }

    pub async fn delete_memory(&self, memory_id: &str) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;
        table.delete(&format!("memory_id = '{}'", memory_id)).await?;
        Ok(())
    }

    pub async fn cleanup_expired_memories(&self) -> Result<usize, AgentDbError> {
        let current_time = chrono::Utc::now().timestamp();
        let table = self.ensure_table().await?;
        
        // 获取过期的记忆
        let mut results = table
            .query()
            .only_if(&format!("expires_at > 0 AND expires_at < {}", current_time))
            .execute()
            .await?;

        let mut expired_count = 0;
        while let Some(batch) = results.try_next().await? {
            expired_count += batch.num_rows();
        }

        // 删除过期记忆
        table.delete(&format!("expires_at > 0 AND expires_at < {}", current_time)).await?;
        
        Ok(expired_count)
    }

    pub async fn get_important_memories(
        &self,
        agent_id: u64,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;
        let current_time = chrono::Utc::now().timestamp();

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {} AND importance >= {}", agent_id, threshold))
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.batch_to_memory(&batch, row)?;
                
                // 计算当前重要性
                let current_importance = memory.calculate_importance(current_time);
                if current_importance >= threshold {
                    memories.push(memory);
                }
            }
        }

        // 按重要性排序
        memories.sort_by(|a, b| {
            let importance_a = a.calculate_importance(current_time);
            let importance_b = b.calculate_importance(current_time);
            importance_b.partial_cmp(&importance_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(memories)
    }

    pub async fn search_memories(
        &self,
        agent_id: u64,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        // 简单的文本搜索（在实际应用中可能需要更复杂的搜索算法）
        let filter = format!(
            "agent_id = {} AND content LIKE '%{}%'",
            agent_id,
            query.replace("'", "''") // 转义单引号
        );

        let mut results = table
            .query()
            .only_if(&filter)
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.batch_to_memory(&batch, row)?;
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    pub async fn get_memory_stats(&self, agent_id: u64) -> Result<HashMap<String, u64>, AgentDbError> {
=======
        let memory = self.extract_memory_from_batch(&batch, 0)?;
        Ok(Some(memory))
    }

    pub async fn get_memories_by_agent(&self, agent_id: u64) -> Result<Vec<Memory>, AgentDbError> {
>>>>>>> origin/feature-module
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .execute()
            .await?;

<<<<<<< HEAD
        let mut stats = HashMap::new();
        stats.insert("total".to_string(), 0);
        stats.insert("episodic".to_string(), 0);
        stats.insert("semantic".to_string(), 0);
        stats.insert("procedural".to_string(), 0);
        stats.insert("working".to_string(), 0);

        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory_type_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
                let memory_type = memory_type_array.value(row);
                
                *stats.get_mut("total").unwrap() += 1;
                *stats.entry(memory_type.to_string()).or_insert(0) += 1;
            }
        }

        Ok(stats)
    }

    fn batch_to_memory(&self, batch: &RecordBatch, row: usize) -> Result<Memory, AgentDbError> {
        let memory_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
        let agent_id_array = batch.column(1).as_any().downcast_ref::<UInt64Array>().unwrap();
        let memory_type_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
        let content_array = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
        let importance_array = batch.column(4).as_any().downcast_ref::<Float32Array>().unwrap();
        let created_at_array = batch.column(5).as_any().downcast_ref::<Int64Array>().unwrap();
        let access_count_array = batch.column(6).as_any().downcast_ref::<UInt32Array>().unwrap();
        let last_access_array = batch.column(7).as_any().downcast_ref::<Int64Array>().unwrap();
        let expires_at_array = batch.column(8).as_any().downcast_ref::<Int64Array>().unwrap();

        let memory_id = memory_id_array.value(row).to_string();
=======
        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.extract_memory_from_batch(&batch, row)?;
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    pub async fn get_memories_by_type(&self, agent_id: u64, memory_type: MemoryType) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {} AND memory_type = '{}'", agent_id, memory_type.to_string()))
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.extract_memory_from_batch(&batch, row)?;
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    pub async fn search_memories(&self, agent_id: u64, query: &str, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        // 简单的文本搜索，实际应用中可以使用更复杂的搜索算法
        let mut results = table
            .query()
            .only_if(&format!("agent_id = {} AND content LIKE '%{}%'", agent_id, query))
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.extract_memory_from_batch(&batch, row)?;
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    pub async fn get_important_memories(&self, agent_id: u64, min_importance: f64, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {} AND importance >= {}", agent_id, min_importance))
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.extract_memory_from_batch(&batch, row)?;
                memories.push(memory);
            }
        }

        // 按重要性排序
        memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());

        Ok(memories)
    }

    pub async fn update_memory(&self, memory_id: &str, updated_memory: &Memory) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;

        // 删除旧记录
        table.delete(&format!("id = '{}'", memory_id)).await?;

        // 添加新记录
        self.store_memory(updated_memory).await?;

        Ok(())
    }

    pub async fn delete_memory(&self, memory_id: &str) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;
        table.delete(&format!("id = '{}'", memory_id)).await?;
        Ok(())
    }

    pub async fn cleanup_expired_memories(&self) -> Result<usize, AgentDbError> {
        let table = self.ensure_table().await?;
        let current_time = chrono::Utc::now().timestamp();

        // 首先计算要删除的记录数
        let mut results = table
            .query()
            .only_if(&format!("expiry_time > 0 AND expiry_time < {}", current_time))
            .execute()
            .await?;

        let mut count = 0;
        while let Some(batch) = results.try_next().await? {
            count += batch.num_rows();
        }

        // 执行删除
        table.delete(&format!("expiry_time > 0 AND expiry_time < {}", current_time)).await?;

        Ok(count)
    }

    pub async fn get_memory_statistics(&self, agent_id: u64) -> Result<MemoryStatistics, AgentDbError> {
        let memories = self.get_memories_by_agent(agent_id).await?;

        let total_count = memories.len();
        let mut type_counts = HashMap::new();
        let mut total_importance = 0.0;
        let mut total_access_count = 0;

        for memory in &memories {
            *type_counts.entry(memory.memory_type.clone()).or_insert(0) += 1;
            total_importance += memory.importance;
            total_access_count += memory.access_count;
        }

        let avg_importance = if total_count > 0 {
            total_importance / total_count as f64
        } else {
            0.0
        };

        let avg_access_count = if total_count > 0 {
            total_access_count as f64 / total_count as f64
        } else {
            0.0
        };

        Ok(MemoryStatistics {
            total_count,
            type_counts,
            avg_importance,
            avg_access_count,
            total_size_bytes: memories.iter().map(|m| m.content.len()).sum(),
        })
    }

    fn extract_memory_from_batch(&self, batch: &RecordBatch, row: usize) -> Result<Memory, AgentDbError> {
        let id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
        let agent_id_array = batch.column(1).as_any().downcast_ref::<UInt64Array>().unwrap();
        let memory_type_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
        let content_array = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
        let importance_array = batch.column(4).as_any().downcast_ref::<Float64Array>().unwrap();
        let timestamp_array = batch.column(5).as_any().downcast_ref::<Int64Array>().unwrap();
        let metadata_array = batch.column(6).as_any().downcast_ref::<StringArray>().unwrap();
        let embedding_array = batch.column(7).as_any().downcast_ref::<BinaryArray>().unwrap();
        let access_count_array = batch.column(8).as_any().downcast_ref::<UInt32Array>().unwrap();
        let last_accessed_array = batch.column(9).as_any().downcast_ref::<Int64Array>().unwrap();
        let expiry_time_array = batch.column(10).as_any().downcast_ref::<Int64Array>().unwrap();

        let id = id_array.value(row).to_string();
>>>>>>> origin/feature-module
        let agent_id = agent_id_array.value(row);
        let memory_type = MemoryType::from_string(memory_type_array.value(row))
            .ok_or_else(|| AgentDbError::InvalidArgument("Invalid memory type".to_string()))?;
        let content = content_array.value(row).to_string();
        let importance = importance_array.value(row);
<<<<<<< HEAD
        let created_at = created_at_array.value(row);
        let access_count = access_count_array.value(row);
        let last_access = last_access_array.value(row);
        let expires_at_value = expires_at_array.value(row);
        let expires_at = if expires_at_value == -1 { None } else { Some(expires_at_value) };

        Ok(Memory {
            memory_id,
=======
        let timestamp = timestamp_array.value(row);
        let metadata_json = metadata_array.value(row);
        let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;
        
        let embedding_bytes = embedding_array.value(row);
        let embedding = if embedding_bytes.is_empty() {
            None
        } else {
            Some(serde_json::from_slice(embedding_bytes)?)
        };

        let access_count = access_count_array.value(row);
        let last_accessed = last_accessed_array.value(row);
        let expiry_time_raw = expiry_time_array.value(row);
        let expiry_time = if expiry_time_raw == -1 { None } else { Some(expiry_time_raw) };

        Ok(Memory {
            id,
>>>>>>> origin/feature-module
            agent_id,
            memory_type,
            content,
            importance,
<<<<<<< HEAD
            embedding: None, // 嵌入向量需要单独处理
            created_at,
            access_count,
            last_access,
            expires_at,
        })
    }

    // 记忆衰减处理
    pub async fn decay_memories(&self, agent_id: u64, decay_factor: f32) -> Result<usize, AgentDbError> {
        let memories = self.get_agent_memories(agent_id, None, 1000).await?;
        let current_time = chrono::Utc::now().timestamp();
        let mut updated_count = 0;

        for mut memory in memories {
            let time_diff = (current_time - memory.created_at) as f32 / (24.0 * 3600.0); // 天数
            let new_importance = memory.importance * (1.0 - decay_factor * time_diff / 365.0).max(0.1);

            if (new_importance - memory.importance).abs() > 0.01 {
                memory.importance = new_importance;
                self.update_memory(&memory).await?;
                updated_count += 1;
            }
        }

        Ok(updated_count)
    }

    // 记忆合并算法
    pub async fn merge_similar_memories(
        &self,
        agent_id: u64,
        similarity_threshold: f32,
    ) -> Result<usize, AgentDbError> {
        let memories = self.get_agent_memories(agent_id, None, 1000).await?;
        let mut merged_count = 0;

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let similarity = self.calculate_memory_similarity(&memories[i], &memories[j]);

                if similarity > similarity_threshold {
                    // 合并记忆
                    let merged_memory = self.merge_memories(&memories[i], &memories[j])?;

                    // 删除原始记忆
                    self.delete_memory(&memories[i].memory_id).await?;
                    self.delete_memory(&memories[j].memory_id).await?;

                    // 存储合并后的记忆
                    self.store_memory(&merged_memory).await?;
                    merged_count += 1;
                }
            }
        }

        Ok(merged_count)
    }

    // 计算记忆相似性
    fn calculate_memory_similarity(&self, memory1: &Memory, memory2: &Memory) -> f32 {
        // 1. 类型相似性
        let type_similarity = if memory1.memory_type == memory2.memory_type { 1.0 } else { 0.0 };

        // 2. 内容相似性（简单的词汇重叠）
        let content_similarity = self.calculate_text_similarity(&memory1.content, &memory2.content);

        // 3. 时间相似性
        let time_diff = (memory1.created_at - memory2.created_at).abs() as f32 / (24.0 * 3600.0);
        let time_similarity = (1.0 / (1.0 + time_diff / 7.0)).max(0.0); // 一周内的记忆相似性更高

        // 4. 向量相似性（如果有嵌入向量）
        let vector_similarity = if let (Some(ref emb1), Some(ref emb2)) = (&memory1.embedding, &memory2.embedding) {
            crate::vector::cosine_similarity(emb1, emb2)
        } else {
            0.0
        };

        // 加权平均
        type_similarity * 0.2 + content_similarity * 0.4 + time_similarity * 0.2 + vector_similarity * 0.2
    }

    // 文本相似性计算
    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f32 {
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count() as f32;
        let union = words1.union(&words2).count() as f32;

        if union == 0.0 {
            0.0
        } else {
            intersection / union
        }
    }

    // 合并两个记忆
    fn merge_memories(&self, memory1: &Memory, memory2: &Memory) -> Result<Memory, AgentDbError> {
        let now = chrono::Utc::now().timestamp();

        // 选择更重要的记忆作为基础
        let (primary, secondary) = if memory1.importance >= memory2.importance {
            (memory1, memory2)
        } else {
            (memory2, memory1)
        };

        // 合并内容
        let merged_content = format!("{}\n[合并记忆]: {}", primary.content, secondary.content);

        // 计算新的重要性（取平均值并增加一点权重）
        let merged_importance = ((primary.importance + secondary.importance) / 2.0 * 1.1).min(1.0);

        // 合并访问次数
        let merged_access_count = primary.access_count + secondary.access_count;

        Ok(Memory {
            memory_id: uuid::Uuid::new_v4().to_string(),
            agent_id: primary.agent_id,
            memory_type: primary.memory_type,
            content: merged_content,
            importance: merged_importance,
            embedding: primary.embedding.clone(), // 使用主要记忆的嵌入
            created_at: primary.created_at.min(secondary.created_at), // 使用较早的创建时间
            access_count: merged_access_count,
            last_access: now,
            expires_at: None, // 合并后的记忆不设置过期时间
        })
    }

    // 更新记忆
    pub async fn update_memory(&self, memory: &Memory) -> Result<(), AgentDbError> {
        // 先删除旧记忆
        self.delete_memory(&memory.memory_id).await?;
        // 存储更新后的记忆
        self.store_memory(memory).await?;
        Ok(())
    }



    // 记忆网络构建（基于相似性）
    pub async fn build_memory_network(&self, agent_id: u64) -> Result<HashMap<String, Vec<String>>, AgentDbError> {
        let memories = self.get_agent_memories(agent_id, None, 1000).await?;
        let mut network = HashMap::new();

        for i in 0..memories.len() {
            let mut connections = Vec::new();

            for j in 0..memories.len() {
                if i != j {
                    let similarity = self.calculate_memory_similarity(&memories[i], &memories[j]);
                    if similarity > 0.3 { // 相似性阈值
                        connections.push(memories[j].memory_id.clone());
                    }
                }
            }

            network.insert(memories[i].memory_id.clone(), connections);
        }

        Ok(network)
    }

    // 记忆压缩策略
    pub async fn compress_memories(&self, agent_id: u64, max_memories: usize) -> Result<usize, AgentDbError> {
        let memories = self.get_agent_memories(agent_id, None, usize::MAX).await?;

        if memories.len() <= max_memories {
            return Ok(0);
        }

        // 按重要性和最近访问时间排序
        let mut sorted_memories = memories;
        sorted_memories.sort_by(|a, b| {
            let score_a = a.importance + (a.access_count as f32 * 0.1);
            let score_b = b.importance + (b.access_count as f32 * 0.1);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 删除重要性较低的记忆
        let _to_delete = sorted_memories.len() - max_memories;
        let mut deleted_count = 0;

        for memory in sorted_memories.iter().skip(max_memories) {
            self.delete_memory(&memory.memory_id).await?;
            deleted_count += 1;
        }

        Ok(deleted_count)
    }
=======
            timestamp,
            metadata,
            embedding,
            access_count,
            last_accessed,
            expiry_time,
        })
    }
}

// 记忆统计信息
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    pub total_count: usize,
    pub type_counts: HashMap<MemoryType, usize>,
    pub avg_importance: f64,
    pub avg_access_count: f64,
    pub total_size_bytes: usize,
}

impl MemoryManager {
    /// 基于向量相似性搜索记忆
    pub async fn search_similar_memories(&self, agent_id: u64, _query_embedding: Vec<f32>, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        // 简化的相似性搜索实现
        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.extract_memory_from_batch(&batch, row)?;
                // 这里可以添加真正的向量相似性计算
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    /// 根据重要性获取记忆
    pub async fn get_memories_by_importance(&self, agent_id: u64, min_importance: f64, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        // 先获取该Agent的所有记忆，然后在内存中过滤
        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.extract_memory_from_batch(&batch, row)?;
                // 在内存中过滤重要性
                if memory.importance >= min_importance {
                    memories.push(memory);
                }
            }
        }

        // 按重要性排序
        memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));

        // 限制结果数量
        memories.truncate(limit);

        Ok(memories)
    }

    /// 更新记忆的访问信息
    pub async fn access_memory(&self, _memory_id: &str) -> Result<(), AgentDbError> {
        // 这里应该实现更新记忆访问计数和最后访问时间的逻辑
        // 由于LanceDB的限制，这里简化处理
        Ok(())
    }
>>>>>>> origin/feature-module
}
