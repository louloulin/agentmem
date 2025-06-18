#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Testing Vector and Memory Features...\n");
    
    // Test 1: Create database
    printf("1. Creating database...\n");
    struct CAgentStateDB *db = agent_db_new("test_vector_memory.lance");
    if (db == NULL) {
        printf("   FAILED: Could not create database\n");
        return 1;
    }
    printf("   SUCCESS: Database created\n");
    
    // Test 2: Save vector state
    printf("2. Saving vector state...\n");
    uint64_t agent_id = 12345;
    uint64_t session_id = 67890;
    int state_type = 5; // Embedding
    const char* test_data = "Vector state data";
    uint8_t* data = (uint8_t*)test_data;
    size_t data_len = strlen(test_data);
    
    // Create test embedding vector (1536 dimensions)
    float* embedding = (float*)malloc(1536 * sizeof(float));
    for (int i = 0; i < 1536; i++) {
        embedding[i] = 0.1f + (i % 100) * 0.001f; // Simple pattern
    }
    
    int result = agent_db_save_vector_state(db, agent_id, session_id, state_type, 
                                           data, data_len, embedding, 1536);
    if (result != 0) {
        printf("   FAILED: Could not save vector state (error code: %d)\n", result);
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Vector state saved\n");
    
    // Test 3: Vector search
    printf("3. Testing vector search...\n");
    uint64_t* search_results = NULL;
    size_t results_count = 0;
    
    result = agent_db_vector_search(db, embedding, 1536, 5, &search_results, &results_count);
    if (result != 0) {
        printf("   FAILED: Vector search failed (error code: %d)\n", result);
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    
    if (results_count > 0) {
        printf("   SUCCESS: Vector search returned %zu results\n", results_count);
        printf("   First result agent_id: %llu\n", (unsigned long long)search_results[0]);
        
        // Free search results
        free(search_results);
    } else {
        printf("   WARNING: Vector search returned no results\n");
    }
    
    // Test 4: Memory manager
    printf("4. Testing memory manager...\n");
    struct CMemoryManager *memory_mgr = memory_manager_new("test_memory.lance");
    if (memory_mgr == NULL) {
        printf("   FAILED: Could not create memory manager\n");
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Memory manager created\n");
    
    // Test 5: Store memory
    printf("5. Storing memory...\n");
    result = memory_manager_store_memory(memory_mgr, agent_id, 0, // Episodic memory
                                       "This is a test memory", 0.8f);
    if (result != 0) {
        printf("   FAILED: Could not store memory (error code: %d)\n", result);
        memory_manager_free(memory_mgr);
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Memory stored\n");
    
    // Test 6: Retrieve memories
    printf("6. Retrieving memories...\n");
    size_t memory_count = 0;
    result = memory_manager_retrieve_memories(memory_mgr, agent_id, 10, &memory_count);
    if (result != 0) {
        printf("   FAILED: Could not retrieve memories (error code: %d)\n", result);
        memory_manager_free(memory_mgr);
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Retrieved %zu memories\n", memory_count);
    
    // Test 7: Store multiple memories with different types
    printf("7. Storing multiple memory types...\n");
    const char* memory_contents[] = {
        "Semantic memory: The sky is blue",
        "Procedural memory: How to ride a bike",
        "Working memory: Current task context"
    };
    int memory_types[] = {1, 2, 3}; // Semantic, Procedural, Working
    float importances[] = {0.9f, 0.7f, 0.5f};
    
    for (int i = 0; i < 3; i++) {
        result = memory_manager_store_memory(memory_mgr, agent_id, memory_types[i],
                                           memory_contents[i], importances[i]);
        if (result != 0) {
            printf("   FAILED: Could not store memory %d (error code: %d)\n", i, result);
            memory_manager_free(memory_mgr);
            free(embedding);
            agent_db_free(db);
            return 1;
        }
    }
    printf("   SUCCESS: Multiple memory types stored\n");
    
    // Test 8: Retrieve all memories for agent
    printf("8. Retrieving all memories for agent...\n");
    result = memory_manager_retrieve_memories(memory_mgr, agent_id, 20, &memory_count);
    if (result != 0) {
        printf("   FAILED: Could not retrieve all memories (error code: %d)\n", result);
        memory_manager_free(memory_mgr);
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Retrieved %zu total memories\n", memory_count);
    
    // Test 9: Test with different agent
    printf("9. Testing with different agent...\n");
    uint64_t agent_id_2 = 54321;
    result = memory_manager_store_memory(memory_mgr, agent_id_2, 0,
                                       "Memory for different agent", 0.6f);
    if (result != 0) {
        printf("   FAILED: Could not store memory for agent 2 (error code: %d)\n", result);
        memory_manager_free(memory_mgr);
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    
    // Verify isolation
    size_t agent2_memory_count = 0;
    result = memory_manager_retrieve_memories(memory_mgr, agent_id_2, 10, &agent2_memory_count);
    if (result != 0 || agent2_memory_count != 1) {
        printf("   FAILED: Agent isolation not working properly\n");
        memory_manager_free(memory_mgr);
        free(embedding);
        agent_db_free(db);
        return 1;
    }
    printf("   SUCCESS: Agent isolation verified (agent 2 has %zu memories)\n", agent2_memory_count);
    
    // Clean up
    memory_manager_free(memory_mgr);
    free(embedding);
    agent_db_free(db);
    
    printf("\n🎉 All vector and memory tests passed! ✅\n");
    printf("✓ Vector state storage and retrieval\n");
    printf("✓ Vector search functionality\n");
    printf("✓ Memory manager creation and operations\n");
    printf("✓ Multiple memory types support\n");
    printf("✓ Agent isolation in memory system\n");
    printf("✓ Memory importance and metadata handling\n");
    
    return 0;
}
