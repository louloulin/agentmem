// C FFI接口模块
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::AgentDB;
use crate::types::{AgentDbError, StateType, MemoryType, Memory};

// C结构体定义
#[repr(C)]
pub struct CAgentStateDB {
    db: *mut AgentDB,
    rt: *mut Runtime,
}

#[repr(C)]
pub struct CMemoryManager {
    db: *mut AgentDB,
    rt: *mut Runtime,
}

// 错误码定义
const SUCCESS: c_int = 0;
const ERROR_GENERAL: c_int = -1;
const ERROR_NOT_FOUND: c_int = 1;

// Agent状态数据库C接口
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

    // 创建运行时
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    // 创建数据库
    let db = match rt.block_on(async {
        AgentDB::new(path_str, 384).await
    }) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => return ptr::null_mut(),
    };

    let rt_ptr = Box::into_raw(Box::new(rt));

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
                let _ = Box::from_raw(c_db.rt);
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
    let rt = unsafe { &*c_db.rt };

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
    let rt = unsafe { &*c_db.rt };

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
    let rt = unsafe { &*c_db.rt };

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
    let embedding_vec = unsafe { std::slice::from_raw_parts(embedding, embedding_len).to_vec() };

    let mut state = crate::types::AgentState::new(agent_id, session_id, state_type, data_vec);
    
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
    limit: usize,
    results_out: *mut *mut u64,
    results_count_out: *mut usize,
) -> c_int {
    if db.is_null() || query_embedding.is_null() || results_out.is_null() || results_count_out.is_null() {
        return ERROR_GENERAL;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };
    let rt = unsafe { &*c_db.rt };

    let query_vec = unsafe { std::slice::from_raw_parts(query_embedding, embedding_len).to_vec() };

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

    let rt_ptr = Box::into_raw(Box::new(rt));

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
                let _ = Box::from_raw(c_mgr.rt);
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
    let rt = unsafe { &*c_mgr.rt };

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
    let rt = unsafe { &*c_mgr.rt };

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
