#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Testing Agent State DB Rust Library...\n");
    
    // Test 1: Create database
    printf("1. Creating database...\n");
    struct CAgentStateDB *db = agent_db_new("test_db.lance");
    if (db == NULL) {
        printf("   FAILED: Could not create database\n");
        return 1;
    }
    printf("   SUCCESS: Database created\n");
    
    // Test 2: Save agent state
    printf("2. Saving agent state...\n");
    uint64_t agent_id = 12345;
    uint64_t session_id = 67890;
    int state_type = 1; // working_memory
    const char* test_data = "Hello, Agent State!";
    uint8_t* data = (uint8_t*)test_data;
    size_t data_len = strlen(test_data);
    
    int result = agent_db_save_state(db, agent_id, session_id, state_type, data, data_len);
    if (result != 0) {
        printf("   FAILED: Could not save state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Agent state saved\n");
    
    // Test 3: Load agent state
    printf("3. Loading agent state...\n");
    uint8_t* loaded_data = NULL;
    size_t loaded_data_len = 0;
    
    result = agent_db_load_state(db, agent_id, &loaded_data, &loaded_data_len);
    if (result != 0) {
        printf("   FAILED: Could not load state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    
    if (loaded_data == NULL || loaded_data_len == 0) {
        printf("   FAILED: No data loaded\n");
        agent_db_free(db);
        return 1;
    }
    
    // Verify data
    if (loaded_data_len == data_len && memcmp(loaded_data, data, data_len) == 0) {
        printf("   SUCCESS: Data loaded correctly: %.*s\n", (int)loaded_data_len, loaded_data);
    } else {
        printf("   FAILED: Data mismatch\n");
        printf("   Expected: %.*s (len=%lu)\n", (int)data_len, data, (unsigned long)data_len);
        printf("   Got: %.*s (len=%lu)\n", (int)loaded_data_len, loaded_data, (unsigned long)loaded_data_len);
        agent_db_free_data(loaded_data, loaded_data_len);
        agent_db_free(db);
        return 1;
    }
    
    // Clean up
    agent_db_free_data(loaded_data, loaded_data_len);
    agent_db_free(db);
    
    printf("\nAll tests passed! ✅\n");
    return 0;
}
