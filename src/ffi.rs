<<<<<<< HEAD
// C FFI接口模块
=======
// FFI 模块 - C语言接口
// 从 lib.rs 自动拆分生成

>>>>>>> origin/feature-module
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;
use tokio::runtime::Runtime;
<<<<<<< HEAD

use crate::api::AgentDB;
use crate::types::{StateType, MemoryType, Memory};

// C结构体定义
#[repr(C)]
pub struct CAgentStateDB {
    db: *mut AgentDB,
    rt: *mut u8, // 使用u8指针代替Runtime
}

#[repr(C)]
pub struct CMemoryManager {
    db: *mut AgentDB,
    rt: *mut u8, // 使用u8指针代替Runtime
}

#[repr(C)]
pub struct CRAGEngine {
    db: *mut AgentDB,
    rt: *mut u8, // 使用u8指针代替Runtime
}

// 错误码定义
const SUCCESS: c_int = 0;
const ERROR_GENERAL: c_int = -1;
const ERROR_NOT_FOUND: c_int = 1;

// Agent状态数据库C接口
#[no_mangle]
pub extern "C" fn agent_db_new(db_path: *const c_char) -> *mut CAgentStateDB {
    eprintln!("agent_db_new called");

    if db_path.is_null() {
        eprintln!("agent_db_new: db_path is null");
=======
use crate::core::*;
use crate::agent_state::*;
use crate::memory::*;

// C FFI接口 - Agent状态数据库
#[repr(C)]
pub struct CAgentStateDB {
    db: *mut AgentStateDB,
}

#[no_mangle]
pub extern "C" fn agent_db_new(db_path: *const c_char) -> *mut CAgentStateDB {
    if db_path.is_null() {
>>>>>>> origin/feature-module
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
<<<<<<< HEAD
            Ok(s) => {
                eprintln!("agent_db_new: path = {}", s);
                s
            },
            Err(e) => {
                eprintln!("agent_db_new: invalid path string: {:?}", e);
                return ptr::null_mut();
            },
        }
    };

    // 创建运行时
    eprintln!("agent_db_new: creating runtime");
    let rt = match Runtime::new() {
        Ok(rt) => {
            eprintln!("agent_db_new: runtime created successfully");
            rt
        },
        Err(e) => {
            eprintln!("agent_db_new: failed to create runtime: {:?}", e);
            return ptr::null_mut();
        },
    };

    // 创建数据库
    eprintln!("agent_db_new: creating database");
    let db = match rt.block_on(async {
        AgentDB::new(path_str, 384).await
    }) {
        Ok(db) => {
            eprintln!("agent_db_new: database created successfully");
            Box::into_raw(Box::new(db))
        },
        Err(e) => {
            eprintln!("agent_db_new: failed to create database: {:?}", e);
            return ptr::null_mut();
        },
    };

    let rt_ptr = Box::into_raw(Box::new(rt)) as *mut u8;

    eprintln!("agent_db_new: returning handle");
    Box::into_raw(Box::new(CAgentStateDB { db, rt: rt_ptr }))
=======
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

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
>>>>>>> origin/feature-module
}

#[no_mangle]
pub extern "C" fn agent_db_free(db: *mut CAgentStateDB) {
    if !db.is_null() {
        unsafe {
            let c_db = Box::from_raw(db);
            if !c_db.db.is_null() {
                let _ = Box::from_raw(c_db.db);
            }
<<<<<<< HEAD
            if !c_db.rt.is_null() {
                let rt_ptr = c_db.rt as *mut Runtime;
                let _ = Box::from_raw(rt_ptr);
            }
=======
>>>>>>> origin/feature-module
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
<<<<<<< HEAD
        return ERROR_GENERAL;
=======
        return -1;
>>>>>>> origin/feature-module
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };
<<<<<<< HEAD
    let rt = unsafe { &*(c_db.rt as *const Runtime) };
=======
>>>>>>> origin/feature-module

    let state_type = match state_type {
        0 => StateType::WorkingMemory,
        1 => StateType::LongTermMemory,
        2 => StateType::Context,
        3 => StateType::TaskState,
        4 => StateType::Relationship,
        5 => StateType::Embedding,
<<<<<<< HEAD
        _ => return ERROR_GENERAL,
    };

    let data_vec = unsafe { std::slice::from_raw_parts(data, data_len).to_vec() };
    let state = crate::types::AgentState::new(agent_id, session_id, state_type, data_vec);

    match rt.block_on(agent_db.save_agent_state(&state)) {
        Ok(_) => SUCCESS,
        Err(_) => ERROR_GENERAL,
=======
        _ => return -1,
    };

    let data_vec = unsafe { std::slice::from_raw_parts(data, data_len).to_vec() };
    let state = AgentState::new(agent_id, session_id, state_type, data_vec);

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.save_state(&state)) {
        Ok(_) => 0,
        Err(_) => -1,
>>>>>>> origin/feature-module
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
<<<<<<< HEAD
        return ERROR_GENERAL;
=======
        return -1;
>>>>>>> origin/feature-module
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };
<<<<<<< HEAD
    let rt = unsafe { &*(c_db.rt as *const Runtime) };

    match rt.block_on(agent_db.load_agent_state(agent_id)) {
=======

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.load_state(agent_id)) {
>>>>>>> origin/feature-module
        Ok(Some(state)) => {
            let data_copy = state.data.into_boxed_slice();
            let len = data_copy.len();
            let ptr = Box::into_raw(data_copy) as *mut u8;

            unsafe {
                *data_out = ptr;
                *data_len_out = len;
            }
<<<<<<< HEAD
            SUCCESS
        }
        Ok(None) => ERROR_NOT_FOUND,
        Err(_) => ERROR_GENERAL,
=======
            0
        }
        Ok(None) => 1, // Not found
        Err(_) => -1,
>>>>>>> origin/feature-module
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

<<<<<<< HEAD
// 向量功能的C接口
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
        return ERROR_GENERAL;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };
    let rt = unsafe { &*(c_db.rt as *const Runtime) };

    let state_type = match state_type {
        0 => StateType::WorkingMemory,
        1 => StateType::LongTermMemory,
        2 => StateType::Context,
        3 => StateType::TaskState,
        4 => StateType::Relationship,
        5 => StateType::Embedding,
        _ => return ERROR_GENERAL,
    };

    let data_vec = unsafe { std::slice::from_raw_parts(data, data_len).to_vec() };
    let _embedding_vec = unsafe { std::slice::from_raw_parts(embedding, embedding_len).to_vec() };

    let state = crate::types::AgentState::new(agent_id, session_id, state_type, data_vec);
    
    // 这里需要扩展AgentState来支持embedding，暂时忽略embedding
    match rt.block_on(agent_db.save_agent_state(&state)) {
        Ok(_) => SUCCESS,
        Err(_) => ERROR_GENERAL,
    }
}

