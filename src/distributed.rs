// Distributed 模块 - 分布式网络支持
// 分布式Agent网络管理

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::core::*;

// Agent节点信息
#[derive(Debug, Clone)]
pub struct AgentNode {
    pub id: u64,
    pub address: SocketAddr,
    pub capabilities: Vec<String>,
    pub status: NodeStatus,
    pub last_heartbeat: Instant,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Busy,
    Maintenance,
}

// 分布式消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedMessage {
    pub id: String,
    pub from_node: u64,
    pub to_node: Option<u64>, // None表示广播
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: i64,
    pub ttl: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Heartbeat,
    StateSync,
    Query,
    Response,
    Broadcast,
    Command,
}

// Agent网络管理器
pub struct AgentNetworkManager {
    nodes: Arc<RwLock<HashMap<u64, AgentNode>>>,
    local_node_id: u64,
    message_queue: Arc<Mutex<Vec<DistributedMessage>>>,
    heartbeat_interval: Duration,
}

impl AgentNetworkManager {
    pub fn new(local_node_id: u64) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            local_node_id,
            message_queue: Arc::new(Mutex::new(Vec::new())),
            heartbeat_interval: Duration::from_secs(30),
        }
    }

    pub fn register_node(&self, node: AgentNode) -> Result<(), AgentDbError> {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(node.id, node);
        Ok(())
    }

    pub fn unregister_node(&self, node_id: u64) -> Result<(), AgentDbError> {
        let mut nodes = self.nodes.write().unwrap();
        nodes.remove(&node_id);
        Ok(())
    }

    pub fn send_message(&self, message: DistributedMessage) -> Result<(), AgentDbError> {
        let mut queue = self.message_queue.lock().unwrap();
        queue.push(message);
        Ok(())
    }

    pub fn broadcast_message(&self, message_type: MessageType, payload: Vec<u8>) -> Result<(), AgentDbError> {
        let message = DistributedMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from_node: self.local_node_id,
            to_node: None,
            message_type,
            payload,
            timestamp: chrono::Utc::now().timestamp(),
            ttl: Duration::from_secs(300),
        };

        self.send_message(message)
    }

    pub fn get_online_nodes(&self) -> Vec<AgentNode> {
        let nodes = self.nodes.read().unwrap();
        nodes.values()
            .filter(|node| matches!(node.status, NodeStatus::Online))
            .cloned()
            .collect()
    }

    pub fn find_nodes_by_capability(&self, capability: &str) -> Vec<AgentNode> {
        let nodes = self.nodes.read().unwrap();
        nodes.values()
            .filter(|node| node.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }

    pub fn update_heartbeat(&self, node_id: u64) -> Result<(), AgentDbError> {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(node) = nodes.get_mut(&node_id) {
            node.last_heartbeat = Instant::now();
            node.status = NodeStatus::Online;
        }
        Ok(())
    }

    pub fn cleanup_offline_nodes(&self) -> usize {
        let mut nodes = self.nodes.write().unwrap();
        let timeout = Duration::from_secs(120); // 2分钟超时
        let now = Instant::now();
        
        let mut to_remove = Vec::new();
        for (id, node) in nodes.iter_mut() {
            if now.duration_since(node.last_heartbeat) > timeout {
                node.status = NodeStatus::Offline;
                to_remove.push(*id);
            }
        }

        for id in &to_remove {
            nodes.remove(id);
        }

        to_remove.len()
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.read().unwrap().len()
    }

    pub fn get_message_queue_size(&self) -> usize {
        self.message_queue.lock().unwrap().len()
    }
}

// 分布式状态管理器
pub struct DistributedStateManager {
    network_manager: Arc<AgentNetworkManager>,
    state_cache: Arc<RwLock<HashMap<u64, Vec<u8>>>>,
    vector_clock: Arc<Mutex<HashMap<u64, u64>>>,
}

impl DistributedStateManager {
    pub fn new(network_manager: Arc<AgentNetworkManager>) -> Self {
        Self {
            network_manager,
            state_cache: Arc::new(RwLock::new(HashMap::new())),
            vector_clock: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn sync_state(&self, agent_id: u64, state_data: Vec<u8>) -> Result<(), AgentDbError> {
        // 更新本地状态缓存
        {
            let mut cache = self.state_cache.write().unwrap();
            cache.insert(agent_id, state_data.clone());
        }

        // 更新向量时钟
        {
            let mut clock = self.vector_clock.lock().unwrap();
            let current = clock.get(&agent_id).copied().unwrap_or(0);
            clock.insert(agent_id, current + 1);
        }

        // 广播状态更新
        self.network_manager.broadcast_message(
            MessageType::StateSync,
            state_data,
        )?;

        Ok(())
    }

    pub fn get_cached_state(&self, agent_id: u64) -> Option<Vec<u8>> {
        let cache = self.state_cache.read().unwrap();
        cache.get(&agent_id).cloned()
    }

    pub fn resolve_conflict(&self, agent_id: u64, local_state: &[u8], remote_state: &[u8]) -> Vec<u8> {
        // 简单的冲突解决策略：选择较新的状态
        // 实际应用中可能需要更复杂的合并逻辑
        
        let local_clock = {
            let clock = self.vector_clock.lock().unwrap();
            clock.get(&agent_id).copied().unwrap_or(0)
        };

        // 这里简化处理，实际应该比较向量时钟
        if local_state.len() > remote_state.len() {
            local_state.to_vec()
        } else {
            remote_state.to_vec()
        }
    }

    pub fn get_vector_clock(&self) -> HashMap<u64, u64> {
        self.vector_clock.lock().unwrap().clone()
    }
}

// 消息路由器
pub struct MessageRouter {
    network_manager: Arc<AgentNetworkManager>,
    routing_table: Arc<RwLock<HashMap<u64, SocketAddr>>>,
}

impl MessageRouter {
    pub fn new(network_manager: Arc<AgentNetworkManager>) -> Self {
        Self {
            network_manager,
            routing_table: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn route_message(&self, message: DistributedMessage) -> Result<(), AgentDbError> {
        match message.to_node {
            Some(target_node) => {
                // 点对点消息
                self.send_to_node(target_node, message)
            }
            None => {
                // 广播消息
                self.broadcast_to_all(message)
            }
        }
    }

    fn send_to_node(&self, node_id: u64, message: DistributedMessage) -> Result<(), AgentDbError> {
        let routing_table = self.routing_table.read().unwrap();
        if let Some(_address) = routing_table.get(&node_id) {
            // 这里应该实际发送网络消息
            // 为了简化，我们只是记录消息
            println!("Routing message {} to node {}", message.id, node_id);
            Ok(())
        } else {
            Err(AgentDbError::NotFound(format!("Node {} not found in routing table", node_id)))
        }
    }

    fn broadcast_to_all(&self, message: DistributedMessage) -> Result<(), AgentDbError> {
        let nodes = self.network_manager.get_online_nodes();
        for node in nodes {
            if node.id != message.from_node {
                self.send_to_node(node.id, message.clone())?;
            }
        }
        Ok(())
    }

    pub fn update_routing_table(&self, node_id: u64, address: SocketAddr) {
        let mut table = self.routing_table.write().unwrap();
        table.insert(node_id, address);
    }

    pub fn remove_from_routing_table(&self, node_id: u64) {
        let mut table = self.routing_table.write().unwrap();
        table.remove(&node_id);
    }
}
