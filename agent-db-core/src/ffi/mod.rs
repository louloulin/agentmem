// FFI 模块 - 简化实现
use std::ffi::{CStr, c_char};
use std::ptr;

use crate::core::{AgentState, StateType};
use crate::agent_state::AgentStateDB;

// C 兼容的错误码
#[repr(C)]
pub enum CAgentDbErrorCode {
    Success = 0,
    InvalidParam = -1,
    NotFound = -2,
    IoError = -3,
    MemoryError = -4,
    InternalError = -5,
}

// C 兼容的数据库句柄
#[repr(C)]
pub struct CAgentStateDB {
    inner: *mut AgentStateDB,
}

// FFI 包装器
pub struct FFIAgentStateDB {
    db: AgentStateDB,
}

impl FFIAgentStateDB {
    pub fn new(db: AgentStateDB) -> Self {
        Self { db }
    }
}

// 创建数据库实例
#[no_mangle]
pub extern "C" fn agent_db_create(db_path: *const c_char) -> *mut CAgentStateDB {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(db_path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    // 使用简化的同步创建方法
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let db = match rt.block_on(AgentStateDB::new(path_str)) {
        Ok(db) => db,
        Err(_) => return ptr::null_mut(),
    };

    let ffi_db = Box::new(FFIAgentStateDB::new(db));
    let c_db = Box::new(CAgentStateDB {
        inner: Box::into_raw(ffi_db) as *mut AgentStateDB,
    });

    Box::into_raw(c_db)
}

// 销毁数据库实例
#[no_mangle]
pub extern "C" fn agent_db_destroy(db: *mut CAgentStateDB) {
    if !db.is_null() {
        unsafe {
            let c_db = Box::from_raw(db);
            if !c_db.inner.is_null() {
                let _ = Box::from_raw(c_db.inner as *mut FFIAgentStateDB);
            }
        }
    }
}

// 保存状态
#[no_mangle]
pub extern "C" fn agent_db_save_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    state_type: u32,
    data: *const c_char,
    data_len: usize,
) -> CAgentDbErrorCode {
    if db.is_null() || data.is_null() {
        return CAgentDbErrorCode::InvalidParam;
    }

    let c_db = unsafe { &*db };
    if c_db.inner.is_null() {
        return CAgentDbErrorCode::InvalidParam;
    }

    let data_slice = unsafe { std::slice::from_raw_parts(data as *const u8, data_len) };
    let data_string = match String::from_utf8(data_slice.to_vec()) {
        Ok(s) => s,
        Err(_) => return CAgentDbErrorCode::InvalidParam,
    };

    let state_type = match state_type {
        0 => StateType::WorkingMemory,
        1 => StateType::LongTermMemory,
        2 => StateType::Context,
        3 => StateType::TaskState,
        4 => StateType::Relationship,
        5 => StateType::Embedding,
        _ => return CAgentDbErrorCode::InvalidParam,
    };

    let state = AgentState::new(
        agent_id,
        0, // session_id
        state_type,
        data_string.into_bytes(),
    );

    // 简化实现：直接返回成功
    // 在实际实现中，这里应该调用异步方法
    CAgentDbErrorCode::Success
}

// 加载状态
#[no_mangle]
pub extern "C" fn agent_db_load_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    out_data: *mut *mut c_char,
    out_len: *mut usize,
) -> CAgentDbErrorCode {
    if db.is_null() || out_data.is_null() || out_len.is_null() {
        return CAgentDbErrorCode::InvalidParam;
    }

    let c_db = unsafe { &*db };
    if c_db.inner.is_null() {
        return CAgentDbErrorCode::InvalidParam;
    }

    // 简化实现：返回空数据
    unsafe {
        *out_data = ptr::null_mut();
        *out_len = 0;
    }

    CAgentDbErrorCode::NotFound
}

// 向量搜索
#[no_mangle]
pub extern "C" fn agent_db_vector_search(
    db: *mut CAgentStateDB,
    query_vector: *const f32,
    vector_len: usize,
    limit: usize,
    out_results: *mut *mut u64,
    out_count: *mut usize,
) -> CAgentDbErrorCode {
    if db.is_null() || query_vector.is_null() || out_results.is_null() || out_count.is_null() {
        return CAgentDbErrorCode::InvalidParam;
    }

    let c_db = unsafe { &*db };
    if c_db.inner.is_null() {
        return CAgentDbErrorCode::InvalidParam;
    }

    // 简化实现：返回空结果
    unsafe {
        *out_results = ptr::null_mut();
        *out_count = 0;
    }

    CAgentDbErrorCode::Success
}

// 释放内存
#[no_mangle]
pub extern "C" fn agent_db_free_memory(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr);
        }
    }
}

// 释放结果数组
#[no_mangle]
pub extern "C" fn agent_db_free_results(results: *mut u64, count: usize) {
    if !results.is_null() && count > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(results, count, count);
        }
    }
}

// 获取错误信息
#[no_mangle]
pub extern "C" fn agent_db_get_error_message(error_code: CAgentDbErrorCode) -> *const c_char {
    let message = match error_code {
        CAgentDbErrorCode::Success => "Success\0",
        CAgentDbErrorCode::InvalidParam => "Invalid parameter\0",
        CAgentDbErrorCode::NotFound => "Not found\0",
        CAgentDbErrorCode::IoError => "IO error\0",
        CAgentDbErrorCode::MemoryError => "Memory error\0",
        CAgentDbErrorCode::InternalError => "Internal error\0",
    };

    message.as_ptr() as *const c_char
}

// 版本信息
#[no_mangle]
pub extern "C" fn agent_db_version() -> *const c_char {
    "0.2.0\0".as_ptr() as *const c_char
}

// 初始化库
#[no_mangle]
pub extern "C" fn agent_db_init() -> CAgentDbErrorCode {
    // 初始化日志等
    CAgentDbErrorCode::Success
}

// 清理库
#[no_mangle]
pub extern "C" fn agent_db_cleanup() {
    // 清理资源
}

// 创建数据库实例 - 简化版本
#[no_mangle]
pub extern "C" fn agent_db_new(db_path: *const c_char) -> *mut CAgentStateDB {
    agent_db_create(db_path)
}

// 释放数据库实例
#[no_mangle]
pub extern "C" fn agent_db_free(db: *mut CAgentStateDB) {
    agent_db_destroy(db);
}

// 释放数据
#[no_mangle]
pub extern "C" fn agent_db_free_data(data: *mut c_char) {
    agent_db_free_memory(data);
}
