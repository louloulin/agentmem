//! Memory managers module
//! 
//! 专门的记忆管理器，实现不同类型记忆的特化管理

pub mod core_memory;

pub use core_memory::{
    CoreMemoryManager, CoreMemoryBlock, CoreMemoryBlockType, 
    CoreMemoryConfig, CoreMemoryStats
};