#[no_mangle]
pub extern "C" fn agent_db_vector_search(
    db: *mut CAgentStateDB,
    query_embedding: *const f32,
    embedding_len: usize,
    _limit: usize,
    results_out: *mut *mut u64,
    results_count_out: *mut usize,
) -> c_int {
    if db.is_null() || query_embedding.is_null() || results_out.is_null() || results_count_out.is_null() {
        return ERROR_GENERAL;
    }

    let c_db = unsafe { &*db };
    let _agent_db = unsafe { &*c_db.db };
    let _rt = unsafe { &*(c_db.rt as *const Runtime) };

    let _query_vec = unsafe { std::slice::from_raw_parts(query_embedding, embedding_len).to_vec() };

    // 这里需要实现向量搜索，暂时返回空结果
    let agent_ids: Vec<u64> = Vec::new();
    let agent_ids_copy = agent_ids.into_boxed_slice();
    let len = agent_ids_copy.len();
    let ptr = Box::into_raw(agent_ids_copy) as *mut u64;

    unsafe {
        *results_out = ptr;
        *results_count_out = len;
    }
    SUCCESS
}

// 记忆管理的C接口
#[no_mangle]
=======
// C FFI接口 - 记忆管理器
#[repr(C)]
pub struct CMemoryManager {
    mgr: *mut MemoryManager,
}

#[no_mangle]
>>>>>>> origin/feature-module
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

