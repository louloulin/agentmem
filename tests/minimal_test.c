#include <stdio.h>
#include <stdlib.h>
#include "../include/agent_state_db.h"

int main() {
    printf("Starting minimal test...\n");
    fflush(stdout);
    
    printf("Creating database...\n");
    fflush(stdout);
    
    struct CAgentStateDB *db = agent_db_new("minimal_test.lance");
    
    printf("Database creation returned: %p\n", (void*)db);
    fflush(stdout);
    
    if (db == NULL) {
        printf("Database creation failed\n");
        fflush(stdout);
        return 1;
    }
    
    printf("Database created successfully\n");
    fflush(stdout);
    
    printf("Freeing database...\n");
    fflush(stdout);
    
    agent_db_free(db);
    
    printf("Test completed successfully\n");
    fflush(stdout);
    
    return 0;
}
