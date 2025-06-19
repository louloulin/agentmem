#ifndef AGENT_STATE_DB_H
#define AGENT_STATE_DB_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "stdint.h"
#include "stddef.h"

/**
 * Agent数据库的主要API接口
 */
typedef struct AgentDB AgentDB;

typedef struct CAgentStateDB {
  struct AgentDB *db;
  uint8_t *rt;
} CAgentStateDB;

typedef struct CMemoryManager {
  struct AgentDB *db;
  uint8_t *rt;
} CMemoryManager;

typedef struct CRAGEngine {
  struct AgentDB *db;
  uint8_t *rt;
} CRAGEngine;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct CAgentStateDB *agent_db_new(const char *db_path);

void agent_db_free(struct CAgentStateDB *db);

int agent_db_save_state(struct CAgentStateDB *db,
                        uint64_t agent_id,
                        uint64_t session_id,
                        int state_type,
                        const uint8_t *data,
                        uintptr_t data_len);

int agent_db_load_state(struct CAgentStateDB *db,
                        uint64_t agent_id,
                        uint8_t **data_out,
                        uintptr_t *data_len_out);

void agent_db_free_data(uint8_t *data, uintptr_t data_len);

int agent_db_save_vector_state(struct CAgentStateDB *db,
                               uint64_t agent_id,
                               uint64_t session_id,
                               int state_type,
                               const uint8_t *data,
                               uintptr_t data_len,
                               const float *embedding,
                               uintptr_t embedding_len);

int agent_db_vector_search(struct CAgentStateDB *db,
                           const float *query_embedding,
                           uintptr_t embedding_len,
                           uintptr_t _limit,
                           uint64_t **results_out,
                           uintptr_t *results_count_out);

struct CMemoryManager *memory_manager_new(const char *db_path);

void memory_manager_free(struct CMemoryManager *mgr);

int memory_manager_store_memory(struct CMemoryManager *mgr,
                                uint64_t agent_id,
                                int memory_type,
                                const char *content,
                                float importance);

int memory_manager_retrieve_memories(struct CMemoryManager *mgr,
                                     uint64_t agent_id,
                                     uintptr_t limit,
                                     uintptr_t *memory_count_out);

struct CRAGEngine *rag_engine_new(const char *db_path);

void rag_engine_free(struct CRAGEngine *engine);

int rag_engine_index_document(struct CRAGEngine *engine,
                              const char *title,
                              const char *content,
                              uintptr_t chunk_size,
                              uintptr_t overlap);

int rag_engine_search_text(struct CRAGEngine *engine,
                           const char *query,
                           uintptr_t limit,
                           uintptr_t *results_count_out);

int rag_engine_build_context(struct CRAGEngine *engine,
                             const char *query,
                             uintptr_t max_tokens,
                             char **context_out,
                             uintptr_t *context_len_out);

void rag_engine_free_context(char *context);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* AGENT_STATE_DB_H */
