//! Hierarchical memory management
//!
//! Implements ContextEngine's layered memory architecture with scoped access control.

use crate::Memory;
use agent_mem_traits::{MemoryType, AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Memory levels following ContextEngine's design
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum MemoryLevel {
    /// Strategic level - long-term planning and goals
    Strategic,
    /// Tactical level - medium-term execution plans
    Tactical,
    /// Operational level - short-term actions and tasks
    Operational,
    /// Contextual level - immediate context and responses
    Contextual,
}

/// Memory scope levels following ContextEngine's hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum MemoryScope {
    /// Global memories accessible across all contexts
    Global,
    /// Agent-specific memories
    Agent(String),
    /// User-specific memories within an agent context
    User { agent_id: String, user_id: String },
    /// Session-specific memories
    Session {
        agent_id: String,
        user_id: String,
        session_id: String,
    },
}

impl MemoryScope {
    /// Check if this scope can access memories from another scope
    pub fn can_access(&self, other: &MemoryScope) -> bool {
        match (self, other) {
            // Global scope can access everything
            (MemoryScope::Global, _) => true,

            // Agent scope can access global and own agent memories
            (MemoryScope::Agent(agent_id), MemoryScope::Global) => true,
            (MemoryScope::Agent(agent_id), MemoryScope::Agent(other_agent_id)) => {
                agent_id == other_agent_id
            }

            // User scope can access global, agent, and own user memories
            (MemoryScope::User { agent_id, user_id }, MemoryScope::Global) => true,
            (MemoryScope::User { agent_id, user_id }, MemoryScope::Agent(other_agent_id)) => {
                agent_id == other_agent_id
            }
            (
                MemoryScope::User { agent_id, user_id },
                MemoryScope::User {
                    agent_id: other_agent_id,
                    user_id: other_user_id,
                },
            ) => agent_id == other_agent_id && user_id == other_user_id,

            // Session scope can access all parent scopes and own session
            (
                MemoryScope::Session {
                    agent_id,
                    user_id,
                    session_id,
                },
                MemoryScope::Global,
            ) => true,
            (
                MemoryScope::Session {
                    agent_id,
                    user_id,
                    session_id,
                },
                MemoryScope::Agent(other_agent_id),
            ) => agent_id == other_agent_id,
            (
                MemoryScope::Session {
                    agent_id,
                    user_id,
                    session_id,
                },
                MemoryScope::User {
                    agent_id: other_agent_id,
                    user_id: other_user_id,
                },
            ) => agent_id == other_agent_id && user_id == other_user_id,
            (
                MemoryScope::Session {
                    agent_id,
                    user_id,
                    session_id,
                },
                MemoryScope::Session {
                    agent_id: other_agent_id,
                    user_id: other_user_id,
                    session_id: other_session_id,
                },
            ) => {
                agent_id == other_agent_id
                    && user_id == other_user_id
                    && session_id == other_session_id
            }

            // All other combinations are not allowed
            _ => false,
        }
    }

    /// Get the hierarchy level (lower number = higher privilege)
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            MemoryScope::Global => 0,
            MemoryScope::Agent(_) => 1,
            MemoryScope::User { .. } => 2,
            MemoryScope::Session { .. } => 3,
        }
    }

    /// Get parent scope
    pub fn parent(&self) -> Option<MemoryScope> {
        match self {
            MemoryScope::Global => None,
            MemoryScope::Agent(_) => Some(MemoryScope::Global),
            MemoryScope::User { agent_id, .. } => Some(MemoryScope::Agent(agent_id.clone())),
            MemoryScope::Session {
                agent_id, user_id, ..
            } => Some(MemoryScope::User {
                agent_id: agent_id.clone(),
                user_id: user_id.clone(),
            }),
        }
    }
}

