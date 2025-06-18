# Makefile for testing Agent State DB

CC = gcc
CFLAGS = -Wall -Wextra -std=c99
INCLUDES = -I../include
LIBDIR = ../target/release
LIBS = -L$(LIBDIR) -lagent_state_db_rust -lws2_32 -ladvapi32 -luserenv -lntdll -lbcrypt

# Windows specific settings
ifeq ($(OS),Windows_NT)
    EXECUTABLE_EXT = .exe
    DLL_EXT = .dll
    LIB_PREFIX = 
else
    EXECUTABLE_EXT = 
    DLL_EXT = .so
    LIB_PREFIX = lib
endif

TARGET = test_rust_lib$(EXECUTABLE_EXT)
SOURCE = test_rust_lib.c

.PHONY: all clean test

all: $(TARGET)

$(TARGET): $(SOURCE)
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ $< $(LIBS)

test: $(TARGET)
	@echo "Running Rust library tests..."
	@echo "Library path: $(LIBDIR)"
	@echo "DLL file: $(LIBDIR)/$(LIB_PREFIX)agent_state_db_rust$(DLL_EXT)"
	@if [ -f "$(LIBDIR)/$(LIB_PREFIX)agent_state_db_rust$(DLL_EXT)" ]; then \
		echo "DLL found, running test..."; \
		./$(TARGET); \
	else \
		echo "ERROR: DLL not found at $(LIBDIR)/$(LIB_PREFIX)agent_state_db_rust$(DLL_EXT)"; \
		exit 1; \
	fi

clean:
	rm -f $(TARGET)
	rm -f test_db.lance
	rm -rf test_db.lance.tmp

help:
	@echo "Available targets:"
	@echo "  all    - Build the test executable"
	@echo "  test   - Build and run the test"
	@echo "  clean  - Remove generated files"
	@echo "  help   - Show this help message"
