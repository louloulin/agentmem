//! Object and memory pools for efficient resource management
//!
//! This module provides object pooling and memory pooling capabilities
//! to reduce allocation overhead and improve performance.

use agent_mem_traits::{AgentMemError, Result};
use bytes::{Bytes, BytesMut};
use crossbeam::queue::SegQueue;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Initial pool size
    pub initial_size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Minimum pool size
    pub min_size: usize,
    /// Enable pool statistics
    pub enable_stats: bool,
    /// Pool cleanup interval (seconds)
    pub cleanup_interval_seconds: u64,
    /// Object lifetime threshold (seconds)
    pub object_lifetime_seconds: u64,
    /// Memory block size for memory pool
    pub memory_block_size: usize,
    /// Maximum memory pool size (bytes)
    pub max_memory_pool_size: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 100,
            max_size: 1000,
            min_size: 10,
            enable_stats: true,
            cleanup_interval_seconds: 300,           // 5 minutes
            object_lifetime_seconds: 3600,           // 1 hour
            memory_block_size: 4096,                 // 4KB blocks
            max_memory_pool_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// Pooled object trait
pub trait Poolable: Send + Sync + 'static {
    /// Reset the object to its initial state
    fn reset(&mut self);

    /// Check if the object is valid for reuse
    fn is_valid(&self) -> bool {
        true
    }

    /// Get the size of the object in bytes
    fn size(&self) -> usize
    where
        Self: Sized,
    {
        std::mem::size_of::<Self>()
    }
}

/// Pooled object wrapper
struct PooledObject<T: Poolable> {
    object: T,
    created_at: Instant,
    last_used: Instant,
    use_count: u64,
}

impl<T: Poolable> PooledObject<T> {
    fn new(object: T) -> Self {
        let now = Instant::now();
        Self {
            object,
            created_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    fn is_expired(&self, lifetime_seconds: u64) -> bool {
        self.created_at.elapsed().as_secs() > lifetime_seconds
    }

    fn borrow(&mut self) -> &mut T {
        self.last_used = Instant::now();
        self.use_count += 1;
        &mut self.object
    }
}

/// Generic object pool
pub struct ObjectPool {
    config: PoolConfig,
    pool: Arc<SegQueue<Box<dyn std::any::Any + Send + Sync>>>,
    stats: Arc<RwLock<PoolStats>>,
    created_count: AtomicU64,
    borrowed_count: AtomicUsize,
}

/// Pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub total_objects: usize,
    pub available_objects: usize,
    pub borrowed_objects: usize,
    pub created_objects: u64,
    pub recycled_objects: u64,
    pub expired_objects: u64,
    pub memory_usage_bytes: usize,
}

impl ObjectPool {
    /// Create a new object pool
    pub fn new(config: PoolConfig) -> Result<Self> {
        let pool = Arc::new(SegQueue::new());
        let stats = Arc::new(RwLock::new(PoolStats::default()));

        let object_pool = Self {
            config,
            pool,
            stats,
            created_count: AtomicU64::new(0),
            borrowed_count: AtomicUsize::new(0),
        };

        info!(
            "Object pool created with max size: {}",
            object_pool.config.max_size
        );
        Ok(object_pool)
    }

    /// Get an object from the pool or create a new one
    pub fn get<T: Poolable + Default>(&self) -> Result<PooledObjectGuard<'_, T>> {
        // Try to get from pool first
        while let Some(boxed_obj) = self.pool.pop() {
            if let Ok(mut pooled_obj) = boxed_obj.downcast::<PooledObject<T>>() {
                if pooled_obj.object.is_valid()
                    && !pooled_obj.is_expired(self.config.object_lifetime_seconds)
                {
                    pooled_obj.object.reset();
                    self.borrowed_count.fetch_add(1, Ordering::Relaxed);

                    return Ok(PooledObjectGuard {
                        object: Some(pooled_obj),
                        pool: Arc::clone(&self.pool),
                        stats: Arc::clone(&self.stats),
                        borrowed_count: &self.borrowed_count,
                    });
                }
            }
        }

        // Create new object if pool is empty or objects are invalid
        let new_object = T::default();
        let pooled_obj = Box::new(PooledObject::new(new_object));

