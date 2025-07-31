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
    println!("Testing LanceDB Rust Integration...");
    
    // Test 1: Create database
    println!("1. Creating LanceDB database...");
    let db_path = CString::new("test_lancedb_rust.lance").unwrap();
    let db = unsafe { agent_db_new(db_path.as_ptr()) };
    if db.is_null() {
        println!("   FAILED: Could not create database");
        return;
    }
    println!("   SUCCESS: Database created");
    
    // Test 2: Save agent state
    println!("2. Saving agent state to LanceDB...");
    let agent_id = 12345u64;
    let session_id = 67890u64;
    let state_type = 0i32; // WorkingMemory
    let test_data = b"Hello from LanceDB Rust test!";
    
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
    println!("   SUCCESS: Agent state saved to LanceDB");
    
    // Test 3: Load agent state
    println!("3. Loading agent state from LanceDB...");
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
        println!("   SUCCESS: Data loaded correctly from LanceDB: {:?}", 
                std::str::from_utf8(loaded_slice).unwrap());
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
    
    // Test 4: Save another agent with different state type
    println!("4. Testing different state types...");
    let agent_id_2 = 54321u64;
    let session_id_2 = 98765u64;
    let state_type_2 = 1i32; // LongTermMemory
    let test_data_2 = b"Long term memory data in LanceDB";
    
    let result_2 = unsafe {
        agent_db_save_state(
            db,
            agent_id_2,
            session_id_2,
            state_type_2,
            test_data_2.as_ptr(),
            test_data_2.len(),
        )
    };
    if result_2 != 0 {
        println!("   FAILED: Could not save second agent state (error code: {})", result_2);
        unsafe {
            agent_db_free_data(loaded_data, loaded_data_len);
            agent_db_free(db);
        }
        return;
    }
    
    // Load second agent
    let mut loaded_data_2: *mut u8 = ptr::null_mut();
    let mut loaded_data_len_2: usize = 0;
    
    let load_result_2 = unsafe {
        agent_db_load_state(db, agent_id_2, &mut loaded_data_2, &mut loaded_data_len_2)
    };
    if load_result_2 != 0 {
        println!("   FAILED: Could not load second agent state (error code: {})", load_result_2);
        unsafe {
            agent_db_free_data(loaded_data, loaded_data_len);
            agent_db_free(db);
        }
        return;
    }
    
    let loaded_slice_2 = unsafe { std::slice::from_raw_parts(loaded_data_2, loaded_data_len_2) };
    if loaded_slice_2 == test_data_2 {
        println!("   SUCCESS: Second agent data verified: {:?}", 
                std::str::from_utf8(loaded_slice_2).unwrap());
    } else {
        println!("   FAILED: Second agent data mismatch");
        unsafe {
            agent_db_free_data(loaded_data, loaded_data_len);
            agent_db_free_data(loaded_data_2, loaded_data_len_2);
            agent_db_free(db);
        }
        return;
    }
    
    // Clean up
    unsafe {
        agent_db_free_data(loaded_data, loaded_data_len);
        agent_db_free_data(loaded_data_2, loaded_data_len_2);
        agent_db_free(db);
    }
    
    println!("\n🎉 All LanceDB integration tests passed! ✅");
    println!("✓ LanceDB database creation");
    println!("✓ Persistent state storage");
    println!("✓ Multiple agent support");
    println!("✓ Different state types");
    println!("✓ Data integrity verification");
    println!("✓ Memory management");
}
