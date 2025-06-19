// 安全和权限管理模块
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::Aead, KeyInit};
use rand::{Rng, thread_rng};

use crate::core::AgentDbError;

// 权限级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    Execute,
    Modify,
}

// 用户角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
    pub description: String,
}

impl Role {
    pub fn new(name: String, permissions: HashSet<Permission>, description: String) -> Self {
        Self {
            name,
            permissions,
            description,
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission) || self.permissions.contains(&Permission::Admin)
    }
}

// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

impl User {
    pub fn new(username: String, password: &str) -> Self {
        let password_hash = hash_password(password);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            password_hash,
            roles: Vec::new(),
            created_at,
            last_login: None,
            is_active: true,
            metadata: HashMap::new(),
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify_password(password, &self.password_hash)
    }

    pub fn update_last_login(&mut self) {
        self.last_login = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }
}

// 访问令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub user_id: String,
    pub expires_at: u64,
    pub permissions: HashSet<Permission>,
    pub metadata: HashMap<String, String>,
}

impl AccessToken {
    pub fn new(user_id: String, permissions: HashSet<Permission>, duration_seconds: u64) -> Self {
        let token = generate_token();
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + duration_seconds;

        Self {
            token,
            user_id,
            expires_at,
            permissions,
            metadata: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        !self.is_expired() && 
        (self.permissions.contains(permission) || self.permissions.contains(&Permission::Admin))
    }
}

// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl AuditLogEntry {
    pub fn new(
        user_id: String,
        action: String,
        resource: String,
        success: bool,
        error_message: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            action,
            resource,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ip_address: None,
            user_agent: None,
            success,
            error_message,
            metadata: HashMap::new(),
        }
    }
}

// 安全管理器
pub struct SecurityManager {
    users: HashMap<String, User>,
    roles: HashMap<String, Role>,
    tokens: HashMap<String, AccessToken>,
    audit_logs: Vec<AuditLogEntry>,
    encryption_key: Option<[u8; 32]>,
}

impl SecurityManager {
    pub fn new() -> Self {
        let mut manager = Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            tokens: HashMap::new(),
            audit_logs: Vec::new(),
            encryption_key: None,
        };

