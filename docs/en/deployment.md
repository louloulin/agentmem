# AgentDB Deployment Guide

## 🚀 Deployment Overview

This guide provides detailed instructions for deploying AgentDB in different environments, including single-node deployment, distributed deployment, and cloud-native deployment.

## 📋 Pre-deployment Preparation

### System Requirements

#### Production Minimum Requirements
- **CPU**: 4 cores 2.0GHz+
- **Memory**: 8GB RAM
- **Storage**: 50GB SSD
- **Network**: 1Gbps bandwidth
- **OS**: Ubuntu 20.04+, CentOS 8+, Windows Server 2019+

#### Recommended Production Configuration
- **CPU**: 8 cores 3.0GHz+
- **Memory**: 32GB RAM
- **Storage**: 500GB NVMe SSD
- **Network**: 10Gbps bandwidth
- **OS**: Ubuntu 22.04 LTS

### Dependencies

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential curl git

# CentOS/RHEL
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Zig
wget https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.xz
tar -xf zig-linux-x86_64-0.14.0.tar.xz
sudo mv zig-linux-x86_64-0.14.0 /opt/zig
echo 'export PATH="/opt/zig:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## 🏠 Single-Node Deployment

### 1. Source Code Compilation Deployment

```bash
# Clone repository
git clone https://github.com/louloulin/AgentDB.git
cd AgentDB

# Build release version
cargo build --release

# Generate C headers
cargo run --bin generate_bindings

# Build Zig components
zig build -Doptimize=ReleaseFast

# Run tests for verification
cargo test --release
zig build test
```

### 2. Configuration File Setup

Create `/etc/agentdb/config.toml`:

```toml
[database]
path = "/var/lib/agentdb/data"
max_connections = 100
connection_timeout = 30
query_timeout = 120
enable_wal = true
cache_size = 1073741824  # 1GB

[vector]
dimension = 384
similarity_algorithm = "cosine"
index_type = "hnsw"
ef_construction = 200
m = 16

[memory]
max_memories_per_agent = 50000
importance_threshold = 0.05
decay_factor = 0.001
cleanup_interval = 3600  # 1 hour

[security]
enable_auth = true
enable_encryption = true
jwt_secret = "your-production-secret-key-here"
session_timeout = 86400  # 24 hours

[performance]
enable_cache = true
batch_size = 5000
worker_threads = 8
io_threads = 4

[logging]
level = "info"
file = "/var/log/agentdb/agentdb.log"
max_size = "100MB"
max_files = 10

[monitoring]
enable_metrics = true
metrics_port = 9090
health_check_port = 8080
```

### 3. System Service Configuration

Create `/etc/systemd/system/agentdb.service`:

```ini
[Unit]
Description=AgentDB High-Performance AI Agent Database
After=network.target
Wants=network.target

[Service]
Type=simple
User=agentdb
Group=agentdb
WorkingDirectory=/opt/agentdb
ExecStart=/opt/agentdb/target/release/agentdb-server --config /etc/agentdb/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
LimitNOFILE=65536
LimitNPROC=32768

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/agentdb /var/log/agentdb

[Install]
WantedBy=multi-user.target
```

### 4. Start Service

```bash
# Create user and directories
sudo useradd -r -s /bin/false agentdb
sudo mkdir -p /var/lib/agentdb/data
sudo mkdir -p /var/log/agentdb
sudo mkdir -p /etc/agentdb
sudo chown -R agentdb:agentdb /var/lib/agentdb /var/log/agentdb

# Copy binary files
sudo cp target/release/agentdb-server /opt/agentdb/
sudo chown agentdb:agentdb /opt/agentdb/agentdb-server
sudo chmod +x /opt/agentdb/agentdb-server

# Start service
sudo systemctl daemon-reload
sudo systemctl enable agentdb
sudo systemctl start agentdb

# Check status
sudo systemctl status agentdb
```

## 🌐 Distributed Deployment

