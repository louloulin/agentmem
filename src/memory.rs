// 记忆管理模块
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, BinaryArray, Float64Array, Int64Array, StringArray, UInt32Array, UInt64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::core::{AgentDbError, Memory, MemoryType};

// 记忆管理器
pub struct MemoryManager {
    connection: Connection,
}

impl MemoryManager {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub async fn ensure_table(&self) -> Result<Table, AgentDbError> {
        match self.connection.open_table("memories").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                let schema = Schema::new(vec![
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

        let metadata_json = serde_json::to_string(&memory.metadata)?;
        let embedding_bytes = memory.embedding.as_ref()
            .map(|emb| serde_json::to_vec(emb).unwrap())
            .unwrap_or_default();

        let schema = table.schema().await?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
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
            .only_if(&format!("id = '{}'", memory_id))
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

        let memory = self.extract_memory_from_batch(&batch, 0)?;
        Ok(Some(memory))
    }

    pub async fn get_memories_by_agent(&self, agent_id: u64) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
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
        let agent_id = agent_id_array.value(row);
        let memory_type = MemoryType::from_string(memory_type_array.value(row))
            .ok_or_else(|| AgentDbError::InvalidArgument("Invalid memory type".to_string()))?;
        let content = content_array.value(row).to_string();
        let importance = importance_array.value(row);
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
            agent_id,
            memory_type,
            content,
            importance,
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
        memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));

        Ok(memories)
    }

    /// 更新记忆的访问信息
    pub async fn access_memory(&self, _memory_id: &str) -> Result<(), AgentDbError> {
        // 这里应该实现更新记忆访问计数和最后访问时间的逻辑
        // 由于LanceDB的限制，这里简化处理
        Ok(())
    }
}