        self.created_count.fetch_add(1, Ordering::Relaxed);
        self.borrowed_count.fetch_add(1, Ordering::Relaxed);

        Ok(PooledObjectGuard {
            object: Some(pooled_obj),
            pool: Arc::clone(&self.pool),
            stats: Arc::clone(&self.stats),
            borrowed_count: &self.borrowed_count,
        })
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> Result<PoolStats> {
        let mut stats = self.stats.read().clone();
        stats.total_objects = self.pool.len() + self.borrowed_count.load(Ordering::Relaxed);
        stats.available_objects = self.pool.len();
        stats.borrowed_objects = self.borrowed_count.load(Ordering::Relaxed);
        stats.created_objects = self.created_count.load(Ordering::Relaxed);

        Ok(stats)
    }

    /// Clear the pool
    pub fn clear(&self) -> Result<()> {
        while self.pool.pop().is_some() {}

        let mut stats = self.stats.write();
        stats.total_objects = 0;
        stats.available_objects = 0;

        info!("Object pool cleared");
        Ok(())
    }
}

/// RAII guard for pooled objects
pub struct PooledObjectGuard<'a, T: Poolable> {
    object: Option<Box<PooledObject<T>>>,
    pool: Arc<SegQueue<Box<dyn std::any::Any + Send + Sync>>>,
    stats: Arc<RwLock<PoolStats>>,
    borrowed_count: &'a AtomicUsize,
}

impl<'a, T: Poolable> PooledObjectGuard<'a, T> {
    /// Get a mutable reference to the pooled object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap().borrow()
    }

    /// Get an immutable reference to the pooled object
    pub fn get(&self) -> &T {
        &self.object.as_ref().unwrap().object
    }
}

