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
    fn agent_db_save_vector_state(
        db: *mut std::ffi::c_void,
        agent_id: u64,
        session_id: u64,
        state_type: i32,
        data: *const u8,
        data_len: usize,
        embedding: *const f32,
        embedding_len: usize,
    ) -> i32;
    fn memory_manager_new(path: *const i8) -> *mut std::ffi::c_void;
    fn memory_manager_free(mgr: *mut std::ffi::c_void);
    fn memory_manager_store_memory(
        mgr: *mut std::ffi::c_void,
        agent_id: u64,
        memory_type: i32,
        content: *const i8,
        importance: f32,
    ) -> i32;
}

fn main() {
    println!("Testing New Features (Rust Internal)...");
    
    // Test 1: Create database
    println!("1. Creating database...");
    let db_path = CString::new("test_new_features_rust.lance").unwrap();
    let db = unsafe { agent_db_new(db_path.as_ptr()) };
    if db.is_null() {
        println!("   FAILED: Could not create database");
        return;
    }
    println!("   SUCCESS: Database created");
    
    // Test 2: Basic state operations
    println!("2. Testing basic state operations...");
    let agent_id = 12345u64;
    let session_id = 67890u64;
    let state_type = 0i32; // WorkingMemory
    let test_data = b"Basic test data";
    
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
    println!("   SUCCESS: State saved");
    
    // Load state
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
    
    let loaded_slice = unsafe { std::slice::from_raw_parts(loaded_data, loaded_data_len) };
    if loaded_slice == test_data {
        println!("   SUCCESS: State loaded and verified");
    } else {
        println!("   FAILED: Data mismatch");
        unsafe {
            agent_db_free_data(loaded_data, loaded_data_len);
            agent_db_free(db);
        }
        return;
    }
    
    unsafe { agent_db_free_data(loaded_data, loaded_data_len); }
    
    // Test 3: Vector state save
    println!("3. Testing vector state save...");
    let test_vector: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    
    let vector_result = unsafe {
        agent_db_save_vector_state(
            db,
            agent_id + 1,
            session_id,
            5, // Embedding type
            test_data.as_ptr(),
            test_data.len(),
            test_vector.as_ptr(),
            test_vector.len(),
        )
    };
    if vector_result != 0 {
        println!("   FAILED: Could not save vector state (error code: {})", vector_result);
        unsafe { agent_db_free(db); }
        return;
    }
    println!("   SUCCESS: Vector state saved");
    
    // Test 4: Memory manager
    println!("4. Testing memory manager...");
    let memory_path = CString::new("test_memory_rust.lance").unwrap();
    let memory_mgr = unsafe { memory_manager_new(memory_path.as_ptr()) };
    if memory_mgr.is_null() {
        println!("   FAILED: Could not create memory manager");
        unsafe { agent_db_free(db); }
        return;
    }
    println!("   SUCCESS: Memory manager created");
    
    // Test 5: Store memory
    println!("5. Testing memory storage...");
    let memory_content = CString::new("Simple test memory").unwrap();
    let memory_result = unsafe {
        memory_manager_store_memory(
            memory_mgr,
            agent_id,
            0, // Episodic
            memory_content.as_ptr(),
            0.8,
        )
    };
    if memory_result != 0 {
        println!("   FAILED: Could not store memory (error code: {})", memory_result);
        unsafe {
            memory_manager_free(memory_mgr);
            agent_db_free(db);
        }
        return;
    }
    println!("   SUCCESS: Memory stored");
    
    // Clean up
    unsafe {
        memory_manager_free(memory_mgr);
        agent_db_free(db);
    }
    
    println!("\n✅ All new feature tests passed!");
    println!("✓ Basic database operations");
    println!("✓ Vector state storage");
    println!("✓ Memory manager functionality");
    println!("✓ Memory storage operations");
}
