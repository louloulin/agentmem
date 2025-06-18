#ifndef AGENT_STATE_DB_H
#define AGENT_STATE_DB_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// 前向声明
typedef struct CAgentStateDB CAgentStateDB;
typedef struct CMemoryManager CMemoryManager;

// 错误码
#define AGENT_DB_SUCCESS 0
#define AGENT_DB_ERROR -1
#define AGENT_DB_NOT_FOUND 1

// 状态类型
#define STATE_TYPE_WORKING_MEMORY 0
#define STATE_TYPE_LONG_TERM_MEMORY 1
#define STATE_TYPE_CONTEXT 2
#define STATE_TYPE_TASK_STATE 3
#define STATE_TYPE_RELATIONSHIP 4
#define STATE_TYPE_EMBEDDING 5

// 记忆类型
#define MEMORY_TYPE_EPISODIC 0
#define MEMORY_TYPE_SEMANTIC 1
#define MEMORY_TYPE_PROCEDURAL 2
#define MEMORY_TYPE_WORKING 3

// Agent状态数据库接口
CAgentStateDB* agent_db_new(const char* db_path);
void agent_db_free(CAgentStateDB* db);

int agent_db_save_state(
    CAgentStateDB* db,
    uint64_t agent_id,
    uint64_t session_id,
    int state_type,
    const uint8_t* data,
    size_t data_len
);

int agent_db_load_state(
    CAgentStateDB* db,
    uint64_t agent_id,
    uint8_t** data_out,
    size_t* data_len_out
);

void agent_db_free_data(uint8_t* data, size_t data_len);

// 向量功能接口
int agent_db_save_vector_state(
    CAgentStateDB* db,
    uint64_t agent_id,
    uint64_t session_id,
    int state_type,
    const uint8_t* data,
    size_t data_len,
    const float* embedding,
    size_t embedding_len
);

int agent_db_vector_search(
    CAgentStateDB* db,
    const float* query_embedding,
    size_t embedding_len,
    size_t limit,
    uint64_t** results_out,
    size_t* results_count_out
);

// 记忆管理接口
CMemoryManager* memory_manager_new(const char* db_path);
void memory_manager_free(CMemoryManager* mgr);

int memory_manager_store_memory(
    CMemoryManager* mgr,
    uint64_t agent_id,
    int memory_type,
    const char* content,
    float importance
);

int memory_manager_retrieve_memories(
    CMemoryManager* mgr,
    uint64_t agent_id,
    size_t limit,
    size_t* memory_count_out
);

#ifdef __cplusplus
}
#endif

#endif // AGENT_STATE_DB_H
