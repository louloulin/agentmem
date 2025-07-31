#include <stdio.h>
#include <windows.h>

int main() {
    printf("Testing DLL loading...\n");
    
    // Try to load the DLL
    HMODULE hModule = LoadLibrary("./target/release/agent_state_db_rust.dll");
    if (hModule == NULL) {
        DWORD error = GetLastError();
        printf("Failed to load DLL. Error code: %lu\n", error);
        return 1;
    }
    
    printf("DLL loaded successfully!\n");
    
    // Try to get a function pointer
    void* func = GetProcAddress(hModule, "agent_db_new");
    if (func == NULL) {
        printf("Failed to find agent_db_new function\n");
        FreeLibrary(hModule);
        return 1;
    }
    
    printf("Function agent_db_new found!\n");
    
    FreeLibrary(hModule);
    printf("Test completed successfully!\n");
    return 0;
}