        // 创建默认角色
        manager.create_default_roles();
        manager
    }

    fn create_default_roles(&mut self) {
        // 管理员角色
        let admin_permissions = [
            Permission::Read,
            Permission::Write,
            Permission::Delete,
            Permission::Admin,
            Permission::Execute,
            Permission::Modify,
        ].iter().cloned().collect();
        
        let admin_role = Role::new(
            "admin".to_string(),
            admin_permissions,
            "Administrator with full access".to_string(),
        );
        self.roles.insert("admin".to_string(), admin_role);

        // 用户角色
        let user_permissions = [Permission::Read, Permission::Write].iter().cloned().collect();
        let user_role = Role::new(
            "user".to_string(),
            user_permissions,
            "Regular user with read/write access".to_string(),
        );
        self.roles.insert("user".to_string(), user_role);

        // 只读角色
        let readonly_permissions = [Permission::Read].iter().cloned().collect();
        let readonly_role = Role::new(
            "readonly".to_string(),
            readonly_permissions,
            "Read-only access".to_string(),
        );
        self.roles.insert("readonly".to_string(), readonly_role);
    }

    pub fn create_user(&mut self, username: String, password: &str, roles: Vec<String>) -> Result<String, AgentDbError> {
        if self.users.values().any(|u| u.username == username) {
            return Err(AgentDbError::InvalidArgument("Username already exists".to_string()));
        }

        // 验证角色是否存在
        for role_name in &roles {
            if !self.roles.contains_key(role_name) {
                return Err(AgentDbError::InvalidArgument(format!("Role '{}' does not exist", role_name)));
            }
        }

        let mut user = User::new(username.clone(), password);
        user.roles = roles;
        let user_id = user.id.clone();

        self.users.insert(user_id.clone(), user);
        
        self.log_audit(
            user_id.clone(),
            "create_user".to_string(),
            format!("user:{}", username),
            true,
            None,
        );

        Ok(user_id)
    }

    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<AccessToken, AgentDbError> {
        let user_id = {
            let user = self.users.values_mut()
                .find(|u| u.username == username && u.is_active)
                .ok_or_else(|| AgentDbError::Unauthorized("Invalid credentials".to_string()))?;

            if !user.verify_password(password) {
                let user_id = user.id.clone();
                self.log_audit(
                    user_id,
                    "authenticate".to_string(),
                    format!("user:{}", username),
                    false,
                    Some("Invalid password".to_string()),
                );
                return Err(AgentDbError::Unauthorized("Invalid credentials".to_string()));
            }

            user.update_last_login();
            user.id.clone()
        };

        // 收集用户权限
        let mut permissions = HashSet::new();
        if let Some(user) = self.users.get(&user_id) {
            for role_name in &user.roles {
                if let Some(role) = self.roles.get(role_name) {
                    permissions.extend(role.permissions.iter().cloned());
                }
            }
        }

        let token = AccessToken::new(user_id.clone(), permissions, 3600); // 1小时有效期
        let token_str = token.token.clone();
        self.tokens.insert(token_str.clone(), token.clone());

        self.log_audit(
            user_id,
            "authenticate".to_string(),
            format!("user:{}", username),
            true,
            None,
        );

        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<&AccessToken, AgentDbError> {
        let access_token = self.tokens.get(token)
            .ok_or_else(|| AgentDbError::Unauthorized("Invalid token".to_string()))?;

        if access_token.is_expired() {
            return Err(AgentDbError::Unauthorized("Token expired".to_string()));
        }

        Ok(access_token)
    }

    pub fn check_permission(&self, token: &str, permission: Permission) -> Result<(), AgentDbError> {
        let access_token = self.validate_token(token)?;
        
        if !access_token.has_permission(&permission) {
            return Err(AgentDbError::Unauthorized(format!("Insufficient permissions: {:?}", permission)));
        }

        Ok(())
    }

    pub fn revoke_token(&mut self, token: &str) -> Result<(), AgentDbError> {
        if let Some(access_token) = self.tokens.remove(token) {
            self.log_audit(
                access_token.user_id,
                "revoke_token".to_string(),
                format!("token:{}", token),
                true,
                None,
            );
        }
        Ok(())
    }

    pub fn set_encryption_key(&mut self, key: [u8; 32]) {
        self.encryption_key = Some(key);
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, AgentDbError> {
        let key = self.encryption_key
            .ok_or_else(|| AgentDbError::Internal("Encryption key not set".to_string()))?;

        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let nonce_bytes: [u8; 12] = thread_rng().gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| AgentDbError::Internal(format!("Encryption failed: {}", e)))?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, AgentDbError> {
        if encrypted_data.len() < 12 {
            return Err(AgentDbError::Internal("Invalid encrypted data".to_string()));
        }

        let key = self.encryption_key
            .ok_or_else(|| AgentDbError::Internal("Encryption key not set".to_string()))?;

        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| AgentDbError::Internal(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    fn log_audit(&mut self, user_id: String, action: String, resource: String, success: bool, error_message: Option<String>) {
        let entry = AuditLogEntry::new(user_id, action, resource, success, error_message);
        self.audit_logs.push(entry);
    }

    pub fn get_audit_logs(&self, limit: Option<usize>) -> Vec<&AuditLogEntry> {
        let logs: Vec<&AuditLogEntry> = self.audit_logs.iter().collect();
        if let Some(limit) = limit {
            logs.into_iter().rev().take(limit).collect()
        } else {
            logs.into_iter().rev().collect()
        }
    }
}

// 密码哈希函数
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

// 密码验证函数
fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

// 生成随机令牌
fn generate_token() -> String {
    let mut rng = thread_rng();
    let token_bytes: [u8; 32] = rng.gen();
    hex::encode(token_bytes)
}
