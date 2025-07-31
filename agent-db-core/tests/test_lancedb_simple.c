#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Testing LanceDB Simple Integration...\n");
    fflush(stdout);
    
    // Test 1: Create database
    printf("1. Creating database...\n");
    fflush(stdout);
    
    struct CAgentStateDB *db = agent_db_new("simple_test.lance");
    
    printf("Database creation returned: %p\n", (void*)db);
    fflush(stdout);
    
    if (db == NULL) {
        printf("Database creation failed\n");
        fflush(stdout);
        return 1;
    }
    
    printf("Database created successfully\n");
    fflush(stdout);
    
    // Test 2: Save a simple state
    printf("2. Saving simple state...\n");
    fflush(stdout);
    
    uint64_t agent_id = 12345;
    uint64_t session_id = 67890;
    int state_type = 0; // WorkingMemory
    const char* test_data = "Hello LanceDB!";
    uint8_t* data = (uint8_t*)test_data;
    size_t data_len = strlen(test_data);
    
    int result = agent_db_save_state(db, agent_id, session_id, state_type, data, data_len);
    printf("Save result: %d\n", result);
    fflush(stdout);
    
    if (result != 0) {
        printf("Save failed with error code: %d\n", result);
        fflush(stdout);
        agent_db_free(db);
        return 1;
    }
    
    printf("State saved successfully\n");
    fflush(stdout);
    
    // Test 3: Load the state
    printf("3. Loading state...\n");
    fflush(stdout);
    
    uint8_t* loaded_data = NULL;
    size_t loaded_data_len = 0;
    
    result = agent_db_load_state(db, agent_id, &loaded_data, &loaded_data_len);
    printf("Load result: %d\n", result);
    fflush(stdout);
    
    if (result != 0) {
        printf("Load failed with error code: %d\n", result);
        fflush(stdout);
        agent_db_free(db);
        return 1;
    }
    
    if (loaded_data == NULL || loaded_data_len == 0) {
        printf("No data loaded\n");
        fflush(stdout);
        agent_db_free(db);
        return 1;
    }
    
    printf("Data loaded: %.*s (length: %zu)\n", (int)loaded_data_len, loaded_data, loaded_data_len);
    fflush(stdout);
    
    // Verify data
    if (loaded_data_len == data_len && memcmp(loaded_data, data, data_len) == 0) {
        printf("Data verification successful!\n");
        fflush(stdout);
    } else {
        printf("Data verification failed!\n");
        fflush(stdout);
        agent_db_free_data(loaded_data, loaded_data_len);
        agent_db_free(db);
        return 1;
    }
    
    // Clean up
    agent_db_free_data(loaded_data, loaded_data_len);
    agent_db_free(db);
    
    printf("All tests passed! ✅\n");
    fflush(stdout);
    
    return 0;
}
