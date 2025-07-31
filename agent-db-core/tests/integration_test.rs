use std::ffi::CString;
use std::ptr;

// Import the C functions from our library
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

#[test]
fn test_agent_state_db_integration() {
    println!("Testing Agent State DB integration...");
    
    // Test 1: Create database
    println!("1. Creating database...");
    let db_path = CString::new("test_integration.lance").unwrap();
    let db = unsafe { agent_db_new(db_path.as_ptr()) };
    assert!(!db.is_null(), "Failed to create database");
    println!("   SUCCESS: Database created");
    
    // Test 2: Save agent state
    println!("2. Saving agent state...");
    let agent_id = 12345u64;
    let session_id = 67890u64;
    let state_type = 1i32; // working_memory
    let test_data = b"Hello from Rust integration test!";
    
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
    assert_eq!(result, 0, "Failed to save state");
    println!("   SUCCESS: Agent state saved");
    
    // Test 3: Load agent state
    println!("3. Loading agent state...");
    let mut loaded_data: *mut u8 = ptr::null_mut();
    let mut loaded_data_len: usize = 0;
    
    let load_result = unsafe {
        agent_db_load_state(db, agent_id, &mut loaded_data, &mut loaded_data_len)
    };
    assert_eq!(load_result, 0, "Failed to load state");
    assert!(!loaded_data.is_null(), "No data loaded");
    assert!(loaded_data_len > 0, "No data length");
    
    // Verify data
    let loaded_slice = unsafe { std::slice::from_raw_parts(loaded_data, loaded_data_len) };
    assert_eq!(loaded_slice, test_data, "Data mismatch");
    println!("   SUCCESS: Data loaded correctly: {:?}", std::str::from_utf8(loaded_slice).unwrap());
    
    // Clean up
    unsafe {
        agent_db_free_data(loaded_data, loaded_data_len);
        agent_db_free(db);
    }
    
    println!("\nAll tests passed! ✅");
}

fn main() {
    test_agent_state_db_integration();
}
