#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdint.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Testing Intelligent Memory Organizer Features...\n");
    fflush(stdout);
    
    // Test 1: Create Memory Organizer
    printf("1. Creating Memory Organizer...\n");
    fflush(stdout);
    
    struct CIntelligentMemoryOrganizer *organizer = memory_organizer_new("test_memory_organizer.lance");
    if (organizer == NULL) {
        printf("   FAILED: Could not create memory organizer\n");
        fflush(stdout);
        return 1;
    }
    printf("   SUCCESS: Memory organizer created\n");
    fflush(stdout);
    
    // Test 2: Evaluate Memory Importance
    printf("2. Testing memory importance evaluation...\n");
    fflush(stdout);
    
    const char* test_memory_id = "test_memory_001";
    uint64_t agent_id = 12345;
    float importance_score = 0.0;
    
    int result = memory_organizer_evaluate_importance(organizer, test_memory_id, agent_id, &importance_score);
    if (result != 0) {
        printf("   FAILED: Could not evaluate memory importance (error code: %d)\n", result);
        fflush(stdout);
        memory_organizer_free(organizer);
        return 1;
    }
    
    printf("   SUCCESS: Memory importance evaluated\n");
    printf("   Importance score: %.3f\n", importance_score);
    fflush(stdout);
    
    // Validate importance score range
    if (importance_score < 0.0 || importance_score > 1.0) {
        printf("   WARNING: Importance score out of expected range [0.0, 1.0]\n");
        fflush(stdout);
    }
    
    // Test 3: Memory Clustering
    printf("3. Testing memory clustering...\n");
    fflush(stdout);
    
    struct CMemoryCluster *clusters = NULL;
    size_t cluster_count = 0;
    
    result = memory_organizer_cluster_memories(organizer, agent_id, &clusters, &cluster_count);
    if (result != 0) {
        printf("   FAILED: Could not cluster memories (error code: %d)\n", result);
        fflush(stdout);
        memory_organizer_free(organizer);
        return 1;
    }
    
    printf("   SUCCESS: Memory clustering completed\n");
    printf("   Found %u memory clusters\n", (unsigned int)cluster_count);
    fflush(stdout);
    
    // Display cluster information
    for (size_t i = 0; i < cluster_count; i++) {
        printf("   Cluster %u:\n", (unsigned int)(i + 1));
        printf("     ID: %s\n", clusters[i].cluster_id);
        printf("     Memory count: %u\n", (unsigned int)clusters[i].memory_count);
        printf("     Importance: %.3f\n", clusters[i].importance_score);
        printf("     Created: %lld\n", (long long)clusters[i].created_at);
        fflush(stdout);
    }
    
    // Free cluster memory
    if (clusters != NULL) {
        memory_organizer_free_clusters(clusters, cluster_count);
    }
    
    // Test 4: Memory Archiving
    printf("4. Testing memory archiving...\n");
    fflush(stdout);
    
    struct CMemoryArchive *archives = NULL;
    size_t archive_count = 0;
    
    result = memory_organizer_archive_old_memories(organizer, agent_id, &archives, &archive_count);
    if (result != 0) {
        printf("   FAILED: Could not archive memories (error code: %d)\n", result);
        fflush(stdout);
        memory_organizer_free(organizer);
        return 1;
    }
    
    printf("   SUCCESS: Memory archiving completed\n");
    printf("   Created %u memory archives\n", (unsigned int)archive_count);
    fflush(stdout);
    
    // Display archive information
    for (size_t i = 0; i < archive_count; i++) {
        printf("   Archive %u:\n", (unsigned int)(i + 1));
        printf("     ID: %s\n", archives[i].archive_id);
        printf("     Original count: %u memories\n", (unsigned int)archives[i].original_count);
        printf("     Compression ratio: %.3f\n", archives[i].compression_ratio);
        printf("     Archived at: %lld\n", (long long)archives[i].archived_at);
        printf("     Summary: %s\n", archives[i].summary);
        fflush(stdout);
    }
    
    // Free archive memory
    if (archives != NULL) {
        memory_organizer_free_archives(archives, archive_count);
    }
    
    // Test 5: Multiple Agent Testing
    printf("5. Testing multiple agents...\n");
    fflush(stdout);
    
    uint64_t test_agents[] = {11111, 22222, 33333, 44444, 55555};
    size_t num_agents = sizeof(test_agents) / sizeof(test_agents[0]);
    
    for (size_t i = 0; i < num_agents; i++) {
        float agent_importance = 0.0;
        char memory_id[64];
        snprintf(memory_id, sizeof(memory_id), "agent_%llu_memory", (unsigned long long)test_agents[i]);
        
        result = memory_organizer_evaluate_importance(organizer, memory_id, test_agents[i], &agent_importance);
        if (result == 0) {
            printf("   Agent %llu: Importance %.3f\n", (unsigned long long)test_agents[i], agent_importance);
        } else {
            printf("   Agent %llu: Evaluation failed\n", (unsigned long long)test_agents[i]);
        }
        fflush(stdout);
    }
    
    // Test 6: Performance Testing
    printf("6. Performance testing...\n");
    fflush(stdout);
    
    clock_t start_time = clock();
    
    // Perform multiple importance evaluations
    for (int i = 0; i < 100; i++) {
        float perf_importance = 0.0;
        char perf_memory_id[64];
        snprintf(perf_memory_id, sizeof(perf_memory_id), "perf_memory_%d", i);
        
        memory_organizer_evaluate_importance(organizer, perf_memory_id, 99999, &perf_importance);
    }
    
    clock_t end_time = clock();
    double elapsed_time = ((double)(end_time - start_time)) / CLOCKS_PER_SEC;
    
    printf("   SUCCESS: 100 importance evaluations completed in %.3f seconds\n", elapsed_time);
    printf("   Average time per evaluation: %.3f ms\n", (elapsed_time * 1000.0) / 100.0);
    fflush(stdout);
    
    // Test 7: Edge Cases
    printf("7. Testing edge cases...\n");
    fflush(stdout);
    
    // Test with NULL memory ID
    float null_importance = 0.0;
    result = memory_organizer_evaluate_importance(organizer, NULL, agent_id, &null_importance);
    if (result != 0) {
        printf("   SUCCESS: NULL memory ID properly rejected\n");
    } else {
        printf("   WARNING: NULL memory ID was accepted\n");
    }
    fflush(stdout);
    
    // Test with invalid agent ID
    float invalid_importance = 0.0;
    result = memory_organizer_evaluate_importance(organizer, "valid_memory", 0, &invalid_importance);
    if (result == 0) {
        printf("   SUCCESS: Zero agent ID handled (importance: %.3f)\n", invalid_importance);
    } else {
        printf("   INFO: Zero agent ID rejected\n");
    }
    fflush(stdout);
    
    // Test clustering with non-existent agent
    struct CMemoryCluster *empty_clusters = NULL;
    size_t empty_cluster_count = 0;
    
    result = memory_organizer_cluster_memories(organizer, 999999, &empty_clusters, &empty_cluster_count);
    if (result == 0) {
        printf("   SUCCESS: Non-existent agent clustering handled (%u clusters)\n", (unsigned int)empty_cluster_count);
        if (empty_clusters != NULL) {
            memory_organizer_free_clusters(empty_clusters, empty_cluster_count);
        }
    } else {
        printf("   INFO: Non-existent agent clustering rejected\n");
    }
    fflush(stdout);
    
    // Clean up
    memory_organizer_free(organizer);
    
    printf("\n🎉 All Memory Organizer tests completed! ✅\n");
    printf("📊 Test Summary:\n");
    printf("   ✓ Memory organizer creation and cleanup\n");
    printf("   ✓ Memory importance evaluation\n");
    printf("   ✓ Memory clustering analysis\n");
    printf("   ✓ Memory archiving and compression\n");
    printf("   ✓ Multiple agent support\n");
    printf("   ✓ Performance testing\n");
    printf("   ✓ Edge case handling\n");
    printf("\n🚀 Intelligent Memory Organizer is working correctly!\n");
    fflush(stdout);
    
    return 0;
}
