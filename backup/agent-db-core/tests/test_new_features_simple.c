#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Testing New Features (Simplified)...\n");
    fflush(stdout);
    
    // Test 1: Basic database creation
    printf("1. Creating database...\n");
    fflush(stdout);
    
    struct CAgentStateDB *db = agent_db_new("test_new_features.lance");
    if (db == NULL) {
        printf("   FAILED: Could not create database\n");
        fflush(stdout);
        return 1;
    }
    printf("   SUCCESS: Database created\n");
    fflush(stdout);
    
    // Test 2: Basic state save/load (existing functionality)
    printf("2. Testing basic state operations...\n");
    fflush(stdout);
    
    uint64_t agent_id = 12345;
    uint64_t session_id = 67890;
    int state_type = 0; // WorkingMemory
    const char* test_data = "Basic test data";
    uint8_t* data = (uint8_t*)test_data;
    size_t data_len = strlen(test_data);
    
    int result = agent_db_save_state(db, agent_id, session_id, state_type, data, data_len);
    if (result != 0) {
        printf("   FAILED: Could not save state (error code: %d)\n", result);
        fflush(stdout);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: State saved\n");
    fflush(stdout);
    
    // Load state
    uint8_t* loaded_data = NULL;
    size_t loaded_data_len = 0;
    result = agent_db_load_state(db, agent_id, &loaded_data, &loaded_data_len);
    if (result != 0) {
        printf("   FAILED: Could not load state (error code: %d)\n", result);
        fflush(stdout);
        agent_db_free(db);
        return 1;
    }
    
    if (loaded_data == NULL || loaded_data_len != data_len || 
        memcmp(loaded_data, data, data_len) != 0) {
        printf("   FAILED: Data mismatch\n");
        fflush(stdout);
        if (loaded_data) agent_db_free_data(loaded_data, loaded_data_len);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: State loaded and verified\n");
    fflush(stdout);
    
    agent_db_free_data(loaded_data, loaded_data_len);
    
    // Test 3: Test vector state save (new functionality)
    printf("3. Testing vector state save...\n");
    fflush(stdout);
    
    // Create a simple test vector
    float test_vector[10] = {0.1f, 0.2f, 0.3f, 0.4f, 0.5f, 0.6f, 0.7f, 0.8f, 0.9f, 1.0f};
    
    result = agent_db_save_vector_state(db, agent_id + 1, session_id, 5, // Embedding type
                                       data, data_len, test_vector, 10);
    if (result != 0) {
        printf("   FAILED: Could not save vector state (error code: %d)\n", result);
        fflush(stdout);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Vector state saved\n");
    fflush(stdout);
    
    // Test 4: Test memory manager creation
    printf("4. Testing memory manager...\n");
    fflush(stdout);
    
    struct CMemoryManager *memory_mgr = memory_manager_new("test_memory_simple.lance");
    if (memory_mgr == NULL) {
        printf("   FAILED: Could not create memory manager\n");
        fflush(stdout);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Memory manager created\n");
    fflush(stdout);
    
    // Test 5: Store a simple memory
    printf("5. Testing memory storage...\n");
    fflush(stdout);
    
    result = memory_manager_store_memory(memory_mgr, agent_id, 0, // Episodic
                                       "Simple test memory", 0.8f);
    if (result != 0) {
        printf("   FAILED: Could not store memory (error code: %d)\n", result);
        fflush(stdout);
        memory_manager_free(memory_mgr);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Memory stored\n");
    fflush(stdout);
    
    // Test 6: Retrieve memories
    printf("6. Testing memory retrieval...\n");
    fflush(stdout);
    
    size_t memory_count = 0;
    result = memory_manager_retrieve_memories(memory_mgr, agent_id, 10, &memory_count);
    if (result != 0) {
        printf("   FAILED: Could not retrieve memories (error code: %d)\n", result);
        fflush(stdout);
        memory_manager_free(memory_mgr);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Memory retrieval completed (count: %d)\n", (int)memory_count);
    fflush(stdout);
    
    // Clean up
    memory_manager_free(memory_mgr);
    agent_db_free(db);
    
    printf("\n✅ All simplified tests passed!\n");
    printf("✓ Basic database operations\n");
    printf("✓ Vector state storage\n");
    printf("✓ Memory manager functionality\n");
    printf("✓ Memory storage and retrieval\n");
    fflush(stdout);
    
    return 0;
}
