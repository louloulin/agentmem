const std = @import("std");
const print = std.debug.print;

// Import the C header
const c = @cImport({
    @cInclude("agent_state_db.h");
});

pub fn main() !void {
    print("Testing Zig integration with Agent State DB...\n", .{});

    // Test 1: Create database
    print("1. Creating database...\n", .{});
    const db = c.agent_db_new("test_db_zig.lance");
    if (db == null) {
        print("   FAILED: Could not create database\n", .{});
        return;
    }
    print("   SUCCESS: Database created\n", .{});

    // Test 2: Save agent state
    print("2. Saving agent state...\n", .{});
    const agent_id: u64 = 12345;
    const session_id: u64 = 67890;
    const state_type: c_int = 1; // working_memory
    const test_data = "Hello from Zig!";
    const data: [*c]const u8 = test_data.ptr;
    const data_len: usize = test_data.len;

    const result = c.agent_db_save_state(db, agent_id, session_id, state_type, data, data_len);
    if (result != 0) {
        print("   FAILED: Could not save state (error code: {})\n", .{result});
        c.agent_db_free(db);
        return;
    }
    print("   SUCCESS: Agent state saved\n", .{});

    // Test 3: Load agent state
    print("3. Loading agent state...\n", .{});
    var loaded_data: [*c]u8 = null;
    var loaded_data_len: usize = 0;

    const load_result = c.agent_db_load_state(db, agent_id, &loaded_data, &loaded_data_len);
    if (load_result != 0) {
        print("   FAILED: Could not load state (error code: {})\n", .{load_result});
        c.agent_db_free(db);
        return;
    }

    if (loaded_data == null or loaded_data_len == 0) {
        print("   FAILED: No data loaded\n", .{});
        c.agent_db_free(db);
        return;
    }

    // Verify data
    const loaded_slice = loaded_data[0..loaded_data_len];
    if (std.mem.eql(u8, loaded_slice, test_data)) {
        print("   SUCCESS: Data loaded correctly: {s}\n", .{loaded_slice});
    } else {
        print("   FAILED: Data mismatch\n", .{});
        print("   Expected: {s}\n", .{test_data});
        print("   Got: {s}\n", .{loaded_slice});
        c.agent_db_free_data(loaded_data, loaded_data_len);
        c.agent_db_free(db);
        return;
    }

    // Clean up
    c.agent_db_free_data(loaded_data, loaded_data_len);
    c.agent_db_free(db);

    print("\nAll tests passed! ✅\n", .{});
}
