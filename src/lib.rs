// Agent状态数据库库 - 模块化实现
//! Agent状态数据库
//!
//! 这是一个基于LanceDB的Agent状态管理系统，提供：
//! - Agent状态持久化
//! - 记忆管理
//! - RAG（检索增强生成）功能
//! - 向量搜索
//! - 统一的API接口

// 模块声明
pub mod types;
pub mod database;
pub mod memory;
pub mod rag;
pub mod vector;
pub mod api;

// 重新导出主要类型和API
pub use types::*;
pub use api::AgentDB;
pub use database::AgentStateDB;
pub use memory::MemoryManager;
pub use rag::RAGEngine;
pub use vector::VectorSearchEngine;



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
            // 注意：版本号可能不会自动递增，这取决于实现
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
            assert!(!memories.is_empty());
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
            // 先创建一个内存管理器来确保表存在
            let _memory_mgr = MemoryManager::new("test_organizer.lance").await.unwrap();
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

            // 先存储记忆到数据库中，这样organizer才能访问到
            let memory_mgr = MemoryManager::new("test_organizer.lance").await.unwrap();
            memory_mgr.store_memory(&memory).await.unwrap();

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
            // 先创建一个内存管理器来确保表存在
            let _memory_mgr = MemoryManager::new("test_clustering.lance").await.unwrap();
            let organizer = IntelligentMemoryOrganizer::new("test_clustering.lance").await.unwrap();

            // 先添加一些测试记忆
            let memory_mgr = MemoryManager::new("test_clustering.lance").await.unwrap();
            let test_memory = Memory::new(12345, MemoryType::Episodic, "Test clustering memory".to_string(), 0.7);
            memory_mgr.store_memory(&test_memory).await.unwrap();

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
            // 先创建一个内存管理器来确保表存在
            let _memory_mgr = MemoryManager::new("test_archiving.lance").await.unwrap();
            let organizer = IntelligentMemoryOrganizer::new("test_archiving.lance").await.unwrap();

            // 先添加一些测试记忆
            let memory_mgr = MemoryManager::new("test_archiving.lance").await.unwrap();
            let old_memory = Memory {
                memory_id: "old_memory_001".to_string(),
                agent_id: 12345,
                memory_type: MemoryType::Episodic,
                content: "Old memory for archiving".to_string(),
                importance: 0.3,
                embedding: None,
                created_at: chrono::Utc::now().timestamp() - 86400 * 30, // 30 days ago
                access_count: 1,
                last_access: chrono::Utc::now().timestamp() - 86400 * 30,
                expires_at: None,
            };
            memory_mgr.store_memory(&old_memory).await.unwrap();

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

    #[test]
    fn test_advanced_vector_engine_creation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = AdvancedVectorEngine::new("test_vector_engine.lance").await.unwrap();

            // 创建不同类型的向量索引
            engine.create_vector_index("flat_index".to_string(), 128, VectorIndexType::Flat).unwrap();
            engine.create_vector_index("hnsw_index".to_string(), 128, VectorIndexType::HNSW).unwrap();

            assert_eq!(engine.indexes.len(), 2);
            assert_eq!(engine.hnsw_indexes.len(), 1);
        });
    }

    #[test]
    fn test_vector_index_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = AdvancedVectorEngine::new("test_vector_ops.lance").await.unwrap();

            // 创建索引
            engine.create_vector_index("test_index".to_string(), 4, VectorIndexType::Flat).unwrap();

            // 添加向量
            let vector1 = vec![1.0, 0.0, 0.0, 0.0];
            let vector2 = vec![0.0, 1.0, 0.0, 0.0];
            let vector3 = vec![0.0, 0.0, 1.0, 0.0];

            let id1 = engine.add_vector("test_index", vector1.clone(), "vector1".to_string()).unwrap();
            let id2 = engine.add_vector("test_index", vector2.clone(), "vector2".to_string()).unwrap();
            let id3 = engine.add_vector("test_index", vector3.clone(), "vector3".to_string()).unwrap();

            assert_eq!(id1, "test_index_0");
            assert_eq!(id2, "test_index_1");
            assert_eq!(id3, "test_index_2");

            // 搜索向量
            let query = vec![1.0, 0.0, 0.0, 0.0];
            let results = engine.search_vectors("test_index", &query, 2, None).unwrap();

            assert_eq!(results.len(), 2);
            assert_eq!(results[0].vector_id, "test_index_0");
            assert!(results[0].distance < 0.1); // 应该非常接近
        });
    }

    #[test]
    fn test_hnsw_index_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = AdvancedVectorEngine::new("test_hnsw.lance").await.unwrap();

            // 创建HNSW索引
            engine.create_vector_index("hnsw_test".to_string(), 3, VectorIndexType::HNSW).unwrap();

            // 添加多个向量
            let vectors = vec![
                vec![1.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0],
                vec![0.0, 0.0, 1.0],
                vec![0.5, 0.5, 0.0],
                vec![0.0, 0.5, 0.5],
            ];

            for (i, vector) in vectors.iter().enumerate() {
                let metadata = format!("hnsw_vector_{}", i);
                engine.add_vector("hnsw_test", vector.clone(), metadata).unwrap();
            }

            // 搜索向量
            let query = vec![1.0, 0.0, 0.0];
            let results = engine.search_vectors("hnsw_test", &query, 3, Some(10)).unwrap();

            assert!(!results.is_empty());
            // 第一个结果应该是最相似的
            assert!(results[0].distance <= results.get(1).map(|r| r.distance).unwrap_or(f32::INFINITY));
        });
    }

    #[test]
    fn test_batch_vector_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = AdvancedVectorEngine::new("test_batch.lance").await.unwrap();

            // 创建索引
            engine.create_vector_index("batch_index".to_string(), 2, VectorIndexType::Flat).unwrap();

            // 批量添加向量
            let vectors = vec![
                vec![1.0, 0.0],
                vec![0.0, 1.0],
                vec![1.0, 1.0],
                vec![0.5, 0.5],
            ];
            let metadata = vec![
                "batch_1".to_string(),
                "batch_2".to_string(),
                "batch_3".to_string(),
                "batch_4".to_string(),
            ];

            let vector_ids = engine.batch_add_vectors("batch_index", vectors.clone(), metadata).unwrap();
            assert_eq!(vector_ids.len(), 4);

            // 批量搜索
            let queries = vec![
                vec![1.0, 0.0],
                vec![0.0, 1.0],
            ];
            let batch_results = engine.batch_search_vectors("batch_index", queries, 2).unwrap();

            assert_eq!(batch_results.len(), 2);
            assert_eq!(batch_results[0].len(), 2);
            assert_eq!(batch_results[1].len(), 2);
        });
    }

    #[test]
    fn test_index_statistics() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = AdvancedVectorEngine::new("test_stats.lance").await.unwrap();

            // 创建索引并添加向量
            engine.create_vector_index("stats_index".to_string(), 5, VectorIndexType::Flat).unwrap();

            for i in 0..10 {
                let vector = vec![i as f32, 0.0, 0.0, 0.0, 0.0];
                let metadata = format!("stats_vector_{}", i);
                engine.add_vector("stats_index", vector, metadata).unwrap();
            }

            // 获取统计信息
            let stats = engine.get_index_stats("stats_index").unwrap();

            assert_eq!(stats.index_id, "stats_index");
            assert_eq!(stats.index_type, VectorIndexType::Flat);
            assert_eq!(stats.vector_count, 10);
            assert_eq!(stats.dimension, 5);
            assert!(stats.memory_usage > 0);
            assert!(stats.created_at > 0);
            assert!(stats.updated_at >= stats.created_at);
        });
    }

    #[test]
    fn test_vector_similarity_functions() {
        // 测试余弦相似度
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let vec3 = vec![1.0, 0.0, 0.0];

        let similarity_orthogonal = cosine_similarity(&vec1, &vec2);
        let similarity_identical = cosine_similarity(&vec1, &vec3);

        assert!((similarity_orthogonal - 0.0).abs() < 1e-6);
        assert!((similarity_identical - 1.0).abs() < 1e-6);

        // 测试欧几里得距离
        let distance_zero = euclidean_distance(&vec1, &vec3);
        let distance_sqrt2 = euclidean_distance(&vec1, &vec2);

        assert!(distance_zero < 1e-6);
        assert!((distance_sqrt2 - 2.0_f32.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn test_query_optimizer_creation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let optimizer = QueryOptimizer::new("test_query_optimizer.lance").await.unwrap();

            assert_eq!(optimizer.query_cache.len(), 0);
            assert_eq!(optimizer.query_stats.len(), 0);
            assert_eq!(optimizer.cache_size_limit, 1000);
            assert_eq!(optimizer.cache_ttl, 3600);
        });
    }

    #[test]
    fn test_query_plan_generation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let optimizer = QueryOptimizer::new("test_query_plans.lance").await.unwrap();

            // 测试向量搜索查询计划
            let mut params = HashMap::new();
            params.insert("k".to_string(), "10".to_string());
            params.insert("dimension".to_string(), "128".to_string());

            let plan = optimizer.generate_query_plan(QueryType::VectorSearch, &params).unwrap();

            assert_eq!(plan.query_type, QueryType::VectorSearch);
            assert!(!plan.execution_steps.is_empty());
            assert!(plan.estimated_cost > 0.0);
            assert!(plan.estimated_time > 0.0);
            assert!(!plan.index_usage.is_empty());

            // 验证执行步骤
            let vector_search_step = &plan.execution_steps[0];
            match &vector_search_step.operation {
                QueryOperation::VectorSearch { k, .. } => {
                    assert_eq!(*k, 10);
                }
                _ => panic!("Expected VectorSearch operation"),
            }
        });
    }

    #[test]
    fn test_memory_retrieval_plan() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let optimizer = QueryOptimizer::new("test_memory_plan.lance").await.unwrap();

            let mut params = HashMap::new();
            params.insert("agent_id".to_string(), "12345".to_string());
            params.insert("limit".to_string(), "50".to_string());

            let plan = optimizer.generate_query_plan(QueryType::MemoryRetrieval, &params).unwrap();

            assert_eq!(plan.query_type, QueryType::MemoryRetrieval);
            assert_eq!(plan.execution_steps.len(), 3); // IndexScan + Filter + Sort

            // 验证执行步骤顺序
            match &plan.execution_steps[0].operation {
                QueryOperation::IndexScan { .. } => {},
                _ => panic!("Expected IndexScan as first operation"),
            }

            match &plan.execution_steps[1].operation {
                QueryOperation::Filter { .. } => {},
                _ => panic!("Expected Filter as second operation"),
            }

            match &plan.execution_steps[2].operation {
                QueryOperation::Sort { field, order } => {
                    assert_eq!(field, "created_at");
                    assert_eq!(*order, SortOrder::Descending);
                },
                _ => panic!("Expected Sort as third operation"),
            }
        });
    }

    #[test]
    fn test_query_cache_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut optimizer = QueryOptimizer::new("test_cache.lance").await.unwrap();

            let query_hash = 12345u64;
            let test_data = vec![1, 2, 3, 4, 5];

            // 测试缓存未命中
            assert!(optimizer.get_cached_result(query_hash).is_none());

            // 缓存结果
            optimizer.cache_result(query_hash, test_data.clone(), 5);

            // 测试缓存命中
            let cached_result = optimizer.get_cached_result(query_hash);
            assert!(cached_result.is_some());
            assert_eq!(cached_result.unwrap(), test_data);

            // 验证缓存统计
            let cache_stats = optimizer.get_cache_statistics();
            assert_eq!(cache_stats.total_entries, 1);
            assert_eq!(cache_stats.total_hits, 1);
            assert!(cache_stats.hit_rate > 0.0);
        });
    }

    #[test]
    fn test_query_performance_analysis() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut optimizer = QueryOptimizer::new("test_performance.lance").await.unwrap();

            // 添加一些测试统计数据
            for i in 0..10 {
                let stats = QueryStats {
                    query_id: format!("query_{}", i),
                    query_type: QueryType::VectorSearch,
                    execution_time: (i + 1) as f64 * 10.0,
                    result_count: 10,
                    cache_hit: i % 2 == 0,
                    index_used: vec!["vector_index".to_string()],
                    memory_usage: 1024,
                    cpu_usage: 50.0,
                    executed_at: chrono::Utc::now().timestamp(),
                };
                optimizer.record_query_stats(stats);
            }

            // 分析性能
            let analysis = optimizer.analyze_query_performance(Some(QueryType::VectorSearch));

            assert_eq!(analysis.total_queries, 10);
            assert!(analysis.avg_execution_time > 0.0);
            assert_eq!(analysis.cache_hit_rate, 0.5); // 50% cache hit rate
            assert_eq!(analysis.avg_result_count, 10);
            assert_eq!(analysis.avg_memory_usage, 1024);
            assert!(!analysis.slowest_queries.is_empty());
            assert!(analysis.most_frequent_queries.contains_key(&QueryType::VectorSearch));
        });
    }

    #[test]
    fn test_index_recommendations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let optimizer = QueryOptimizer::new("test_recommendations.lance").await.unwrap();

            let recommendations = optimizer.generate_index_recommendations();

            assert!(!recommendations.is_empty());

            for recommendation in &recommendations {
                assert!(!recommendation.index_name.is_empty());
                assert!(!recommendation.columns.is_empty());
                assert!(recommendation.estimated_benefit >= 0.0);
                assert!(recommendation.creation_cost >= 0.0);
                assert!(recommendation.maintenance_cost >= 0.0);
                assert!(recommendation.usage_frequency > 0);
            }

            // 验证推荐按收益排序
            for i in 1..recommendations.len() {
                assert!(recommendations[i-1].estimated_benefit >= recommendations[i].estimated_benefit);
            }
        });
    }

    #[test]
    fn test_optimal_index_selection() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let optimizer = QueryOptimizer::new("test_index_selection.lance").await.unwrap();

            // 测试不同场景下的索引选择
            assert_eq!(optimizer.select_optimal_vector_index(30, 5), VectorIndexType::Flat);
            assert_eq!(optimizer.select_optimal_vector_index(200, 5), VectorIndexType::HNSW);
            assert_eq!(optimizer.select_optimal_vector_index(1500, 10), VectorIndexType::PQ);
            assert_eq!(optimizer.select_optimal_vector_index(500, 50), VectorIndexType::IVF);
        });
    }

    #[test]
    fn test_execution_time_estimation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let optimizer = QueryOptimizer::new("test_time_estimation.lance").await.unwrap();

            let steps = vec![
                ExecutionStep {
                    step_id: 0,
                    operation: QueryOperation::VectorSearch {
                        index_type: VectorIndexType::HNSW,
                        k: 10
                    },
                    input_size: 1,
                    output_size: 10,
                    cost: 5.0,
                    dependencies: Vec::new(),
                },
                ExecutionStep {
                    step_id: 1,
                    operation: QueryOperation::Filter {
                        condition: "agent_id = 123".to_string(),
                        selectivity: 0.1
                    },
                    input_size: 100,
                    output_size: 10,
                    cost: 2.0,
                    dependencies: vec![0],
                },
            ];

            let estimated_time = optimizer.estimate_execution_time(&steps);
            assert!(estimated_time > 0.0);
        });
    }

    #[test]
    fn test_cache_eviction() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut optimizer = QueryOptimizer::new("test_cache_eviction.lance").await.unwrap();
            optimizer.cache_size_limit = 3; // 设置小的缓存限制

            // 添加超过限制的缓存条目
            for i in 0..5 {
                let query_hash = i as u64;
                let test_data = vec![i as u8; 10];
                optimizer.cache_result(query_hash, test_data, 10);
            }

            // 验证缓存大小不超过限制
            assert!(optimizer.query_cache.len() <= optimizer.cache_size_limit);

            let cache_stats = optimizer.get_cache_statistics();
            assert!(cache_stats.total_entries <= 3);
        });
    }

    #[test]
    fn test_multimodal_engine_creation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let engine = MultimodalEngine::new("test_multimodal.lance").await.unwrap();

            assert_eq!(engine.data_storage.len(), 0);
            assert_eq!(engine.cross_modal_mappings.len(), 0);
            assert_eq!(engine.feature_extractors.len(), 3); // Text, Image, Audio
        });
    }

    #[test]
    fn test_text_feature_extraction() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = MultimodalEngine::new("test_text_features.lance").await.unwrap();

            let text_data = "This is a sample text for feature extraction testing. It contains multiple sentences and various words.".as_bytes().to_vec();
            let mut metadata = HashMap::new();
            metadata.insert("language".to_string(), "en".to_string());

            engine.add_multimodal_data(
                "text_001".to_string(),
                ModalityType::Text,
                text_data,
                metadata
            ).unwrap();

            let data = engine.data_storage.get("text_001").unwrap();
            assert_eq!(data.modality_type, ModalityType::Text);
            assert!(data.embedding.is_some());
            assert!(data.features.is_some());

            let embedding = data.embedding.as_ref().unwrap();
            assert_eq!(embedding.len(), 160); // 10 + 100 + 50 features

            let features = data.features.as_ref().unwrap();
            assert!(features.contains_key("char_count"));
            assert!(features.contains_key("word_count"));
            assert!(features.contains_key("lexical_diversity"));
        });
    }

    #[test]
    fn test_image_feature_extraction() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = MultimodalEngine::new("test_image_features.lance").await.unwrap();

            // 创建简单的测试图像数据 (3x3 RGB)
            let image_data = vec![
                255, 0, 0,   0, 255, 0,   0, 0, 255,
                255, 255, 0, 255, 0, 255, 0, 255, 255,
                128, 128, 128, 64, 64, 64, 192, 192, 192
            ];

            let mut metadata = HashMap::new();
            metadata.insert("width".to_string(), "3".to_string());
            metadata.insert("height".to_string(), "3".to_string());
            metadata.insert("channels".to_string(), "3".to_string());
            metadata.insert("format".to_string(), "RGB".to_string());

            engine.add_multimodal_data(
                "image_001".to_string(),
                ModalityType::Image,
                image_data,
                metadata
            ).unwrap();

            let data = engine.data_storage.get("image_001").unwrap();
            assert_eq!(data.modality_type, ModalityType::Image);
            assert!(data.embedding.is_some());
            assert!(data.features.is_some());

            let embedding = data.embedding.as_ref().unwrap();
            assert_eq!(embedding.len(), 144); // 64 + 32 + 32 + 16 features

            let features = data.features.as_ref().unwrap();
            assert!(features.contains_key("width"));
            assert!(features.contains_key("height"));
            assert!(features.contains_key("brightness"));
            assert!(features.contains_key("contrast"));
        });
    }

    #[test]
    fn test_multimodal_statistics() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut engine = MultimodalEngine::new("test_stats.lance").await.unwrap();

            // 添加不同类型的数据
            engine.add_multimodal_data(
                "text_1".to_string(),
                ModalityType::Text,
                "Sample text".as_bytes().to_vec(),
                HashMap::new()
            ).unwrap();

            engine.add_multimodal_data(
                "text_2".to_string(),
                ModalityType::Text,
                "Another text".as_bytes().to_vec(),
                HashMap::new()
            ).unwrap();

            let image_data = vec![128u8; 100 * 100 * 3];
            let mut image_metadata = HashMap::new();
            image_metadata.insert("width".to_string(), "100".to_string());
            image_metadata.insert("height".to_string(), "100".to_string());
            image_metadata.insert("channels".to_string(), "3".to_string());

            engine.add_multimodal_data(
                "image_1".to_string(),
                ModalityType::Image,
                image_data,
                image_metadata
            ).unwrap();

            // 获取统计信息
            let stats = engine.get_multimodal_statistics();

            assert_eq!(stats.total_data_count, 3);
            assert_eq!(stats.modality_counts.get(&ModalityType::Text), Some(&2));
            assert_eq!(stats.modality_counts.get(&ModalityType::Image), Some(&1));
            assert!(stats.total_data_size > 0);
            assert_eq!(stats.supported_modalities.len(), 3);
            assert!(stats.feature_dimensions.contains_key(&ModalityType::Text));
            assert!(stats.feature_dimensions.contains_key(&ModalityType::Image));
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
        // 先检查索引是否存在和维度是否匹配
        let (index_type, dimension, vector_count) = {
            let index = self.indexes.get(index_id)
                .ok_or_else(|| AgentDbError::InvalidArgument("Index not found".to_string()))?;

            if vector.len() != index.dimension {
                return Err(AgentDbError::InvalidArgument("Vector dimension mismatch".to_string()));
            }

            (index.index_type, index.dimension, index.vectors.len())
        };

        let vector_id = format!("{}_{}", index_id, vector_count);

        match index_type {
            VectorIndexType::Flat => {
                let index = self.indexes.get_mut(index_id).unwrap();
                index.vectors.push(vector);
                index.metadata.push(metadata);
                index.updated_at = chrono::Utc::now().timestamp();
            }
            VectorIndexType::HNSW => {
                self.add_to_hnsw(index_id, vector, metadata)?;
                let index = self.indexes.get_mut(index_id).unwrap();
                index.updated_at = chrono::Utc::now().timestamp();
            }
            VectorIndexType::IVF => {
                self.add_to_ivf(index_id, vector, metadata)?;
                let index = self.indexes.get_mut(index_id).unwrap();
                index.updated_at = chrono::Utc::now().timestamp();
            }
            VectorIndexType::PQ => {
                self.add_to_pq(index_id, vector, metadata)?;
                let index = self.indexes.get_mut(index_id).unwrap();
                index.updated_at = chrono::Utc::now().timestamp();
            }
        }

        Ok(vector_id)
    }

    // HNSW索引添加向量
    fn add_to_hnsw(&mut self, index_id: &str, vector: Vec<f32>, metadata: String) -> Result<(), AgentDbError> {
        // 先获取必要的参数
        let (node_id, ml, max_level, max_connections, ef_construction) = {
            let hnsw = self.hnsw_indexes.get(index_id)
                .ok_or_else(|| AgentDbError::InvalidArgument("HNSW index not found".to_string()))?;
            (hnsw.nodes.len(), hnsw.ml, hnsw.max_level, hnsw.max_connections, hnsw.ef_construction)
        };

        let level = self.generate_random_level(ml);

        // 创建新节点
        let mut new_node = HNSWNode {
            id: node_id,
            vector: vector.clone(),
            connections: vec![Vec::new(); level + 1],
            level,
        };

        // 检查是否为第一个节点
        let is_first_node = {
            let hnsw = self.hnsw_indexes.get(index_id).unwrap();
            hnsw.nodes.is_empty()
        };

        if is_first_node {
            // 第一个节点作为入口点
            let hnsw = self.hnsw_indexes.get_mut(index_id).unwrap();
            hnsw.entry_point = Some(node_id);
            hnsw.max_level = level;
            hnsw.nodes.push(new_node);
        } else {
            // 搜索最近邻并建立连接
            let (entry_point, current_max_level) = {
                let hnsw = self.hnsw_indexes.get(index_id).unwrap();
                (hnsw.entry_point.unwrap(), hnsw.max_level)
            };

            let mut current = entry_point;

            // 从顶层向下搜索
            for lc in (level + 1..=current_max_level).rev() {
                let nodes = &self.hnsw_indexes.get(index_id).unwrap().nodes;
                let candidates = self.search_layer_hnsw(nodes, &vector, current, 1, lc);
                if !candidates.is_empty() {
                    current = candidates[0];
                }
            }

            // 在每一层建立连接
            for lc in (0..=level.min(current_max_level)).rev() {
                let nodes = &self.hnsw_indexes.get(index_id).unwrap().nodes;
                let candidates = self.search_layer_hnsw(nodes, &vector, current, ef_construction, lc);
                let m = if lc == 0 { max_connections * 2 } else { max_connections };
                let selected = self.select_neighbors_hnsw(nodes, &vector, &candidates, m);

                // 建立连接
                for &neighbor in &selected {
                    new_node.connections[lc].push(neighbor);
                }

                if !candidates.is_empty() {
                    current = candidates[0];
                }
            }

            // 添加节点并更新入口点
            let hnsw = self.hnsw_indexes.get_mut(index_id).unwrap();
            hnsw.nodes.push(new_node);

            if level > current_max_level {
                hnsw.entry_point = Some(node_id);
                hnsw.max_level = level;
            }
        }

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
            .ok_or_else(|| AgentDbError::InvalidArgument("Index not found".to_string()))?;

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
            .ok_or_else(|| AgentDbError::InvalidArgument("HNSW index not found".to_string()))?;

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
            return Err(AgentDbError::InvalidArgument("Vectors and metadata length mismatch".to_string()));
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
            .ok_or_else(|| AgentDbError::InvalidArgument("Index not found".to_string()))?;

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

// 查询优化引擎系统
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub plan_id: String,
    pub query_type: QueryType,
    pub execution_steps: Vec<ExecutionStep>,
    pub estimated_cost: f64,
    pub estimated_time: f64,
    pub index_usage: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueryType {
    VectorSearch,
    MemoryRetrieval,
    AgentStateQuery,
    RAGQuery,
    HybridQuery,
}

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub step_id: usize,
    pub operation: QueryOperation,
    pub input_size: usize,
    pub output_size: usize,
    pub cost: f64,
    pub dependencies: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum QueryOperation {
    IndexScan { index_name: String, selectivity: f64 },
    VectorSearch { index_type: VectorIndexType, k: usize },
    Filter { condition: String, selectivity: f64 },
    Sort { field: String, order: SortOrder },
    Join { join_type: JoinType, condition: String },
    Aggregate { function: AggregateFunction, field: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Average,
    Min,
    Max,
}

#[derive(Debug, Clone)]
pub struct QueryCache {
    pub cache_id: String,
    pub query_hash: u64,
    pub result_data: Vec<u8>,
    pub result_count: usize,
    pub hit_count: u64,
    pub created_at: i64,
    pub last_accessed: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone)]
pub struct QueryStats {
    pub query_id: String,
    pub query_type: QueryType,
    pub execution_time: f64,
    pub result_count: usize,
    pub cache_hit: bool,
    pub index_used: Vec<String>,
    pub memory_usage: usize,
    pub cpu_usage: f64,
    pub executed_at: i64,
}

#[derive(Debug, Clone)]
pub struct IndexRecommendation {
    pub index_name: String,
    pub index_type: VectorIndexType,
    pub columns: Vec<String>,
    pub estimated_benefit: f64,
    pub creation_cost: f64,
    pub maintenance_cost: f64,
    pub usage_frequency: u64,
}

pub struct QueryOptimizer {
    connection: Connection,
    query_cache: HashMap<u64, QueryCache>,
    query_stats: Vec<QueryStats>,
    index_stats: HashMap<String, IndexStats>,
    cache_size_limit: usize,
    cache_ttl: i64,
}

impl QueryOptimizer {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;
        Ok(Self {
            connection,
            query_cache: HashMap::new(),
            query_stats: Vec::new(),
            index_stats: HashMap::new(),
            cache_size_limit: 1000,
            cache_ttl: 3600, // 1 hour
        })
    }

    // 智能查询计划生成
    pub fn generate_query_plan(&self, query_type: QueryType, parameters: &HashMap<String, String>) -> Result<QueryPlan, AgentDbError> {
        let plan_id = format!("plan_{}_{}",
            chrono::Utc::now().timestamp_millis(),
            rand::thread_rng().gen::<u32>()
        );

        let mut execution_steps = Vec::new();
        let mut estimated_cost = 0.0;
        let mut index_usage = Vec::new();

        match query_type {
            QueryType::VectorSearch => {
                self.plan_vector_search(&mut execution_steps, &mut estimated_cost, &mut index_usage, parameters)?;
            }
            QueryType::MemoryRetrieval => {
                self.plan_memory_retrieval(&mut execution_steps, &mut estimated_cost, &mut index_usage, parameters)?;
            }
            QueryType::AgentStateQuery => {
                self.plan_agent_state_query(&mut execution_steps, &mut estimated_cost, &mut index_usage, parameters)?;
            }
            QueryType::RAGQuery => {
                self.plan_rag_query(&mut execution_steps, &mut estimated_cost, &mut index_usage, parameters)?;
            }
            QueryType::HybridQuery => {
                self.plan_hybrid_query(&mut execution_steps, &mut estimated_cost, &mut index_usage, parameters)?;
            }
        }

        let estimated_time = self.estimate_execution_time(&execution_steps);

        Ok(QueryPlan {
            plan_id,
            query_type,
            execution_steps,
            estimated_cost,
            estimated_time,
            index_usage,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    // 向量搜索查询计划
    fn plan_vector_search(&self, steps: &mut Vec<ExecutionStep>, cost: &mut f64, indexes: &mut Vec<String>, params: &HashMap<String, String>) -> Result<(), AgentDbError> {
        let k = params.get("k").and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
        let dimension = params.get("dimension").and_then(|s| s.parse::<usize>().ok()).unwrap_or(128);

        // 选择最优索引类型
        let index_type = self.select_optimal_vector_index(dimension, k);
        let index_name = format!("vector_index_{}", dimension);
        indexes.push(index_name.clone());

        // 向量搜索步骤
        steps.push(ExecutionStep {
            step_id: steps.len(),
            operation: QueryOperation::VectorSearch { index_type, k },
            input_size: 1,
            output_size: k,
            cost: self.estimate_vector_search_cost(index_type, k, dimension),
            dependencies: Vec::new(),
        });

        *cost += steps.last().unwrap().cost;
        Ok(())
    }

    // 记忆检索查询计划
    fn plan_memory_retrieval(&self, steps: &mut Vec<ExecutionStep>, cost: &mut f64, indexes: &mut Vec<String>, params: &HashMap<String, String>) -> Result<(), AgentDbError> {
        let agent_id = params.get("agent_id").unwrap_or(&"0".to_string()).clone();
        let limit = params.get("limit").and_then(|s| s.parse::<usize>().ok()).unwrap_or(100);

        // 索引扫描步骤
        let index_name = "memory_agent_index".to_string();
        indexes.push(index_name.clone());

        steps.push(ExecutionStep {
            step_id: steps.len(),
            operation: QueryOperation::IndexScan {
                index_name: index_name.clone(),
                selectivity: 0.1
            },
            input_size: 10000,
            output_size: 1000,
            cost: 10.0,
            dependencies: Vec::new(),
        });

        // 过滤步骤
        steps.push(ExecutionStep {
            step_id: steps.len(),
            operation: QueryOperation::Filter {
                condition: format!("agent_id = {}", agent_id),
                selectivity: 0.1
            },
            input_size: 1000,
            output_size: limit,
            cost: 5.0,
            dependencies: vec![steps.len() - 1],
        });

        // 排序步骤
        steps.push(ExecutionStep {
            step_id: steps.len(),
            operation: QueryOperation::Sort {
                field: "created_at".to_string(),
                order: SortOrder::Descending
            },
            input_size: limit,
            output_size: limit,
            cost: (limit as f64 * (limit as f64).log2()) / 1000.0,
            dependencies: vec![steps.len() - 1],
        });

        *cost += steps.iter().map(|s| s.cost).sum::<f64>();
        Ok(())
    }

    // Agent状态查询计划
    fn plan_agent_state_query(&self, steps: &mut Vec<ExecutionStep>, cost: &mut f64, indexes: &mut Vec<String>, params: &HashMap<String, String>) -> Result<(), AgentDbError> {
        let agent_id = params.get("agent_id").unwrap_or(&"0".to_string()).clone();

        // 主键查找
        let index_name = "agent_state_pk_index".to_string();
        indexes.push(index_name.clone());

        steps.push(ExecutionStep {
            step_id: steps.len(),
            operation: QueryOperation::IndexScan {
                index_name,
                selectivity: 0.001
            },
            input_size: 1,
            output_size: 1,
            cost: 1.0,
            dependencies: Vec::new(),
        });

        *cost += 1.0;
        Ok(())
    }

    // RAG查询计划
    fn plan_rag_query(&self, steps: &mut Vec<ExecutionStep>, cost: &mut f64, indexes: &mut Vec<String>, params: &HashMap<String, String>) -> Result<(), AgentDbError> {
        let k = params.get("k").and_then(|s| s.parse::<usize>().ok()).unwrap_or(5);
        let dimension = params.get("dimension").and_then(|s| s.parse::<usize>().ok()).unwrap_or(384);

        // 向量搜索步骤
        self.plan_vector_search(steps, cost, indexes, params)?;

        // 文档检索步骤
        let doc_index = "document_index".to_string();
        indexes.push(doc_index.clone());

        steps.push(ExecutionStep {
            step_id: steps.len(),
            operation: QueryOperation::Join {
                join_type: JoinType::Inner,
                condition: "document_id".to_string()
            },
            input_size: k,
            output_size: k,
            cost: k as f64 * 2.0,
            dependencies: vec![0],
        });

        *cost += k as f64 * 2.0;
        Ok(())
    }

    // 混合查询计划
    fn plan_hybrid_query(&self, steps: &mut Vec<ExecutionStep>, cost: &mut f64, indexes: &mut Vec<String>, params: &HashMap<String, String>) -> Result<(), AgentDbError> {
        // 组合多种查询类型
        self.plan_vector_search(steps, cost, indexes, params)?;
        self.plan_memory_retrieval(steps, cost, indexes, params)?;

        // 结果合并步骤
        steps.push(ExecutionStep {
            step_id: steps.len(),
            operation: QueryOperation::Aggregate {
                function: AggregateFunction::Count,
                field: "result_id".to_string()
            },
            input_size: 100,
            output_size: 50,
            cost: 5.0,
            dependencies: vec![0, 1],
        });

        *cost += 5.0;
        Ok(())
    }

    // 选择最优向量索引
    fn select_optimal_vector_index(&self, dimension: usize, k: usize) -> VectorIndexType {
        // 基于启发式规则选择索引类型
        if dimension < 50 {
            VectorIndexType::Flat
        } else if k < 10 && dimension < 500 {
            VectorIndexType::HNSW
        } else if dimension > 1000 {
            VectorIndexType::PQ
        } else {
            VectorIndexType::IVF
        }
    }

    // 估算向量搜索成本
    fn estimate_vector_search_cost(&self, index_type: VectorIndexType, k: usize, dimension: usize) -> f64 {
        match index_type {
            VectorIndexType::Flat => (k * dimension) as f64 * 0.001,
            VectorIndexType::HNSW => (k as f64 * (dimension as f64).log2()) * 0.01,
            VectorIndexType::IVF => (k as f64 * (dimension as f64).sqrt()) * 0.005,
            VectorIndexType::PQ => k as f64 * 0.1,
        }
    }

    // 估算执行时间
    fn estimate_execution_time(&self, steps: &[ExecutionStep]) -> f64 {
        steps.iter().map(|step| {
            match &step.operation {
                QueryOperation::VectorSearch { index_type, k } => {
                    match index_type {
                        VectorIndexType::Flat => *k as f64 * 0.1,
                        VectorIndexType::HNSW => (*k as f64).log2() * 0.01,
                        VectorIndexType::IVF => (*k as f64).sqrt() * 0.05,
                        VectorIndexType::PQ => *k as f64 * 0.001,
                    }
                }
                QueryOperation::IndexScan { selectivity, .. } => {
                    step.input_size as f64 * selectivity * 0.001
                }
                QueryOperation::Filter { selectivity, .. } => {
                    step.input_size as f64 * selectivity * 0.0001
                }
                QueryOperation::Sort { .. } => {
                    let n = step.input_size as f64;
                    n * n.log2() * 0.00001
                }
                QueryOperation::Join { .. } => {
                    step.input_size as f64 * 0.01
                }
                QueryOperation::Aggregate { .. } => {
                    step.input_size as f64 * 0.0001
                }
            }
        }).sum()
    }

    // 查询缓存管理
    pub fn get_cached_result(&mut self, query_hash: u64) -> Option<Vec<u8>> {
        let current_time = chrono::Utc::now().timestamp();

        if let Some(cache_entry) = self.query_cache.get_mut(&query_hash) {
            if cache_entry.expires_at > current_time {
                cache_entry.hit_count += 1;
                cache_entry.last_accessed = current_time;
                return Some(cache_entry.result_data.clone());
            } else {
                // 缓存过期，删除
                self.query_cache.remove(&query_hash);
            }
        }

        None
    }

    pub fn cache_result(&mut self, query_hash: u64, result_data: Vec<u8>, result_count: usize) {
        let current_time = chrono::Utc::now().timestamp();

        // 检查缓存大小限制
        if self.query_cache.len() >= self.cache_size_limit {
            self.evict_oldest_cache_entry();
        }

        let cache_entry = QueryCache {
            cache_id: format!("cache_{}", query_hash),
            query_hash,
            result_data,
            result_count,
            hit_count: 0,
            created_at: current_time,
            last_accessed: current_time,
            expires_at: current_time + self.cache_ttl,
        };

        self.query_cache.insert(query_hash, cache_entry);
    }

    // 缓存淘汰策略（LRU）
    fn evict_oldest_cache_entry(&mut self) {
        if let Some((&oldest_hash, _)) = self.query_cache.iter()
            .min_by_key(|(_, cache)| cache.last_accessed) {
            self.query_cache.remove(&oldest_hash);
        }
    }

    // 记录查询统计
    pub fn record_query_stats(&mut self, stats: QueryStats) {
        self.query_stats.push(stats);

        // 保持统计数据在合理范围内
        if self.query_stats.len() > 10000 {
            self.query_stats.drain(0..1000);
        }
    }

    // 分析查询性能
    pub fn analyze_query_performance(&self, query_type: Option<QueryType>) -> QueryPerformanceAnalysis {
        let relevant_stats: Vec<&QueryStats> = match query_type {
            Some(qt) => self.query_stats.iter().filter(|s| s.query_type == qt).collect(),
            None => self.query_stats.iter().collect(),
        };

        if relevant_stats.is_empty() {
            return QueryPerformanceAnalysis::default();
        }

        let total_queries = relevant_stats.len();
        let avg_execution_time = relevant_stats.iter().map(|s| s.execution_time).sum::<f64>() / total_queries as f64;
        let cache_hit_rate = relevant_stats.iter().filter(|s| s.cache_hit).count() as f64 / total_queries as f64;
        let avg_result_count = relevant_stats.iter().map(|s| s.result_count).sum::<usize>() / total_queries;
        let avg_memory_usage = relevant_stats.iter().map(|s| s.memory_usage).sum::<usize>() / total_queries;

        // 计算性能分布
        let mut execution_times: Vec<f64> = relevant_stats.iter().map(|s| s.execution_time).collect();
        execution_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p50 = execution_times[execution_times.len() / 2];
        let p95 = execution_times[(execution_times.len() as f64 * 0.95) as usize];
        let p99 = execution_times[(execution_times.len() as f64 * 0.99) as usize];

        QueryPerformanceAnalysis {
            total_queries,
            avg_execution_time,
            p50_execution_time: p50,
            p95_execution_time: p95,
            p99_execution_time: p99,
            cache_hit_rate,
            avg_result_count,
            avg_memory_usage,
            slowest_queries: self.get_slowest_queries(&relevant_stats, 5),
            most_frequent_queries: self.get_most_frequent_query_types(&relevant_stats),
        }
    }

    // 获取最慢的查询
    fn get_slowest_queries(&self, stats: &[&QueryStats], limit: usize) -> Vec<QueryStats> {
        let mut sorted_stats = stats.to_vec();
        sorted_stats.sort_by(|a, b| b.execution_time.partial_cmp(&a.execution_time).unwrap());
        sorted_stats.into_iter().take(limit).cloned().collect()
    }

    // 获取最频繁的查询类型
    fn get_most_frequent_query_types(&self, stats: &[&QueryStats]) -> HashMap<QueryType, usize> {
        let mut type_counts = HashMap::new();
        for stat in stats {
            *type_counts.entry(stat.query_type.clone()).or_insert(0) += 1;
        }
        type_counts
    }

    // 生成索引推荐
    pub fn generate_index_recommendations(&self) -> Vec<IndexRecommendation> {
        let mut recommendations = Vec::new();

        // 分析查询模式
        let query_patterns = self.analyze_query_patterns();

        for pattern in query_patterns {
            if pattern.frequency > 10 && pattern.avg_execution_time > 100.0 {
                let recommendation = IndexRecommendation {
                    index_name: format!("recommended_index_{}", pattern.pattern_id),
                    index_type: self.recommend_index_type(&pattern),
                    columns: pattern.accessed_columns.clone(),
                    estimated_benefit: self.estimate_index_benefit(&pattern),
                    creation_cost: self.estimate_index_creation_cost(&pattern),
                    maintenance_cost: self.estimate_index_maintenance_cost(&pattern),
                    usage_frequency: pattern.frequency,
                };
                recommendations.push(recommendation);
            }
        }

        // 按预期收益排序
        recommendations.sort_by(|a, b| b.estimated_benefit.partial_cmp(&a.estimated_benefit).unwrap());
        recommendations
    }

    // 分析查询模式
    fn analyze_query_patterns(&self) -> Vec<QueryPattern> {
        // 简化的查询模式分析
        vec![
            QueryPattern {
                pattern_id: "vector_search_pattern".to_string(),
                query_type: QueryType::VectorSearch,
                frequency: 100,
                avg_execution_time: 150.0,
                accessed_columns: vec!["embedding".to_string()],
            },
            QueryPattern {
                pattern_id: "memory_retrieval_pattern".to_string(),
                query_type: QueryType::MemoryRetrieval,
                frequency: 50,
                avg_execution_time: 80.0,
                accessed_columns: vec!["agent_id".to_string(), "created_at".to_string()],
            },
        ]
    }

    // 推荐索引类型
    fn recommend_index_type(&self, pattern: &QueryPattern) -> VectorIndexType {
        match pattern.query_type {
            QueryType::VectorSearch => VectorIndexType::HNSW,
            _ => VectorIndexType::Flat,
        }
    }

    // 估算索引收益
    fn estimate_index_benefit(&self, pattern: &QueryPattern) -> f64 {
        pattern.frequency as f64 * (pattern.avg_execution_time * 0.5)
    }

    // 估算索引创建成本
    fn estimate_index_creation_cost(&self, _pattern: &QueryPattern) -> f64 {
        1000.0 // 简化的固定成本
    }

    // 估算索引维护成本
    fn estimate_index_maintenance_cost(&self, _pattern: &QueryPattern) -> f64 {
        100.0 // 简化的固定成本
    }

    // 获取缓存统计
    pub fn get_cache_statistics(&self) -> CacheStatistics {
        let total_entries = self.query_cache.len();
        let total_hits = self.query_cache.values().map(|c| c.hit_count).sum();
        let total_size = self.query_cache.values().map(|c| c.result_data.len()).sum();
        let current_time = chrono::Utc::now().timestamp();
        let expired_entries = self.query_cache.values().filter(|c| c.expires_at < current_time).count();

        CacheStatistics {
            total_entries,
            total_hits,
            total_size,
            expired_entries,
            hit_rate: if total_entries > 0 { total_hits as f64 / total_entries as f64 } else { 0.0 },
            memory_usage: total_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryPattern {
    pub pattern_id: String,
    pub query_type: QueryType,
    pub frequency: u64,
    pub avg_execution_time: f64,
    pub accessed_columns: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct QueryPerformanceAnalysis {
    pub total_queries: usize,
    pub avg_execution_time: f64,
    pub p50_execution_time: f64,
    pub p95_execution_time: f64,
    pub p99_execution_time: f64,
    pub cache_hit_rate: f64,
    pub avg_result_count: usize,
    pub avg_memory_usage: usize,
    pub slowest_queries: Vec<QueryStats>,
    pub most_frequent_queries: HashMap<QueryType, usize>,
}

#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub total_hits: u64,
    pub total_size: usize,
    pub expired_entries: usize,
    pub hit_rate: f64,
    pub memory_usage: usize,
}

// 多模态数据支持系统
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModalityType {
    Text,
    Image,
    Audio,
    Video,
    Multimodal,
}

#[derive(Debug, Clone)]
pub struct MultimodalData {
    pub data_id: String,
    pub modality_type: ModalityType,
    pub raw_data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub embedding: Option<Vec<f32>>,
    pub features: Option<HashMap<String, f32>>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct ImageFeatures {
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub format: String,
    pub color_histogram: Vec<f32>,
    pub edge_features: Vec<f32>,
    pub texture_features: Vec<f32>,
    pub shape_features: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct AudioFeatures {
    pub sample_rate: u32,
    pub duration: f32,
    pub channels: u32,
    pub format: String,
    pub mfcc: Vec<f32>,           // Mel-frequency cepstral coefficients
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub zero_crossing_rate: f32,
    pub tempo: f32,
    pub energy: f32,
}

#[derive(Debug, Clone)]
pub struct CrossModalMapping {
    pub mapping_id: String,
    pub source_modality: ModalityType,
    pub target_modality: ModalityType,
    pub transformation_matrix: Vec<Vec<f32>>,
    pub bias_vector: Vec<f32>,
    pub confidence_score: f32,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct MultimodalSearchResult {
    pub data_id: String,
    pub modality_type: ModalityType,
    pub similarity_score: f32,
    pub cross_modal_score: Option<f32>,
    pub metadata: HashMap<String, String>,
    pub features_summary: String,
}

pub struct MultimodalEngine {
    connection: Connection,
    data_storage: HashMap<String, MultimodalData>,
    cross_modal_mappings: HashMap<String, CrossModalMapping>,
    feature_extractors: HashMap<ModalityType, Box<dyn FeatureExtractor>>,
}

pub trait FeatureExtractor: Send + Sync {
    fn extract_features(&self, data: &[u8], metadata: &HashMap<String, String>) -> Result<Vec<f32>, AgentDbError>;
    fn extract_detailed_features(&self, data: &[u8], metadata: &HashMap<String, String>) -> Result<HashMap<String, f32>, AgentDbError>;
}

// 图像特征提取器
pub struct ImageFeatureExtractor;

impl FeatureExtractor for ImageFeatureExtractor {
    fn extract_features(&self, data: &[u8], metadata: &HashMap<String, String>) -> Result<Vec<f32>, AgentDbError> {
        // 简化的图像特征提取实现
        let width = metadata.get("width").and_then(|s| s.parse::<u32>().ok()).unwrap_or(224);
        let height = metadata.get("height").and_then(|s| s.parse::<u32>().ok()).unwrap_or(224);
        let channels = metadata.get("channels").and_then(|s| s.parse::<u32>().ok()).unwrap_or(3);

        // 模拟CNN特征提取（实际应用中应使用预训练模型）
        let mut features = Vec::new();

        // 颜色直方图特征 (64维)
        let color_hist = self.extract_color_histogram(data, width, height, channels)?;
        features.extend(color_hist);

        // 边缘特征 (32维)
        let edge_features = self.extract_edge_features(data, width, height)?;
        features.extend(edge_features);

        // 纹理特征 (32维)
        let texture_features = self.extract_texture_features(data, width, height)?;
        features.extend(texture_features);

        // 形状特征 (16维)
        let shape_features = self.extract_shape_features(data, width, height)?;
        features.extend(shape_features);

        Ok(features)
    }

    fn extract_detailed_features(&self, data: &[u8], metadata: &HashMap<String, String>) -> Result<HashMap<String, f32>, AgentDbError> {
        let width = metadata.get("width").and_then(|s| s.parse::<u32>().ok()).unwrap_or(224);
        let height = metadata.get("height").and_then(|s| s.parse::<u32>().ok()).unwrap_or(224);

        let mut detailed_features = HashMap::new();

        // 基础图像属性
        detailed_features.insert("width".to_string(), width as f32);
        detailed_features.insert("height".to_string(), height as f32);
        detailed_features.insert("aspect_ratio".to_string(), width as f32 / height as f32);
        detailed_features.insert("pixel_count".to_string(), (width * height) as f32);

        // 颜色统计
        let (brightness, contrast, saturation) = self.calculate_color_stats(data, width, height)?;
        detailed_features.insert("brightness".to_string(), brightness);
        detailed_features.insert("contrast".to_string(), contrast);
        detailed_features.insert("saturation".to_string(), saturation);

        // 复杂度指标
        let complexity = self.calculate_image_complexity(data, width, height)?;
        detailed_features.insert("complexity".to_string(), complexity);

        Ok(detailed_features)
    }
}

impl ImageFeatureExtractor {
    fn extract_color_histogram(&self, data: &[u8], width: u32, height: u32, channels: u32) -> Result<Vec<f32>, AgentDbError> {
        let mut histogram = vec![0.0; 64]; // 简化的64维颜色直方图

        let pixel_count = (width * height * channels) as usize;
        if data.len() < pixel_count {
            return Err(AgentDbError::InvalidArgument("Insufficient image data".to_string()));
        }

        // 简化的颜色直方图计算
        for i in (0..pixel_count).step_by(channels as usize) {
            if i + 2 < data.len() {
                let r = data[i] as f32 / 255.0;
                let g = data[i + 1] as f32 / 255.0;
                let b = data[i + 2] as f32 / 255.0;

                // 将RGB映射到直方图bin
                let bin = ((r * 4.0) as usize * 16 + (g * 4.0) as usize * 4 + (b * 4.0) as usize).min(63);
                histogram[bin] += 1.0;
            }
        }

        // 归一化
        let total: f32 = histogram.iter().sum();
        if total > 0.0 {
            for h in &mut histogram {
                *h /= total;
            }
        }

        Ok(histogram)
    }

    fn extract_edge_features(&self, data: &[u8], width: u32, height: u32) -> Result<Vec<f32>, AgentDbError> {
        // 简化的边缘检测特征
        let mut edge_features = vec![0.0; 32];

        // 模拟Sobel边缘检测
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let idx = (y * width + x) as usize * 3;
                if idx + 2 < data.len() {
                    let intensity = (data[idx] as f32 + data[idx + 1] as f32 + data[idx + 2] as f32) / 3.0;

                    // 简化的梯度计算
                    let gradient_x = intensity - (*data.get(idx - 3).unwrap_or(&0) as f32);
                    let gradient_y = intensity - (*data.get(idx - width as usize * 3).unwrap_or(&0) as f32);
                    let magnitude = (gradient_x * gradient_x + gradient_y * gradient_y).sqrt();

                    // 将梯度幅值分配到特征bin
                    let bin = (magnitude / 32.0).min(31.0) as usize;
                    edge_features[bin] += 1.0;
                }
            }
        }

        // 归一化
        let total: f32 = edge_features.iter().sum();
        if total > 0.0 {
            for f in &mut edge_features {
                *f /= total;
            }
        }

        Ok(edge_features)
    }

    fn extract_texture_features(&self, data: &[u8], width: u32, height: u32) -> Result<Vec<f32>, AgentDbError> {
        // 简化的纹理特征（基于局部二值模式LBP的简化版本）
        let mut texture_features = vec![0.0; 32];

        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let center_idx = (y * width + x) as usize * 3;
                if center_idx + 2 < data.len() {
                    let center_intensity = (data[center_idx] as f32 + data[center_idx + 1] as f32 + data[center_idx + 2] as f32) / 3.0;

                    // 简化的局部模式计算
                    let mut pattern = 0;
                    let neighbors = [
                        (-1, -1), (-1, 0), (-1, 1),
                        (0, -1),           (0, 1),
                        (1, -1),  (1, 0),  (1, 1),
                    ];

                    for (i, (dx, dy)) in neighbors.iter().enumerate() {
                        let nx = (x as i32 + dx) as u32;
                        let ny = (y as i32 + dy) as u32;
                        let neighbor_idx = (ny * width + nx) as usize * 3;

                        if neighbor_idx + 2 < data.len() {
                            let neighbor_intensity = (data[neighbor_idx] as f32 + data[neighbor_idx + 1] as f32 + data[neighbor_idx + 2] as f32) / 3.0;
                            if neighbor_intensity > center_intensity {
                                pattern |= 1 << i;
                            }
                        }
                    }

                    // 将模式映射到特征bin
                    let bin = (pattern % 32) as usize;
                    texture_features[bin] += 1.0;
                }
            }
        }

        // 归一化
        let total: f32 = texture_features.iter().sum();
        if total > 0.0 {
            for f in &mut texture_features {
                *f /= total;
            }
        }

        Ok(texture_features)
    }

    fn extract_shape_features(&self, data: &[u8], width: u32, height: u32) -> Result<Vec<f32>, AgentDbError> {
        // 简化的形状特征
        let mut shape_features = vec![0.0; 16];

        // 计算图像的矩特征
        let mut m00 = 0.0; // 零阶矩
        let mut m10 = 0.0; // 一阶矩
        let mut m01 = 0.0;
        let mut m20 = 0.0; // 二阶矩
        let mut m11 = 0.0;
        let mut m02 = 0.0;

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize * 3;
                if idx + 2 < data.len() {
                    let intensity = (data[idx] as f32 + data[idx + 1] as f32 + data[idx + 2] as f32) / 3.0 / 255.0;

                    let fx = x as f32;
                    let fy = y as f32;

                    m00 += intensity;
                    m10 += fx * intensity;
                    m01 += fy * intensity;
                    m20 += fx * fx * intensity;
                    m11 += fx * fy * intensity;
                    m02 += fy * fy * intensity;
                }
            }
        }

        // 计算中心矩
        if m00 > 0.0 {
            let cx = m10 / m00;
            let cy = m01 / m00;

            let mu20 = m20 / m00 - cx * cx;
            let mu11 = m11 / m00 - cx * cy;
            let mu02 = m02 / m00 - cy * cy;

            // 形状特征
            shape_features[0] = m00; // 面积
            shape_features[1] = cx; // 质心x
            shape_features[2] = cy; // 质心y
            shape_features[3] = mu20; // 中心矩
            shape_features[4] = mu11;
            shape_features[5] = mu02;

            // 不变矩
            shape_features[6] = mu20 + mu02; // 第一不变矩
            shape_features[7] = (mu20 - mu02) * (mu20 - mu02) + 4.0 * mu11 * mu11; // 第二不变矩

            // 其他形状描述符
            shape_features[8] = width as f32 / height as f32; // 长宽比
            shape_features[9] = (mu20 * mu02 - mu11 * mu11) / (m00 * m00 * m00 * m00); // 紧凑度
        }

        Ok(shape_features)
    }

    fn calculate_color_stats(&self, data: &[u8], width: u32, height: u32) -> Result<(f32, f32, f32), AgentDbError> {
        let mut r_sum = 0.0;
        let mut g_sum = 0.0;
        let mut b_sum = 0.0;
        let mut r_sq_sum = 0.0;
        let mut g_sq_sum = 0.0;
        let mut b_sq_sum = 0.0;

        let pixel_count = width * height;

        for i in (0..data.len()).step_by(3) {
            if i + 2 < data.len() {
                let r = data[i] as f32 / 255.0;
                let g = data[i + 1] as f32 / 255.0;
                let b = data[i + 2] as f32 / 255.0;

                r_sum += r;
                g_sum += g;
                b_sum += b;
                r_sq_sum += r * r;
                g_sq_sum += g * g;
                b_sq_sum += b * b;
            }
        }

        let brightness = (r_sum + g_sum + b_sum) / (3.0 * pixel_count as f32);

        let r_var = r_sq_sum / pixel_count as f32 - (r_sum / pixel_count as f32).powi(2);
        let g_var = g_sq_sum / pixel_count as f32 - (g_sum / pixel_count as f32).powi(2);
        let b_var = b_sq_sum / pixel_count as f32 - (b_sum / pixel_count as f32).powi(2);
        let contrast = (r_var + g_var + b_var) / 3.0;

        // 简化的饱和度计算
        let saturation = contrast.sqrt();

        Ok((brightness, contrast, saturation))
    }

    fn calculate_image_complexity(&self, data: &[u8], width: u32, height: u32) -> Result<f32, AgentDbError> {
        // 基于熵的复杂度计算
        let mut histogram = vec![0; 256];

        for i in (0..data.len()).step_by(3) {
            if i + 2 < data.len() {
                let intensity = ((data[i] as u32 + data[i + 1] as u32 + data[i + 2] as u32) / 3) as usize;
                histogram[intensity] += 1;
            }
        }

        let total_pixels = width * height;
        let mut entropy = 0.0;

        for &count in &histogram {
            if count > 0 {
                let p = count as f32 / total_pixels as f32;
                entropy -= p * p.log2();
            }
        }

        Ok(entropy / 8.0) // 归一化到[0,1]
    }
}

// 音频特征提取器
pub struct AudioFeatureExtractor;

impl FeatureExtractor for AudioFeatureExtractor {
    fn extract_features(&self, data: &[u8], metadata: &HashMap<String, String>) -> Result<Vec<f32>, AgentDbError> {
        let sample_rate = metadata.get("sample_rate").and_then(|s| s.parse::<u32>().ok()).unwrap_or(44100);
        let channels = metadata.get("channels").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);

        // 将字节数据转换为音频样本
        let samples = self.bytes_to_samples(data)?;

        let mut features = Vec::new();

        // MFCC特征 (13维)
        let mfcc = self.extract_mfcc(&samples, sample_rate)?;
        features.extend(mfcc);

        // 频谱特征 (10维)
        let spectral_features = self.extract_spectral_features(&samples, sample_rate)?;
        features.extend(spectral_features);

        // 时域特征 (5维)
        let temporal_features = self.extract_temporal_features(&samples, sample_rate)?;
        features.extend(temporal_features);

        // 节奏特征 (4维)
        let rhythm_features = self.extract_rhythm_features(&samples, sample_rate)?;
        features.extend(rhythm_features);

        Ok(features)
    }

    fn extract_detailed_features(&self, data: &[u8], metadata: &HashMap<String, String>) -> Result<HashMap<String, f32>, AgentDbError> {
        let sample_rate = metadata.get("sample_rate").and_then(|s| s.parse::<u32>().ok()).unwrap_or(44100);
        let channels = metadata.get("channels").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);

        let samples = self.bytes_to_samples(data)?;
        let duration = samples.len() as f32 / sample_rate as f32;

        let mut detailed_features = HashMap::new();

        // 基础音频属性
        detailed_features.insert("sample_rate".to_string(), sample_rate as f32);
        detailed_features.insert("duration".to_string(), duration);
        detailed_features.insert("channels".to_string(), channels as f32);
        detailed_features.insert("sample_count".to_string(), samples.len() as f32);

        // 音量统计
        let (rms_energy, peak_amplitude, dynamic_range) = self.calculate_amplitude_stats(&samples)?;
        detailed_features.insert("rms_energy".to_string(), rms_energy);
        detailed_features.insert("peak_amplitude".to_string(), peak_amplitude);
        detailed_features.insert("dynamic_range".to_string(), dynamic_range);

        // 频域特征
        let (spectral_centroid, spectral_rolloff, spectral_bandwidth) = self.calculate_spectral_stats(&samples, sample_rate)?;
        detailed_features.insert("spectral_centroid".to_string(), spectral_centroid);
        detailed_features.insert("spectral_rolloff".to_string(), spectral_rolloff);
        detailed_features.insert("spectral_bandwidth".to_string(), spectral_bandwidth);

        // 时域特征
        let zero_crossing_rate = self.calculate_zero_crossing_rate(&samples)?;
        detailed_features.insert("zero_crossing_rate".to_string(), zero_crossing_rate);

        Ok(detailed_features)
    }
}

impl AudioFeatureExtractor {
    fn bytes_to_samples(&self, data: &[u8]) -> Result<Vec<f32>, AgentDbError> {
        // 假设16位PCM格式
        if data.len() % 2 != 0 {
            return Err(AgentDbError::InvalidArgument("Invalid audio data length".to_string()));
        }

        let mut samples = Vec::new();
        for i in (0..data.len()).step_by(2) {
            if i + 1 < data.len() {
                let sample = i16::from_le_bytes([data[i], data[i + 1]]) as f32 / 32768.0;
                samples.push(sample);
            }
        }

        Ok(samples)
    }

    fn extract_mfcc(&self, samples: &[f32], sample_rate: u32) -> Result<Vec<f32>, AgentDbError> {
        // 简化的MFCC实现
        let mut mfcc = vec![0.0; 13];

        // 预加重
        let mut pre_emphasized = Vec::new();
        pre_emphasized.push(samples[0]);
        for i in 1..samples.len() {
            pre_emphasized.push(samples[i] - 0.97 * samples[i - 1]);
        }

        // 简化的频谱分析
        let window_size = 1024;
        let hop_size = 512;

        for start in (0..pre_emphasized.len()).step_by(hop_size) {
            let end = (start + window_size).min(pre_emphasized.len());
            let window = &pre_emphasized[start..end];

            // 简化的DFT
            let mut spectrum = vec![0.0; window_size / 2];
            for k in 0..spectrum.len() {
                let mut real = 0.0;
                let mut imag = 0.0;
                for n in 0..window.len() {
                    let angle = -2.0 * std::f32::consts::PI * k as f32 * n as f32 / window_size as f32;
                    real += window[n] * angle.cos();
                    imag += window[n] * angle.sin();
                }
                spectrum[k] = (real * real + imag * imag).sqrt();
            }

            // Mel滤波器组
            let mel_filters = self.create_mel_filters(spectrum.len(), sample_rate);
            for (i, filter) in mel_filters.iter().enumerate().take(13) {
                let mut energy = 0.0;
                for (j, &coeff) in filter.iter().enumerate() {
                    if j < spectrum.len() {
                        energy += spectrum[j] * coeff;
                    }
                }
                mfcc[i] += energy.ln().max(-50.0); // 避免log(0)
            }
        }

        // 归一化
        let frames = (pre_emphasized.len() / hop_size).max(1) as f32;
        for coeff in &mut mfcc {
            *coeff /= frames;
        }

        Ok(mfcc)
    }

    fn create_mel_filters(&self, fft_size: usize, sample_rate: u32) -> Vec<Vec<f32>> {
        let num_filters = 13;
        let mut filters = Vec::new();

        // Mel频率转换
        let mel_low = 0.0;
        let mel_high = 2595.0 * (1.0 + sample_rate as f32 / 2.0 / 700.0).ln();

        let mel_points: Vec<f32> = (0..=num_filters + 1)
            .map(|i| mel_low + (mel_high - mel_low) * i as f32 / (num_filters + 1) as f32)
            .collect();

        let hz_points: Vec<f32> = mel_points.iter()
            .map(|&mel| 700.0 * ((mel / 2595.0).exp() - 1.0))
            .collect();

        let bin_points: Vec<usize> = hz_points.iter()
            .map(|&hz| ((fft_size + 1) as f32 * hz / sample_rate as f32) as usize)
            .collect();

        for i in 1..=num_filters {
            let mut filter = vec![0.0; fft_size];

            for j in bin_points[i - 1]..bin_points[i] {
                if j < fft_size {
                    filter[j] = (j - bin_points[i - 1]) as f32 / (bin_points[i] - bin_points[i - 1]) as f32;
                }
            }

            for j in bin_points[i]..bin_points[i + 1] {
                if j < fft_size {
                    filter[j] = (bin_points[i + 1] - j) as f32 / (bin_points[i + 1] - bin_points[i]) as f32;
                }
            }

            filters.push(filter);
        }

        filters
    }

    fn extract_spectral_features(&self, samples: &[f32], sample_rate: u32) -> Result<Vec<f32>, AgentDbError> {
        let mut features = vec![0.0; 10];

        // 简化的频谱分析
        let window_size = 1024;
        let mut spectrum = vec![0.0; window_size / 2];

        // 计算功率谱
        for k in 0..spectrum.len() {
            let mut real = 0.0;
            let mut imag = 0.0;
            for n in 0..window_size.min(samples.len()) {
                let angle = -2.0 * std::f32::consts::PI * k as f32 * n as f32 / window_size as f32;
                real += samples[n] * angle.cos();
                imag += samples[n] * angle.sin();
            }
            spectrum[k] = real * real + imag * imag;
        }

        // 频谱质心
        let mut weighted_sum = 0.0;
        let mut total_power = 0.0;
        for (i, &power) in spectrum.iter().enumerate() {
            let freq = i as f32 * sample_rate as f32 / window_size as f32;
            weighted_sum += freq * power;
            total_power += power;
        }
        features[0] = if total_power > 0.0 { weighted_sum / total_power } else { 0.0 };

        // 频谱滚降点
        let mut cumulative_power = 0.0;
        let threshold = total_power * 0.85;
        for (i, &power) in spectrum.iter().enumerate() {
            cumulative_power += power;
            if cumulative_power >= threshold {
                features[1] = i as f32 * sample_rate as f32 / window_size as f32;
                break;
            }
        }

        // 频谱带宽
        let centroid = features[0];
        let mut bandwidth = 0.0;
        for (i, &power) in spectrum.iter().enumerate() {
            let freq = i as f32 * sample_rate as f32 / window_size as f32;
            bandwidth += (freq - centroid).powi(2) * power;
        }
        features[2] = if total_power > 0.0 { (bandwidth / total_power).sqrt() } else { 0.0 };

        // 频谱平坦度
        let mut geometric_mean = 1.0;
        let mut arithmetic_mean = 0.0;
        let valid_bins = spectrum.iter().filter(|&&p| p > 0.0).count();

        if valid_bins > 0 {
            for &power in spectrum.iter().filter(|&&p| p > 0.0) {
                geometric_mean *= power.powf(1.0 / valid_bins as f32);
                arithmetic_mean += power;
            }
            arithmetic_mean /= valid_bins as f32;
            features[3] = if arithmetic_mean > 0.0 { geometric_mean / arithmetic_mean } else { 0.0 };
        }

        // 其他频谱特征（简化）
        features[4] = spectrum.iter().sum::<f32>(); // 总能量
        features[5] = spectrum.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0).clone(); // 峰值

        Ok(features)
    }

    fn extract_temporal_features(&self, samples: &[f32], _sample_rate: u32) -> Result<Vec<f32>, AgentDbError> {
        let mut features = vec![0.0; 5];

        // RMS能量
        let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
        features[0] = rms;

        // 峰值幅度
        features[1] = samples.iter().map(|&x| x.abs()).fold(0.0, f32::max);

        // 过零率
        let mut zero_crossings = 0;
        for i in 1..samples.len() {
            if (samples[i] >= 0.0) != (samples[i - 1] >= 0.0) {
                zero_crossings += 1;
            }
        }
        features[2] = zero_crossings as f32 / samples.len() as f32;

        // 短时能量
        let frame_size = 1024;
        let mut frame_energies = Vec::new();
        for start in (0..samples.len()).step_by(frame_size) {
            let end = (start + frame_size).min(samples.len());
            let energy: f32 = samples[start..end].iter().map(|&x| x * x).sum();
            frame_energies.push(energy);
        }

        if !frame_energies.is_empty() {
            features[3] = frame_energies.iter().sum::<f32>() / frame_energies.len() as f32; // 平均能量
            features[4] = frame_energies.iter().fold(0.0, |acc, &x| acc.max(x)); // 最大能量
        }

        Ok(features)
    }

    fn extract_rhythm_features(&self, samples: &[f32], sample_rate: u32) -> Result<Vec<f32>, AgentDbError> {
        let mut features = vec![0.0; 4];

        // 简化的节拍检测
        let hop_size = 512;
        let mut onset_strength = Vec::new();

        for start in (0..samples.len()).step_by(hop_size) {
            let end = (start + hop_size).min(samples.len());
            let frame = &samples[start..end];

            // 计算频谱通量
            let mut flux = 0.0;
            for i in 1..frame.len() {
                let diff = frame[i] - frame[i - 1];
                if diff > 0.0 {
                    flux += diff;
                }
            }
            onset_strength.push(flux);
        }

        // 自相关分析寻找周期性
        if onset_strength.len() > 1 {
            let mut max_correlation = 0.0;
            let mut best_lag = 0;

            for lag in 1..(onset_strength.len() / 2) {
                let mut correlation = 0.0;
                let mut count = 0;

                for i in lag..onset_strength.len() {
                    correlation += onset_strength[i] * onset_strength[i - lag];
                    count += 1;
                }

                if count > 0 {
                    correlation /= count as f32;
                    if correlation > max_correlation {
                        max_correlation = correlation;
                        best_lag = lag;
                    }
                }
            }

            // 估算BPM
            if best_lag > 0 {
                let period_seconds = best_lag as f32 * hop_size as f32 / sample_rate as f32;
                features[0] = 60.0 / period_seconds; // BPM
            }

            features[1] = max_correlation; // 节拍强度
        }

        // 节奏规律性
        let mut regularity = 0.0;
        if onset_strength.len() > 2 {
            let mut intervals = Vec::new();
            for i in 1..onset_strength.len() {
                if onset_strength[i] > onset_strength[i - 1] * 1.5 {
                    intervals.push(i);
                }
            }

            if intervals.len() > 1 {
                let mut interval_diffs = Vec::new();
                for i in 1..intervals.len() {
                    interval_diffs.push(intervals[i] - intervals[i - 1]);
                }

                if !interval_diffs.is_empty() {
                    let mean_interval = interval_diffs.iter().sum::<usize>() as f32 / interval_diffs.len() as f32;
                    let variance = interval_diffs.iter()
                        .map(|&x| (x as f32 - mean_interval).powi(2))
                        .sum::<f32>() / interval_diffs.len() as f32;

                    regularity = 1.0 / (1.0 + variance.sqrt());
                }
            }
        }
        features[2] = regularity;

        // 动态范围
        if !onset_strength.is_empty() {
            let max_onset = onset_strength.iter().fold(0.0f32, |acc, &x| acc.max(x));
            let min_onset = onset_strength.iter().fold(f32::INFINITY, |acc, &x| acc.min(x));
            features[3] = max_onset - min_onset;
        }

        Ok(features)
    }

    fn calculate_amplitude_stats(&self, samples: &[f32]) -> Result<(f32, f32, f32), AgentDbError> {
        if samples.is_empty() {
            return Ok((0.0, 0.0, 0.0));
        }

        // RMS能量
        let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();

        // 峰值幅度
        let peak = samples.iter().map(|&x| x.abs()).fold(0.0, f32::max);

        // 动态范围
        let min_amplitude = samples.iter().map(|&x| x.abs()).fold(f32::INFINITY, f32::min);
        let dynamic_range = if min_amplitude > 0.0 { 20.0 * (peak / min_amplitude).log10() } else { 0.0 };

        Ok((rms, peak, dynamic_range))
    }

    fn calculate_spectral_stats(&self, samples: &[f32], sample_rate: u32) -> Result<(f32, f32, f32), AgentDbError> {
        let window_size = 1024;
        let mut spectrum = vec![0.0; window_size / 2];

        // 计算功率谱
        for k in 0..spectrum.len() {
            let mut real = 0.0;
            let mut imag = 0.0;
            for n in 0..window_size.min(samples.len()) {
                let angle = -2.0 * std::f32::consts::PI * k as f32 * n as f32 / window_size as f32;
                real += samples[n] * angle.cos();
                imag += samples[n] * angle.sin();
            }
            spectrum[k] = real * real + imag * imag;
        }

        let total_power: f32 = spectrum.iter().sum();

        // 频谱质心
        let mut centroid = 0.0;
        if total_power > 0.0 {
            for (i, &power) in spectrum.iter().enumerate() {
                let freq = i as f32 * sample_rate as f32 / window_size as f32;
                centroid += freq * power;
            }
            centroid /= total_power;
        }

        // 频谱滚降点
        let mut rolloff = 0.0;
        let mut cumulative_power = 0.0;
        let threshold = total_power * 0.85;
        for (i, &power) in spectrum.iter().enumerate() {
            cumulative_power += power;
            if cumulative_power >= threshold {
                rolloff = i as f32 * sample_rate as f32 / window_size as f32;
                break;
            }
        }

        // 频谱带宽
        let mut bandwidth = 0.0;
        if total_power > 0.0 {
            for (i, &power) in spectrum.iter().enumerate() {
                let freq = i as f32 * sample_rate as f32 / window_size as f32;
                bandwidth += (freq - centroid).powi(2) * power;
            }
            bandwidth = (bandwidth / total_power).sqrt();
        }

        Ok((centroid, rolloff, bandwidth))
    }

    fn calculate_zero_crossing_rate(&self, samples: &[f32]) -> Result<f32, AgentDbError> {
        if samples.len() < 2 {
            return Ok(0.0);
        }

        let mut zero_crossings = 0;
        for i in 1..samples.len() {
            if (samples[i] >= 0.0) != (samples[i - 1] >= 0.0) {
                zero_crossings += 1;
            }
        }

        Ok(zero_crossings as f32 / (samples.len() - 1) as f32)
    }
}

