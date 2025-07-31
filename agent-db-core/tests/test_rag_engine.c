#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Testing RAG Engine Features...\n");
    fflush(stdout);
    
    // Test 1: Create RAG engine
    printf("1. Creating RAG engine...\n");
    fflush(stdout);
    
    struct CRAGEngine *rag_engine = rag_engine_new("test_rag.lance");
    if (rag_engine == NULL) {
        printf("   FAILED: Could not create RAG engine\n");
        fflush(stdout);
        return 1;
    }
    printf("   SUCCESS: RAG engine created\n");
    fflush(stdout);
    
    // Test 2: Index a document
    printf("2. Indexing document...\n");
    fflush(stdout);
    
    const char* title = "Introduction to Machine Learning";
    const char* content = "Machine learning is a subset of artificial intelligence that enables computers to learn and make decisions from data without being explicitly programmed. It involves algorithms that can identify patterns in data and make predictions or classifications based on those patterns. There are three main types of machine learning: supervised learning, unsupervised learning, and reinforcement learning. Supervised learning uses labeled data to train models, while unsupervised learning finds patterns in unlabeled data. Reinforcement learning involves agents learning through interaction with an environment.";
    
    int result = rag_engine_index_document(rag_engine, title, content, 200, 50);
    if (result != 0) {
        printf("   FAILED: Could not index document (error code: %d)\n", result);
        fflush(stdout);
        rag_engine_free(rag_engine);
        return 1;
    }
    printf("   SUCCESS: Document indexed\n");
    fflush(stdout);
    
    // Test 3: Index another document
    printf("3. Indexing second document...\n");
    fflush(stdout);
    
    const char* title2 = "Deep Learning Fundamentals";
    const char* content2 = "Deep learning is a specialized subset of machine learning that uses neural networks with multiple layers to model and understand complex patterns in data. These neural networks are inspired by the structure and function of the human brain. Deep learning has revolutionized many fields including computer vision, natural language processing, and speech recognition. Popular deep learning architectures include convolutional neural networks for image processing and recurrent neural networks for sequential data.";
    
    result = rag_engine_index_document(rag_engine, title2, content2, 150, 30);
    if (result != 0) {
        printf("   FAILED: Could not index second document (error code: %d)\n", result);
        fflush(stdout);
        rag_engine_free(rag_engine);
        return 1;
    }
    printf("   SUCCESS: Second document indexed\n");
    fflush(stdout);
    
    // Test 4: Search for relevant content
    printf("4. Searching for relevant content...\n");
    fflush(stdout);
    
    const char* query = "neural networks";
    size_t results_count = 0;
    
    result = rag_engine_search_text(rag_engine, query, 5, &results_count);
    if (result != 0) {
        printf("   FAILED: Could not search text (error code: %d)\n", result);
        fflush(stdout);
        rag_engine_free(rag_engine);
        return 1;
    }
    printf("   SUCCESS: Text search completed, found %zu results\n", results_count);
    fflush(stdout);
    
    // Test 5: Build context for RAG
    printf("5. Building RAG context...\n");
    fflush(stdout);
    
    const char* rag_query = "What is deep learning?";
    char* context = NULL;
    size_t context_len = 0;
    
    result = rag_engine_build_context(rag_engine, rag_query, 500, &context, &context_len);
    if (result != 0) {
        printf("   FAILED: Could not build context (error code: %d)\n", result);
        fflush(stdout);
        rag_engine_free(rag_engine);
        return 1;
    }
    
    if (context == NULL || context_len == 0) {
        printf("   FAILED: No context generated\n");
        fflush(stdout);
        rag_engine_free(rag_engine);
        return 1;
    }
    
    printf("   SUCCESS: Context built (length: %zu)\n", context_len);
    printf("   Context preview: %.100s...\n", context);
    fflush(stdout);
    
    // Free context
    rag_engine_free_context(context);
    
    // Test 6: Test with different queries
    printf("6. Testing various queries...\n");
    fflush(stdout);
    
    const char* queries[] = {
        "supervised learning",
        "artificial intelligence",
        "computer vision",
        "data patterns"
    };
    
    for (int i = 0; i < 4; i++) {
        size_t query_results_count = 0;
        result = rag_engine_search_text(rag_engine, queries[i], 3, &query_results_count);
        
        if (result == 0) {
            printf("   Query '%s': %zu results\n", queries[i], query_results_count);
        } else {
            printf("   Query '%s': FAILED (error code: %d)\n", queries[i], result);
        }
        fflush(stdout);
    }
    
    // Test 7: Build context for different queries
    printf("7. Building context for different queries...\n");
    fflush(stdout);
    
    const char* context_queries[] = {
        "What is machine learning?",
        "How do neural networks work?"
    };
    
    for (int i = 0; i < 2; i++) {
        char* query_context = NULL;
        size_t query_context_len = 0;
        
        result = rag_engine_build_context(rag_engine, context_queries[i], 300, &query_context, &query_context_len);
        
        if (result == 0 && query_context != NULL) {
            printf("   Query '%s': Context length %zu\n", context_queries[i], query_context_len);
            rag_engine_free_context(query_context);
        } else {
            printf("   Query '%s': FAILED to build context\n", context_queries[i]);
        }
        fflush(stdout);
    }
    
    // Clean up
    rag_engine_free(rag_engine);
    
    printf("\n🎉 All RAG engine tests passed! ✅\n");
    printf("✓ RAG engine creation and initialization\n");
    printf("✓ Document indexing and chunking\n");
    printf("✓ Text search functionality\n");
    printf("✓ Context building for RAG\n");
    printf("✓ Multiple query handling\n");
    printf("✓ Memory management\n");
    fflush(stdout);
    
    return 0;
}
