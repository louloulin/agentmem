#ifndef AGENT_STATE_DB_H
#define AGENT_STATE_DB_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "stdint.h"
#include "stddef.h"

typedef struct AgentNetworkManager AgentNetworkManager;

typedef struct AgentStateDB AgentStateDB;

typedef struct IntelligentMemoryOrganizer IntelligentMemoryOrganizer;

typedef struct MemoryManager MemoryManager;

typedef struct RAGEngine RAGEngine;

typedef struct CAgentStateDB {
  struct AgentStateDB *db;
} CAgentStateDB;

typedef struct CMemoryManager {
  struct MemoryManager *mgr;
} CMemoryManager;

typedef struct CRAGEngine {
  struct RAGEngine *engine;
} CRAGEngine;

typedef struct CIntelligentMemoryOrganizer {
  struct IntelligentMemoryOrganizer *organizer;
} CIntelligentMemoryOrganizer;

typedef struct CMemoryCluster {
  char *cluster_id;
  uintptr_t memory_count;
  float importance_score;
  int64_t created_at;
} CMemoryCluster;

typedef struct CMemoryArchive {
  char *archive_id;
  uintptr_t original_count;
  float compression_ratio;
  int64_t archived_at;
  char *summary;
} CMemoryArchive;

typedef struct CAgentNetworkManager {
  struct AgentNetworkManager *manager;
} CAgentNetworkManager;

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
                           uintptr_t limit,
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

struct CIntelligentMemoryOrganizer *memory_organizer_new(const char *db_path);

void memory_organizer_free(struct CIntelligentMemoryOrganizer *organizer);

int memory_organizer_evaluate_importance(struct CIntelligentMemoryOrganizer *organizer,
                                         const char *memory_id,
                                         uint64_t agent_id,
                                         float *importance_out);

int memory_organizer_cluster_memories(struct CIntelligentMemoryOrganizer *organizer,
                                      uint64_t agent_id,
                                      struct CMemoryCluster **clusters_out,
                                      uintptr_t *cluster_count_out);

int memory_organizer_archive_old_memories(struct CIntelligentMemoryOrganizer *organizer,
                                          uint64_t agent_id,
                                          struct CMemoryArchive **archives_out,
                                          uintptr_t *archive_count_out);

void memory_organizer_free_clusters(struct CMemoryCluster *clusters, uintptr_t count);

void memory_organizer_free_archives(struct CMemoryArchive *archives, uintptr_t count);

struct CAgentNetworkManager *agent_network_manager_new(uint64_t agent_id,
                                                       const char *address,
                                                       uint16_t port,
                                                       const char *const *capabilities,
                                                       uintptr_t capabilities_count);

void agent_network_manager_free(struct CAgentNetworkManager *manager);

int agent_network_join_network(struct CAgentNetworkManager *manager,
                               const char *const *bootstrap_nodes,
                               uintptr_t bootstrap_count);

int agent_network_send_message(struct CAgentNetworkManager *manager,
                               uint64_t from_agent,
                               uint64_t to_agent,
                               int message_type,
                               const uint8_t *payload,
                               uintptr_t payload_len);

int agent_network_broadcast_message(struct CAgentNetworkManager *manager,
                                    const uint8_t *payload,
                                    uintptr_t payload_len);

int agent_network_leave_network(struct CAgentNetworkManager *manager);

int agent_network_get_active_nodes_count(struct CAgentNetworkManager *manager);

int agent_network_find_nodes_by_capability(struct CAgentNetworkManager *manager,
                                           const char *capability,
                                           uint64_t **nodes_out,
                                           uintptr_t *nodes_count_out);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* AGENT_STATE_DB_H */
