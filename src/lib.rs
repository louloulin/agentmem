// Agent状态数据库 - 基于LanceDB的Rust实现
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Arc;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
}







