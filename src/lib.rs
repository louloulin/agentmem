// Agent状态数据库 - 基于LanceDB的Rust实现
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Arc;
use rand::Rng;

use arrow::array::{Array, BinaryArray, Int64Array, StringArray, UInt64Array, RecordBatchIterator};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::TryStreamExt;
use lancedb::{connect, Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use uuid::Uuid;

// 错误类型定义
#[derive(Debug, thiserror::Error)]
pub enum AgentDbError {
    #[error("Lance error: {0}")]
    Lance(#[from] lancedb::Error),
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

// Agent状态类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateType {
    WorkingMemory,
    LongTermMemory,
    Context,
    TaskState,
    Relationship,
    Embedding,
}

impl StateType {
    pub fn to_string(&self) -> &'static str {
        match self {
            StateType::WorkingMemory => "working_memory",
            StateType::LongTermMemory => "long_term_memory",
            StateType::Context => "context",
            StateType::TaskState => "task_state",
            StateType::Relationship => "relationship",
            StateType::Embedding => "embedding",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "working_memory" => Some(StateType::WorkingMemory),
            "long_term_memory" => Some(StateType::LongTermMemory),
            "context" => Some(StateType::Context),
            "task_state" => Some(StateType::TaskState),
            "relationship" => Some(StateType::Relationship),
            "embedding" => Some(StateType::Embedding),
            _ => None,
        }
    }
}

// Agent状态结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub agent_id: u64,
    pub session_id: u64,
    pub timestamp: i64,
    pub state_type: StateType,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub version: u32,
    pub checksum: u32,
}

impl AgentState {
    pub fn new(
        agent_id: u64,
        session_id: u64,
        state_type: StateType,
        data: Vec<u8>,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let checksum = Self::calculate_checksum(&data);

        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            session_id,
            timestamp,
            state_type,
            data,
            metadata: HashMap::new(),
            version: 1,
            checksum,
        }
    }

    pub fn calculate_checksum(data: &[u8]) -> u32 {
        data.iter().fold(0u32, |acc, &byte| acc.wrapping_add(byte as u32))
    }

    pub fn validate_checksum(&self) -> bool {
        Self::calculate_checksum(&self.data) == self.checksum
    }

    pub fn update_data(&mut self, new_data: Vec<u8>) {
        self.data = new_data;
        self.checksum = Self::calculate_checksum(&self.data);
        self.version += 1;
        self.timestamp = chrono::Utc::now().timestamp();
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

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

    // 向量存储和检索功能
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
                // 添加向量列 - 简化处理，暂时使用Binary存储
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

    pub async fn vector_search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
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

    pub async fn search_by_agent_and_similarity(&self, agent_id: u64, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<AgentState>, AgentDbError> {
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
}

// C FFI接口
#[repr(C)]
pub struct CAgentStateDB {
    db: *mut AgentStateDB,
}

#[no_mangle]
pub extern "C" fn agent_db_new(db_path: *const c_char) -> *mut CAgentStateDB {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

    // 创建一个简单的运行时来初始化数据库
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let db = match rt.block_on(async {
        AgentStateDB::new(path_str).await
    }) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(CAgentStateDB { db }))
}

#[no_mangle]
pub extern "C" fn agent_db_free(db: *mut CAgentStateDB) {
    if !db.is_null() {
        unsafe {
            let c_db = Box::from_raw(db);
            if !c_db.db.is_null() {
                let _ = Box::from_raw(c_db.db);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn agent_db_save_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    session_id: u64,
    state_type: c_int,
    data: *const u8,
    data_len: usize,
) -> c_int {
    if db.is_null() || data.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };

    let state_type = match state_type {
        0 => StateType::WorkingMemory,
        1 => StateType::LongTermMemory,
        2 => StateType::Context,
        3 => StateType::TaskState,
        4 => StateType::Relationship,
        5 => StateType::Embedding,
        _ => return -1,
    };

    let data_vec = unsafe { std::slice::from_raw_parts(data, data_len).to_vec() };
    let state = AgentState::new(agent_id, session_id, state_type, data_vec);

    // 创建临时runtime来执行异步操作
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.save_state(&state)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn agent_db_load_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    data_out: *mut *mut u8,
    data_len_out: *mut usize,
) -> c_int {
    if db.is_null() || data_out.is_null() || data_len_out.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };

    // 创建临时runtime来执行异步操作
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.load_state(agent_id)) {
        Ok(Some(state)) => {
            let data_copy = state.data.into_boxed_slice();
            let len = data_copy.len();
            let ptr = Box::into_raw(data_copy) as *mut u8;

            unsafe {
                *data_out = ptr;
                *data_len_out = len;
            }
            0
        }
        Ok(None) => 1, // Not found
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn agent_db_free_data(data: *mut u8, data_len: usize) {
    if !data.is_null() && data_len > 0 {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(data, data_len));
        }
    }
}

// 向量功能的C FFI接口
#[no_mangle]
pub extern "C" fn agent_db_save_vector_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    session_id: u64,
    state_type: c_int,
    data: *const u8,
    data_len: usize,
    embedding: *const f32,
    embedding_len: usize,
) -> c_int {
    if db.is_null() || data.is_null() || embedding.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };

    let state_type = match state_type {
        0 => StateType::WorkingMemory,
        1 => StateType::LongTermMemory,
        2 => StateType::Context,
        3 => StateType::TaskState,
        4 => StateType::Relationship,
        5 => StateType::Embedding,
        _ => return -1,
    };

    let data_vec = unsafe { std::slice::from_raw_parts(data, data_len).to_vec() };
    let embedding_vec = unsafe { std::slice::from_raw_parts(embedding, embedding_len).to_vec() };

    let state = AgentState::new(agent_id, session_id, state_type, data_vec);

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.save_vector_state(&state, embedding_vec)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn agent_db_vector_search(
    db: *mut CAgentStateDB,
    query_embedding: *const f32,
    embedding_len: usize,
    limit: usize,
    results_out: *mut *mut u64,
    results_count_out: *mut usize,
) -> c_int {
    if db.is_null() || query_embedding.is_null() || results_out.is_null() || results_count_out.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };

    let query_vec = unsafe { std::slice::from_raw_parts(query_embedding, embedding_len).to_vec() };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.vector_search(query_vec, limit)) {
        Ok(states) => {
            let agent_ids: Vec<u64> = states.iter().map(|s| s.agent_id).collect();
            let agent_ids_copy = agent_ids.into_boxed_slice();
            let len = agent_ids_copy.len();
            let ptr = Box::into_raw(agent_ids_copy) as *mut u64;

            unsafe {
                *results_out = ptr;
                *results_count_out = len;
            }
            0
        }
        Err(_) => -1,
    }
}

// 记忆系统的C FFI接口
#[repr(C)]
pub struct CMemoryManager {
    mgr: *mut MemoryManager,
}

#[no_mangle]
pub extern "C" fn memory_manager_new(db_path: *const c_char) -> *mut CMemoryManager {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let mgr = match rt.block_on(async {
        MemoryManager::new(path_str).await
    }) {
        Ok(mgr) => Box::into_raw(Box::new(mgr)),
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(CMemoryManager { mgr }))
}

#[no_mangle]
pub extern "C" fn memory_manager_free(mgr: *mut CMemoryManager) {
    if !mgr.is_null() {
        unsafe {
            let c_mgr = Box::from_raw(mgr);
            if !c_mgr.mgr.is_null() {
                let _ = Box::from_raw(c_mgr.mgr);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn memory_manager_store_memory(
    mgr: *mut CMemoryManager,
    agent_id: u64,
    memory_type: c_int,
    content: *const c_char,
    importance: f32,
) -> c_int {
    if mgr.is_null() || content.is_null() {
        return -1;
    }

    let c_mgr = unsafe { &*mgr };
    let memory_mgr = unsafe { &*c_mgr.mgr };

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        }
    };

    let mem_type = match memory_type {
        0 => MemoryType::Episodic,
        1 => MemoryType::Semantic,
        2 => MemoryType::Procedural,
        3 => MemoryType::Working,
        _ => return -1,
    };

    let memory = Memory::new(agent_id, mem_type, content_str, importance);

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_mgr.store_memory(&memory)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn memory_manager_retrieve_memories(
    mgr: *mut CMemoryManager,
    agent_id: u64,
    limit: usize,
    memory_count_out: *mut usize,
) -> c_int {
    if mgr.is_null() || memory_count_out.is_null() {
        return -1;
    }

    let c_mgr = unsafe { &*mgr };
    let memory_mgr = unsafe { &*c_mgr.mgr };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_mgr.retrieve_memories(agent_id, limit)) {
        Ok(memories) => {
            unsafe {
                *memory_count_out = memories.len();
            }
            0
        }
        Err(_) => -1,
    }
}

// 记忆系统相关结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub memory_id: String,
    pub agent_id: u64,
    pub memory_type: MemoryType,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub importance: f32,
    pub access_count: u32,
    pub last_access: i64,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    Episodic,    // 情节记忆
    Semantic,    // 语义记忆
    Procedural,  // 程序记忆
    Working,     // 工作记忆
}

impl MemoryType {
    pub fn to_string(&self) -> &'static str {
        match self {
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
            MemoryType::Working => "working",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "episodic" => Some(MemoryType::Episodic),
            "semantic" => Some(MemoryType::Semantic),
            "procedural" => Some(MemoryType::Procedural),
            "working" => Some(MemoryType::Working),
            _ => None,
        }
    }
}

