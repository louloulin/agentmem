// 数据库核心模块
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, BinaryArray, Int64Array, StringArray, UInt64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{connect, Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::core::{AgentDbError, AgentState, StateType};

// Agent状态数据库核心
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

        let id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
        let agent_id_array = batch.column(1).as_any().downcast_ref::<UInt64Array>().unwrap();
        let session_id_array = batch.column(2).as_any().downcast_ref::<UInt64Array>().unwrap();
        let timestamp_array = batch.column(3).as_any().downcast_ref::<Int64Array>().unwrap();
        let state_type_array = batch.column(4).as_any().downcast_ref::<StringArray>().unwrap();
        let data_array = batch.column(5).as_any().downcast_ref::<BinaryArray>().unwrap();
        let metadata_array = batch.column(6).as_any().downcast_ref::<StringArray>().unwrap();
        let version_array = batch.column(7).as_any().downcast_ref::<UInt64Array>().unwrap();
        let checksum_array = batch.column(8).as_any().downcast_ref::<UInt64Array>().unwrap();

        let id = id_array.value(0).to_string();
        let agent_id = agent_id_array.value(0);
        let session_id = session_id_array.value(0);
        let timestamp = timestamp_array.value(0);
        let state_type = StateType::from_string(state_type_array.value(0))
            .ok_or_else(|| AgentDbError::InvalidArgument("Invalid state type".to_string()))?;
        let data = data_array.value(0).to_vec();
        let metadata_json = metadata_array.value(0);
        let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;
        let version = version_array.value(0) as u32;
        let checksum = checksum_array.value(0) as u32;

        Ok(Some(AgentState {
            id,
            agent_id,
            session_id,
            timestamp,
            state_type,
            data,
            metadata,
            version,
            checksum,
        }))
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

    pub async fn list_agents(&self) -> Result<Vec<u64>, AgentDbError> {
        let table = self.ensure_table().await?;
        
        let mut results = table
            .query()
            .execute()
            .await?;

        let mut agent_ids = Vec::new();
        while let Some(batch) = results.try_next().await? {
            let agent_id_array = batch.column(1).as_any().downcast_ref::<UInt64Array>().unwrap();
            for i in 0..batch.num_rows() {
                agent_ids.push(agent_id_array.value(i));
            }
        }

        // 去重
        agent_ids.sort();
        agent_ids.dedup();
        Ok(agent_ids)
    }

    pub async fn get_state_history(&self, agent_id: u64, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
        let table = self.ensure_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .limit(limit)
            .execute()
            .await?;

        let mut states = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
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

                states.push(AgentState {
                    id,
                    agent_id,
                    session_id,
                    timestamp,
                    state_type,
                    data,
                    metadata,
                    version,
                    checksum,
                });
            }
        }

        Ok(states)
    }

    pub fn get_connection(&self) -> &Connection {
        &self.connection
    }
}