### 1. Cluster Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AgentDB Distributed Cluster              │
├─────────────────────────────────────────────────────────────┤
│  Load Balancer Layer                                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   HAProxy   │ │   Nginx     │ │   Consul    │           │
│  │  (Primary)  │ │  (Backup)   │ │ (Discovery) │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  AgentDB Node Layer                                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │  Node-1     │ │  Node-2     │ │  Node-3     │           │
│  │ (Master)    │ │ (Worker)    │ │ (Worker)    │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  Storage Layer                                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │  LanceDB    │ │   Redis     │ │   MinIO     │           │
│  │ (Primary)   │ │  (Cache)    │ │ (Object)    │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

### 2. Node Configuration

#### Master Node Configuration (node-1)

```toml
[cluster]
node_id = "node-1"
node_type = "master"
bind_address = "0.0.0.0:7000"
cluster_members = [
    "node-1:7000",
    "node-2:7000", 
    "node-3:7000"
]
election_timeout = 5000
heartbeat_interval = 1000

[replication]
enable_replication = true
replication_factor = 3
sync_mode = "async"
backup_interval = 3600

[sharding]
enable_sharding = true
shard_count = 16
hash_algorithm = "consistent"
```

#### Worker Node Configuration (node-2, node-3)

```toml
[cluster]
node_id = "node-2"  # Use "node-3" for node-3
node_type = "worker"
bind_address = "0.0.0.0:7000"
master_address = "node-1:7000"
cluster_members = [
    "node-1:7000",
    "node-2:7000",
    "node-3:7000"
]
```

### 3. Load Balancer Configuration

#### HAProxy Configuration (`/etc/haproxy/haproxy.cfg`)

```
global
    daemon
    maxconn 4096
    log stdout local0

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    option httplog

frontend agentdb_frontend
    bind *:8080
    default_backend agentdb_backend

backend agentdb_backend
    balance roundrobin
    option httpchk GET /health
    server node1 node-1:8080 check
    server node2 node-2:8080 check
    server node3 node-3:8080 check

frontend agentdb_api
    bind *:9000
    default_backend agentdb_api_backend

backend agentdb_api_backend
    balance leastconn
    server node1 node-1:9000 check
    server node2 node-2:9000 check
    server node3 node-3:9000 check
```

## ☁️ Cloud-Native Deployment

### 1. Docker Containerization

#### Dockerfile

```dockerfile
# Multi-stage build
FROM rust:1.70 as rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM ziglang/zig:0.14.0 as zig-builder
WORKDIR /app
COPY build.zig ./
COPY src ./src
COPY --from=rust-builder /app/target/release/libagent_db_rust.so ./target/release/
RUN zig build -Doptimize=ReleaseFast

# Runtime image
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=rust-builder /app/target/release/agentdb-server ./
COPY --from=zig-builder /app/zig-out/bin/* ./
COPY config/docker.toml ./config.toml

EXPOSE 8080 9000 9090
USER 1000:1000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./agentdb-server", "--config", "config.toml"]
```

#### Docker Compose

```yaml
version: '3.8'

services:
  agentdb-node1:
    build: .
    container_name: agentdb-node1
    hostname: node1
    ports:
      - "8081:8080"
      - "9001:9000"
      - "9091:9090"
    volumes:
      - agentdb-data1:/var/lib/agentdb
      - ./config/node1.toml:/app/config.toml
    environment:
      - AGENTDB_NODE_ID=node-1
      - AGENTDB_NODE_TYPE=master
    networks:
      - agentdb-network

  agentdb-node2:
    build: .
    container_name: agentdb-node2
    hostname: node2
    ports:
      - "8082:8080"
      - "9002:9000"
      - "9092:9090"
    volumes:
      - agentdb-data2:/var/lib/agentdb
      - ./config/node2.toml:/app/config.toml
    environment:
      - AGENTDB_NODE_ID=node-2
      - AGENTDB_NODE_TYPE=worker
    depends_on:
      - agentdb-node1
    networks:
      - agentdb-network

  redis:
    image: redis:7-alpine
    container_name: agentdb-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    networks:
      - agentdb-network

  prometheus:
    image: prom/prometheus:latest
    container_name: agentdb-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    networks:
      - agentdb-network

volumes:
  agentdb-data1:
  agentdb-data2:
  redis-data:
  prometheus-data:

networks:
  agentdb-network:
    driver: bridge
```

