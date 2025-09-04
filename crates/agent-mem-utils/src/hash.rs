//! Hashing utilities

use sha2::{Digest, Sha256};

/// Generate SHA256 hash of text content
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generate a short hash (first 8 characters of SHA256)
pub fn short_hash(content: &str) -> String {
    let full_hash = hash_content(content);
    full_hash[..8].to_string()
}

/// Generate hash for memory deduplication
pub fn memory_hash(content: &str) -> String {
    // Normalize content before hashing
    let normalized = content.trim().to_lowercase();
    hash_content(&normalized)
}

/// Check if two content strings have the same hash
pub fn same_content_hash(content1: &str, content2: &str) -> bool {
    hash_content(content1) == hash_content(content2)
}

/// Generate a unique ID based on content and timestamp
pub fn generate_content_id(content: &str, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let combined = format!("{}_{}", content, timestamp.timestamp());
    short_hash(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_hash_content() {
        let content = "test content";
        let hash = hash_content(content);
        assert_eq!(hash.len(), 64); // SHA256 produces 64 character hex string

        // Same content should produce same hash
        let hash2 = hash_content(content);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_short_hash() {
        let content = "test content";
        let hash = short_hash(content);
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn test_memory_hash() {
        let content1 = "  Test Content  ";
        let content2 = "  test content  ";

        let hash1 = memory_hash(content1);
        let hash2 = memory_hash(content2);

        // Should be the same after normalization
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_same_content_hash() {
        let content1 = "identical content";
        let content2 = "identical content";
        let content3 = "different content";

        assert!(same_content_hash(content1, content2));
        assert!(!same_content_hash(content1, content3));
    }

    #[test]
    fn test_generate_content_id() {
        let content = "test content";
        let timestamp = Utc::now();
        let id = generate_content_id(content, &timestamp);
        assert_eq!(id.len(), 8);

        // Different timestamps should produce different IDs
        let timestamp2 = timestamp + chrono::Duration::seconds(1);
        let id2 = generate_content_id(content, &timestamp2);
        assert_ne!(id, id2);
    }
}
