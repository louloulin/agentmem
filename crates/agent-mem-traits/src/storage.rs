//! Storage trait definitions

use async_trait::async_trait;
use crate::{Result, Vector, SearchResult, Filters, Metadata, Entity, Relation, Session, HistoryEntry};

/// Vector store trait
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Insert vectors with metadata
    async fn insert(&self, vectors: &[Vector], metadata: &[Metadata]) -> Result<Vec<String>>;
    
    /// Search for similar vectors
    async fn search(&self, query: &Vector, limit: usize, filters: &Filters) -> Result<Vec<SearchResult>>;
    
    /// Update a vector and its metadata
    async fn update(&self, id: &str, vector: &Vector, metadata: &Metadata) -> Result<()>;
    
    /// Delete a vector
    async fn delete(&self, id: &str) -> Result<()>;
    
    /// Reset the vector store (for testing)
    async fn reset(&self) -> Result<()>;
    
    /// Get store statistics
    async fn stats(&self) -> Result<VectorStoreStats>;
}

/// Graph store trait
#[async_trait]
pub trait GraphStore: Send + Sync {
    /// Add entities to the graph
    async fn add_entities(&self, entities: &[Entity], session: &Session) -> Result<()>;
    
    /// Add relations to the graph
    async fn add_relations(&self, relations: &[Relation], session: &Session) -> Result<()>;
    
    /// Search the graph
    async fn search_graph(&self, query: &str, session: &Session) -> Result<Vec<GraphResult>>;
    
    /// Get neighbors of an entity
    async fn get_neighbors(&self, entity_id: &str, depth: usize) -> Result<Vec<Entity>>;
    
    /// Reset the graph store (for testing)
    async fn reset(&self) -> Result<()>;
}

/// Key-value store trait
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    /// Set a key-value pair
    async fn set(&self, key: &str, value: &str) -> Result<()>;
    
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<String>>;
    
    /// Delete a key
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// Set with expiration
    async fn set_with_expiry(&self, key: &str, value: &str, seconds: u64) -> Result<()>;
}

/// History store trait
#[async_trait]
pub trait HistoryStore: Send + Sync {
    /// Add a history entry
    async fn add_history_entry(&self, memory_id: &str, entry: &HistoryEntry) -> Result<()>;
    
    /// Get history for a memory
    async fn get_history(&self, memory_id: &str) -> Result<Vec<HistoryEntry>>;
    
    /// Delete history for a memory
    async fn delete_history(&self, memory_id: &str) -> Result<()>;
}

/// Vector store statistics
#[derive(Debug, Clone)]
pub struct VectorStoreStats {
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_size: usize,
}

/// Graph search result
#[derive(Debug, Clone)]
pub struct GraphResult {
    pub entity: Entity,
    pub relations: Vec<Relation>,
    pub score: f32,
}
