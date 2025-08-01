// 安全管理模块
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 移除未使用的导入

// 用户结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: HashSet<String>,
    pub permissions: HashSet<Permission>,
    pub created_at: i64,
    pub last_login: Option<i64>,
    pub is_active: bool,
}

// 角色结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
    pub description: String,
}

// 权限枚举
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ReadAgentState,
    WriteAgentState,
    DeleteAgentState,
    ReadMemory,
    WriteMemory,
    DeleteMemory,
    ReadDocument,
    WriteDocument,
    DeleteDocument,
    ManageUsers,
    ManageRoles,
    ViewMetrics,
    AdminAccess,
}

// 访问令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: i64,
    pub permissions: HashSet<Permission>,
}

// 安全管理器
pub struct SecurityManager {
    users: HashMap<Uuid, User>,
    roles: HashMap<String, Role>,
    sessions: HashMap<String, AccessToken>,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    pub fn create_user(&mut self, username: String, email: String, password_hash: String, roles: HashSet<String>, permissions: HashSet<Permission>) -> Result<Uuid, String> {
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            username,
            email,
            password_hash,
            roles,
            permissions,
            created_at: chrono::Utc::now().timestamp(),
            last_login: None,
            is_active: true,
        };
        
        self.users.insert(user_id, user);
        Ok(user_id)
    }

    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<String, String> {
        // 查找用户
        for (user_id, user) in &self.users {
            if user.username == username && user.password_hash == password {
                // 创建会话令牌
                let token = Uuid::new_v4().to_string();
                let access_token = AccessToken {
                    token: token.clone(),
                    user_id: *user_id,
                    expires_at: chrono::Utc::now().timestamp() + 3600, // 1小时过期
                    permissions: user.permissions.clone(),
                };
                self.sessions.insert(token.clone(), access_token);
                return Ok(token);
            }
        }
        Err("Invalid credentials".to_string())
    }

    pub fn check_permission(&self, session_token: &str, permission: Permission) -> bool {
        if let Some(access_token) = self.sessions.get(session_token) {
            // 检查令牌是否过期
            if access_token.expires_at < chrono::Utc::now().timestamp() {
                return false;
            }
            // 检查权限
            access_token.permissions.contains(&permission)
        } else {
            false
        }
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}