<<<<<<< HEAD
    let db = match rt.block_on(async {
        AgentDB::new(path_str, 384).await
    }) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => return ptr::null_mut(),
    };

    let rt_ptr = Box::into_raw(Box::new(rt)) as *mut u8;

    Box::into_raw(Box::new(CMemoryManager { db, rt: rt_ptr }))
=======
    let connection = match rt.block_on(async {
        lancedb::connect(path_str).execute().await
    }) {
        Ok(conn) => conn,
        Err(_) => return ptr::null_mut(),
    };

    let mgr = MemoryManager::new(connection);
    let mgr_ptr = Box::into_raw(Box::new(mgr));

    Box::into_raw(Box::new(CMemoryManager { mgr: mgr_ptr }))
>>>>>>> origin/feature-module
}

#[no_mangle]
pub extern "C" fn memory_manager_free(mgr: *mut CMemoryManager) {
    if !mgr.is_null() {
        unsafe {
            let c_mgr = Box::from_raw(mgr);
<<<<<<< HEAD
            if !c_mgr.db.is_null() {
                let _ = Box::from_raw(c_mgr.db);
            }
            if !c_mgr.rt.is_null() {
                let rt_ptr = c_mgr.rt as *mut Runtime;
                let _ = Box::from_raw(rt_ptr);
=======
            if !c_mgr.mgr.is_null() {
                let _ = Box::from_raw(c_mgr.mgr);
>>>>>>> origin/feature-module
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
<<<<<<< HEAD
    importance: f32,
) -> c_int {
    if mgr.is_null() || content.is_null() {
        return ERROR_GENERAL;
    }

    let c_mgr = unsafe { &*mgr };
    let agent_db = unsafe { &*c_mgr.db };
    let rt = unsafe { &*(c_mgr.rt as *const Runtime) };
=======
    importance: f64,
) -> c_int {
    if mgr.is_null() || content.is_null() {
        return -1;
    }

    let c_mgr = unsafe { &*mgr };
    let memory_mgr = unsafe { &*c_mgr.mgr };
>>>>>>> origin/feature-module

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s.to_string(),
<<<<<<< HEAD
            Err(_) => return ERROR_GENERAL,
=======
            Err(_) => return -1,
>>>>>>> origin/feature-module
        }
    };

    let mem_type = match memory_type {
        0 => MemoryType::Episodic,
        1 => MemoryType::Semantic,
        2 => MemoryType::Procedural,
        3 => MemoryType::Working,
<<<<<<< HEAD
        _ => return ERROR_GENERAL,
=======
        _ => return -1,
>>>>>>> origin/feature-module
    };

    let memory = Memory::new(agent_id, mem_type, content_str, importance);

<<<<<<< HEAD
    match rt.block_on(agent_db.store_memory(&memory)) {
        Ok(_) => SUCCESS,
        Err(_) => ERROR_GENERAL,
=======
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_mgr.store_memory(&memory)) {
        Ok(_) => 0,
        Err(_) => -1,
>>>>>>> origin/feature-module
    }
}

#[no_mangle]
pub extern "C" fn memory_manager_retrieve_memories(
    mgr: *mut CMemoryManager,
    agent_id: u64,
<<<<<<< HEAD
    limit: usize,
    memory_count_out: *mut usize,
) -> c_int {
    if mgr.is_null() || memory_count_out.is_null() {
        return ERROR_GENERAL;
    }

    let c_mgr = unsafe { &*mgr };
    let agent_db = unsafe { &*c_mgr.db };
    let rt = unsafe { &*(c_mgr.rt as *const Runtime) };

    match rt.block_on(agent_db.get_agent_memories(agent_id, None, limit)) {
=======
    _limit: usize,
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

    match rt.block_on(memory_mgr.get_memories_by_agent(agent_id)) {
>>>>>>> origin/feature-module
        Ok(memories) => {
            unsafe {
                *memory_count_out = memories.len();
            }
<<<<<<< HEAD
            SUCCESS
        }
        Err(_) => ERROR_GENERAL,
    }
}

// RAG引擎C接口
=======
            0
        }
        Err(_) => -1,
    }
}

// C FFI接口 - RAG引擎
#[repr(C)]
pub struct CRAGEngine {
    engine: *mut RAGEngine,
}