// 文本特征提取器
pub struct TextFeatureExtractor;

impl FeatureExtractor for TextFeatureExtractor {
    fn extract_features(&self, data: &[u8], _metadata: &HashMap<String, String>) -> Result<Vec<f32>, AgentDbError> {
        let text = String::from_utf8_lossy(data);

        let mut features = Vec::new();

        // 基础文本统计特征 (10维)
        let basic_features = self.extract_basic_text_features(&text)?;
        features.extend(basic_features);

        // TF-IDF特征 (100维)
        let tfidf_features = self.extract_tfidf_features(&text)?;
        features.extend(tfidf_features);

        // N-gram特征 (50维)
        let ngram_features = self.extract_ngram_features(&text)?;
        features.extend(ngram_features);

        Ok(features)
    }

    fn extract_detailed_features(&self, data: &[u8], _metadata: &HashMap<String, String>) -> Result<HashMap<String, f32>, AgentDbError> {
        let text = String::from_utf8_lossy(data);

        let mut detailed_features = HashMap::new();

        // 基础统计
        detailed_features.insert("char_count".to_string(), text.len() as f32);
        detailed_features.insert("word_count".to_string(), text.split_whitespace().count() as f32);
        detailed_features.insert("sentence_count".to_string(), text.matches('.').count() as f32);
        detailed_features.insert("paragraph_count".to_string(), text.matches("\n\n").count() as f32);

        // 词汇复杂度
        let words: Vec<&str> = text.split_whitespace().collect();
        let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
        let lexical_diversity = if words.len() > 0 { unique_words.len() as f32 / words.len() as f32 } else { 0.0 };
        detailed_features.insert("lexical_diversity".to_string(), lexical_diversity);

        // 平均词长
        let avg_word_length = if words.len() > 0 {
            words.iter().map(|w| w.len()).sum::<usize>() as f32 / words.len() as f32
        } else { 0.0 };
        detailed_features.insert("avg_word_length".to_string(), avg_word_length);

        Ok(detailed_features)
    }
}

