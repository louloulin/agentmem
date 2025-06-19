// Agent状态管理模块
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, BinaryArray, Int64Array, StringArray, UInt64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{connect, Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::core::{AgentDbError, AgentState, StateType, QueryResult, PaginationParams};

// Agent状态数据库
pub struct AgentStateDB {
    connection: Connection,
}

impl AgentStateDB {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;
        Ok(Self { connection })
    }

    pub async fn ensure_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有表
        match self.connection.open_table("agent_states").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新表
                let schema = Schema::new(vec![
                    Field::new("id", DataType::Utf8, false),
                    Field::new("agent_id", DataType::UInt64, false),
                    Field::new("session_id", DataType::UInt64, false),
                    Field::new("timestamp", DataType::Int64, false),
                    Field::new("state_type", DataType::Utf8, false),
                    Field::new("data", DataType::Binary, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("version", DataType::UInt64, false),
                    Field::new("checksum", DataType::UInt64, false),
                ]);

                // 创建空的RecordBatch迭代器
                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("agent_states", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    pub async fn save_state(&self, state: &AgentState) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;

        // 首先检查是否已存在相同agent_id的记录
        let existing = self.load_state(state.agent_id).await?;

        if existing.is_some() {
            // 如果存在，先删除旧记录
            table.delete(&format!("agent_id = {}", state.agent_id)).await?;
        }

        let metadata_json = serde_json::to_string(&state.metadata)?;

        // 获取表的schema
        let schema = table.schema().await?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![state.id.clone()])),
                Arc::new(UInt64Array::from(vec![state.agent_id])),
                Arc::new(UInt64Array::from(vec![state.session_id])),
                Arc::new(Int64Array::from(vec![state.timestamp])),
                Arc::new(StringArray::from(vec![state.state_type.to_string()])),
                Arc::new(BinaryArray::from(vec![state.data.as_slice()])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(UInt64Array::from(vec![state.version as u64])),
                Arc::new(UInt64Array::from(vec![state.checksum as u64])),
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    pub async fn load_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
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

        let state = self.extract_state_from_batch(&batch, 0)?;
        Ok(Some(state))
    }

    pub async fn update_state(&self, agent_id: u64, new_data: Vec<u8>) -> Result<(), AgentDbError> {
        if let Some(mut state) = self.load_state(agent_id).await? {
            state.update_data(new_data);
            self.save_state(&state).await?;
            Ok(())
        } else {
            Err(AgentDbError::NotFound(format!("Agent {} not found", agent_id)))
        }
    }

    pub async fn delete_state(&self, agent_id: u64) -> Result<(), AgentDbError> {
        let table = self.ensure_table().await?;
        table.delete(&format!("agent_id = {}", agent_id)).await?;
        Ok(())
    }

    pub async fn list_states(&self, pagination: PaginationParams) -> Result<QueryResult<AgentState>, AgentDbError> {
        let table = self.ensure_table().await?;
        let start_time = std::time::Instant::now();

        let offset = (pagination.page - 1) * pagination.page_size;
        
        let mut query = table.query();
        
        // 添加排序
        if let Some(_sort_field) = &pagination.sort_by {
            // LanceDB的排序语法可能需要调整
            query = query.limit(pagination.page_size);
        } else {
            query = query.limit(pagination.page_size);
        }

        let mut results = query.execute().await?;
        let mut states = Vec::new();
        let mut count = 0;

        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                if count >= offset && states.len() < pagination.page_size {
                    let state = self.extract_state_from_batch(&batch, row)?;
                    states.push(state);
                }
                count += 1;
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(QueryResult::new(
            states,
            count,
            pagination.page,
            pagination.page_size,
            execution_time,
        ))
    }

    pub async fn search_states_by_type(&self, state_type: StateType) -> Result<Vec<AgentState>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("state_type = '{}'", state_type.to_string()))
            .execute()
            .await?;

        let mut states = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let state = self.extract_state_from_batch(&batch, row)?;
                states.push(state);
            }
        }

        Ok(states)
    }

    pub async fn get_states_by_session(&self, session_id: u64) -> Result<Vec<AgentState>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("session_id = {}", session_id))
            .execute()
            .await?;

        let mut states = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let state = self.extract_state_from_batch(&batch, row)?;
                states.push(state);
            }
        }

        Ok(states)
    }

    fn extract_state_from_batch(&self, batch: &RecordBatch, row: usize) -> Result<AgentState, AgentDbError> {
        let id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
        let agent_id_array = batch.column(1).as_any().downcast_ref::<UInt64Array>().unwrap();
        let session_id_array = batch.column(2).as_any().downcast_ref::<UInt64Array>().unwrap();
        let timestamp_array = batch.column(3).as_any().downcast_ref::<Int64Array>().unwrap();
        let state_type_array = batch.column(4).as_any().downcast_ref::<StringArray>().unwrap();
        let data_array = batch.column(5).as_any().downcast_ref::<BinaryArray>().unwrap();
        let metadata_array = batch.column(6).as_any().downcast_ref::<StringArray>().unwrap();
        let version_array = batch.column(7).as_any().downcast_ref::<UInt64Array>().unwrap();
        let checksum_array = batch.column(8).as_any().downcast_ref::<UInt64Array>().unwrap();

        let id = id_array.value(row).to_string();
        let agent_id = agent_id_array.value(row);
        let session_id = session_id_array.value(row);
        let timestamp = timestamp_array.value(row);
        let state_type = StateType::from_string(state_type_array.value(row))
            .ok_or_else(|| AgentDbError::InvalidArgument("Invalid state type".to_string()))?;
        let data = data_array.value(row).to_vec();
        let metadata_json = metadata_array.value(row);
        let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;
        let version = version_array.value(row) as u32;
        let checksum = checksum_array.value(row) as u32;

        Ok(AgentState {
            id,
            agent_id,
            session_id,
            timestamp,
            state_type,
            data,
            metadata,
            version,
            checksum,
        })
    }

    pub async fn get_state_count(&self) -> Result<usize, AgentDbError> {
        let table = self.ensure_table().await?;
        let mut results = table.query().execute().await?;
        let mut count = 0;

        while let Some(batch) = results.try_next().await? {
            count += batch.num_rows();
        }

        Ok(count)
    }

    pub async fn cleanup_old_states(&self, older_than_seconds: i64) -> Result<usize, AgentDbError> {
        let table = self.ensure_table().await?;
        let cutoff_time = chrono::Utc::now().timestamp() - older_than_seconds;
        
        // 首先计算要删除的记录数
        let mut results = table
            .query()
            .only_if(&format!("timestamp < {}", cutoff_time))
            .execute()
            .await?;

        let mut count = 0;
        while let Some(batch) = results.try_next().await? {
            count += batch.num_rows();
        }

        // 执行删除
        table.delete(&format!("timestamp < {}", cutoff_time)).await?;

        Ok(count)
    }

    // 向量状态管理功能
    /// 确保向量表存在，如果不存在则创建
    pub async fn ensure_vector_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有向量表
        match self.connection.open_table("agent_vector_states").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新的向量表
                let schema = Schema::new(vec![
                    Field::new("id", DataType::Utf8, false),
                    Field::new("agent_id", DataType::UInt64, false),
                    Field::new("session_id", DataType::UInt64, false),
                    Field::new("timestamp", DataType::Int64, false),
                    Field::new("state_type", DataType::Utf8, false),
                    Field::new("data", DataType::Binary, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("version", DataType::UInt64, false),
                    Field::new("checksum", DataType::UInt64, false),
                    // 向量列 - 使用Binary存储序列化的向量
                    Field::new("embedding", DataType::Binary, false),
                ]);

                // 创建空的RecordBatch迭代器
                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("agent_vector_states", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    /// 保存带向量嵌入的Agent状态
    pub async fn save_vector_state(&self, state: &AgentState, embedding: Vec<f32>) -> Result<(), AgentDbError> {
        let table = self.ensure_vector_table().await?;

        let metadata_json = serde_json::to_string(&state.metadata)?;

        // 获取表的schema
        let schema = table.schema().await?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![state.id.clone()])),
                Arc::new(UInt64Array::from(vec![state.agent_id])),
                Arc::new(UInt64Array::from(vec![state.session_id])),
                Arc::new(Int64Array::from(vec![state.timestamp])),
                Arc::new(StringArray::from(vec![state.state_type.to_string()])),
                Arc::new(BinaryArray::from(vec![state.data.as_slice()])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(UInt64Array::from(vec![state.version as u64])),
                Arc::new(UInt64Array::from(vec![state.checksum as u64])),
                // 添加向量列 - 序列化向量为二进制数据
                Arc::new(BinaryArray::from(vec![serde_json::to_vec(&embedding).unwrap().as_slice()])),
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    /// 向量相似性搜索 - 基础实现
    pub async fn vector_search(&self, _query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        let table = self.ensure_vector_table().await?;

        // 暂时使用简单查询，后续可以优化为真正的向量搜索
        let mut results = table
            .query()
            .limit(limit)
            .execute()
            .await?;

        let mut states = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let state = self.extract_vector_state_from_batch(&batch, row)?;
                states.push(state);
            }
        }

        Ok(states)
    }

    /// 基于Agent ID和向量相似性的搜索
    pub async fn search_by_agent_and_similarity(&self, agent_id: u64, _query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        let table = self.ensure_vector_table().await?;

        // 结合Agent ID过滤的查询
        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .limit(limit)
            .execute()
            .await?;

        let mut states = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let state = self.extract_vector_state_from_batch(&batch, row)?;
                states.push(state);
            }
        }

        Ok(states)
    }

    /// 从向量表的RecordBatch中提取AgentState
    fn extract_vector_state_from_batch(&self, batch: &RecordBatch, row: usize) -> Result<AgentState, AgentDbError> {
        let id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
        let agent_id_array = batch.column(1).as_any().downcast_ref::<UInt64Array>().unwrap();
        let session_id_array = batch.column(2).as_any().downcast_ref::<UInt64Array>().unwrap();
        let timestamp_array = batch.column(3).as_any().downcast_ref::<Int64Array>().unwrap();
        let state_type_array = batch.column(4).as_any().downcast_ref::<StringArray>().unwrap();
        let data_array = batch.column(5).as_any().downcast_ref::<BinaryArray>().unwrap();
        let metadata_array = batch.column(6).as_any().downcast_ref::<StringArray>().unwrap();
        let version_array = batch.column(7).as_any().downcast_ref::<UInt64Array>().unwrap();
        let checksum_array = batch.column(8).as_any().downcast_ref::<UInt64Array>().unwrap();
        // 注意：第9列是embedding，这里暂时不处理

        let id = id_array.value(row).to_string();
        let agent_id = agent_id_array.value(row);
        let session_id = session_id_array.value(row);
        let timestamp = timestamp_array.value(row);
        let state_type = StateType::from_string(state_type_array.value(row))
            .ok_or_else(|| AgentDbError::InvalidArgument("Invalid state type".to_string()))?;
        let data = data_array.value(row).to_vec();
        let metadata_json = metadata_array.value(row);
        let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;
        let version = version_array.value(row) as u32;
        let checksum = checksum_array.value(row) as u32;

        Ok(AgentState {
            id,
            agent_id,
            session_id,
            timestamp,
            state_type,
            data,
            metadata,
            version,
            checksum,
        })
    }
}
