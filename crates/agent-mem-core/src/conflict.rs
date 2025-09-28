//! Conflict detection and resolution

use serde::{Deserialize, Serialize};

/// Conflict detector
pub struct ConflictDetector {
    // TODO: Implement conflict detector
}

impl ConflictDetector {
    /// Create new conflict detector
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Keep the first memory
    KeepFirst,

    /// Keep the last memory
    KeepLast,

    /// Merge memories
    Merge,

    /// Manual resolution required
    Manual,
}
