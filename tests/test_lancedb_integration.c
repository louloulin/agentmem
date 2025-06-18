#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Testing LanceDB Integration...\n");
    
    // Test 1: Create database with LanceDB backend
    printf("1. Creating LanceDB database...\n");
    struct CAgentStateDB *db = agent_db_new("test_lancedb.lance");
    if (db == NULL) {
        printf("   FAILED: Could not create LanceDB database\n");
        return 1;
    }
    printf("   SUCCESS: LanceDB database created\n");
    
    // Test 2: Save multiple agent states
    printf("2. Saving multiple agent states...\n");
    
    // Agent 1 - Working Memory
    uint64_t agent_id_1 = 12345;
    uint64_t session_id_1 = 67890;
    int state_type_1 = 0; // WorkingMemory
    const char* test_data_1 = "Agent 1 working memory data";
    uint8_t* data_1 = (uint8_t*)test_data_1;
    size_t data_len_1 = strlen(test_data_1);
    
    int result = agent_db_save_state(db, agent_id_1, session_id_1, state_type_1, data_1, data_len_1);
    if (result != 0) {
        printf("   FAILED: Could not save agent 1 state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    
    // Agent 2 - Long Term Memory
    uint64_t agent_id_2 = 54321;
    uint64_t session_id_2 = 98765;
    int state_type_2 = 1; // LongTermMemory
    const char* test_data_2 = "Agent 2 long term memory data";
    uint8_t* data_2 = (uint8_t*)test_data_2;
    size_t data_len_2 = strlen(test_data_2);
    
    result = agent_db_save_state(db, agent_id_2, session_id_2, state_type_2, data_2, data_len_2);
    if (result != 0) {
        printf("   FAILED: Could not save agent 2 state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    
    // Agent 3 - Context
    uint64_t agent_id_3 = 11111;
    uint64_t session_id_3 = 22222;
    int state_type_3 = 2; // Context
    const char* test_data_3 = "Agent 3 context data with special chars: 中文测试 🚀";
    uint8_t* data_3 = (uint8_t*)test_data_3;
    size_t data_len_3 = strlen(test_data_3);
    
    result = agent_db_save_state(db, agent_id_3, session_id_3, state_type_3, data_3, data_len_3);
    if (result != 0) {
        printf("   FAILED: Could not save agent 3 state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    
    printf("   SUCCESS: All agent states saved to LanceDB\n");
    
    // Test 3: Load and verify agent states
    printf("3. Loading and verifying agent states...\n");
    
    // Load Agent 1
    uint8_t* loaded_data_1 = NULL;
    size_t loaded_data_len_1 = 0;
    result = agent_db_load_state(db, agent_id_1, &loaded_data_1, &loaded_data_len_1);
    if (result != 0) {
        printf("   FAILED: Could not load agent 1 state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    
    if (loaded_data_1 == NULL || loaded_data_len_1 != data_len_1 || 
        memcmp(loaded_data_1, data_1, data_len_1) != 0) {
        printf("   FAILED: Agent 1 data mismatch\n");
        if (loaded_data_1) agent_db_free_data(loaded_data_1, loaded_data_len_1);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Agent 1 data verified: %.*s\n", (int)loaded_data_len_1, loaded_data_1);
    agent_db_free_data(loaded_data_1, loaded_data_len_1);
    
    // Load Agent 2
    uint8_t* loaded_data_2 = NULL;
    size_t loaded_data_len_2 = 0;
    result = agent_db_load_state(db, agent_id_2, &loaded_data_2, &loaded_data_len_2);
    if (result != 0) {
        printf("   FAILED: Could not load agent 2 state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    
    if (loaded_data_2 == NULL || loaded_data_len_2 != data_len_2 || 
        memcmp(loaded_data_2, data_2, data_len_2) != 0) {
        printf("   FAILED: Agent 2 data mismatch\n");
        if (loaded_data_2) agent_db_free_data(loaded_data_2, loaded_data_len_2);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Agent 2 data verified: %.*s\n", (int)loaded_data_len_2, loaded_data_2);
    agent_db_free_data(loaded_data_2, loaded_data_len_2);
    
    // Load Agent 3
    uint8_t* loaded_data_3 = NULL;
    size_t loaded_data_len_3 = 0;
    result = agent_db_load_state(db, agent_id_3, &loaded_data_3, &loaded_data_len_3);
    if (result != 0) {
        printf("   FAILED: Could not load agent 3 state (error code: %d)\n", result);
        agent_db_free(db);
        return 1;
    }
    
    if (loaded_data_3 == NULL || loaded_data_len_3 != data_len_3 || 
        memcmp(loaded_data_3, data_3, data_len_3) != 0) {
        printf("   FAILED: Agent 3 data mismatch\n");
        if (loaded_data_3) agent_db_free_data(loaded_data_3, loaded_data_len_3);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Agent 3 data verified: %.*s\n", (int)loaded_data_len_3, loaded_data_3);
    agent_db_free_data(loaded_data_3, loaded_data_len_3);
    
    // Test 4: Test non-existent agent
    printf("4. Testing non-existent agent...\n");
    uint8_t* loaded_data_404 = NULL;
    size_t loaded_data_len_404 = 0;
    result = agent_db_load_state(db, 99999, &loaded_data_404, &loaded_data_len_404);
    if (result == 1) {
        printf("   SUCCESS: Non-existent agent correctly returned 'not found'\n");
    } else {
        printf("   FAILED: Expected 'not found' but got error code: %d\n", result);
        agent_db_free(db);
        return 1;
    }
    
    // Clean up
    agent_db_free(db);
    
    printf("\n🎉 All LanceDB integration tests passed! ✅\n");
    printf("✓ Database creation with LanceDB backend\n");
    printf("✓ Multiple agent state persistence\n");
    printf("✓ Data integrity verification\n");
    printf("✓ Unicode and special character support\n");
    printf("✓ Non-existent record handling\n");
    printf("✓ Memory management\n");
    
    return 0;
}
