// C FFI 接口模块
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use crate::core::AgentDbError;

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
    _private: [u8; 0],
}

// 导出的 C 函数
#[no_mangle]
pub extern "C" fn agent_db_new(db_path: *const c_char) -> *mut CAgentStateDB {
    if db_path.is_null() {
        return ptr::null_mut();
    }
    
    // 简化实现，返回一个虚拟指针
    Box::into_raw(Box::new(CAgentStateDB { _private: [] }))
}

#[no_mangle]
pub extern "C" fn agent_db_free(db: *mut CAgentStateDB) {
    if !db.is_null() {
        unsafe {
            let _ = Box::from_raw(db);
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
        return CAgentDbErrorCode::InvalidParam as c_int;
    }
    
    // 简化实现
    CAgentDbErrorCode::Success as c_int
}

#[no_mangle]
pub extern "C" fn agent_db_load_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    data: *mut *mut u8,
    data_len: *mut usize,
) -> c_int {
    if db.is_null() || data.is_null() || data_len.is_null() {
        return CAgentDbErrorCode::InvalidParam as c_int;
    }
    
    // 简化实现
    unsafe {
        *data = ptr::null_mut();
        *data_len = 0;
    }
    
    CAgentDbErrorCode::NotFound as c_int
}

#[no_mangle]
pub extern "C" fn agent_db_free_data(data: *mut u8, data_len: usize) {
    if !data.is_null() && data_len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(data, data_len, data_len);
        }
    }
}

#[no_mangle]
pub extern "C" fn agent_db_get_last_error(db: *mut CAgentStateDB) -> *const c_char {
    if db.is_null() {
        return ptr::null();
    }
    
    // 简化实现，返回空字符串
    b"\0".as_ptr() as *const c_char
}