/// Hierarchical memory with scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalMemory {
    /// Base memory data
    pub memory: Memory,

    /// Memory scope
    pub scope: MemoryScope,

    /// Inheritance rules
    pub inheritance: MemoryInheritance,

    /// Access permissions
    pub permissions: MemoryPermissions,
}

/// Memory inheritance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInheritance {
    /// Whether this memory can be inherited by child scopes
    pub inheritable: bool,

    /// Whether this memory was inherited from a parent scope
    pub inherited: bool,

    /// Original scope if inherited
    pub original_scope: Option<MemoryScope>,

    /// Inheritance decay factor (reduces importance over scope levels)
    pub decay_factor: f32,
}

impl Default for MemoryInheritance {
    fn default() -> Self {
        Self {
            inheritable: true,
            inherited: false,
            original_scope: None,
            decay_factor: 0.9, // 10% importance reduction per level
        }
    }
}

/// Memory access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPermissions {
    /// Can read this memory
    pub readable: bool,

    /// Can modify this memory
    pub writable: bool,

    /// Can delete this memory
    pub deletable: bool,

    /// Can share this memory with other scopes
    pub shareable: bool,
}

impl Default for MemoryPermissions {
    fn default() -> Self {
        Self {
            readable: true,
            writable: true,
            deletable: true,
            shareable: true,
        }
    }
}

/// Hierarchical memory manager
pub struct HierarchicalMemoryManager {
    /// Memories organized by scope
    memories: HashMap<MemoryScope, Vec<HierarchicalMemory>>,

    /// Scope inheritance cache
    inheritance_cache: HashMap<MemoryScope, Vec<MemoryScope>>,
}