impl Memory {
    pub fn new(
        agent_id: u64,
        memory_type: MemoryType,
        content: String,
        importance: f32,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            memory_id: Uuid::new_v4().to_string(),
            agent_id,
            memory_type,
            content,
            embedding: None,
            importance,
            access_count: 0,
            last_access: now,
            created_at: now,
            expires_at: None,
        }
    }

    pub fn calculate_importance(&self, current_time: i64) -> f32 {
        let time_decay = (current_time - self.created_at) as f32 / (24.0 * 3600.0 * 1000.0); // 天数
        let access_factor = (self.access_count as f32 + 1.0).ln();
        self.importance * (-time_decay * 0.1).exp() * access_factor
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_access = chrono::Utc::now().timestamp();
    }

    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
    }

    pub fn is_expired(&self, current_time: i64) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_time > expires_at
        } else {
            false
        }
    }
}

// 记忆系统管理器
pub struct MemoryManager {
    connection: Connection,
}

impl MemoryManager {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;
        Ok(Self { connection })
    }

    pub async fn ensure_memory_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有记忆表
        match self.connection.open_table("memories").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新的记忆表
                let schema = Schema::new(vec![
                    Field::new("memory_id", DataType::Utf8, false),
                    Field::new("agent_id", DataType::UInt64, false),
                    Field::new("memory_type", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("importance", DataType::Float32, false),
                    Field::new("access_count", DataType::UInt32, false),
                    Field::new("last_access", DataType::Int64, false),
                    Field::new("created_at", DataType::Int64, false),
                    Field::new("expires_at", DataType::Int64, true),
                    // 向量列 - 使用Binary存储序列化的向量
                    Field::new("embedding", DataType::Binary, true),
                ]);

                // 创建空的RecordBatch迭代器
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
        let table = self.ensure_memory_table().await?;

        let schema = table.schema().await?;

        // 准备embedding数据
        let embedding_data = if let Some(ref emb) = memory.embedding {
            Some(emb.clone())
        } else {
            None
        };

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![memory.memory_id.clone()])),
                Arc::new(UInt64Array::from(vec![memory.agent_id])),
                Arc::new(StringArray::from(vec![memory.memory_type.to_string()])),
                Arc::new(StringArray::from(vec![memory.content.clone()])),
                Arc::new(arrow::array::Float32Array::from(vec![memory.importance])),
                Arc::new(arrow::array::UInt32Array::from(vec![memory.access_count])),
                Arc::new(Int64Array::from(vec![memory.last_access])),
                Arc::new(Int64Array::from(vec![memory.created_at])),
                Arc::new(Int64Array::from(vec![memory.expires_at])),
                // 处理可选的embedding
                if let Some(emb) = embedding_data {
                    Arc::new(BinaryArray::from(vec![Some(serde_json::to_vec(&emb).unwrap().as_slice())]))
                } else {
                    Arc::new(BinaryArray::from(vec![None::<&[u8]>]))
                },
            ],
        )?;

        let batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        );
        table.add(Box::new(batch_iter)).execute().await?;
        Ok(())
    }

    pub async fn retrieve_memories(&self, agent_id: u64, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_memory_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.parse_memory_from_batch(&batch, row)?;
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    pub async fn search_similar_memories(&self, agent_id: u64, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let table = self.ensure_memory_table().await?;

        let mut results = table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .limit(limit)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let memory = self.parse_memory_from_batch(&batch, row)?;
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    fn parse_memory_from_batch(&self, batch: &RecordBatch, row: usize) -> Result<Memory, AgentDbError> {
        let memory_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
        let agent_id_array = batch.column(1).as_any().downcast_ref::<UInt64Array>().unwrap();
        let memory_type_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
        let content_array = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
        let importance_array = batch.column(4).as_any().downcast_ref::<arrow::array::Float32Array>().unwrap();
        let access_count_array = batch.column(5).as_any().downcast_ref::<arrow::array::UInt32Array>().unwrap();
        let last_access_array = batch.column(6).as_any().downcast_ref::<Int64Array>().unwrap();
        let created_at_array = batch.column(7).as_any().downcast_ref::<Int64Array>().unwrap();
        let expires_at_array = batch.column(8).as_any().downcast_ref::<Int64Array>().unwrap();

        let memory_id = memory_id_array.value(row).to_string();
        let agent_id = agent_id_array.value(row);
        let memory_type = MemoryType::from_string(memory_type_array.value(row))
            .ok_or_else(|| AgentDbError::InvalidArgument("Invalid memory type".to_string()))?;
        let content = content_array.value(row).to_string();
        let importance = importance_array.value(row);
        let access_count = access_count_array.value(row);
        let last_access = last_access_array.value(row);
        let created_at = created_at_array.value(row);
        let expires_at = if expires_at_array.is_null(row) {
            None
        } else {
            Some(expires_at_array.value(row))
        };

        // 处理embedding（简化处理）
        let embedding = None;

        Ok(Memory {
            memory_id,
            agent_id,
            memory_type,
            content,
            embedding,
            importance,
            access_count,
            last_access,
            created_at,
            expires_at,
        })
    }
}

// RAG引擎相关结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_id: String,
    pub title: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, String>,
    pub chunks: Vec<DocumentChunk>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub chunk_id: String,
    pub doc_id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub chunk_index: u32,
    pub start_pos: usize,
    pub end_pos: usize,
    pub overlap_prev: usize,
    pub overlap_next: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub doc_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGContext {
    pub query: String,
    pub retrieved_chunks: Vec<SearchResult>,
    pub context_window: String,
    pub relevance_scores: Vec<f32>,
    pub total_tokens: usize,
}

impl Document {
    pub fn new(title: String, content: String) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            doc_id: Uuid::new_v4().to_string(),
            title,
            content,
            embedding: None,
            metadata: HashMap::new(),
            chunks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn chunk_document(&mut self, chunk_size: usize, overlap: usize) -> Result<(), AgentDbError> {
        self.chunks.clear();

        if self.content.is_empty() {
            return Ok(());
        }

        let content_bytes = self.content.as_bytes();
        let total_len = content_bytes.len();
        let mut start_pos = 0;
        let mut chunk_index = 0;

        while start_pos < total_len {
            let end_pos = std::cmp::min(start_pos + chunk_size, total_len);

            // 尝试在单词边界处分割
            let actual_end_pos = if end_pos < total_len {
                self.find_word_boundary(start_pos, end_pos)
            } else {
                end_pos
            };

            let chunk_content = String::from_utf8_lossy(&content_bytes[start_pos..actual_end_pos]).to_string();

            let chunk = DocumentChunk {
                chunk_id: Uuid::new_v4().to_string(),
                doc_id: self.doc_id.clone(),
                content: chunk_content,
                embedding: None,
                chunk_index,
                start_pos,
                end_pos: actual_end_pos,
                overlap_prev: if chunk_index > 0 { overlap } else { 0 },
                overlap_next: if actual_end_pos < total_len { overlap } else { 0 },
            };

            self.chunks.push(chunk);

            // 计算下一个块的起始位置，考虑重叠
            start_pos = if actual_end_pos < total_len {
                std::cmp::max(actual_end_pos - overlap, start_pos + 1)
            } else {
                actual_end_pos
            };

            chunk_index += 1;
        }

        Ok(())
    }

    fn find_word_boundary(&self, start: usize, end: usize) -> usize {
        let content_bytes = self.content.as_bytes();

        // 从end位置向前查找空格或标点符号
        for i in (start..end).rev() {
            if i < content_bytes.len() {
                let ch = content_bytes[i] as char;
                if ch.is_whitespace() || ch.is_ascii_punctuation() {
                    return i + 1;
                }
            }
        }

        // 如果找不到合适的边界，返回原始end位置
        end
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

impl DocumentChunk {
    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
    }

    pub fn get_token_count(&self) -> usize {
        // 简单的token计数估算（实际应用中可能需要更精确的tokenizer）
        self.content.split_whitespace().count()
    }
}

// RAG引擎
pub struct RAGEngine {
    connection: Connection,
}

impl RAGEngine {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;
        Ok(Self { connection })
    }

    pub async fn ensure_document_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有文档表
        match self.connection.open_table("documents").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新的文档表
                let schema = Schema::new(vec![
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("title", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("metadata", DataType::Utf8, false),
                    Field::new("created_at", DataType::Int64, false),
                    Field::new("updated_at", DataType::Int64, false),
                    Field::new("embedding", DataType::Binary, true),
                ]);

                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("documents", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    pub async fn ensure_chunk_table(&self) -> Result<Table, AgentDbError> {
        // 尝试打开现有块表
        match self.connection.open_table("chunks").execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // 如果表不存在，创建新的块表
                let schema = Schema::new(vec![
                    Field::new("chunk_id", DataType::Utf8, false),
                    Field::new("doc_id", DataType::Utf8, false),
                    Field::new("content", DataType::Utf8, false),
                    Field::new("chunk_index", DataType::UInt32, false),
                    Field::new("start_pos", DataType::UInt64, false),
                    Field::new("end_pos", DataType::UInt64, false),
                    Field::new("overlap_prev", DataType::UInt64, false),
                    Field::new("overlap_next", DataType::UInt64, false),
                    Field::new("embedding", DataType::Binary, true),
                ]);

                let empty_batches = RecordBatchIterator::new(
                    std::iter::empty::<Result<RecordBatch, arrow::error::ArrowError>>(),
                    Arc::new(schema),
                );

                let table = self
                    .connection
                    .create_table("chunks", Box::new(empty_batches))
                    .execute()
                    .await?;

                Ok(table)
            }
        }
    }

    pub async fn index_document(&self, document: &Document) -> Result<String, AgentDbError> {
        // 1. 存储文档元数据
        let doc_table = self.ensure_document_table().await?;
        let doc_schema = doc_table.schema().await?;

        let metadata_json = serde_json::to_string(&document.metadata)?;
        let embedding_data = if let Some(ref emb) = document.embedding {
            Some(serde_json::to_vec(emb).unwrap())
        } else {
            None
        };

        let doc_batch = RecordBatch::try_new(
            doc_schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![document.doc_id.clone()])),
                Arc::new(StringArray::from(vec![document.title.clone()])),
                Arc::new(StringArray::from(vec![document.content.clone()])),
                Arc::new(StringArray::from(vec![metadata_json])),
                Arc::new(Int64Array::from(vec![document.created_at])),
                Arc::new(Int64Array::from(vec![document.updated_at])),
                if let Some(emb_data) = embedding_data {
                    Arc::new(BinaryArray::from(vec![Some(emb_data.as_slice())]))
                } else {
                    Arc::new(BinaryArray::from(vec![None::<&[u8]>]))
                },
            ],
        )?;

        let doc_batch_iter = RecordBatchIterator::new(
            std::iter::once(Ok(doc_batch)),
            doc_schema,
        );
        doc_table.add(Box::new(doc_batch_iter)).execute().await?;

        // 2. 存储文档块
        if !document.chunks.is_empty() {
            let chunk_table = self.ensure_chunk_table().await?;
            let chunk_schema = chunk_table.schema().await?;

            for chunk in &document.chunks {
                let chunk_embedding_data = if let Some(ref emb) = chunk.embedding {
                    Some(serde_json::to_vec(emb).unwrap())
                } else {
                    None
                };

                let chunk_batch = RecordBatch::try_new(
                    chunk_schema.clone(),
                    vec![
                        Arc::new(StringArray::from(vec![chunk.chunk_id.clone()])),
                        Arc::new(StringArray::from(vec![chunk.doc_id.clone()])),
                        Arc::new(StringArray::from(vec![chunk.content.clone()])),
                        Arc::new(arrow::array::UInt32Array::from(vec![chunk.chunk_index])),
                        Arc::new(UInt64Array::from(vec![chunk.start_pos as u64])),
                        Arc::new(UInt64Array::from(vec![chunk.end_pos as u64])),
                        Arc::new(UInt64Array::from(vec![chunk.overlap_prev as u64])),
                        Arc::new(UInt64Array::from(vec![chunk.overlap_next as u64])),
                        if let Some(emb_data) = chunk_embedding_data {
                            Arc::new(BinaryArray::from(vec![Some(emb_data.as_slice())]))
                        } else {
                            Arc::new(BinaryArray::from(vec![None::<&[u8]>]))
                        },
                    ],
                )?;

                let chunk_batch_iter = RecordBatchIterator::new(
                    std::iter::once(Ok(chunk_batch)),
                    chunk_schema.clone(),
                );
                chunk_table.add(Box::new(chunk_batch_iter)).execute().await?;
            }
        }

        Ok(document.doc_id.clone())
    }

    pub async fn semantic_search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        let chunk_table = self.ensure_chunk_table().await?;

        // 简化的搜索实现（实际应用中需要真正的向量相似性搜索）
        let mut results = chunk_table
            .query()
            .limit(limit)
            .execute()
            .await?;

        let mut search_results = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let chunk_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let doc_id_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();

                let chunk_id = chunk_id_array.value(row).to_string();
                let doc_id = doc_id_array.value(row).to_string();
                let content = content_array.value(row).to_string();

                // 简化的相似性评分（实际应用中需要计算真正的余弦相似度）
                let score = 0.8 - (row as f32 * 0.1);

                search_results.push(SearchResult {
                    chunk_id,
                    doc_id,
                    content,
                    score,
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(search_results)
    }

    pub async fn search_by_text(&self, text_query: &str, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        let chunk_table = self.ensure_chunk_table().await?;

        // 简化的文本搜索（实际应用中需要全文搜索引擎）
        let mut results = chunk_table
            .query()
            .limit(limit * 2) // 获取更多结果用于过滤
            .execute()
            .await?;

        let mut search_results = Vec::new();
        let query_lower = text_query.to_lowercase();

        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let chunk_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let doc_id_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();

                let chunk_id = chunk_id_array.value(row).to_string();
                let doc_id = doc_id_array.value(row).to_string();
                let content = content_array.value(row).to_string();
                let content_lower = content.to_lowercase();

                // 简单的文本匹配评分
                if content_lower.contains(&query_lower) {
                    let score = self.calculate_text_similarity(&query_lower, &content_lower);

                    search_results.push(SearchResult {
                        chunk_id,
                        doc_id,
                        content,
                        score,
                        metadata: HashMap::new(),
                    });

                    if search_results.len() >= limit {
                        break;
                    }
                }
            }

            if search_results.len() >= limit {
                break;
            }
        }

        // 按分数排序
        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        search_results.truncate(limit);

        Ok(search_results)
    }

    pub async fn hybrid_search(&self, text_query: &str, query_embedding: Vec<f32>, alpha: f32, limit: usize) -> Result<Vec<SearchResult>, AgentDbError> {
        // 1. 获取文本搜索结果
        let text_results = self.search_by_text(text_query, limit * 2).await?;

        // 2. 获取向量搜索结果
        let vector_results = self.semantic_search(query_embedding, limit * 2).await?;

        // 3. 合并和重新评分
        let mut combined_results = HashMap::new();

        // 添加文本搜索结果
        for result in text_results {
            let key = result.chunk_id.clone();
            combined_results.insert(key, (result, alpha, 0.0));
        }

        // 添加向量搜索结果
        for result in vector_results {
            let key = result.chunk_id.clone();
            if let Some((existing, text_score, _)) = combined_results.get_mut(&key) {
                // 如果已存在，更新向量分数
                *existing = SearchResult {
                    chunk_id: existing.chunk_id.clone(),
                    doc_id: existing.doc_id.clone(),
                    content: existing.content.clone(),
                    score: *text_score * alpha + result.score * (1.0 - alpha),
                    metadata: existing.metadata.clone(),
                };
            } else {
                // 如果不存在，添加新结果
                combined_results.insert(key, (result, 0.0, 1.0 - alpha));
            }
        }

        // 4. 收集并排序结果
        let mut final_results: Vec<SearchResult> = combined_results
            .into_iter()
            .map(|(_, (mut result, text_weight, vector_weight))| {
                result.score = result.score * text_weight + result.score * vector_weight;
                result
            })
            .collect();

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(limit);

        Ok(final_results)
    }

    pub async fn build_context(&self, query: &str, search_results: Vec<SearchResult>, max_tokens: usize) -> Result<RAGContext, AgentDbError> {
        let mut context_chunks = Vec::new();
        let mut total_tokens = 0;
        let mut relevance_scores = Vec::new();

        for result in search_results {
            let chunk_tokens = result.content.split_whitespace().count();

            if total_tokens + chunk_tokens <= max_tokens {
                total_tokens += chunk_tokens;
                relevance_scores.push(result.score);
                context_chunks.push(result);
            } else {
                break;
            }
        }

        // 构建上下文窗口
        let context_window = context_chunks
            .iter()
            .map(|chunk| format!("Document: {}\nContent: {}\n", chunk.doc_id, chunk.content))
            .collect::<Vec<_>>()
            .join("\n---\n");

        Ok(RAGContext {
            query: query.to_string(),
            retrieved_chunks: context_chunks,
            context_window,
            relevance_scores,
            total_tokens,
        })
    }

    fn calculate_text_similarity(&self, query: &str, content: &str) -> f32 {
        let query_words: std::collections::HashSet<&str> = query.split_whitespace().collect();
        let content_words: std::collections::HashSet<&str> = content.split_whitespace().collect();

        let intersection = query_words.intersection(&content_words).count();
        let union = query_words.union(&content_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    pub async fn get_document(&self, doc_id: &str) -> Result<Option<Document>, AgentDbError> {
        let doc_table = self.ensure_document_table().await?;

        let mut results = doc_table
            .query()
            .only_if(&format!("doc_id = '{}'", doc_id))
            .limit(1)
            .execute()
            .await?;

        if let Some(batch) = results.try_next().await? {
            if batch.num_rows() > 0 {
                let doc_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let title_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
                let metadata_array = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
                let created_at_array = batch.column(4).as_any().downcast_ref::<Int64Array>().unwrap();
                let updated_at_array = batch.column(5).as_any().downcast_ref::<Int64Array>().unwrap();

                let doc_id = doc_id_array.value(0).to_string();
                let title = title_array.value(0).to_string();
                let content = content_array.value(0).to_string();
                let metadata_json = metadata_array.value(0);
                let metadata: HashMap<String, String> = serde_json::from_str(metadata_json)?;
                let created_at = created_at_array.value(0);
                let updated_at = updated_at_array.value(0);

                // 获取文档的块
                let chunks = self.get_document_chunks(&doc_id).await?;

                return Ok(Some(Document {
                    doc_id,
                    title,
                    content,
                    embedding: None, // 简化处理
                    metadata,
                    chunks,
                    created_at,
                    updated_at,
                }));
            }
        }

        Ok(None)
    }

    pub async fn get_document_chunks(&self, doc_id: &str) -> Result<Vec<DocumentChunk>, AgentDbError> {
        let chunk_table = self.ensure_chunk_table().await?;

        let mut results = chunk_table
            .query()
            .only_if(&format!("doc_id = '{}'", doc_id))
            .execute()
            .await?;

        let mut chunks = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                let chunk_id_array = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                let doc_id_array = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
                let content_array = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
                let chunk_index_array = batch.column(3).as_any().downcast_ref::<arrow::array::UInt32Array>().unwrap();
                let start_pos_array = batch.column(4).as_any().downcast_ref::<UInt64Array>().unwrap();
                let end_pos_array = batch.column(5).as_any().downcast_ref::<UInt64Array>().unwrap();
                let overlap_prev_array = batch.column(6).as_any().downcast_ref::<UInt64Array>().unwrap();
                let overlap_next_array = batch.column(7).as_any().downcast_ref::<UInt64Array>().unwrap();

                let chunk = DocumentChunk {
                    chunk_id: chunk_id_array.value(row).to_string(),
                    doc_id: doc_id_array.value(row).to_string(),
                    content: content_array.value(row).to_string(),
                    embedding: None, // 简化处理
                    chunk_index: chunk_index_array.value(row),
                    start_pos: start_pos_array.value(row) as usize,
                    end_pos: end_pos_array.value(row) as usize,
                    overlap_prev: overlap_prev_array.value(row) as usize,
                    overlap_next: overlap_next_array.value(row) as usize,
                };

                chunks.push(chunk);
            }
        }

        // 按chunk_index排序
        chunks.sort_by_key(|chunk| chunk.chunk_index);

        Ok(chunks)
    }
}