// 简单的RAG引擎实现
pub struct RAGEngine {
    db_path: String,
    documents: Vec<Document>,
}

pub struct Document {
    title: String,
    content: String,
    chunks: Vec<String>,
}

impl RAGEngine {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
            documents: Vec::new(),
        }
    }

    pub fn index_document(&mut self, title: &str, content: &str, chunk_size: usize, overlap: usize) -> Result<(), Box<dyn std::error::Error>> {
        let chunks = self.chunk_text(content, chunk_size, overlap);
        let document = Document {
            title: title.to_string(),
            content: content.to_string(),
            chunks,
        };
        self.documents.push(document);
        Ok(())
    }

    pub fn search_text(&self, query: &str, limit: usize) -> Result<usize, Box<dyn std::error::Error>> {
        let mut matches = 0;
        for document in &self.documents {
            for chunk in &document.chunks {
                if chunk.to_lowercase().contains(&query.to_lowercase()) {
                    matches += 1;
                    if matches >= limit {
                        break;
                    }
                }
            }
            if matches >= limit {
                break;
            }
        }
        Ok(matches)
    }

    pub fn build_context(&self, query: &str, max_tokens: usize) -> Result<String, Box<dyn std::error::Error>> {
        let mut context = String::new();
        let mut token_count = 0;

        for document in &self.documents {
            for chunk in &document.chunks {
                if chunk.to_lowercase().contains(&query.to_lowercase()) {
                    let chunk_tokens = chunk.split_whitespace().count();
                    if token_count + chunk_tokens <= max_tokens {
                        if !context.is_empty() {
                            context.push_str("\n\n");
                        }
                        context.push_str(chunk);
                        token_count += chunk_tokens;
                    } else {
                        break;
                    }
                }
            }
            if token_count >= max_tokens {
                break;
            }
        }

        if context.is_empty() {
            context = format!("No relevant context found for query: {}", query);
        }

        Ok(context)
    }

    fn chunk_text(&self, text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();

        if words.is_empty() {
            return chunks;
        }

        let mut start = 0;
        while start < words.len() {
            let end = std::cmp::min(start + chunk_size, words.len());
            let chunk = words[start..end].join(" ");
            chunks.push(chunk);

            if end >= words.len() {
                break;
            }

            start = if chunk_size > overlap { end - overlap } else { end };
        }

        chunks
    }
}

>>>>>>> origin/feature-module
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

<<<<<<< HEAD
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let db = match rt.block_on(async {
        AgentDB::new(path_str, 384).await
    }) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => return ptr::null_mut(),
    };

    let rt_ptr = Box::into_raw(Box::new(rt)) as *mut u8;

    Box::into_raw(Box::new(CRAGEngine { db, rt: rt_ptr }))
=======
    let engine = RAGEngine::new(path_str);
    let engine_ptr = Box::into_raw(Box::new(engine));

    Box::into_raw(Box::new(CRAGEngine { engine: engine_ptr }))
>>>>>>> origin/feature-module
}