impl HierarchicalMemoryManager {
    /// Create a new hierarchical memory manager
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            inheritance_cache: HashMap::new(),
        }
    }

    /// Add a memory to a specific scope
    pub fn add_memory(
        &mut self,
        memory: Memory,
        scope: MemoryScope,
        inheritance: Option<MemoryInheritance>,
        permissions: Option<MemoryPermissions>,
    ) -> Result<String> {
        let hierarchical_memory = HierarchicalMemory {
            memory,
            scope: scope.clone(),
            inheritance: inheritance.unwrap_or_default(),
            permissions: permissions.unwrap_or_default(),
        };

        self.memories
            .entry(scope)
            .or_insert_with(Vec::new)
            .push(hierarchical_memory);

        Ok(Uuid::new_v4().to_string())
    }

    /// Get memories accessible from a specific scope
    pub fn get_accessible_memories(&self, scope: &MemoryScope) -> Vec<&HierarchicalMemory> {
        let mut accessible = Vec::new();

        // Get all scopes this scope can access
        let accessible_scopes = self.get_accessible_scopes(scope);

        for accessible_scope in accessible_scopes {
            if let Some(memories) = self.memories.get(&accessible_scope) {
                for memory in memories {
                    if memory.permissions.readable && scope.can_access(&memory.scope) {
                        accessible.push(memory);
                    }
                }
            }
        }

        accessible
    }

    /// Get inherited memories for a scope
    pub fn get_inherited_memories(&self, scope: &MemoryScope) -> Vec<HierarchicalMemory> {
        let mut inherited = Vec::new();
        let mut current_scope = scope.parent();
        let mut level = 1;

        while let Some(parent_scope) = current_scope {
            if let Some(memories) = self.memories.get(&parent_scope) {
                for memory in memories {
                    if memory.inheritance.inheritable && memory.permissions.shareable {
                        let mut inherited_memory = memory.clone();

                        // Apply inheritance decay
                        inherited_memory.memory.importance = inherited_memory.memory.importance
                            * memory.inheritance.decay_factor.powi(level);

                        // Mark as inherited
                        inherited_memory.inheritance.inherited = true;
                        inherited_memory.inheritance.original_scope = Some(parent_scope.clone());
                        inherited_memory.scope = scope.clone();

                        inherited.push(inherited_memory);
                    }
                }
            }

            current_scope = parent_scope.parent();
            level += 1;
        }

        inherited
    }

    /// Get all scopes accessible from a given scope
    fn get_accessible_scopes(&self, scope: &MemoryScope) -> Vec<MemoryScope> {
        if let Some(cached) = self.inheritance_cache.get(scope) {
            return cached.clone();
        }

        let mut accessible = vec![scope.clone()];
        let mut current = scope.parent();

        while let Some(parent) = current {
            accessible.push(parent.clone());
            current = parent.parent();
        }

        accessible
    }

    /// Update memory permissions
    pub fn update_permissions(
        &mut self,
        memory_id: &str,
        scope: &MemoryScope,
        permissions: MemoryPermissions,
    ) -> Result<()> {
        if let Some(memories) = self.memories.get_mut(scope) {
            for memory in memories {
                if memory.memory.id == memory_id {
                    if memory.permissions.writable {
                        memory.permissions = permissions;
                        return Ok(());
                    } else {
                        return Err(AgentMemError::memory_error("Memory is not writable"));
                    }
                }
            }
        }

        Err(AgentMemError::not_found("Memory not found"))
    }

    /// Delete memory from scope
    pub fn delete_memory(&mut self, memory_id: &str, scope: &MemoryScope) -> Result<()> {
        if let Some(memories) = self.memories.get_mut(scope) {
            if let Some(pos) = memories.iter().position(|m| m.memory.id == memory_id) {
                let memory = &memories[pos];
                if memory.permissions.deletable {
                    memories.remove(pos);
                    Ok(())
                } else {
                    Err(AgentMemError::memory_error("Memory is not deletable"))
                }
            } else {
                Err(AgentMemError::not_found("Memory not found"))
            }
        } else {
            Err(AgentMemError::not_found("Scope not found"))
        }
    }

    /// Get memory statistics by scope
    pub fn get_scope_statistics(&self) -> HashMap<MemoryScope, ScopeStatistics> {
        let mut stats = HashMap::new();

        for (scope, memories) in &self.memories {
            let mut scope_stats = ScopeStatistics::default();

            for memory in memories {
                scope_stats.total_memories += 1;

                match memory.memory.memory_type {
                    MemoryType::Episodic => scope_stats.episodic_memories += 1,
                    MemoryType::Semantic => scope_stats.semantic_memories += 1,
                    MemoryType::Procedural => scope_stats.procedural_memories += 1,
                    MemoryType::Working => scope_stats.untyped_memories += 1,
                }

                if memory.inheritance.inherited {
                    scope_stats.inherited_memories += 1;
                }

                let importance = memory.memory.importance;
                scope_stats.total_importance += importance;
                if importance > scope_stats.max_importance {
                    scope_stats.max_importance = importance;
                }
                if importance < scope_stats.min_importance {
                    scope_stats.min_importance = importance;
                }
            }

            if scope_stats.total_memories > 0 {
                scope_stats.avg_importance =
                    scope_stats.total_importance / scope_stats.total_memories as f32;
            }

            stats.insert(scope.clone(), scope_stats);
        }

        stats
    }
}