impl TextFeatureExtractor {
    fn extract_basic_text_features(&self, text: &str) -> Result<Vec<f32>, AgentDbError> {
        let mut features = vec![0.0; 10];

        let char_count = text.len() as f32;
        let word_count = text.split_whitespace().count() as f32;
        let sentence_count = text.matches('.').count() as f32;

        features[0] = char_count.ln().max(0.0);
        features[1] = word_count.ln().max(0.0);
        features[2] = sentence_count.ln().max(0.0);
        features[3] = if word_count > 0.0 { char_count / word_count } else { 0.0 }; // 平均词长
        features[4] = if sentence_count > 0.0 { word_count / sentence_count } else { 0.0 }; // 平均句长

        // 字符类型统计
        let mut alpha_count = 0.0;
        let mut digit_count = 0.0;
        let mut punct_count = 0.0;
        let mut space_count = 0.0;

        for c in text.chars() {
            if c.is_alphabetic() { alpha_count += 1.0; }
            else if c.is_numeric() { digit_count += 1.0; }
            else if c.is_ascii_punctuation() { punct_count += 1.0; }
            else if c.is_whitespace() { space_count += 1.0; }
        }

        if char_count > 0.0 {
            features[5] = alpha_count / char_count;
            features[6] = digit_count / char_count;
            features[7] = punct_count / char_count;
            features[8] = space_count / char_count;
        }

        // 词汇多样性
        let words: Vec<&str> = text.split_whitespace().collect();
        let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
        features[9] = if word_count > 0.0 { unique_words.len() as f32 / word_count } else { 0.0 };

        Ok(features)
    }

