// 记忆管理模块
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, Float32Array, Int64Array, StringArray, UInt32Array, UInt64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::types::{AgentDbError, Memory, MemoryType};

pub struct MemoryManager {
    connection: Arc<Connection>,
}

impl MemoryManager {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self { connection }
    }

    pub async fn ensure_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("memories").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                let schema = Schema::new(vec![
                    Field::new("memory_id", DataType::Utf8, false),
                    Field::new("agent_id", DataType::UInt64, false),
                    Field::new("memory_type", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("importance", DataType::Float32, false),
                    Field::new("created_at", DataType::Int64, false),
                    Field::new("access_count", DataType::UInt32, false),
                    Field::new("last_access", DataType::Int64, false),
                    Field::new("expires_at", DataType::Int64, true),
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
        let schema = table.schema().await?;

        let expires_at_value = memory.expires_at.unwrap_or(-1);

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![memory.memory_id.clone()])),
                Arc::new(UInt64Array::from(vec![memory.agent_id])),
                Arc::new(StringArray::from(vec![memory.memory_type.to_string()])),
                Arc::new(StringArray::from(vec![memory.content.clone()])),
                Arc::new(Float32Array::from(vec![memory.importance])),
                Arc::new(Int64Array::from(vec![memory.created_at])),
                Arc::new(UInt32Array::from(vec![memory.access_count])),
                Arc::new(Int64Array::from(vec![memory.last_access])),
                Arc::new(Int64Array::from(vec![expires_at_value])),
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
            .only_if(&format!("memory_id = '{}'", memory_id))
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
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .execute()
            .await?;

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
        let agent_id = agent_id_array.value(row);
        let memory_type = MemoryType::from_string(memory_type_array.value(row))
            .ok_or_else(|| AgentDbError::InvalidArgument("Invalid memory type".to_string()))?;
        let content = content_array.value(row).to_string();
        let importance = importance_array.value(row);
        let created_at = created_at_array.value(row);
        let access_count = access_count_array.value(row);
        let last_access = last_access_array.value(row);
        let expires_at_value = expires_at_array.value(row);
        let expires_at = if expires_at_value == -1 { None } else { Some(expires_at_value) };

        Ok(Memory {
            memory_id,
            agent_id,
            memory_type,
            content,
            importance,
            embedding: None, // 嵌入向量需要单独处理
            created_at,
            access_count,
            last_access,
            expires_at,
        })
    }
}
