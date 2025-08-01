use criterion::{black_box, criterion_group, criterion_main, Criterion};
use agent_db_core::*;
use std::collections::HashMap;
use tempfile::TempDir;

async fn create_test_database() -> (AgentDatabase, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let config = DatabaseConfig {
        db_path: db_path.to_string(),
        max_connections: 10,
        cache_size: 1024 * 1024,
        enable_wal: true,
        sync_mode: "NORMAL".to_string(),
    };
    let db = AgentDatabase::new(config).await.unwrap();
    (db, temp_dir)
}

fn bench_agent_state_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("agent_state_save", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (db, _temp_dir) = create_test_database().await;
                let state = AgentState::new(
                    black_box(1001),
                    1,
                    StateType::WorkingMemory,
                    black_box(b"test data".to_vec()),
                );
                db.save_agent_state(&state).await.unwrap();
            });
        });
    });

    c.bench_function("agent_state_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (db, _temp_dir) = create_test_database().await;
                let state = AgentState::new(1001, 1, StateType::WorkingMemory, b"test data".to_vec());
                db.save_agent_state(&state).await.unwrap();
                
                let _loaded = db.load_agent_state(black_box(1001)).await.unwrap();
            });
        });
    });
}

fn bench_memory_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("memory_store", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (db, _temp_dir) = create_test_database().await;
                let memory = Memory::new(
                    black_box(1001),
                    MemoryType::Episodic,
                    black_box("Test memory content".to_string()),
                    0.8,
                );
                db.store_memory(&memory).await.unwrap();
            });
        });
    });

    c.bench_function("memory_retrieve", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (db, _temp_dir) = create_test_database().await;
                let memory = Memory::new(1001, MemoryType::Episodic, "Test memory content".to_string(), 0.8);
                db.store_memory(&memory).await.unwrap();
                
                let _memories = db.get_memories(black_box(1001)).await.unwrap();
            });
        });
    });
}

fn bench_vector_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("vector_add", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (mut db, _temp_dir) = create_test_database().await;
                let vector_config = vector::VectorIndexConfig {
                    dimension: 128,
                    metric: "cosine".to_string(),
                    index_type: "flat".to_string(),
                    ef_construction: 200,
                    m: 16,
                };
                db = db.with_vector_engine(vector_config).await.unwrap();

                let vector: Vec<f32> = (0..128).map(|i| i as f32 / 128.0).collect();
                let mut metadata = HashMap::new();
                metadata.insert("type".to_string(), "benchmark".to_string());

                db.add_vector(black_box(1), black_box(vector), metadata).await.unwrap();
            });
        });
    });

    c.bench_function("vector_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (mut db, _temp_dir) = create_test_database().await;
                let vector_config = vector::VectorIndexConfig {
                    dimension: 128,
                    metric: "cosine".to_string(),
                    index_type: "flat".to_string(),
                    ef_construction: 200,
                    m: 16,
                };
                db = db.with_vector_engine(vector_config).await.unwrap();

                // 添加一些向量
                for i in 0..100 {
                    let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 128.0).collect();
                    let mut metadata = HashMap::new();
                    metadata.insert("id".to_string(), i.to_string());
                    db.add_vector(i, vector, metadata).await.unwrap();
                }

                let query_vector: Vec<f32> = (0..128).map(|i| i as f32 / 128.0).collect();
                let _results = db.search_vectors(black_box(&query_vector), black_box(10)).await.unwrap();
            });
        });
    });
}

fn bench_rag_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("document_index", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (mut db, _temp_dir) = create_test_database().await;
                db = db.with_rag_engine().await.unwrap();

                let document = Document::new(
                    black_box("Test Document".to_string()),
                    black_box("This is a test document for RAG functionality benchmarking.".to_string()),
                );
                db.index_document(&document).await.unwrap();
            });
        });
    });

    c.bench_function("document_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (mut db, _temp_dir) = create_test_database().await;
                db = db.with_rag_engine().await.unwrap();

                // 索引一些文档
                for i in 0..50 {
                    let document = Document::new(
                        format!("Document {}", i),
                        format!("This is test document number {} with some content for searching.", i),
                    );
                    db.index_document(&document).await.unwrap();
                }

                let _results = db.search_documents(black_box("test"), black_box(10)).await.unwrap();
            });
        });
    });
}

fn bench_batch_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("batch_memory_store", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (db, _temp_dir) = create_test_database().await;
                
                let memories: Vec<Memory> = (0..100).map(|i| {
                    Memory::new(
                        1001,
                        MemoryType::Episodic,
                        format!("Batch memory {}", i),
                        0.8,
                    )
                }).collect();

                db.memory_manager.store_memories_batch(black_box(memories)).await.unwrap();
            });
        });
    });
}

criterion_group!(
    benches,
    bench_agent_state_operations,
    bench_memory_operations,
    bench_vector_operations,
    bench_rag_operations,
    bench_batch_operations
);
criterion_main!(benches);
