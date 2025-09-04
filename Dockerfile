# Multi-stage Docker build for AgentMem production deployment
# Optimized for security, performance, and minimal image size

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 agentmem

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/*/

# Create dummy source files to build dependencies
RUN mkdir -p crates/agent-mem-core/src \
    crates/agent-mem-traits/src \
    crates/agent-mem-llm/src \
    crates/agent-mem-storage/src \
    crates/agent-mem-intelligence/src \
    crates/agent-mem-graph/src \
    crates/agent-mem-server/src \
    crates/agent-mem-client/src \
    crates/agent-mem-performance/src \
    crates/agent-mem-distributed/src \
    && echo "fn main() {}" > crates/agent-mem-server/src/main.rs \
    && echo "// dummy" > crates/agent-mem-core/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-traits/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-llm/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-storage/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-intelligence/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-graph/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-client/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-performance/src/lib.rs \
    && echo "// dummy" > crates/agent-mem-distributed/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --bin agent-mem-server

# Remove dummy files
RUN rm -rf crates/*/src

# Copy actual source code
COPY . .

# Build the actual application
RUN cargo build --release --bin agent-mem-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user and group
RUN groupadd -r agentmem && useradd -r -g agentmem -u 1001 agentmem

# Create necessary directories
RUN mkdir -p /app/data /app/logs /app/config \
    && chown -R agentmem:agentmem /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/agent-mem-server /app/agent-mem-server

# Copy configuration files
COPY --chown=agentmem:agentmem docker/config/ /app/config/

# Set permissions
RUN chmod +x /app/agent-mem-server

# Switch to non-root user
USER agentmem

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV AGENT_MEM_PORT=8080
ENV AGENT_MEM_HOST=0.0.0.0
ENV AGENT_MEM_DATA_DIR=/app/data
ENV AGENT_MEM_LOG_DIR=/app/logs
ENV AGENT_MEM_CONFIG_DIR=/app/config

# Run the application
CMD ["./agent-mem-server"]