#[no_mangle]
pub extern "C" fn rag_engine_free(engine: *mut CRAGEngine) {
    if !engine.is_null() {
<<<<<<< HEAD
        let c_engine = unsafe { Box::from_raw(engine) };
        if !c_engine.db.is_null() {
            unsafe { Box::from_raw(c_engine.db) };
        }
        if !c_engine.rt.is_null() {
            unsafe {
                let rt_ptr = c_engine.rt as *mut Runtime;
                let _ = Box::from_raw(rt_ptr);
            };
=======
        unsafe {
            let c_engine = Box::from_raw(engine);
            if !c_engine.engine.is_null() {
                let _ = Box::from_raw(c_engine.engine);
            }
>>>>>>> origin/feature-module
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
<<<<<<< HEAD
        return ERROR_GENERAL;
    }

    let c_engine = unsafe { &*engine };
    let agent_db = unsafe { &*c_engine.db };
    let rt = unsafe { &*(c_engine.rt as *const Runtime) };
=======
        return -1;
    }

    let c_engine = unsafe { &mut *engine };
    let rag_engine = unsafe { &mut *c_engine.engine };
>>>>>>> origin/feature-module

    let title_str = unsafe {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s,
<<<<<<< HEAD
            Err(_) => return ERROR_GENERAL,
=======
            Err(_) => return -1,
>>>>>>> origin/feature-module
        }
    };

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s,
<<<<<<< HEAD
            Err(_) => return ERROR_GENERAL,
        }
    };

    // 创建文档并添加到数据库
    let doc = crate::types::Document {
        doc_id: format!("doc_{}", title_str),
        title: title_str.to_string(),
        content: content_str.to_string(),
        metadata: std::collections::HashMap::new(),
        chunks: Vec::new(), // 空的chunks，会在add_document中处理
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    match rt.block_on(agent_db.add_document(doc)) {
        Ok(_) => SUCCESS,
        Err(_) => ERROR_GENERAL,
=======
            Err(_) => return -1,
        }
    };

    match rag_engine.index_document(title_str, content_str, chunk_size, overlap) {
        Ok(_) => 0,
        Err(_) => -1,
>>>>>>> origin/feature-module
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
<<<<<<< HEAD
        return ERROR_GENERAL;
    }

    let c_engine = unsafe { &*engine };
    let agent_db = unsafe { &*c_engine.db };
    let rt = unsafe { &*(c_engine.rt as *const Runtime) };
=======
        return -1;
    }

    let c_engine = unsafe { &*engine };
    let rag_engine = unsafe { &*c_engine.engine };
>>>>>>> origin/feature-module

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
<<<<<<< HEAD
            Err(_) => return ERROR_GENERAL,
        }
    };

    match rt.block_on(agent_db.search_documents(query_str, limit)) {
        Ok(results) => {
            unsafe {
                *results_count_out = results.len();
            }
            SUCCESS
        }
        Err(_) => ERROR_GENERAL,
=======
            Err(_) => return -1,
        }
    };

    match rag_engine.search_text(query_str, limit) {
        Ok(count) => {
            unsafe {
                *results_count_out = count;
            }
            0
        }
        Err(_) => -1,
>>>>>>> origin/feature-module
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
<<<<<<< HEAD
        return ERROR_GENERAL;
    }

    let c_engine = unsafe { &*engine };
    let agent_db = unsafe { &*c_engine.db };
    let rt = unsafe { &*(c_engine.rt as *const Runtime) };
=======
        return -1;
    }

    let c_engine = unsafe { &*engine };
    let rag_engine = unsafe { &*c_engine.engine };
>>>>>>> origin/feature-module

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
<<<<<<< HEAD
            Err(_) => return ERROR_GENERAL,
        }
    };

    // 搜索相关文档并构建上下文
    match rt.block_on(agent_db.search_documents(query_str, 5)) {
        Ok(results) => {
            let mut context = String::new();
            let mut token_count = 0;

            for result in results {
                let chunk_tokens = result.content.split_whitespace().count();
                if token_count + chunk_tokens > max_tokens {
                    break;
                }
                context.push_str(&result.content);
                context.push('\n');
                token_count += chunk_tokens;
            }

            if context.is_empty() {
                context = "No relevant context found.".to_string();
            }

            let context_bytes = context.into_bytes();
=======
            Err(_) => return -1,
        }
    };

    match rag_engine.build_context(query_str, max_tokens) {
        Ok(context) => {
            let context_cstring = match std::ffi::CString::new(context) {
                Ok(s) => s,
                Err(_) => return -1,
            };

            let context_bytes = context_cstring.into_bytes_with_nul();
>>>>>>> origin/feature-module
            let len = context_bytes.len();
            let ptr = Box::into_raw(context_bytes.into_boxed_slice()) as *mut c_char;

            unsafe {
                *context_out = ptr;
                *context_len_out = len;
            }
<<<<<<< HEAD
            SUCCESS
        }
        Err(_) => ERROR_GENERAL,
=======
            0
        }
        Err(_) => -1,
>>>>>>> origin/feature-module
    }
}

#[no_mangle]
pub extern "C" fn rag_engine_free_context(context: *mut c_char) {
    if !context.is_null() {
        unsafe {
<<<<<<< HEAD
            // 这里需要知道原始长度，但C接口中没有传递
            // 简化处理，假设是以null结尾的字符串
=======
>>>>>>> origin/feature-module
            let _ = std::ffi::CString::from_raw(context);
        }
    }
}
<<<<<<< HEAD
=======

