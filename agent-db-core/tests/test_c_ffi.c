#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "include/agent_state_db.h"

int main() {
    printf("Testing C FFI interface...\n");
    
    // 测试创建数据库
    printf("Creating database...\n");
    CAgentStateDB* db = agent_db_new("test_c_ffi.lance");
    if (db == NULL) {
        printf("Failed to create database\n");
        return 1;
    }
    printf("Database created successfully\n");
    
    // 测试保存状态
    printf("Testing save state...\n");
    const char* test_data = "Hello from C!";
    int result = agent_db_save_state(db, 12345, 67890, 0, (const uint8_t*)test_data, strlen(test_data));
    if (result != 0) {
        printf("Failed to save state: %d\n", result);
        agent_db_free(db);
        return 1;
    }
    printf("State saved successfully\n");
    
    // 测试加载状态
    printf("Testing load state...\n");
    uint8_t* loaded_data = NULL;
    size_t loaded_len = 0;
    result = agent_db_load_state(db, 12345, &loaded_data, &loaded_len);
    if (result == 0 && loaded_data != NULL) {
        printf("State loaded successfully: %.*s\n", (int)loaded_len, loaded_data);
        agent_db_free_data(loaded_data, loaded_len);
    } else {
        printf("Failed to load state or no data found: %d\n", result);
    }
    
    // 清理
    printf("Cleaning up...\n");
    agent_db_free(db);
    printf("Test completed successfully\n");
    
    return 0;
}