### 2. Kubernetes Deployment

#### Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: agentdb
  labels:
    name: agentdb
```

#### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: agentdb-config
  namespace: agentdb
data:
  config.toml: |
    [database]
    path = "/var/lib/agentdb/data"
    max_connections = 200
    connection_timeout = 30
    query_timeout = 120
    enable_wal = true
    cache_size = 2147483648  # 2GB
    
    [cluster]
    enable_cluster = true
    node_id = "${NODE_ID}"
    node_type = "${NODE_TYPE}"
    bind_address = "0.0.0.0:7000"
    
    [monitoring]
    enable_metrics = true
    metrics_port = 9090
    health_check_port = 8080
```

#### StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: agentdb
  namespace: agentdb
spec:
  serviceName: agentdb-headless
  replicas: 3
  selector:
    matchLabels:
      app: agentdb
  template:
    metadata:
      labels:
        app: agentdb
    spec:
      containers:
      - name: agentdb
        image: agentdb:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9000
          name: api
        - containerPort: 9090
          name: metrics
        - containerPort: 7000
          name: cluster
        env:
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: NODE_TYPE
          value: "worker"
        volumeMounts:
        - name: data
          mountPath: /var/lib/agentdb
        - name: config
          mountPath: /app/config.toml
          subPath: config.toml
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
      volumes:
      - name: config
        configMap:
          name: agentdb-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 100Gi
```

## 📊 Monitoring and Operations

### 1. Health Checks

```bash
# Check service status
curl http://localhost:8080/health

# Check cluster status
curl http://localhost:8080/cluster/status

# Check performance metrics
curl http://localhost:9090/metrics
```

### 2. Log Management

```bash
# View service logs
sudo journalctl -u agentdb -f

# View application logs
tail -f /var/log/agentdb/agentdb.log

# Log rotation configuration
sudo logrotate -d /etc/logrotate.d/agentdb
```

### 3. Backup Strategy

```bash
#!/bin/bash
# Backup script backup.sh

BACKUP_DIR="/backup/agentdb"
DATE=$(date +%Y%m%d_%H%M%S)
DATA_DIR="/var/lib/agentdb/data"

# Create backup directory
mkdir -p $BACKUP_DIR

# Data backup
tar -czf $BACKUP_DIR/agentdb_data_$DATE.tar.gz -C $DATA_DIR .

# Configuration backup
cp /etc/agentdb/config.toml $BACKUP_DIR/config_$DATE.toml

# Clean old backups (keep 30 days)
find $BACKUP_DIR -name "*.tar.gz" -mtime +30 -delete
find $BACKUP_DIR -name "config_*.toml" -mtime +30 -delete

echo "Backup completed: $DATE"
```

## 🔧 Performance Tuning

### 1. System-level Optimization

```bash
# Kernel parameter optimization
echo 'net.core.somaxconn = 65535' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65535' >> /etc/sysctl.conf
echo 'fs.file-max = 1000000' >> /etc/sysctl.conf
sysctl -p

# File descriptor limits
echo '* soft nofile 1000000' >> /etc/security/limits.conf
echo '* hard nofile 1000000' >> /etc/security/limits.conf
```

### 2. Application-level Optimization

```toml
[performance]
# Worker threads = CPU cores
worker_threads = 16

# I/O threads = CPU cores / 2
io_threads = 8

# Batch size
batch_size = 10000

# Cache size = 50% of available memory
cache_size = 16777216000  # 16GB

# Connection pool size
max_connections = 1000
```

---

**Document Version**: v1.0  
**Last Updated**: June 19, 2025  
**Maintainer**: AgentDB Development Team