/// Statistics for a memory scope
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeStatistics {
    pub total_memories: usize,
    pub episodic_memories: usize,
    pub semantic_memories: usize,
    pub procedural_memories: usize,
    pub untyped_memories: usize,
    pub inherited_memories: usize,
    pub total_importance: f32,
    pub avg_importance: f32,
    pub max_importance: f32,
    pub min_importance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_memory(id: &str, content: &str) -> Memory {
        Memory {
            id: id.to_string(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            memory_type: MemoryType::Episodic,
            content: content.to_string(),
            importance: 0.8,
            embedding: None,
            created_at: chrono::Utc::now().timestamp(),
            last_accessed_at: chrono::Utc::now().timestamp(),
            access_count: 0,
            expires_at: None,
            metadata: std::collections::HashMap::new(),
            version: 1,
        }
    }

    #[test]
    fn test_memory_scope_access() {
        let global = MemoryScope::Global;
        let agent = MemoryScope::Agent("agent1".to_string());
        let user = MemoryScope::User {
            agent_id: "agent1".to_string(),
            user_id: "user1".to_string(),
        };
        let session = MemoryScope::Session {
            agent_id: "agent1".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
        };

        // Global can access everything
        assert!(global.can_access(&global));
        assert!(global.can_access(&agent));
        assert!(global.can_access(&user));
        assert!(global.can_access(&session));

        // Agent can access global and own agent
        assert!(agent.can_access(&global));
        assert!(agent.can_access(&agent));
        assert!(!agent.can_access(&user));
        assert!(!agent.can_access(&session));

        // User can access global, agent, and own user
        assert!(user.can_access(&global));
        assert!(user.can_access(&agent));
        assert!(user.can_access(&user));
        assert!(!user.can_access(&session));

        // Session can access all parent scopes
        assert!(session.can_access(&global));
        assert!(session.can_access(&agent));
        assert!(session.can_access(&user));
        assert!(session.can_access(&session));
    }

    #[test]
    fn test_hierarchical_memory_manager() {
        let mut manager = HierarchicalMemoryManager::new();

        let memory1 = create_test_memory("mem1", "Global memory");
        let memory2 = create_test_memory("mem2", "Agent memory");
        let memory3 = create_test_memory("mem3", "User memory");

        let global_scope = MemoryScope::Global;
        let agent_scope = MemoryScope::Agent("agent1".to_string());
        let user_scope = MemoryScope::User {
            agent_id: "agent1".to_string(),
            user_id: "user1".to_string(),
        };

        // Add memories to different scopes
        manager
            .add_memory(memory1, global_scope.clone(), None, None)
            .unwrap();
        manager
            .add_memory(memory2, agent_scope.clone(), None, None)
            .unwrap();
        manager
            .add_memory(memory3, user_scope.clone(), None, None)
            .unwrap();

        // Test accessible memories from user scope
        let accessible = manager.get_accessible_memories(&user_scope);
        assert_eq!(accessible.len(), 3); // Should access all three

        // Test accessible memories from agent scope
        let accessible = manager.get_accessible_memories(&agent_scope);
        assert_eq!(accessible.len(), 2); // Should access global and agent only

        // Test statistics
        let stats = manager.get_scope_statistics();
        assert_eq!(stats.len(), 3);
        assert_eq!(stats[&global_scope].total_memories, 1);
        assert_eq!(stats[&agent_scope].total_memories, 1);
        assert_eq!(stats[&user_scope].total_memories, 1);
    }

    #[test]
    fn test_memory_inheritance() {
        let mut manager = HierarchicalMemoryManager::new();

        let global_memory = create_test_memory("global", "Global knowledge");
        let global_scope = MemoryScope::Global;

        let user_scope = MemoryScope::User {
            agent_id: "agent1".to_string(),
            user_id: "user1".to_string(),
        };

        // Add inheritable memory to global scope
        let inheritance = MemoryInheritance {
            inheritable: true,
            inherited: false,
            original_scope: None,
            decay_factor: 0.8,
        };

        manager
            .add_memory(global_memory, global_scope, Some(inheritance), None)
            .unwrap();

        // Get inherited memories for user scope
        let inherited = manager.get_inherited_memories(&user_scope);
        assert_eq!(inherited.len(), 1);
        assert!(inherited[0].inheritance.inherited);
        assert_eq!(
            inherited[0].inheritance.original_scope,
            Some(MemoryScope::Global)
        );

        // Check importance decay (0.8 * 0.8^2 = 0.512)
        let expected_importance = 0.8 * 0.8_f32.powi(2);
        assert!((inherited[0].memory.importance - expected_importance).abs() < 0.001);
    }
}
