use std::ffi::CString;
use std::ptr;

// 导入我们的库函数
extern "C" {
    fn agent_db_new(path: *const i8) -> *mut std::ffi::c_void;
    fn agent_db_free(db: *mut std::ffi::c_void);
    fn agent_db_save_state(
        db: *mut std::ffi::c_void,
        agent_id: u64,
        session_id: u64,
        state_type: i32,
        data: *const u8,
        data_len: usize,
    ) -> i32;
    fn agent_db_load_state(
        db: *mut std::ffi::c_void,
        agent_id: u64,
        data: *mut *mut u8,
        data_len: *mut usize,
    ) -> i32;
    fn agent_db_free_data(data: *mut u8, data_len: usize);
}

fn main() {
    println!("Testing Rust internal integration...");
    
    // Test 1: Create database
    println!("1. Creating database...");
    let db_path = CString::new("test_internal.db").unwrap();
    let db = unsafe { agent_db_new(db_path.as_ptr()) };
    if db.is_null() {
        println!("   FAILED: Could not create database");
        return;
    }
    println!("   SUCCESS: Database created");
    
    // Test 2: Save agent state
    println!("2. Saving agent state...");
    let agent_id = 12345u64;
    let session_id = 67890u64;
    let state_type = 1i32; // working_memory
    let test_data = b"Hello from Rust internal test!";
    
    let result = unsafe {
        agent_db_save_state(
            db,
            agent_id,
            session_id,
            state_type,
            test_data.as_ptr(),
            test_data.len(),
        )
    };
    if result != 0 {
        println!("   FAILED: Could not save state (error code: {})", result);
        unsafe { agent_db_free(db); }
        return;
    }
    println!("   SUCCESS: Agent state saved");
    
    // Test 3: Load agent state
    println!("3. Loading agent state...");
    let mut loaded_data: *mut u8 = ptr::null_mut();
    let mut loaded_data_len: usize = 0;
    
    let load_result = unsafe {
        agent_db_load_state(db, agent_id, &mut loaded_data, &mut loaded_data_len)
    };
    if load_result != 0 {
        println!("   FAILED: Could not load state (error code: {})", load_result);
        unsafe { agent_db_free(db); }
        return;
    }
    
    if loaded_data.is_null() || loaded_data_len == 0 {
        println!("   FAILED: No data loaded");
        unsafe { agent_db_free(db); }
        return;
    }
    
    // Verify data
    let loaded_slice = unsafe { std::slice::from_raw_parts(loaded_data, loaded_data_len) };
    if loaded_slice == test_data {
        println!("   SUCCESS: Data loaded correctly: {:?}", std::str::from_utf8(loaded_slice).unwrap());
    } else {
        println!("   FAILED: Data mismatch");
        println!("   Expected: {:?}", std::str::from_utf8(test_data).unwrap());
        println!("   Got: {:?}", std::str::from_utf8(loaded_slice).unwrap());
        unsafe {
            agent_db_free_data(loaded_data, loaded_data_len);
            agent_db_free(db);
        }
        return;
    }
    
    // Clean up
    unsafe {
        agent_db_free_data(loaded_data, loaded_data_len);
        agent_db_free(db);
    }
    
    println!("\nAll tests passed! ✅");
}