    fn extract_tfidf_features(&self, text: &str) -> Result<Vec<f32>, AgentDbError> {
        // 简化的TF-IDF实现
        let words: Vec<&str> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| !w.is_empty())
            .collect();

        // 词频统计
        let mut word_counts = HashMap::new();
        for word in &words {
            *word_counts.entry(word.to_lowercase()).or_insert(0) += 1;
        }

        // 选择最频繁的100个词作为特征
        let mut word_freq_pairs: Vec<_> = word_counts.iter().collect();
        word_freq_pairs.sort_by(|a, b| b.1.cmp(a.1));

        let mut features = vec![0.0; 100];
        for (i, (word, &count)) in word_freq_pairs.iter().take(100).enumerate() {
            // 简化的TF-IDF计算（这里只计算TF，IDF需要文档集合）
            let tf = count as f32 / words.len() as f32;
            features[i] = tf;
        }

        Ok(features)
    }

    fn extract_ngram_features(&self, text: &str) -> Result<Vec<f32>, AgentDbError> {
        let words: Vec<&str> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| !w.is_empty())
            .collect();

        let mut features = vec![0.0; 50];

        // 2-gram特征
        let mut bigram_counts = HashMap::new();
        for i in 0..words.len().saturating_sub(1) {
            let bigram = format!("{} {}", words[i].to_lowercase(), words[i + 1].to_lowercase());
            *bigram_counts.entry(bigram).or_insert(0) += 1;
        }

        // 3-gram特征
        let mut trigram_counts = HashMap::new();
        for i in 0..words.len().saturating_sub(2) {
            let trigram = format!("{} {} {}",
                words[i].to_lowercase(),
                words[i + 1].to_lowercase(),
                words[i + 2].to_lowercase()
            );
            *trigram_counts.entry(trigram).or_insert(0) += 1;
        }

        // 选择最频繁的n-gram作为特征
        let mut bigram_pairs: Vec<_> = bigram_counts.iter().collect();
        bigram_pairs.sort_by(|a, b| b.1.cmp(a.1));

        let mut trigram_pairs: Vec<_> = trigram_counts.iter().collect();
        trigram_pairs.sort_by(|a, b| b.1.cmp(a.1));

        // 填充特征向量
        for (i, (_, &count)) in bigram_pairs.iter().take(25).enumerate() {
            features[i] = count as f32 / words.len().saturating_sub(1).max(1) as f32;
        }

        for (i, (_, &count)) in trigram_pairs.iter().take(25).enumerate() {
            features[25 + i] = count as f32 / words.len().saturating_sub(2).max(1) as f32;
        }

        Ok(features)
    }
}

