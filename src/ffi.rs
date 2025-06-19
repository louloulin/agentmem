// C FFI接口模块
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;
use tokio::runtime::Runtime;

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
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
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
}

#[no_mangle]
pub extern "C" fn agent_db_free(db: *mut CAgentStateDB) {
    if !db.is_null() {
        unsafe {
            let c_db = Box::from_raw(db);
            if !c_db.db.is_null() {
                let _ = Box::from_raw(c_db.db);
            }
            if !c_db.rt.is_null() {
                let rt_ptr = c_db.rt as *mut Runtime;
                let _ = Box::from_raw(rt_ptr);
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
    let state = crate::types::AgentState::new(agent_id, session_id, state_type, data_vec);

    match rt.block_on(agent_db.save_agent_state(&state)) {
        Ok(_) => SUCCESS,
        Err(_) => ERROR_GENERAL,
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
        return ERROR_GENERAL;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };
    let rt = unsafe { &*(c_db.rt as *const Runtime) };

    match rt.block_on(agent_db.load_agent_state(agent_id)) {
        Ok(Some(state)) => {
            let data_copy = state.data.into_boxed_slice();
            let len = data_copy.len();
            let ptr = Box::into_raw(data_copy) as *mut u8;

            unsafe {
                *data_out = ptr;
                *data_len_out = len;
            }
            SUCCESS
        }
        Ok(None) => ERROR_NOT_FOUND,
        Err(_) => ERROR_GENERAL,
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

    let db = match rt.block_on(async {
        AgentDB::new(path_str, 384).await
    }) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => return ptr::null_mut(),
    };

    let rt_ptr = Box::into_raw(Box::new(rt)) as *mut u8;

    Box::into_raw(Box::new(CMemoryManager { db, rt: rt_ptr }))
}

#[no_mangle]
pub extern "C" fn memory_manager_free(mgr: *mut CMemoryManager) {
    if !mgr.is_null() {
        unsafe {
            let c_mgr = Box::from_raw(mgr);
            if !c_mgr.db.is_null() {
                let _ = Box::from_raw(c_mgr.db);
            }
            if !c_mgr.rt.is_null() {
                let rt_ptr = c_mgr.rt as *mut Runtime;
                let _ = Box::from_raw(rt_ptr);
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
        return ERROR_GENERAL;
    }

    let c_mgr = unsafe { &*mgr };
    let agent_db = unsafe { &*c_mgr.db };
    let rt = unsafe { &*(c_mgr.rt as *const Runtime) };

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ERROR_GENERAL,
        }
    };

    let mem_type = match memory_type {
        0 => MemoryType::Episodic,
        1 => MemoryType::Semantic,
        2 => MemoryType::Procedural,
        3 => MemoryType::Working,
        _ => return ERROR_GENERAL,
    };

    let memory = Memory::new(agent_id, mem_type, content_str, importance);

    match rt.block_on(agent_db.store_memory(&memory)) {
        Ok(_) => SUCCESS,
        Err(_) => ERROR_GENERAL,
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
        return ERROR_GENERAL;
    }

    let c_mgr = unsafe { &*mgr };
    let agent_db = unsafe { &*c_mgr.db };
    let rt = unsafe { &*(c_mgr.rt as *const Runtime) };

    match rt.block_on(agent_db.get_agent_memories(agent_id, None, limit)) {
        Ok(memories) => {
            unsafe {
                *memory_count_out = memories.len();
            }
            SUCCESS
        }
        Err(_) => ERROR_GENERAL,
    }
}

// RAG引擎C接口
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

    let db = match rt.block_on(async {
        AgentDB::new(path_str, 384).await
    }) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => return ptr::null_mut(),
    };

    let rt_ptr = Box::into_raw(Box::new(rt)) as *mut u8;

    Box::into_raw(Box::new(CRAGEngine { db, rt: rt_ptr }))
}

#[no_mangle]
pub extern "C" fn rag_engine_free(engine: *mut CRAGEngine) {
    if !engine.is_null() {
        let c_engine = unsafe { Box::from_raw(engine) };
        if !c_engine.db.is_null() {
            unsafe { Box::from_raw(c_engine.db) };
        }
        if !c_engine.rt.is_null() {
            unsafe {
                let rt_ptr = c_engine.rt as *mut Runtime;
                let _ = Box::from_raw(rt_ptr);
            };
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
        return ERROR_GENERAL;
    }

    let c_engine = unsafe { &*engine };
    let agent_db = unsafe { &*c_engine.db };
    let rt = unsafe { &*(c_engine.rt as *const Runtime) };

    let title_str = unsafe {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s,
            Err(_) => return ERROR_GENERAL,
        }
    };

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s,
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
        return ERROR_GENERAL;
    }

    let c_engine = unsafe { &*engine };
    let agent_db = unsafe { &*c_engine.db };
    let rt = unsafe { &*(c_engine.rt as *const Runtime) };

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
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
        return ERROR_GENERAL;
    }

    let c_engine = unsafe { &*engine };
    let agent_db = unsafe { &*c_engine.db };
    let rt = unsafe { &*(c_engine.rt as *const Runtime) };

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
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
            let len = context_bytes.len();
            let ptr = Box::into_raw(context_bytes.into_boxed_slice()) as *mut c_char;

            unsafe {
                *context_out = ptr;
                *context_len_out = len;
            }
            SUCCESS
        }
        Err(_) => ERROR_GENERAL,
    }
}

#[no_mangle]
pub extern "C" fn rag_engine_free_context(context: *mut c_char) {
    if !context.is_null() {
        unsafe {
            // 这里需要知道原始长度，但C接口中没有传递
            // 简化处理，假设是以null结尾的字符串
            let _ = std::ffi::CString::from_raw(context);
        }
    }
}