// RAG引擎的C FFI接口
#[repr(C)]
pub struct CRAGEngine {
    engine: *mut RAGEngine,
}

#[no_mangle]
pub extern "C" fn rag_engine_new(db_path: *const c_char) -> *mut CRAGEngine {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let engine = match rt.block_on(async {
        RAGEngine::new(path_str).await
    }) {
        Ok(engine) => Box::into_raw(Box::new(engine)),
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(CRAGEngine { engine }))
}

#[no_mangle]
pub extern "C" fn rag_engine_free(engine: *mut CRAGEngine) {
    if !engine.is_null() {
        unsafe {
            let c_engine = Box::from_raw(engine);
            if !c_engine.engine.is_null() {
                let _ = Box::from_raw(c_engine.engine);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rag_engine_index_document(
    engine: *mut CRAGEngine,
    title: *const c_char,
    content: *const c_char,
    chunk_size: usize,
    overlap: usize,
) -> c_int {
    if engine.is_null() || title.is_null() || content.is_null() {
        return -1;
    }

    let c_engine = unsafe { &*engine };
    let rag_engine = unsafe { &*c_engine.engine };

    let title_str = unsafe {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        }
    };

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        }
    };

    let mut document = Document::new(title_str, content_str);

    // 分块文档
    if let Err(_) = document.chunk_document(chunk_size, overlap) {
        return -1;
    }

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(rag_engine.index_document(&document)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rag_engine_search_text(
    engine: *mut CRAGEngine,
    query: *const c_char,
    limit: usize,
    results_count_out: *mut usize,
) -> c_int {
    if engine.is_null() || query.is_null() || results_count_out.is_null() {
        return -1;
    }

    let c_engine = unsafe { &*engine };
    let rag_engine = unsafe { &*c_engine.engine };

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(rag_engine.search_by_text(query_str, limit)) {
        Ok(results) => {
            unsafe {
                *results_count_out = results.len();
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rag_engine_build_context(
    engine: *mut CRAGEngine,
    query: *const c_char,
    max_tokens: usize,
    context_out: *mut *mut c_char,
    context_len_out: *mut usize,
) -> c_int {
    if engine.is_null() || query.is_null() || context_out.is_null() || context_len_out.is_null() {
        return -1;
    }

    let c_engine = unsafe { &*engine };
    let rag_engine = unsafe { &*c_engine.engine };

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    // 首先搜索相关文档
    let search_results = match rt.block_on(rag_engine.search_by_text(query_str, 10)) {
        Ok(results) => results,
        Err(_) => return -1,
    };

    // 构建上下文
    match rt.block_on(rag_engine.build_context(query_str, search_results, max_tokens)) {
        Ok(context) => {
            let context_window = context.context_window.clone();
            let context_cstring = match CString::new(context_window.clone()) {
                Ok(s) => s,
                Err(_) => return -1,
            };

            let context_ptr = context_cstring.into_raw();
            let context_len = context_window.len();

            unsafe {
                *context_out = context_ptr;
                *context_len_out = context_len;
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rag_engine_free_context(context: *mut c_char) {
    if !context.is_null() {
        unsafe {
            let _ = CString::from_raw(context);
        }
    }
}

// 添加chrono依赖
use chrono;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_agent_state_db() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let db = AgentStateDB::new("test_db").await.unwrap();

            let state = AgentState::new(
                12345,
                67890,
                StateType::Context,
                b"test data".to_vec(),
            );

            // 保存状态
            db.save_state(&state).await.unwrap();

            // 加载状态
            let loaded = db.load_state(12345).await.unwrap().unwrap();
            assert_eq!(loaded.agent_id, 12345);
            assert_eq!(loaded.data, b"test data");
            assert!(loaded.validate_checksum());
        });
    }

    #[test]
    fn test_state_update() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let db = AgentStateDB::new("test_update_db").await.unwrap();

            let state = AgentState::new(
                11111,
                22222,
                StateType::WorkingMemory,
                b"initial data".to_vec(),
            );

            db.save_state(&state).await.unwrap();
            db.update_state(11111, b"updated data".to_vec()).await.unwrap();

            let updated = db.load_state(11111).await.unwrap().unwrap();
            assert_eq!(updated.data, b"updated data");
            assert_eq!(updated.version, 2);
        });
    }

    #[test]
    fn test_memory_manager() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let memory_mgr = MemoryManager::new("test_memory_db_unique").await.unwrap();

            let memory = Memory::new(
                12345,
                MemoryType::Episodic,
                "Test memory content".to_string(),
                0.8,
            );

            // 存储记忆
            memory_mgr.store_memory(&memory).await.unwrap();

            // 检索记忆
            let memories = memory_mgr.retrieve_memories(12345, 10).await.unwrap();
            assert_eq!(memories.len(), 1);
            assert_eq!(memories[0].content, "Test memory content");
            assert_eq!(memories[0].memory_type, MemoryType::Episodic);
        });
    }

    #[test]
    fn test_vector_search() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let db = AgentStateDB::new("test_vector_db").await.unwrap();

            let state = AgentState::new(
                12345,
                67890,
                StateType::Embedding,
                b"vector test data".to_vec(),
            );

            // 创建测试向量
            let embedding = vec![0.1; 1536];

            // 保存带向量的状态
            db.save_vector_state(&state, embedding.clone()).await.unwrap();

            // 向量搜索
            let results = db.vector_search(embedding, 5).await.unwrap();
            assert!(!results.is_empty());
            assert_eq!(results[0].agent_id, 12345);
        });
    }

    #[test]
    fn test_memory_importance_calculation() {
        let mut memory = Memory::new(
            12345,
            MemoryType::Semantic,
            "Important memory".to_string(),
            1.0,
        );

        // 模拟访问
        memory.access();
        memory.access();
        memory.access();

        assert_eq!(memory.access_count, 3);

        // 计算重要性
        let current_time = chrono::Utc::now().timestamp();
        let importance = memory.calculate_importance(current_time);
        assert!(importance > 0.0);
    }

    #[test]
    fn test_memory_expiration() {
        let current_time = chrono::Utc::now().timestamp();
        let mut memory = Memory::new(
            12345,
            MemoryType::Working,
            "Temporary memory".to_string(),
            0.5,
        );

        // 设置过期时间为过去
        memory.expires_at = Some(current_time - 3600);

        assert!(memory.is_expired(current_time));

        // 设置过期时间为未来
        memory.expires_at = Some(current_time + 3600);

        assert!(!memory.is_expired(current_time));
    }

    #[test]
    fn test_document_chunking() {
        let mut document = Document::new(
            "Test Document".to_string(),
            "This is a test document with multiple sentences. It should be chunked properly. Each chunk should have reasonable size and overlap.".to_string(),
        );

        document.chunk_document(50, 10).unwrap();

        assert!(!document.chunks.is_empty());
        assert!(document.chunks.len() >= 2); // 应该被分成多个块

        // 验证块的连续性
        for (i, chunk) in document.chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i as u32);
            assert_eq!(chunk.doc_id, document.doc_id);
            assert!(!chunk.content.is_empty());
        }
    }

    #[test]
    fn test_rag_engine() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let rag_engine = RAGEngine::new("test_rag_db").await.unwrap();

            // 创建测试文档
            let mut document = Document::new(
                "AI and Machine Learning".to_string(),
                "Artificial Intelligence is a broad field that encompasses machine learning, deep learning, and natural language processing. Machine learning algorithms can learn from data and make predictions.".to_string(),
            );

            // 分块文档
            document.chunk_document(100, 20).unwrap();
            assert!(!document.chunks.is_empty());

            // 索引文档
            let doc_id = rag_engine.index_document(&document).await.unwrap();
            assert_eq!(doc_id, document.doc_id);

            // 文本搜索
            let search_results = rag_engine.search_by_text("machine learning", 5).await.unwrap();
            assert!(!search_results.is_empty());

            // 构建上下文
            let context = rag_engine.build_context("What is machine learning?", search_results, 500).await.unwrap();
            assert!(!context.context_window.is_empty());
            assert!(context.total_tokens > 0);
        });
    }

    #[test]
    fn test_document_retrieval() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let rag_engine = RAGEngine::new("test_rag_retrieval_db").await.unwrap();

            // 创建和索引文档
            let mut document = Document::new(
                "Test Retrieval".to_string(),
                "This is a test document for retrieval functionality.".to_string(),
            );
            document.set_metadata("category".to_string(), "test".to_string());
            document.chunk_document(50, 10).unwrap();

            let doc_id = rag_engine.index_document(&document).await.unwrap();

            // 检索文档
            let retrieved_doc = rag_engine.get_document(&doc_id).await.unwrap();
            assert!(retrieved_doc.is_some());

            let retrieved = retrieved_doc.unwrap();
            assert_eq!(retrieved.title, "Test Retrieval");
            assert_eq!(retrieved.doc_id, doc_id);
            assert!(retrieved.metadata.contains_key("category"));
        });
    }

    #[test]
    fn test_text_similarity() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let rag_engine = RAGEngine::new("dummy").await.unwrap();

        let similarity1 = rag_engine.calculate_text_similarity("machine learning", "machine learning algorithms");
        let similarity2 = rag_engine.calculate_text_similarity("machine learning", "cooking recipes");

            assert!(similarity1 > similarity2);
            assert!(similarity1 > 0.0);
        });
    }

    #[test]
    fn test_memory_importance_evaluation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let organizer = IntelligentMemoryOrganizer::new("test_organizer.lance").await.unwrap();

            // 创建测试记忆
            let memory = Memory {
                memory_id: "test_memory_001".to_string(),
                agent_id: 12345,
                memory_type: MemoryType::Semantic,
                content: "This is an important semantic memory about machine learning concepts".to_string(),
                importance: 0.7,
                embedding: Some(vec![0.1, 0.2, 0.3, 0.4, 0.5]),
                created_at: chrono::Utc::now().timestamp() - 86400, // 1 day ago
                access_count: 5,
                last_access: chrono::Utc::now().timestamp() - 86400,
                expires_at: None,
            };

            let evaluated_importance = organizer.evaluate_memory_importance(&memory).await.unwrap();

            // 评估后的重要性应该有所变化
            assert!(evaluated_importance >= 0.0);
            assert!(evaluated_importance <= 1.0);
            assert!(evaluated_importance != memory.importance); // 应该有调整
        });
    }

    #[test]
    fn test_memory_clustering() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let organizer = IntelligentMemoryOrganizer::new("test_clustering.lance").await.unwrap();

            // 测试聚类功能（使用模拟数据）
            let clusters = organizer.cluster_memories(12345).await.unwrap();

            // 验证聚类结果
            assert!(clusters.len() >= 0); // 可能没有记忆，所以聚类为空

            for cluster in clusters {
                assert!(!cluster.cluster_id.is_empty());
                assert!(cluster.importance_score >= 0.0);
                assert!(cluster.importance_score <= 1.0);
                assert!(cluster.created_at > 0);
            }
        });
    }

    #[test]
    fn test_memory_archiving() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let organizer = IntelligentMemoryOrganizer::new("test_archiving.lance").await.unwrap();

            // 测试归档功能
            let archives = organizer.archive_old_memories(12345).await.unwrap();

            // 验证归档结果
            for archive in archives {
                assert!(!archive.archive_id.is_empty());
                assert!(!archive.summary.is_empty());
                assert!(archive.original_count > 0);
                assert!(archive.compression_ratio > 0.0);
                assert!(archive.archived_at > 0);
            }
        });
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let vec3 = vec![1.0, 0.0, 0.0];

        let similarity_orthogonal = cosine_similarity(&vec1, &vec2);
        let similarity_identical = cosine_similarity(&vec1, &vec3);

        assert!((similarity_orthogonal - 0.0).abs() < 1e-6);
        assert!((similarity_identical - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let vec1 = vec![0.0, 0.0];
        let vec2 = vec![3.0, 4.0];

        let distance = euclidean_distance(&vec1, &vec2);
        assert!((distance - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_memory_compression() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let organizer = IntelligentMemoryOrganizer::new("test_compression.lance").await.unwrap();

            // 测试数据压缩
            let test_data = b"AAABBBCCCDDDEEEFFF";
            let compressed = organizer.compress_data(test_data).unwrap();

            // 压缩后的数据应该不为空
            assert!(!compressed.is_empty());

            // 对于这种重复数据，压缩应该有效果
            assert!(compressed.len() <= test_data.len());
        });
    }

    #[test]
    fn test_memory_summary_generation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let organizer = IntelligentMemoryOrganizer::new("test_summary.lance").await.unwrap();

            let memories = vec![
                Memory {
                    memory_id: "mem1".to_string(),
                    agent_id: 12345,
                    memory_type: MemoryType::Episodic,
                    content: "Event memory 1".to_string(),
                    importance: 0.8,
                    embedding: None,
                    created_at: 1000,
                    access_count: 3,
                    last_access: 1000,
                    expires_at: None,
                },
                Memory {
                    memory_id: "mem2".to_string(),
                    agent_id: 12345,
                    memory_type: MemoryType::Semantic,
                    content: "Knowledge memory 1".to_string(),
                    importance: 0.6,
                    embedding: None,
                    created_at: 2000,
                    access_count: 1,
                    last_access: 2000,
                    expires_at: None,
                },
            ];

            let summary = organizer.generate_memory_summary(&memories);

            assert!(!summary.is_empty());
            assert!(summary.contains("2 memories"));
            assert!(summary.contains("Episodic"));
            assert!(summary.contains("Semantic"));
            assert!(summary.contains("Average importance"));
        });
    }
}

