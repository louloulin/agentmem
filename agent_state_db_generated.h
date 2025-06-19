#ifndef AGENT_STATE_DB_H
#define AGENT_STATE_DB_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "stdint.h"
#include "stddef.h"

typedef struct AgentStateDB AgentStateDB;

typedef struct MemoryManager MemoryManager;

typedef struct CAgentStateDB {
  struct AgentStateDB *db;
} CAgentStateDB;

typedef struct CMemoryManager {
  struct MemoryManager *mgr;
} CMemoryManager;

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

struct CMemoryManager *memory_manager_new(const char *db_path);

void memory_manager_free(struct CMemoryManager *mgr);

int memory_manager_store_memory(struct CMemoryManager *mgr,
                                uint64_t agent_id,
                                int memory_type,
                                const char *content,
                                double importance);

int memory_manager_get_memories(struct CMemoryManager *mgr,
                                uint64_t agent_id,
                                uintptr_t *memory_count_out);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* AGENT_STATE_DB_H */
