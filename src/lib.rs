// Agent状态数据库 - 简化版本用于测试
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Mutex;

// 简化版本 - 使用内存存储而不是LanceDB
static mut GLOBAL_STORAGE: Option<Mutex<HashMap<u64, Vec<u8>>>> = None;

// 初始化全局存储
fn init_storage() {
    unsafe {
        if GLOBAL_STORAGE.is_none() {
            GLOBAL_STORAGE = Some(Mutex::new(HashMap::new()));
        }
    }
}

// C FFI结构
#[repr(C)]
pub struct CAgentStateDB {
    initialized: bool,
}

#[no_mangle]
pub extern "C" fn agent_db_new(db_path: *const c_char) -> *mut CAgentStateDB {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    // 验证路径字符串
    let _path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

    // 初始化存储
    init_storage();

    // 创建数据库实例
    Box::into_raw(Box::new(CAgentStateDB { initialized: true }))
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
    _session_id: u64,
    _state_type: c_int,
    data: *const u8,
    data_len: usize,
) -> c_int {
    if db.is_null() || data.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    if !c_db.initialized {
        return -1;
    }

    let data_vec = unsafe { std::slice::from_raw_parts(data, data_len).to_vec() };

    unsafe {
        if let Some(ref storage) = GLOBAL_STORAGE {
            match storage.lock() {
                Ok(mut map) => {
                    map.insert(agent_id, data_vec);
                    0
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
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
    if !c_db.initialized {
        return -1;
    }

    unsafe {
        if let Some(ref storage) = GLOBAL_STORAGE {
            match storage.lock() {
                Ok(map) => {
                    if let Some(data) = map.get(&agent_id) {
                        let data_copy = data.clone().into_boxed_slice();
                        let len = data_copy.len();
                        let ptr = Box::into_raw(data_copy) as *mut u8;

                        *data_out = ptr;
                        *data_len_out = len;
                        0
                    } else {
                        1 // Not found
                    }
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
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