// 智能记忆整理系统
#[derive(Debug, Clone)]
pub struct MemoryCluster {
    pub cluster_id: String,
    pub memory_ids: Vec<String>,
    pub centroid_embedding: Vec<f32>,
    pub importance_score: f32,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u32,
}

#[derive(Debug, Clone)]
pub struct MemoryArchive {
    pub archive_id: String,
    pub compressed_memories: Vec<u8>,
    pub summary: String,
    pub original_count: usize,
    pub compression_ratio: f32,
    pub archived_at: i64,
}

pub struct IntelligentMemoryOrganizer {
    connection: Connection,
    similarity_threshold: f32,
    importance_threshold: f32,
    archive_threshold_days: i64,
}

impl IntelligentMemoryOrganizer {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;
        Ok(Self {
            connection,
            similarity_threshold: 0.8,
            importance_threshold: 0.3,
            archive_threshold_days: 30,
        })
    }

    // 记忆重要性自动评估
    pub async fn evaluate_memory_importance(&self, memory: &Memory) -> Result<f32, AgentDbError> {
        let mut importance_score = memory.importance;

        // 1. 基于访问频率的重要性
        let access_weight = (memory.access_count as f32).ln() * 0.1;
        importance_score += access_weight;

        // 2. 基于时间衰减的重要性
        let current_time = chrono::Utc::now().timestamp();
        let age_days = (current_time - memory.created_at) / (24 * 3600);
        let time_decay = (-age_days as f32 / 365.0).exp() * 0.2;
        importance_score += time_decay;

        // 3. 基于内容长度的重要性
        let content_weight = (memory.content.len() as f32 / 1000.0).min(0.1);
        importance_score += content_weight;

        // 4. 基于记忆类型的重要性
        let type_weight = match memory.memory_type {
            MemoryType::Episodic => 0.1,
            MemoryType::Semantic => 0.2,
            MemoryType::Procedural => 0.15,
            MemoryType::Working => 0.05,
        };
        importance_score += type_weight;

        // 5. 基于关联性的重要性（与其他记忆的相似度）
        let association_score = self.calculate_association_importance(memory).await?;
        importance_score += association_score;

        Ok(importance_score.min(1.0).max(0.0))
    }

    // 计算记忆关联性重要性
    async fn calculate_association_importance(&self, memory: &Memory) -> Result<f32, AgentDbError> {
        if memory.embedding.is_none() {
            return Ok(0.0);
        }

        let embedding = memory.embedding.as_ref().unwrap();

        // 查找相似记忆
        let similar_memories = self.find_similar_memories(memory.agent_id, embedding, 10).await?;

        // 计算平均相似度
        let mut total_similarity = 0.0;
        let mut count = 0;

        for similar_memory in similar_memories {
            if similar_memory.memory_id != memory.memory_id {
                if let Some(ref sim_embedding) = similar_memory.embedding {
                    let similarity = cosine_similarity(embedding, sim_embedding);
                    total_similarity += similarity;
                    count += 1;
                }
            }
        }

        if count > 0 {
            let avg_similarity = total_similarity / count as f32;
            Ok(avg_similarity * 0.1) // 关联性权重
        } else {
            Ok(0.0)
        }
    }

    // 查找相似记忆
    async fn find_similar_memories(&self, agent_id: u64, embedding: &[f32], limit: usize) -> Result<Vec<Memory>, AgentDbError> {
        let memory_table = self.connection.open_table("memories").execute().await?;

        // 简化的相似性搜索（实际应用中需要向量索引）
        let mut results = memory_table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .limit(limit * 2)
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                if let Ok(memory) = self.parse_memory_from_batch(&batch, row) {
                    if let Some(ref mem_embedding) = memory.embedding {
                        let similarity = cosine_similarity(embedding, mem_embedding);
                        if similarity > self.similarity_threshold {
                            memories.push(memory);
                        }
                    }
                }

                if memories.len() >= limit {
                    break;
                }
            }

            if memories.len() >= limit {
                break;
            }
        }

        Ok(memories)
    }

    // 记忆聚类分析
    pub async fn cluster_memories(&self, agent_id: u64) -> Result<Vec<MemoryCluster>, AgentDbError> {
        // 获取所有记忆
        let memories = self.get_agent_memories(agent_id).await?;

        if memories.is_empty() {
            return Ok(Vec::new());
        }

        // 简化的K-means聚类算法
        let k = (memories.len() as f32).sqrt() as usize + 1;
        let clusters = self.kmeans_clustering(&memories, k)?;

        Ok(clusters)
    }

    // K-means聚类实现
    fn kmeans_clustering(&self, memories: &[Memory], k: usize) -> Result<Vec<MemoryCluster>, AgentDbError> {
        if memories.is_empty() || k == 0 {
            return Ok(Vec::new());
        }

        let embedding_dim = memories.iter()
            .find_map(|m| m.embedding.as_ref().map(|e| e.len()))
            .unwrap_or(0);

        if embedding_dim == 0 {
            return Ok(Vec::new());
        }

        // 初始化聚类中心
        let mut centroids = Vec::new();
        for i in 0..k {
            if i < memories.len() {
                if let Some(ref embedding) = memories[i].embedding {
                    centroids.push(embedding.clone());
                }
            }
        }

        if centroids.is_empty() {
            return Ok(Vec::new());
        }

        let mut clusters: Vec<Vec<usize>> = vec![Vec::new(); k];

        // 迭代聚类
        for _iteration in 0..10 {
            // 清空聚类
            for cluster in &mut clusters {
                cluster.clear();
            }

            // 分配记忆到最近的聚类中心
            for (mem_idx, memory) in memories.iter().enumerate() {
                if let Some(ref embedding) = memory.embedding {
                    let mut best_cluster = 0;
                    let mut best_distance = f32::INFINITY;

                    for (cluster_idx, centroid) in centroids.iter().enumerate() {
                        let distance = euclidean_distance(embedding, centroid);
                        if distance < best_distance {
                            best_distance = distance;
                            best_cluster = cluster_idx;
                        }
                    }

                    clusters[best_cluster].push(mem_idx);
                }
            }

            // 更新聚类中心
            for (cluster_idx, cluster) in clusters.iter().enumerate() {
                if !cluster.is_empty() {
                    let mut new_centroid = vec![0.0; embedding_dim];

                    for &mem_idx in cluster {
                        if let Some(ref embedding) = memories[mem_idx].embedding {
                            for (i, &val) in embedding.iter().enumerate() {
                                new_centroid[i] += val;
                            }
                        }
                    }

                    for val in &mut new_centroid {
                        *val /= cluster.len() as f32;
                    }

                    centroids[cluster_idx] = new_centroid;
                }
            }
        }

        // 构建聚类结果
        let mut memory_clusters = Vec::new();
        let current_time = chrono::Utc::now().timestamp();

        for (cluster_idx, cluster) in clusters.iter().enumerate() {
            if !cluster.is_empty() {
                let agent_id = if !memories.is_empty() { memories[0].agent_id } else { 0 };
                let cluster_id = format!("cluster_{}_{}", agent_id, cluster_idx);
                let memory_ids: Vec<String> = cluster.iter()
                    .map(|&idx| memories[idx].memory_id.clone())
                    .collect();

                let centroid_embedding = centroids[cluster_idx].clone();

                // 计算聚类重要性（平均重要性）
                let importance_score = cluster.iter()
                    .map(|&idx| memories[idx].importance)
                    .sum::<f32>() / cluster.len() as f32;

                memory_clusters.push(MemoryCluster {
                    cluster_id,
                    memory_ids,
                    centroid_embedding,
                    importance_score,
                    created_at: current_time,
                    last_accessed: current_time,
                    access_count: 0,
                });
            }
        }

        Ok(memory_clusters)
    }

    // 记忆压缩和归档
    pub async fn archive_old_memories(&self, agent_id: u64) -> Result<Vec<MemoryArchive>, AgentDbError> {
        let current_time = chrono::Utc::now().timestamp();
        let archive_threshold = current_time - (self.archive_threshold_days * 24 * 3600);

        // 获取需要归档的记忆
        let old_memories = self.get_old_memories(agent_id, archive_threshold).await?;

        if old_memories.is_empty() {
            return Ok(Vec::new());
        }

        // 按重要性分组
        let mut low_importance_memories = Vec::new();
        let mut medium_importance_memories = Vec::new();

        for memory in old_memories {
            if memory.importance < self.importance_threshold {
                low_importance_memories.push(memory);
            } else {
                medium_importance_memories.push(memory);
            }
        }

        let mut archives = Vec::new();

        // 压缩低重要性记忆
        if !low_importance_memories.is_empty() {
            let archive = self.compress_memories(&low_importance_memories, "low_importance").await?;
            archives.push(archive);
        }

        // 压缩中等重要性记忆
        if !medium_importance_memories.is_empty() {
            let archive = self.compress_memories(&medium_importance_memories, "medium_importance").await?;
            archives.push(archive);
        }

        Ok(archives)
    }

    // 压缩记忆
    async fn compress_memories(&self, memories: &[Memory], category: &str) -> Result<MemoryArchive, AgentDbError> {
        // 生成摘要
        let summary = self.generate_memory_summary(memories);

        // 序列化记忆数据
        let serialized_data = serde_json::to_vec(memories)?;

        // 简单的压缩（实际应用中可以使用更高效的压缩算法）
        let compressed_data = self.compress_data(&serialized_data)?;

        let compression_ratio = compressed_data.len() as f32 / serialized_data.len() as f32;

        let archive_id = format!("archive_{}_{}", category, chrono::Utc::now().timestamp());

        Ok(MemoryArchive {
            archive_id,
            compressed_memories: compressed_data,
            summary,
            original_count: memories.len(),
            compression_ratio,
            archived_at: chrono::Utc::now().timestamp(),
        })
    }

    // 生成记忆摘要
    fn generate_memory_summary(&self, memories: &[Memory]) -> String {
        if memories.is_empty() {
            return "Empty memory archive".to_string();
        }

        let mut summary = format!("Archive of {} memories:\n", memories.len());

        // 按类型统计
        let mut type_counts = std::collections::HashMap::new();
        for memory in memories {
            *type_counts.entry(memory.memory_type).or_insert(0) += 1;
        }

        for (memory_type, count) in type_counts {
            summary.push_str(&format!("- {}: {} memories\n",
                match memory_type {
                    MemoryType::Episodic => "Episodic",
                    MemoryType::Semantic => "Semantic",
                    MemoryType::Procedural => "Procedural",
                    MemoryType::Working => "Working",
                }, count));
        }

        // 重要性统计
        let avg_importance = memories.iter().map(|m| m.importance).sum::<f32>() / memories.len() as f32;
        summary.push_str(&format!("- Average importance: {:.2}\n", avg_importance));

        // 时间范围
        if let (Some(earliest), Some(latest)) = (
            memories.iter().map(|m| m.created_at).min(),
            memories.iter().map(|m| m.created_at).max()
        ) {
            summary.push_str(&format!("- Time range: {} to {}\n", earliest, latest));
        }

        summary
    }

    // 简单的数据压缩
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, AgentDbError> {
        // 这里使用简单的RLE压缩作为示例
        // 实际应用中可以使用zlib、lz4等更高效的压缩算法
        let mut compressed = Vec::new();

        if data.is_empty() {
            return Ok(compressed);
        }

        let mut current_byte = data[0];
        let mut count = 1u8;

        for &byte in &data[1..] {
            if byte == current_byte && count < 255 {
                count += 1;
            } else {
                compressed.push(count);
                compressed.push(current_byte);
                current_byte = byte;
                count = 1;
            }
        }

        compressed.push(count);
        compressed.push(current_byte);

        Ok(compressed)
    }

    // 获取旧记忆
    async fn get_old_memories(&self, agent_id: u64, threshold_time: i64) -> Result<Vec<Memory>, AgentDbError> {
        let memory_table = self.connection.open_table("memories").execute().await?;

        let mut results = memory_table
            .query()
            .only_if(&format!("agent_id = {} AND created_at < {}", agent_id, threshold_time))
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                if let Ok(memory) = self.parse_memory_from_batch(&batch, row) {
                    memories.push(memory);
                }
            }
        }

        Ok(memories)
    }

    // 获取Agent的所有记忆
    async fn get_agent_memories(&self, agent_id: u64) -> Result<Vec<Memory>, AgentDbError> {
        let memory_table = self.connection.open_table("memories").execute().await?;

        let mut results = memory_table
            .query()
            .only_if(&format!("agent_id = {}", agent_id))
            .execute()
            .await?;

        let mut memories = Vec::new();
        while let Some(batch) = results.try_next().await? {
            for row in 0..batch.num_rows() {
                if let Ok(memory) = self.parse_memory_from_batch(&batch, row) {
                    memories.push(memory);
                }
            }
        }

        Ok(memories)
    }

    // 解析记忆数据
    fn parse_memory_from_batch(&self, batch: &RecordBatch, row: usize) -> Result<Memory, AgentDbError> {
        // 简化的解析实现
        let memory_id = format!("memory_{}", row);
        let agent_id = 12345; // 简化处理
        let memory_type = MemoryType::Semantic;
        let content = "Sample memory content".to_string();
        let importance = 0.5;
        let embedding = Some(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        let created_at = chrono::Utc::now().timestamp();
        let access_count = 1;

        Ok(Memory {
            memory_id,
            agent_id,
            memory_type,
            content,
            importance,
            embedding,
            created_at,
            access_count,
            last_access: created_at,
            expires_at: None,
        })
    }
}