// 向量状态保存功能
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

    let c_db = unsafe { &mut *db };
    let agent_db = unsafe { &mut *c_db.db };

    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let embedding_slice = unsafe { std::slice::from_raw_parts(embedding, embedding_len) };

    // 将向量数据序列化为字节
    let mut vector_data = Vec::new();
    vector_data.extend_from_slice(data_slice);
    vector_data.push(b'|'); // 分隔符

    // 将embedding转换为字节
    for &value in embedding_slice {
        vector_data.extend_from_slice(&value.to_le_bytes());
    }

    let state = AgentState {
        id: format!("{}_{}", agent_id, session_id),
        agent_id,
        session_id,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        state_type: match state_type {
            0 => StateType::WorkingMemory,
            1 => StateType::LongTermMemory,
            2 => StateType::Context,
            3 => StateType::TaskState,
            4 => StateType::Relationship,
            5 => StateType::Embedding,
            _ => StateType::WorkingMemory,
        },
        data: vector_data,
        metadata: std::collections::HashMap::new(),
        version: 1,
        checksum: 0,
    };

    // 使用同步版本或者创建一个运行时
    match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            match rt.block_on(agent_db.save_state(&state)) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

// 向量状态加载功能
#[no_mangle]
pub extern "C" fn agent_db_load_vector_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    data_out: *mut *mut u8,
    data_len_out: *mut usize,
    embedding_out: *mut *mut f32,
    embedding_len_out: *mut usize,
) -> c_int {
    if db.is_null() || data_out.is_null() || data_len_out.is_null() ||
       embedding_out.is_null() || embedding_len_out.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };

    match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            match rt.block_on(agent_db.load_state(agent_id)) {
                Ok(Some(state)) => {
                    // 查找分隔符
                    if let Some(separator_pos) = state.data.iter().position(|&x| x == b'|') {
                        let data_part = &state.data[..separator_pos];
                        let embedding_part = &state.data[separator_pos + 1..];

                        // 分配数据内存
                        let data_len = data_part.len();
                        let data_ptr = Box::into_raw(data_part.to_vec().into_boxed_slice()) as *mut u8;

                        // 转换embedding
                        let embedding_len = embedding_part.len() / 4; // f32 = 4 bytes
                        let mut embedding_vec = Vec::with_capacity(embedding_len);

                        for chunk in embedding_part.chunks_exact(4) {
                            if let Ok(bytes) = chunk.try_into() {
                                embedding_vec.push(f32::from_le_bytes(bytes));
                            }
                        }

                        let embedding_ptr = Box::into_raw(embedding_vec.into_boxed_slice()) as *mut f32;

                        unsafe {
                            *data_out = data_ptr;
                            *data_len_out = data_len;
                            *embedding_out = embedding_ptr;
                            *embedding_len_out = embedding_len;
                        }
                        0
                    } else {
                        -1
                    }
                }
                Ok(None) => 1, // Not found
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

// 释放向量数据
#[no_mangle]
pub extern "C" fn agent_db_free_vector_data(
    data: *mut u8,
    data_len: usize,
    embedding: *mut f32,
    embedding_len: usize,
) {
    if !data.is_null() {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(data, data_len));
        }
    }
    if !embedding.is_null() {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(embedding, embedding_len));
        }
    }
}

// 向量搜索功能
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
    let _agent_db = unsafe { &*c_db.db };

    let _query_slice = unsafe { std::slice::from_raw_parts(query_embedding, embedding_len) };

    // 简单的向量搜索实现 - 在实际应用中应该使用更高效的算法
    let mut results = Vec::new();

    // 这里应该实现真正的向量搜索逻辑
    // 目前返回一些示例结果
    for i in 0..std::cmp::min(limit, 5) {
        results.push(1000 + i as u64);
    }

    let results_len = results.len();
    let results_ptr = Box::into_raw(results.into_boxed_slice()) as *mut u64;

    unsafe {
        *results_out = results_ptr;
        *results_count_out = results_len;
    }

    0
}
>>>>>>> origin/feature-module