impl<'a, T: Poolable> Drop for PooledObjectGuard<'a, T> {
    fn drop(&mut self) {
        if let Some(mut pooled_obj) = self.object.take() {
            // Reset the object before returning to pool
            pooled_obj.object.reset();

            // Return to pool if it's still valid
            if pooled_obj.object.is_valid() {
                self.pool.push(pooled_obj);

                let mut stats = self.stats.write();
                stats.recycled_objects += 1;
            }

            self.borrowed_count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

/// Memory pool for efficient byte buffer management
pub struct MemoryPool {
    config: PoolConfig,
    small_blocks: Arc<SegQueue<BytesMut>>,  // < 1KB
    medium_blocks: Arc<SegQueue<BytesMut>>, // 1KB - 64KB
    large_blocks: Arc<SegQueue<BytesMut>>,  // > 64KB
    stats: Arc<RwLock<MemoryStats>>,
    total_allocated: AtomicUsize,
}

/// Memory pool statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_used: usize,
    pub total_free: usize,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub small_blocks_count: usize,
    pub medium_blocks_count: usize,
    pub large_blocks_count: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(config: PoolConfig) -> Result<Self> {
        let small_blocks = Arc::new(SegQueue::new());
        let medium_blocks = Arc::new(SegQueue::new());
        let large_blocks = Arc::new(SegQueue::new());
        let stats = Arc::new(RwLock::new(MemoryStats::default()));

        // Pre-allocate some blocks
        for _ in 0..config.initial_size / 3 {
            small_blocks.push(BytesMut::with_capacity(1024));
            medium_blocks.push(BytesMut::with_capacity(8192));
            large_blocks.push(BytesMut::with_capacity(65536));
        }

        let memory_pool = Self {
            config,
            small_blocks,
            medium_blocks,
            large_blocks,
            stats,
            total_allocated: AtomicUsize::new(0),
        };

        info!(
            "Memory pool created with max size: {} bytes",
            memory_pool.config.max_memory_pool_size
        );
        Ok(memory_pool)
    }

    /// Allocate a buffer of the specified size
    pub fn allocate(&self, size: usize) -> Result<MemoryBlock> {
        let buffer = if size <= 1024 {
            self.get_or_create_buffer(&self.small_blocks, 1024)
        } else if size <= 65536 {
            self.get_or_create_buffer(&self.medium_blocks, size.max(8192))
        } else {
            self.get_or_create_buffer(&self.large_blocks, size)
        };

        self.total_allocated
            .fetch_add(buffer.capacity(), Ordering::Relaxed);

        let mut stats = self.stats.write();
        stats.allocation_count += 1;
        stats.total_allocated = self.total_allocated.load(Ordering::Relaxed);

        Ok(MemoryBlock {
            buffer,
            pool: self,
            size,
            frozen: false,
        })
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> Result<MemoryStats> {
        let mut stats = self.stats.read().clone();
        stats.small_blocks_count = self.small_blocks.len();
        stats.medium_blocks_count = self.medium_blocks.len();
        stats.large_blocks_count = self.large_blocks.len();
        stats.total_allocated = self.total_allocated.load(Ordering::Relaxed);

        Ok(stats)
    }

    fn get_or_create_buffer(&self, queue: &SegQueue<BytesMut>, capacity: usize) -> BytesMut {
        if let Some(mut buffer) = queue.pop() {
            buffer.clear();
            if buffer.capacity() >= capacity {
                return buffer;
            }
        }

        BytesMut::with_capacity(capacity)
    }

    fn return_buffer(&self, mut buffer: BytesMut) {
        buffer.clear();

        let capacity = buffer.capacity();
        if capacity <= 1024 {
            self.small_blocks.push(buffer);
        } else if capacity <= 65536 {
            self.medium_blocks.push(buffer);
        } else {
            self.large_blocks.push(buffer);
        }

        let mut stats = self.stats.write();
        stats.deallocation_count += 1;
    }
}

/// RAII wrapper for memory blocks
pub struct MemoryBlock<'a> {
    buffer: BytesMut,
    pool: &'a MemoryPool,
    size: usize,
    frozen: bool,
}

impl<'a> MemoryBlock<'a> {
    /// Get a mutable reference to the buffer
    pub fn as_mut(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    /// Get an immutable reference to the buffer
    pub fn as_ref(&self) -> &BytesMut {
        &self.buffer
    }

    /// Convert to Bytes (immutable)
    pub fn freeze(mut self) -> Bytes {
        self.frozen = true;
        let bytes = self.buffer.clone().freeze();
        // Note: We can't return the buffer to pool after freeze
        bytes
    }
}

impl<'a> Drop for MemoryBlock<'a> {
    fn drop(&mut self) {
        // Only return buffer to pool if it hasn't been frozen
        if !self.frozen {
            let buffer = std::mem::replace(&mut self.buffer, BytesMut::new());
            self.pool.return_buffer(buffer);
        }
        self.pool
            .total_allocated
            .fetch_sub(self.size, Ordering::Relaxed);
    }
}

// Example poolable objects
#[derive(Default)]
pub struct StringBuffer {
    buffer: String,
}

impl Poolable for StringBuffer {
    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn size(&self) -> usize {
        self.buffer.capacity()
    }
}

impl StringBuffer {
    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.buffer
    }

    pub fn as_str(&self) -> &str {
        &self.buffer
    }
}

#[derive(Default)]
pub struct VecBuffer<T: Default + Clone> {
    buffer: Vec<T>,
}

impl<T: Default + Clone + Send + Sync + 'static> Poolable for VecBuffer<T> {
    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn size(&self) -> usize {
        self.buffer.capacity() * std::mem::size_of::<T>()
    }
}

impl<T: Default + Clone> VecBuffer<T> {
    pub fn as_mut_vec(&mut self) -> &mut Vec<T> {
        &mut self.buffer
    }

    pub fn as_slice(&self) -> &[T] {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_creation() {
        let config = PoolConfig::default();
        let pool = ObjectPool::new(config);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_object_pool_get() {
        let config = PoolConfig::default();
        let pool = ObjectPool::new(config).unwrap();

        let guard = pool.get::<StringBuffer>();
        assert!(guard.is_ok());
    }

    #[test]
    fn test_memory_pool_creation() {
        let config = PoolConfig::default();
        let pool = MemoryPool::new(config);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_memory_pool_allocate() {
        let config = PoolConfig::default();
        let pool = MemoryPool::new(config).unwrap();

        let block = pool.allocate(1024);
        assert!(block.is_ok());
    }

    #[test]
    fn test_pool_stats() {
        let config = PoolConfig::default();
        let pool = ObjectPool::new(config).unwrap();

        let stats = pool.get_stats().unwrap();
        assert_eq!(stats.created_objects, 0);
    }
}