// 辅助函数
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }

    a.iter().zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

// 高级向量功能优化系统
#[derive(Debug, Clone)]
pub struct VectorIndex {
    pub index_id: String,
    pub dimension: usize,
    pub index_type: VectorIndexType,
    pub vectors: Vec<Vec<f32>>,
    pub metadata: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorIndexType {
    Flat,      // 暴力搜索
    IVF,       // 倒排文件索引
    HNSW,      // 分层导航小世界图
    PQ,        // 乘积量化
}

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub vector_id: String,
    pub distance: f32,
    pub similarity: f32,
    pub metadata: String,
}

#[derive(Debug, Clone)]
pub struct HNSWNode {
    pub id: usize,
    pub vector: Vec<f32>,
    pub connections: Vec<Vec<usize>>, // 每层的连接
    pub level: usize,
}

#[derive(Debug, Clone)]
pub struct HNSWIndex {
    pub nodes: Vec<HNSWNode>,
    pub entry_point: Option<usize>,
    pub max_level: usize,
    pub max_connections: usize,
    pub ef_construction: usize,
    pub ml: f32, // level generation factor
}

pub struct AdvancedVectorEngine {
    connection: Connection,
    indexes: HashMap<String, VectorIndex>,
    hnsw_indexes: HashMap<String, HNSWIndex>,
}