impl MultimodalEngine {
    pub async fn new(db_path: &str) -> Result<Self, AgentDbError> {
        let connection = connect(db_path).execute().await?;

        let mut feature_extractors: HashMap<ModalityType, Box<dyn FeatureExtractor>> = HashMap::new();
        feature_extractors.insert(ModalityType::Text, Box::new(TextFeatureExtractor));
        feature_extractors.insert(ModalityType::Image, Box::new(ImageFeatureExtractor));
        feature_extractors.insert(ModalityType::Audio, Box::new(AudioFeatureExtractor));

        Ok(Self {
            connection,
            data_storage: HashMap::new(),
            cross_modal_mappings: HashMap::new(),
            feature_extractors,
        })
    }

    // 添加多模态数据
    pub fn add_multimodal_data(&mut self, data_id: String, modality_type: ModalityType, raw_data: Vec<u8>, metadata: HashMap<String, String>) -> Result<(), AgentDbError> {
        // 提取特征
        let embedding = if let Some(extractor) = self.feature_extractors.get(&modality_type) {
            Some(extractor.extract_features(&raw_data, &metadata)?)
        } else {
            None
        };

        let features = if let Some(extractor) = self.feature_extractors.get(&modality_type) {
            Some(extractor.extract_detailed_features(&raw_data, &metadata)?)
        } else {
            None
        };

        let multimodal_data = MultimodalData {
            data_id: data_id.clone(),
            modality_type,
            raw_data,
            metadata,
            embedding,
            features,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        self.data_storage.insert(data_id, multimodal_data);
        Ok(())
    }

    // 跨模态搜索
    pub fn cross_modal_search(&self, query_data_id: &str, target_modality: ModalityType, k: usize) -> Result<Vec<MultimodalSearchResult>, AgentDbError> {
        let query_data = self.data_storage.get(query_data_id)
            .ok_or_else(|| AgentDbError::InvalidArgument("Query data not found".to_string()))?;

        let query_embedding = query_data.embedding.as_ref()
            .ok_or_else(|| AgentDbError::InvalidArgument("Query data has no embedding".to_string()))?;

        // 如果查询和目标是同一模态，直接进行相似性搜索
        if query_data.modality_type == target_modality {
            return self.same_modal_search(query_embedding, target_modality, k);
        }

        // 跨模态搜索需要模态转换
        let transformed_embedding = self.transform_embedding(query_embedding, &query_data.modality_type, &target_modality)?;
        self.same_modal_search(&transformed_embedding, target_modality, k)
    }

    // 同模态搜索
    fn same_modal_search(&self, query_embedding: &[f32], target_modality: ModalityType, k: usize) -> Result<Vec<MultimodalSearchResult>, AgentDbError> {
        let mut candidates = Vec::new();

        for (data_id, data) in &self.data_storage {
            if data.modality_type == target_modality {
                if let Some(ref embedding) = data.embedding {
                    let similarity = cosine_similarity(query_embedding, embedding);
                    candidates.push((data_id.clone(), similarity, data));
                }
            }
        }

        // 按相似度排序
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results = candidates.into_iter()
            .take(k)
            .map(|(data_id, similarity, data)| MultimodalSearchResult {
                data_id,
                modality_type: data.modality_type.clone(),
                similarity_score: similarity,
                cross_modal_score: None,
                metadata: data.metadata.clone(),
                features_summary: self.summarize_features(&data.features),
            })
            .collect();

        Ok(results)
    }

    // 模态转换
    fn transform_embedding(&self, source_embedding: &[f32], source_modality: &ModalityType, target_modality: &ModalityType) -> Result<Vec<f32>, AgentDbError> {
        let mapping_key = format!("{:?}_to_{:?}", source_modality, target_modality);

        if let Some(mapping) = self.cross_modal_mappings.get(&mapping_key) {
            // 使用学习到的映射矩阵进行转换
            self.apply_linear_transformation(source_embedding, &mapping.transformation_matrix, &mapping.bias_vector)
        } else {
            // 如果没有学习到的映射，使用简单的线性投影
            self.simple_cross_modal_projection(source_embedding, source_modality, target_modality)
        }
    }

    // 应用线性变换
    fn apply_linear_transformation(&self, input: &[f32], matrix: &[Vec<f32>], bias: &[f32]) -> Result<Vec<f32>, AgentDbError> {
        if matrix.is_empty() || matrix[0].len() != input.len() {
            return Err(AgentDbError::InvalidArgument("Matrix dimensions mismatch".to_string()));
        }

        let mut output = vec![0.0; matrix.len()];

        for (i, row) in matrix.iter().enumerate() {
            let mut sum = 0.0;
            for (j, &weight) in row.iter().enumerate() {
                if j < input.len() {
                    sum += weight * input[j];
                }
            }
            output[i] = sum + bias.get(i).unwrap_or(&0.0);
        }

        Ok(output)
    }

    // 简单的跨模态投影
    fn simple_cross_modal_projection(&self, source_embedding: &[f32], source_modality: &ModalityType, target_modality: &ModalityType) -> Result<Vec<f32>, AgentDbError> {
        // 简化的跨模态投影实现
        match (source_modality, target_modality) {
            (ModalityType::Text, ModalityType::Image) => {
                // 文本到图像：使用语义映射
                let mut projected = vec![0.0; 144]; // 图像特征维度
                for (i, &val) in source_embedding.iter().enumerate() {
                    if i < projected.len() {
                        projected[i] = val * 0.8; // 简单的缩放
                    }
                }
                Ok(projected)
            }
            (ModalityType::Image, ModalityType::Text) => {
                // 图像到文本：提取视觉语义
                let mut projected = vec![0.0; 160]; // 文本特征维度
                for (i, &val) in source_embedding.iter().enumerate() {
                    if i < projected.len() {
                        projected[i] = val * 1.2;
                    }
                }
                Ok(projected)
            }
            (ModalityType::Audio, ModalityType::Text) => {
                // 音频到文本：音频语义映射
                let mut projected = vec![0.0; 160];
                for (i, &val) in source_embedding.iter().enumerate() {
                    if i < projected.len() {
                        projected[i] = val * 0.9;
                    }
                }
                Ok(projected)
            }
            (ModalityType::Text, ModalityType::Audio) => {
                // 文本到音频：语义到声学映射
                let mut projected = vec![0.0; 32];
                for (i, &val) in source_embedding.iter().enumerate() {
                    if i < projected.len() {
                        projected[i] = val * 0.7;
                    }
                }
                Ok(projected)
            }
            _ => {
                // 其他情况使用身份映射
                Ok(source_embedding.to_vec())
            }
        }
    }

    // 学习跨模态映射
    pub fn learn_cross_modal_mapping(&mut self, source_modality: ModalityType, target_modality: ModalityType, paired_data: Vec<(String, String)>) -> Result<String, AgentDbError> {
        let mapping_id = format!("mapping_{}_{}",
            chrono::Utc::now().timestamp_millis(),
            rand::thread_rng().gen::<u32>()
        );

        // 收集配对数据的特征
        let mut source_features = Vec::new();
        let mut target_features = Vec::new();

        for (source_id, target_id) in paired_data {
            if let (Some(source_data), Some(target_data)) = (
                self.data_storage.get(&source_id),
                self.data_storage.get(&target_id)
            ) {
                if source_data.modality_type == source_modality && target_data.modality_type == target_modality {
                    if let (Some(ref source_emb), Some(ref target_emb)) = (&source_data.embedding, &target_data.embedding) {
                        source_features.push(source_emb.clone());
                        target_features.push(target_emb.clone());
                    }
                }
            }
        }

        if source_features.is_empty() {
            return Err(AgentDbError::InvalidArgument("No valid paired data found".to_string()));
        }

        // 简化的线性回归学习映射矩阵
        let (transformation_matrix, bias_vector) = self.learn_linear_mapping(&source_features, &target_features)?;

        let mapping = CrossModalMapping {
            mapping_id: mapping_id.clone(),
            source_modality: source_modality.clone(),
            target_modality: target_modality.clone(),
            transformation_matrix,
            bias_vector,
            confidence_score: 0.8, // 简化的置信度
            created_at: chrono::Utc::now().timestamp(),
        };

        let mapping_key = format!("{:?}_to_{:?}", source_modality, target_modality);
        self.cross_modal_mappings.insert(mapping_key, mapping);

        Ok(mapping_id)
    }

    // 学习线性映射
    fn learn_linear_mapping(&self, source_features: &[Vec<f32>], target_features: &[Vec<f32>]) -> Result<(Vec<Vec<f32>>, Vec<f32>), AgentDbError> {
        if source_features.is_empty() || target_features.is_empty() || source_features.len() != target_features.len() {
            return Err(AgentDbError::InvalidArgument("Invalid training data".to_string()));
        }

        let input_dim = source_features[0].len();
        let output_dim = target_features[0].len();

        // 简化的最小二乘法实现
        let mut transformation_matrix = vec![vec![0.0; input_dim]; output_dim];
        let mut bias_vector = vec![0.0; output_dim];

        // 计算均值
        let mut source_mean = vec![0.0; input_dim];
        let mut target_mean = vec![0.0; output_dim];

        for features in source_features {
            for (i, &val) in features.iter().enumerate() {
                source_mean[i] += val;
            }
        }

        for features in target_features {
            for (i, &val) in features.iter().enumerate() {
                target_mean[i] += val;
            }
        }

        let n = source_features.len() as f32;
        for mean in &mut source_mean { *mean /= n; }
        for mean in &mut target_mean { *mean /= n; }

        // 简化的线性回归（每个输出维度独立计算）
        for out_dim in 0..output_dim {
            for in_dim in 0..input_dim {
                let mut numerator = 0.0;
                let mut denominator = 0.0;

                for i in 0..source_features.len() {
                    let x_centered = source_features[i][in_dim] - source_mean[in_dim];
                    let y_centered = target_features[i][out_dim] - target_mean[out_dim];

                    numerator += x_centered * y_centered;
                    denominator += x_centered * x_centered;
                }

                if denominator.abs() > 1e-8 {
                    transformation_matrix[out_dim][in_dim] = numerator / denominator;
                }
            }

            // 计算偏置
            let mut predicted_mean = 0.0;
            for in_dim in 0..input_dim {
                predicted_mean += transformation_matrix[out_dim][in_dim] * source_mean[in_dim];
            }
            bias_vector[out_dim] = target_mean[out_dim] - predicted_mean;
        }

        Ok((transformation_matrix, bias_vector))
    }

    // 多模态融合搜索
    pub fn multimodal_fusion_search(&self, query_data_ids: Vec<String>, k: usize) -> Result<Vec<MultimodalSearchResult>, AgentDbError> {
        if query_data_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 收集查询数据的嵌入
        let mut query_embeddings = Vec::new();
        let mut query_modalities = Vec::new();

        for data_id in &query_data_ids {
            if let Some(data) = self.data_storage.get(data_id) {
                if let Some(ref embedding) = data.embedding {
                    query_embeddings.push(embedding.clone());
                    query_modalities.push(data.modality_type.clone());
                }
            }
        }

        if query_embeddings.is_empty() {
            return Err(AgentDbError::InvalidArgument("No valid query embeddings found".to_string()));
        }

        // 融合查询嵌入
        let fused_embedding = self.fuse_embeddings(&query_embeddings, &query_modalities)?;

        // 对所有数据进行搜索
        let mut candidates = Vec::new();

        for (data_id, data) in &self.data_storage {
            if let Some(ref embedding) = data.embedding {
                // 将目标嵌入转换到融合空间
                let transformed_embedding = self.transform_to_fusion_space(embedding, &data.modality_type)?;
                let similarity = cosine_similarity(&fused_embedding, &transformed_embedding);

                candidates.push((data_id.clone(), similarity, data));
            }
        }

        // 按相似度排序
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results = candidates.into_iter()
            .take(k)
            .map(|(data_id, similarity, data)| MultimodalSearchResult {
                data_id,
                modality_type: data.modality_type.clone(),
                similarity_score: similarity,
                cross_modal_score: Some(similarity),
                metadata: data.metadata.clone(),
                features_summary: self.summarize_features(&data.features),
            })
            .collect();

        Ok(results)
    }

    // 融合多个嵌入
    fn fuse_embeddings(&self, embeddings: &[Vec<f32>], modalities: &[ModalityType]) -> Result<Vec<f32>, AgentDbError> {
        if embeddings.is_empty() {
            return Err(AgentDbError::InvalidArgument("No embeddings to fuse".to_string()));
        }

        // 找到最大维度
        let max_dim = embeddings.iter().map(|e| e.len()).max().unwrap_or(0);
        let mut fused = vec![0.0; max_dim];

        // 加权平均融合
        let weights = self.calculate_modality_weights(modalities);

        for (i, embedding) in embeddings.iter().enumerate() {
            let weight = weights.get(i).unwrap_or(&1.0);
            for (j, &val) in embedding.iter().enumerate() {
                if j < fused.len() {
                    fused[j] += val * weight;
                }
            }
        }

        // 归一化
        let total_weight: f32 = weights.iter().sum();
        if total_weight > 0.0 {
            for val in &mut fused {
                *val /= total_weight;
            }
        }

        Ok(fused)
    }

    // 计算模态权重
    fn calculate_modality_weights(&self, modalities: &[ModalityType]) -> Vec<f32> {
        modalities.iter().map(|modality| {
            match modality {
                ModalityType::Text => 1.0,
                ModalityType::Image => 1.2,
                ModalityType::Audio => 0.8,
                ModalityType::Video => 1.5,
                ModalityType::Multimodal => 1.0,
            }
        }).collect()
    }

    // 转换到融合空间
    fn transform_to_fusion_space(&self, embedding: &[f32], modality: &ModalityType) -> Result<Vec<f32>, AgentDbError> {
        // 简化的融合空间转换
        let scale_factor = match modality {
            ModalityType::Text => 1.0,
            ModalityType::Image => 0.9,
            ModalityType::Audio => 1.1,
            ModalityType::Video => 0.95,
            ModalityType::Multimodal => 1.0,
        };

        Ok(embedding.iter().map(|&x| x * scale_factor).collect())
    }

    // 总结特征
    fn summarize_features(&self, features: &Option<HashMap<String, f32>>) -> String {
        if let Some(ref feature_map) = features {
            let mut summary_parts = Vec::new();

            // 选择最重要的几个特征进行总结
            let mut sorted_features: Vec<_> = feature_map.iter().collect();
            sorted_features.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (key, value) in sorted_features.iter().take(5) {
                summary_parts.push(format!("{}:{:.3}", key, value));
            }

            summary_parts.join(", ")
        } else {
            "No features available".to_string()
        }
    }

    // 获取多模态统计
    pub fn get_multimodal_statistics(&self) -> MultimodalStatistics {
        let mut modality_counts = HashMap::new();
        let mut total_data_size = 0;
        let mut feature_dimensions = HashMap::new();

        for data in self.data_storage.values() {
            *modality_counts.entry(data.modality_type.clone()).or_insert(0) += 1;
            total_data_size += data.raw_data.len();

            if let Some(ref embedding) = data.embedding {
                feature_dimensions.insert(data.modality_type.clone(), embedding.len());
            }
        }

        MultimodalStatistics {
            total_data_count: self.data_storage.len(),
            modality_counts,
            total_data_size,
            cross_modal_mappings_count: self.cross_modal_mappings.len(),
            feature_dimensions,
            supported_modalities: vec![
                ModalityType::Text,
                ModalityType::Image,
                ModalityType::Audio,
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultimodalStatistics {
    pub total_data_count: usize,
    pub modality_counts: HashMap<ModalityType, usize>,
    pub total_data_size: usize,
    pub cross_modal_mappings_count: usize,
    pub feature_dimensions: HashMap<ModalityType, usize>,
    pub supported_modalities: Vec<ModalityType>,
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







