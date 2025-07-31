// 分布式支持模块
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::AgentDbError;

// 智能体节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: u64,
    pub address: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub status: NodeStatus,
    pub last_heartbeat: i64,
}

// 节点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Busy,
    Maintenance,
}

// 智能体网络管理器
pub struct AgentNetworkManager {
    nodes: HashMap<u64, AgentNode>,
}

impl AgentNetworkManager {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn register_node(&mut self, node: AgentNode) -> Result<(), AgentDbError> {
        self.nodes.insert(node.id, node);
        Ok(())
    }

    pub fn get_node(&self, node_id: u64) -> Option<&AgentNode> {
        self.nodes.get(&node_id)
    }

    pub fn list_nodes(&self) -> Vec<&AgentNode> {
        self.nodes.values().collect()
    }
}

impl Default for AgentNetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

// 分布式状态管理器
pub struct DistributedStateManager {
    network: AgentNetworkManager,
}

impl DistributedStateManager {
    pub fn new() -> Self {
        Self {
            network: AgentNetworkManager::new(),
        }
    }
}

impl Default for DistributedStateManager {
    fn default() -> Self {
        Self::new()
    }
}

// 消息路由器
pub struct MessageRouter {
    routes: HashMap<String, u64>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, pattern: String, node_id: u64) {
        self.routes.insert(pattern, node_id);
    }

    pub fn route_message(&self, message_type: &str) -> Option<u64> {
        self.routes.get(message_type).copied()
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}