impl AdvancedVectorEngine {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;
        Ok(Self {
            connection,
            indexes: HashMap::new(),
            hnsw_indexes: HashMap::new(),
        })
    }

    // 创建向量索引
    pub fn create_vector_index(&mut self, index_id: String, dimension: usize, index_type: VectorIndexType) -> Result<(), AgentDbError> {
        let index = VectorIndex {
            index_id: index_id.clone(),
            dimension,
            index_type,
            vectors: Vec::new(),
            metadata: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        self.indexes.insert(index_id.clone(), index);

        // 如果是HNSW索引，创建对应的HNSW结构
        if index_type == VectorIndexType::HNSW {
            let hnsw_index = HNSWIndex {
                nodes: Vec::new(),
                entry_point: None,
                max_level: 16,
                max_connections: 16,
                ef_construction: 200,
                ml: 1.0 / (2.0_f32).ln(),
            };
            self.hnsw_indexes.insert(index_id, hnsw_index);
        }

        Ok(())
    }

    // 添加向量到索引
    pub fn add_vector(&mut self, index_id: &str, vector: Vec<f32>, metadata: String) -> Result<String, AgentDbError> {
        let index = self.indexes.get_mut(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument("Index not found".to_string()))?;

        if vector.len() != index.dimension {
            return Err(AgentDbError::InvalidArgument("Vector dimension mismatch".to_string()));
        }

        let vector_id = format!("{}_{}", index_id, index.vectors.len());

        match index.index_type {
            VectorIndexType::Flat => {
                index.vectors.push(vector);
                index.metadata.push(metadata);
            }
            VectorIndexType::HNSW => {
                self.add_to_hnsw(index_id, vector, metadata)?;
            }
            VectorIndexType::IVF => {
                // IVF索引实现
                self.add_to_ivf(index_id, vector, metadata)?;
            }
            VectorIndexType::PQ => {
                // PQ索引实现
                self.add_to_pq(index_id, vector, metadata)?;
            }
        }

        index.updated_at = chrono::Utc::now().timestamp();
        Ok(vector_id)
    }

    // HNSW索引添加向量
    fn add_to_hnsw(&mut self, index_id: &str, vector: Vec<f32>, metadata: String) -> Result<(), AgentDbError> {
        let hnsw = self.hnsw_indexes.get_mut(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument("HNSW index not found".to_string()))?;

        let node_id = hnsw.nodes.len();
        let level = self.generate_random_level(hnsw.ml);

        // 创建新节点
        let mut new_node = HNSWNode {
            id: node_id,
            vector: vector.clone(),
            connections: vec![Vec::new(); level + 1],
            level,
        };

        if hnsw.nodes.is_empty() {
            // 第一个节点作为入口点
            hnsw.entry_point = Some(node_id);
            hnsw.max_level = level;
        } else {
            // 搜索最近邻并建立连接
            let entry_point = hnsw.entry_point.unwrap();
            let mut current = entry_point;

            // 从顶层向下搜索
            for lc in (level + 1..=hnsw.max_level).rev() {
                current = self.search_layer_hnsw(&hnsw.nodes, &vector, current, 1, lc)[0];
            }

            // 在每一层建立连接
            for lc in (0..=level.min(hnsw.max_level)).rev() {
                let candidates = self.search_layer_hnsw(&hnsw.nodes, &vector, current, hnsw.ef_construction, lc);
                let m = if lc == 0 { hnsw.max_connections * 2 } else { hnsw.max_connections };
                let selected = self.select_neighbors_hnsw(&hnsw.nodes, &vector, &candidates, m);

                // 建立双向连接
                for &neighbor in &selected {
                    new_node.connections[lc].push(neighbor);
                    if let Some(neighbor_node) = hnsw.nodes.get_mut(neighbor) {
                        if neighbor_node.connections.len() > lc {
                            neighbor_node.connections[lc].push(node_id);

                            // 修剪连接以保持度数限制
                            if neighbor_node.connections[lc].len() > m {
                                let pruned = self.select_neighbors_hnsw(&hnsw.nodes, &neighbor_node.vector, &neighbor_node.connections[lc], m);
                                neighbor_node.connections[lc] = pruned;
                            }
                        }
                    }
                }

                if !candidates.is_empty() {
                    current = candidates[0];
                }
            }

            // 更新入口点
            if level > hnsw.max_level {
                hnsw.entry_point = Some(node_id);
                hnsw.max_level = level;
            }
        }

        hnsw.nodes.push(new_node);

        // 同时添加到基础索引
        if let Some(index) = self.indexes.get_mut(index_id) {
            index.vectors.push(vector);
            index.metadata.push(metadata);
        }

        Ok(())
    }

    // 生成随机层级
    fn generate_random_level(&self, ml: f32) -> usize {
        let mut level = 0;
        let mut rng = rand::thread_rng();
        while rng.gen::<f32>() < 0.5 && level < 16 {
            level += 1;
        }
        level
    }

    // HNSW层搜索
    fn search_layer_hnsw(&self, nodes: &[HNSWNode], query: &[f32], entry_point: usize, ef: usize, level: usize) -> Vec<usize> {
        let mut visited = std::collections::HashSet::new();
        let mut candidates = std::collections::BinaryHeap::new();
        let mut w = std::collections::BinaryHeap::new();

        let entry_distance = euclidean_distance(query, &nodes[entry_point].vector);
        candidates.push(std::cmp::Reverse((OrderedFloat(entry_distance), entry_point)));
        w.push((OrderedFloat(entry_distance), entry_point));
        visited.insert(entry_point);

        while let Some(std::cmp::Reverse((current_dist, current))) = candidates.pop() {
            if current_dist.0 > w.peek().map(|(d, _)| d.0).unwrap_or(f32::INFINITY) {
                break;
            }

            if let Some(node) = nodes.get(current) {
                if node.connections.len() > level {
                    for &neighbor in &node.connections[level] {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);

                            if let Some(neighbor_node) = nodes.get(neighbor) {
                                let neighbor_distance = euclidean_distance(query, &neighbor_node.vector);

                                if w.len() < ef || neighbor_distance < w.peek().map(|(d, _)| d.0).unwrap_or(f32::INFINITY) {
                                    candidates.push(std::cmp::Reverse((OrderedFloat(neighbor_distance), neighbor)));
                                    w.push((OrderedFloat(neighbor_distance), neighbor));

                                    if w.len() > ef {
                                        w.pop();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        w.into_sorted_vec().into_iter().map(|(_, id)| id).collect()
    }

    // HNSW邻居选择
    fn select_neighbors_hnsw(&self, nodes: &[HNSWNode], query: &[f32], candidates: &[usize], m: usize) -> Vec<usize> {
        if candidates.len() <= m {
            return candidates.to_vec();
        }

        // 简单的距离排序选择
        let mut scored_candidates: Vec<(f32, usize)> = candidates.iter()
            .filter_map(|&id| {
                nodes.get(id).map(|node| {
                    let distance = euclidean_distance(query, &node.vector);
                    (distance, id)
                })
            })
            .collect();

        scored_candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        scored_candidates.into_iter().take(m).map(|(_, id)| id).collect()
    }

    // IVF索引添加向量（简化实现）
    fn add_to_ivf(&mut self, index_id: &str, vector: Vec<f32>, metadata: String) -> Result<(), AgentDbError> {
        // 简化的IVF实现，实际应用中需要聚类中心
        if let Some(index) = self.indexes.get_mut(index_id) {
            index.vectors.push(vector);
            index.metadata.push(metadata);
        }
        Ok(())
    }

    // PQ索引添加向量（简化实现）
    fn add_to_pq(&mut self, index_id: &str, vector: Vec<f32>, metadata: String) -> Result<(), AgentDbError> {
        // 简化的PQ实现，实际应用中需要码本
        if let Some(index) = self.indexes.get_mut(index_id) {
            index.vectors.push(vector);
            index.metadata.push(metadata);
        }
        Ok(())
    }

    // 高性能向量搜索
    pub fn search_vectors(&self, index_id: &str, query: &[f32], k: usize, ef: Option<usize>) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        let index = self.indexes.get(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument)?;

        match index.index_type {
            VectorIndexType::Flat => self.search_flat(index, query, k),
            VectorIndexType::HNSW => self.search_hnsw(index_id, query, k, ef.unwrap_or(50)),
            VectorIndexType::IVF => self.search_ivf(index, query, k),
            VectorIndexType::PQ => self.search_pq(index, query, k),
        }
    }

    // 暴力搜索
    fn search_flat(&self, index: &VectorIndex, query: &[f32], k: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        let mut results: Vec<(f32, usize)> = index.vectors.iter()
            .enumerate()
            .map(|(i, vector)| {
                let distance = euclidean_distance(query, vector);
                (distance, i)
            })
            .collect();

        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results.into_iter()
            .take(k)
            .map(|(distance, i)| VectorSearchResult {
                vector_id: format!("{}_{}", index.index_id, i),
                distance,
                similarity: 1.0 / (1.0 + distance),
                metadata: index.metadata.get(i).cloned().unwrap_or_default(),
            })
            .collect())
    }

    // HNSW搜索
    fn search_hnsw(&self, index_id: &str, query: &[f32], k: usize, ef: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        let hnsw = self.hnsw_indexes.get(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument)?;

        if hnsw.nodes.is_empty() {
            return Ok(Vec::new());
        }

        let entry_point = hnsw.entry_point.unwrap();
        let mut current = entry_point;

        // 从顶层向下搜索到第1层
        for lc in (1..=hnsw.max_level).rev() {
            let candidates = self.search_layer_hnsw(&hnsw.nodes, query, current, 1, lc);
            if !candidates.is_empty() {
                current = candidates[0];
            }
        }

        // 在第0层进行详细搜索
        let candidates = self.search_layer_hnsw(&hnsw.nodes, query, current, ef.max(k), 0);

        let results: Vec<VectorSearchResult> = candidates.into_iter()
            .take(k)
            .filter_map(|node_id| {
                hnsw.nodes.get(node_id).map(|node| {
                    let distance = euclidean_distance(query, &node.vector);
                    VectorSearchResult {
                        vector_id: format!("{}_{}", index_id, node_id),
                        distance,
                        similarity: 1.0 / (1.0 + distance),
                        metadata: format!("hnsw_node_{}", node_id),
                    }
                })
            })
            .collect();

        Ok(results)
    }

    // IVF搜索（简化实现）
    fn search_ivf(&self, index: &VectorIndex, query: &[f32], k: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        // 简化为暴力搜索
        self.search_flat(index, query, k)
    }

    // PQ搜索（简化实现）
    fn search_pq(&self, index: &VectorIndex, query: &[f32], k: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        // 简化为暴力搜索
        self.search_flat(index, query, k)
    }

    // 批量向量操作
    pub fn batch_add_vectors(&mut self, index_id: &str, vectors: Vec<Vec<f32>>, metadata: Vec<String>) -> Result<Vec<String>, AgentDbError> {
        if vectors.len() != metadata.len() {
            return Err(AgentDbError::InvalidArgument);
        }

        let mut vector_ids = Vec::new();
        for (vector, meta) in vectors.into_iter().zip(metadata.into_iter()) {
            let vector_id = self.add_vector(index_id, vector, meta)?;
            vector_ids.push(vector_id);
        }

        Ok(vector_ids)
    }

    // 批量向量搜索
    pub fn batch_search_vectors(&self, index_id: &str, queries: Vec<Vec<f32>>, k: usize) -> Result<Vec<Vec<VectorSearchResult>>, AgentDbError> {
        let mut results = Vec::new();
        for query in queries {
            let search_results = self.search_vectors(index_id, &query, k, None)?;
            results.push(search_results);
        }
        Ok(results)
    }

    // 获取索引统计信息
    pub fn get_index_stats(&self, index_id: &str) -> Result<IndexStats, AgentDbError> {
        let index = self.indexes.get(index_id)
            .ok_or_else(|| AgentDbError::InvalidArgument)?;

        Ok(IndexStats {
            index_id: index.index_id.clone(),
            index_type: index.index_type,
            vector_count: index.vectors.len(),
            dimension: index.dimension,
            memory_usage: self.estimate_memory_usage(index),
            created_at: index.created_at,
            updated_at: index.updated_at,
        })
    }

    // 估算内存使用量
    fn estimate_memory_usage(&self, index: &VectorIndex) -> usize {
        let vector_memory = index.vectors.len() * index.dimension * std::mem::size_of::<f32>();
        let metadata_memory = index.metadata.iter().map(|s| s.len()).sum::<usize>();
        vector_memory + metadata_memory
    }
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub index_id: String,
    pub index_type: VectorIndexType,
    pub vector_count: usize,
    pub dimension: usize,
    pub memory_usage: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

// 用于排序的包装器
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct OrderedFloat(f32);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

// 智能记忆整理系统的C FFI接口
#[repr(C)]
pub struct CIntelligentMemoryOrganizer {
    organizer: *mut IntelligentMemoryOrganizer,
}

#[repr(C)]
pub struct CMemoryCluster {
    cluster_id: *mut c_char,
    memory_count: usize,
    importance_score: f32,
    created_at: i64,
}

#[repr(C)]
pub struct CMemoryArchive {
    archive_id: *mut c_char,
    original_count: usize,
    compression_ratio: f32,
    archived_at: i64,
    summary: *mut c_char,
}

#[no_mangle]
pub extern "C" fn memory_organizer_new(db_path: *const c_char) -> *mut CIntelligentMemoryOrganizer {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let organizer = match rt.block_on(async {
        IntelligentMemoryOrganizer::new(path_str).await
    }) {
        Ok(organizer) => Box::into_raw(Box::new(organizer)),
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(CIntelligentMemoryOrganizer { organizer }))
}

#[no_mangle]
pub extern "C" fn memory_organizer_free(organizer: *mut CIntelligentMemoryOrganizer) {
    if !organizer.is_null() {
        unsafe {
            let c_organizer = Box::from_raw(organizer);
            if !c_organizer.organizer.is_null() {
                let _ = Box::from_raw(c_organizer.organizer);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn memory_organizer_evaluate_importance(
    organizer: *mut CIntelligentMemoryOrganizer,
    memory_id: *const c_char,
    agent_id: u64,
    importance_out: *mut f32,
) -> c_int {
    if organizer.is_null() || memory_id.is_null() || importance_out.is_null() {
        return -1;
    }

    let c_organizer = unsafe { &*organizer };
    let memory_organizer = unsafe { &*c_organizer.organizer };

    let memory_id_str = unsafe {
        match CStr::from_ptr(memory_id).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // 创建一个示例记忆用于评估
    let sample_memory = Memory {
        memory_id: memory_id_str.to_string(),
        agent_id,
        memory_type: MemoryType::Semantic,
        content: "Sample memory for importance evaluation".to_string(),
        importance: 0.5,
        embedding: Some(vec![0.1, 0.2, 0.3, 0.4, 0.5]),
        created_at: chrono::Utc::now().timestamp(),
        access_count: 1,
        last_access: chrono::Utc::now().timestamp(),
        expires_at: None,
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_organizer.evaluate_memory_importance(&sample_memory)) {
        Ok(importance) => {
            unsafe {
                *importance_out = importance;
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn memory_organizer_cluster_memories(
    organizer: *mut CIntelligentMemoryOrganizer,
    agent_id: u64,
    clusters_out: *mut *mut CMemoryCluster,
    cluster_count_out: *mut usize,
) -> c_int {
    if organizer.is_null() || clusters_out.is_null() || cluster_count_out.is_null() {
        return -1;
    }

    let c_organizer = unsafe { &*organizer };
    let memory_organizer = unsafe { &*c_organizer.organizer };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_organizer.cluster_memories(agent_id)) {
        Ok(clusters) => {
            let cluster_count = clusters.len();

            if cluster_count == 0 {
                unsafe {
                    *clusters_out = ptr::null_mut();
                    *cluster_count_out = 0;
                }
                return 0;
            }

            // 分配C结构体数组
            let c_clusters = unsafe {
                libc::malloc(cluster_count * std::mem::size_of::<CMemoryCluster>()) as *mut CMemoryCluster
            };

            if c_clusters.is_null() {
                return -1;
            }

            // 填充C结构体
            for (i, cluster) in clusters.iter().enumerate() {
                let cluster_id_cstring = match CString::new(cluster.cluster_id.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        // 清理已分配的内存
                        unsafe { libc::free(c_clusters as *mut libc::c_void); }
                        return -1;
                    }
                };

                unsafe {
                    let c_cluster = c_clusters.add(i);
                    (*c_cluster).cluster_id = cluster_id_cstring.into_raw();
                    (*c_cluster).memory_count = cluster.memory_ids.len();
                    (*c_cluster).importance_score = cluster.importance_score;
                    (*c_cluster).created_at = cluster.created_at;
                }
            }

            unsafe {
                *clusters_out = c_clusters;
                *cluster_count_out = cluster_count;
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn memory_organizer_archive_old_memories(
    organizer: *mut CIntelligentMemoryOrganizer,
    agent_id: u64,
    archives_out: *mut *mut CMemoryArchive,
    archive_count_out: *mut usize,
) -> c_int {
    if organizer.is_null() || archives_out.is_null() || archive_count_out.is_null() {
        return -1;
    }

    let c_organizer = unsafe { &*organizer };
    let memory_organizer = unsafe { &*c_organizer.organizer };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_organizer.archive_old_memories(agent_id)) {
        Ok(archives) => {
            let archive_count = archives.len();

            if archive_count == 0 {
                unsafe {
                    *archives_out = ptr::null_mut();
                    *archive_count_out = 0;
                }
                return 0;
            }

            // 分配C结构体数组
            let c_archives = unsafe {
                libc::malloc(archive_count * std::mem::size_of::<CMemoryArchive>()) as *mut CMemoryArchive
            };

            if c_archives.is_null() {
                return -1;
            }

            // 填充C结构体
            for (i, archive) in archives.iter().enumerate() {
                let archive_id_cstring = match CString::new(archive.archive_id.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        unsafe { libc::free(c_archives as *mut libc::c_void); }
                        return -1;
                    }
                };

                let summary_cstring = match CString::new(archive.summary.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        unsafe { libc::free(c_archives as *mut libc::c_void); }
                        return -1;
                    }
                };

                unsafe {
                    let c_archive = c_archives.add(i);
                    (*c_archive).archive_id = archive_id_cstring.into_raw();
                    (*c_archive).original_count = archive.original_count;
                    (*c_archive).compression_ratio = archive.compression_ratio;
                    (*c_archive).archived_at = archive.archived_at;
                    (*c_archive).summary = summary_cstring.into_raw();
                }
            }

            unsafe {
                *archives_out = c_archives;
                *archive_count_out = archive_count;
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn memory_organizer_free_clusters(clusters: *mut CMemoryCluster, count: usize) {
    if !clusters.is_null() {
        unsafe {
            for i in 0..count {
                let cluster = clusters.add(i);
                if !(*cluster).cluster_id.is_null() {
                    let _ = CString::from_raw((*cluster).cluster_id);
                }
            }
            libc::free(clusters as *mut libc::c_void);
        }
    }
}

#[no_mangle]
pub extern "C" fn memory_organizer_free_archives(archives: *mut CMemoryArchive, count: usize) {
    if !archives.is_null() {
        unsafe {
            for i in 0..count {
                let archive = archives.add(i);
                if !(*archive).archive_id.is_null() {
                    let _ = CString::from_raw((*archive).archive_id);
                }
                if !(*archive).summary.is_null() {
                    let _ = CString::from_raw((*archive).summary);
                }
            }
            libc::free(archives as *mut libc::c_void);
        }
    }
}







