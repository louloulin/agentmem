// 模拟LanceDB C API头文件
// 实际项目中需要使用真正的LanceDB C绑定

#ifndef LANCE_MOCK_H
#define LANCE_MOCK_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// 错误码定义
typedef enum {
    LANCE_OK = 0,
    LANCE_ERROR_INVALID_ARGUMENT = -1,
    LANCE_ERROR_IO = -2,
    LANCE_ERROR_NOT_FOUND = -3,
    LANCE_ERROR_ALREADY_EXISTS = -4,
    LANCE_ERROR_INTERNAL = -5
} lance_error_t;

// 不透明指针类型
typedef struct lance_database lance_database_t;
typedef struct lance_table lance_table_t;
typedef struct lance_record lance_record_t;
typedef struct lance_vector_record lance_vector_record_t;
typedef struct lance_search_result lance_search_result_t;

// 数据库操作
lance_database_t* lance_database_open(const char* path);
void lance_database_close(lance_database_t* db);
lance_error_t lance_database_create_table(lance_database_t* db, const char* name, lance_table_t** table);
lance_error_t lance_database_open_table(lance_database_t* db, const char* name, lance_table_t** table);

// 表操作
void lance_table_close(lance_table_t* table);
lance_error_t lance_table_insert(lance_table_t* table, const uint8_t* data, size_t len);
lance_error_t lance_table_search(lance_table_t* table, const char* query, lance_search_result_t** results, size_t* count);
lance_error_t lance_table_vector_search(lance_table_t* table, const float* vector, size_t dim, size_t limit, lance_search_result_t** results, size_t* count);

// 记录操作
lance_record_t* lance_record_create(void);
void lance_record_destroy(lance_record_t* record);
lance_error_t lance_record_set_field_u64(lance_record_t* record, const char* name, uint64_t value);
lance_error_t lance_record_set_field_i64(lance_record_t* record, const char* name, int64_t value);
lance_error_t lance_record_set_field_string(lance_record_t* record, const char* name, const char* value);
lance_error_t lance_record_set_field_binary(lance_record_t* record, const char* name, const uint8_t* data, size_t len);
lance_error_t lance_record_get_field_u64(lance_record_t* record, const char* name, uint64_t* value);
lance_error_t lance_record_get_field_i64(lance_record_t* record, const char* name, int64_t* value);
lance_error_t lance_record_get_field_string(lance_record_t* record, const char* name, const char** value);
lance_error_t lance_record_get_field_binary(lance_record_t* record, const char* name, const uint8_t** data, size_t* len);

// 向量记录操作
lance_vector_record_t* lance_vector_record_create(uint64_t id);
void lance_vector_record_destroy(lance_vector_record_t* record);
lance_error_t lance_vector_record_set_vector(lance_vector_record_t* record, const float* vector, size_t dim);
lance_error_t lance_vector_record_set_metadata(lance_vector_record_t* record, const char* key, const char* value);
lance_error_t lance_vector_record_get_id(lance_vector_record_t* record, uint64_t* id);
lance_error_t lance_vector_record_get_vector(lance_vector_record_t* record, const float** vector, size_t* dim);
lance_error_t lance_vector_record_get_metadata(lance_vector_record_t* record, const char* key, const char** value);

// 搜索结果操作
void lance_search_results_destroy(lance_search_result_t* results, size_t count);
lance_error_t lance_search_result_get_record(lance_search_result_t* result, lance_record_t** record);
lance_error_t lance_search_result_get_score(lance_search_result_t* result, float* score);
lance_error_t lance_search_result_get_id(lance_search_result_t* result, uint64_t* id);

#ifdef __cplusplus
}
#endif

#endif // LANCE_MOCK_H
