// 安全管理系统测试
const std = @import("std");
const testing = std.testing;
const print = std.debug.print;

// 权限枚举
const Permission = enum {
    Read,
    Write,
    Delete,
    Admin,
    Execute,
    CreateAgent,
    ModifyAgent,
    ViewMetrics,
    ManageUsers,
};

// 角色结构体
const Role = struct {
    role_id: []const u8,
    name: []const u8,
    description: []const u8,
    permissions: []const Permission,
    created_at: i64,
    updated_at: i64,
};

// 用户结构体
const User = struct {
    user_id: []const u8,
    username: []const u8,
    email: []const u8,
    password_hash: []const u8,
    salt: []const u8,
    roles: []const []const u8, // role_ids
    is_active: bool,
    last_login: ?i64,
    failed_login_attempts: u32,
    locked_until: ?i64,
    created_at: i64,
    updated_at: i64,
};

// 访问令牌结构体
const AccessToken = struct {
    token_id: []const u8,
    user_id: []const u8,
    token_hash: []const u8,
    expires_at: i64,
    scopes: []const []const u8,
    created_at: i64,
    last_used: ?i64,
};

// 审计日志结构体
const AuditLog = struct {
    log_id: []const u8,
    user_id: []const u8,
    action: []const u8,
    resource: []const u8,
    resource_id: ?[]const u8,
    ip_address: []const u8,
    user_agent: []const u8,
    success: bool,
    error_message: ?[]const u8,
    timestamp: i64,
};

// 密码策略结构体
const PasswordPolicy = struct {
    min_length: usize,
    require_uppercase: bool,
    require_lowercase: bool,
    require_numbers: bool,
    require_special_chars: bool,
    max_age_days: u32,
    history_count: usize,

    const default = PasswordPolicy{
        .min_length = 8,
        .require_uppercase = true,
        .require_lowercase = true,
        .require_numbers = true,
        .require_special_chars = true,
        .max_age_days = 90,
        .history_count = 5,
    };
};

// 模拟安全管理器
const MockSecurityManager = struct {
    users: std.HashMap([]const u8, User, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    roles: std.HashMap([]const u8, Role, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    tokens: std.HashMap([]const u8, AccessToken, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    audit_logs: std.ArrayList(AuditLog),
    password_policy: PasswordPolicy,
    max_failed_attempts: u32,
    lockout_duration_seconds: i64,
    session_timeout_seconds: i64,
    allocator: std.mem.Allocator,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator) Self {
        var roles = std.HashMap([]const u8, Role, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator);

        // 创建默认角色
        const admin_permissions = [_]Permission{ Permission.Read, Permission.Write, Permission.Delete, Permission.Admin, Permission.Execute, Permission.CreateAgent, Permission.ModifyAgent, Permission.ViewMetrics, Permission.ManageUsers };
        const user_permissions = [_]Permission{ Permission.Read, Permission.Write };
        const readonly_permissions = [_]Permission{Permission.Read};

        const admin_role = Role{
            .role_id = "admin",
            .name = "Administrator",
            .description = "Full system access",
            .permissions = &admin_permissions,
            .created_at = std.time.timestamp(),
            .updated_at = std.time.timestamp(),
        };

        const user_role = Role{
            .role_id = "user",
            .name = "Regular User",
            .description = "Basic read/write access",
            .permissions = &user_permissions,
            .created_at = std.time.timestamp(),
            .updated_at = std.time.timestamp(),
        };

        const readonly_role = Role{
            .role_id = "readonly",
            .name = "Read Only",
            .description = "Read-only access",
            .permissions = &readonly_permissions,
            .created_at = std.time.timestamp(),
            .updated_at = std.time.timestamp(),
        };

        roles.put("admin", admin_role) catch {};
        roles.put("user", user_role) catch {};
        roles.put("readonly", readonly_role) catch {};

        return Self{
            .users = std.HashMap([]const u8, User, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
            .roles = roles,
            .tokens = std.HashMap([]const u8, AccessToken, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
            .audit_logs = std.ArrayList(AuditLog).init(allocator),
            .password_policy = PasswordPolicy.default,
            .max_failed_attempts = 5,
            .lockout_duration_seconds = 900, // 15分钟
            .session_timeout_seconds = 3600, // 1小时
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Self) void {
        self.users.deinit();
        self.roles.deinit();
        self.tokens.deinit();
        self.audit_logs.deinit();
    }

    pub fn validatePassword(self: *const Self, password: []const u8) !void {
        if (password.len < self.password_policy.min_length) {
            return error.PasswordTooShort;
        }

        if (self.password_policy.require_uppercase) {
            var has_upper = false;
            for (password) |c| {
                if (c >= 'A' and c <= 'Z') {
                    has_upper = true;
                    break;
                }
            }
            if (!has_upper) return error.PasswordMissingUppercase;
        }

        if (self.password_policy.require_lowercase) {
            var has_lower = false;
            for (password) |c| {
                if (c >= 'a' and c <= 'z') {
                    has_lower = true;
                    break;
                }
            }
            if (!has_lower) return error.PasswordMissingLowercase;
        }

        if (self.password_policy.require_numbers) {
            var has_number = false;
            for (password) |c| {
                if (c >= '0' and c <= '9') {
                    has_number = true;
                    break;
                }
            }
            if (!has_number) return error.PasswordMissingNumber;
        }

        if (self.password_policy.require_special_chars) {
            var has_special = false;
            const special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            for (password) |c| {
                for (special_chars) |sc| {
                    if (c == sc) {
                        has_special = true;
                        break;
                    }
                }
                if (has_special) break;
            }
            if (!has_special) return error.PasswordMissingSpecialChar;
        }
    }

    pub fn hashPassword(self: *const Self, password: []const u8, salt: []const u8) []const u8 {
        // 简化的哈希实现（实际应用中应使用bcrypt或类似算法）
        var hasher = std.hash.Wyhash.init(0);
        hasher.update(password);
        hasher.update(salt);
        const hash_value = hasher.final();

        // 将哈希值转换为十六进制字符串
        const hash_str = self.allocator.alloc(u8, 16) catch return "";
        _ = std.fmt.bufPrint(hash_str, "{x:0>16}", .{hash_value}) catch return "";
        return hash_str;
    }

    pub fn generateSalt(self: *const Self) []const u8 {
        // 简化的盐生成
        const timestamp = std.time.timestamp();
        const salt = self.allocator.alloc(u8, 16) catch return "";
        _ = std.fmt.bufPrint(salt, "{x:0>16}", .{@as(u64, @intCast(timestamp))}) catch return "";
        return salt;
    }

    pub fn createUser(self: *Self, username: []const u8, email: []const u8, password: []const u8, role_ids: []const []const u8) ![]const u8 {
        // 验证密码策略
        try self.validatePassword(password);

        // 生成盐和密码哈希
        const salt = self.generateSalt();
        const password_hash = self.hashPassword(password, salt);

        const user_id = try std.fmt.allocPrint(self.allocator, "user_{d}", .{std.time.timestamp()});

        const user = User{
            .user_id = user_id,
            .username = username,
            .email = email,
            .password_hash = password_hash,
            .salt = salt,
            .roles = role_ids,
            .is_active = true,
            .last_login = null,
            .failed_login_attempts = 0,
            .locked_until = null,
            .created_at = std.time.timestamp(),
            .updated_at = std.time.timestamp(),
        };

        try self.users.put(user_id, user);

        // 记录审计日志
        self.logAction("system", "create_user", "user", user_id, "127.0.0.1", "system", true, null);

        return user_id;
    }

    pub fn authenticate(self: *Self, username: []const u8, password: []const u8, ip_address: []const u8, user_agent: []const u8) ![]const u8 {
        // 查找用户
        var user_entry = self.users.iterator();
        var found_user: ?*User = null;

        while (user_entry.next()) |entry| {
            if (std.mem.eql(u8, entry.value_ptr.username, username)) {
                found_user = entry.value_ptr;
                break;
            }
        }

        if (found_user == null) {
            return error.InvalidCredentials;
        }

        const user = found_user.?;

        // 检查账户是否被锁定
        if (user.locked_until) |locked_until| {
            if (std.time.timestamp() < locked_until) {
                self.logAction(user.user_id, "login", "user", user.user_id, ip_address, user_agent, false, "Account locked");
                return error.AccountLocked;
            } else {
                // 锁定期已过，重置失败次数
                user.locked_until = null;
                user.failed_login_attempts = 0;
            }
        }

        // 检查账户是否激活
        if (!user.is_active) {
            self.logAction(user.user_id, "login", "user", user.user_id, ip_address, user_agent, false, "Account inactive");
            return error.AccountInactive;
        }

        // 验证密码
        const computed_hash = self.hashPassword(password, user.salt);
        if (!std.mem.eql(u8, computed_hash, user.password_hash)) {
            user.failed_login_attempts += 1;

            // 检查是否需要锁定账户
            if (user.failed_login_attempts >= self.max_failed_attempts) {
                user.locked_until = std.time.timestamp() + self.lockout_duration_seconds;
                self.logAction(user.user_id, "login", "user", user.user_id, ip_address, user_agent, false, "Too many failed attempts");
                return error.TooManyFailedAttempts;
            }

            self.logAction(user.user_id, "login", "user", user.user_id, ip_address, user_agent, false, "Invalid password");
            return error.InvalidCredentials;
        }

        // 登录成功，重置失败次数
        user.failed_login_attempts = 0;
        user.last_login = std.time.timestamp();
        user.updated_at = std.time.timestamp();

        // 生成访问令牌
        const token = try self.generateAccessToken(user.user_id, user.roles);

        self.logAction(user.user_id, "login", "user", user.user_id, ip_address, user_agent, true, null);

        return token;
    }

    pub fn checkPermission(self: *const Self, user_id: []const u8, permission: Permission) !bool {
        const user = self.users.get(user_id) orelse return error.UserNotFound;

        for (user.roles) |role_id| {
            if (self.roles.get(role_id)) |role| {
                for (role.permissions) |perm| {
                    if (perm == permission) {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    fn generateAccessToken(self: *Self, user_id: []const u8, roles: []const []const u8) ![]const u8 {
        const token_id = try std.fmt.allocPrint(self.allocator, "token_{d}", .{std.time.timestamp()});
        const token = try std.fmt.allocPrint(self.allocator, "{s}:{d}", .{ token_id, std.time.timestamp() });
        const token_hash = self.hashPassword(token, "token_salt");

        const access_token = AccessToken{
            .token_id = token_id,
            .user_id = user_id,
            .token_hash = token_hash,
            .expires_at = std.time.timestamp() + self.session_timeout_seconds,
            .scopes = roles,
            .created_at = std.time.timestamp(),
            .last_used = null,
        };

        try self.tokens.put(token_id, access_token);

        return token;
    }

    fn logAction(self: *Self, user_id: []const u8, action: []const u8, resource: []const u8, resource_id: []const u8, ip_address: []const u8, user_agent: []const u8, success: bool, error_message: ?[]const u8) void {
        const log_id = std.fmt.allocPrint(self.allocator, "log_{d}", .{std.time.timestamp()}) catch return;

        const log = AuditLog{
            .log_id = log_id,
            .user_id = user_id,
            .action = action,
            .resource = resource,
            .resource_id = resource_id,
            .ip_address = ip_address,
            .user_agent = user_agent,
            .success = success,
            .error_message = error_message,
            .timestamp = std.time.timestamp(),
        };

        self.audit_logs.append(log) catch {};

        // 保持最近1000条日志
        if (self.audit_logs.items.len > 1000) {
            _ = self.audit_logs.orderedRemove(0);
        }
    }
};

// 测试密码策略验证
test "Password policy validation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var security_manager = MockSecurityManager.init(allocator);
    defer security_manager.deinit();

    // 测试有效密码
    try security_manager.validatePassword("Password123!");

    // 测试无效密码
    try testing.expectError(error.PasswordTooShort, security_manager.validatePassword("123"));
    try testing.expectError(error.PasswordMissingUppercase, security_manager.validatePassword("password123!"));
    try testing.expectError(error.PasswordMissingLowercase, security_manager.validatePassword("PASSWORD123!"));
    try testing.expectError(error.PasswordMissingNumber, security_manager.validatePassword("Password!"));
    try testing.expectError(error.PasswordMissingSpecialChar, security_manager.validatePassword("Password123"));

    print("✓ 密码策略验证测试通过\n", .{});
}

// 测试用户创建
test "User creation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var security_manager = MockSecurityManager.init(allocator);
    defer security_manager.deinit();

    const role_ids = [_][]const u8{"user"};
    const user_id = try security_manager.createUser("testuser", "test@example.com", "Password123!", &role_ids);

    try testing.expect(user_id.len > 0);

    const user = security_manager.users.get(user_id).?;
    try testing.expect(std.mem.eql(u8, user.username, "testuser"));
    try testing.expect(std.mem.eql(u8, user.email, "test@example.com"));
    try testing.expect(user.is_active);
    try testing.expect(user.failed_login_attempts == 0);

    print("✓ 用户创建测试通过\n", .{});
}

// 测试用户认证
test "User authentication" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var security_manager = MockSecurityManager.init(allocator);
    defer security_manager.deinit();

    // 创建测试用户
    const role_ids = [_][]const u8{"user"};
    _ = try security_manager.createUser("testuser", "test@example.com", "Password123!", &role_ids);

    // 测试成功认证
    const token = try security_manager.authenticate("testuser", "Password123!", "127.0.0.1", "test-agent");
    try testing.expect(token.len > 0);

    // 测试失败认证
    try testing.expectError(error.InvalidCredentials, security_manager.authenticate("testuser", "wrongpassword", "127.0.0.1", "test-agent"));
    try testing.expectError(error.InvalidCredentials, security_manager.authenticate("nonexistent", "Password123!", "127.0.0.1", "test-agent"));

    print("✓ 用户认证测试通过\n", .{});
}

// 测试权限检查
test "Permission checking" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var security_manager = MockSecurityManager.init(allocator);
    defer security_manager.deinit();

    // 创建不同角色的用户
    const admin_roles = [_][]const u8{"admin"};
    const user_roles = [_][]const u8{"user"};
    const readonly_roles = [_][]const u8{"readonly"};

    const admin_id = try security_manager.createUser("admin", "admin@example.com", "Password123!", &admin_roles);
    const user_id = try security_manager.createUser("user", "user@example.com", "Password123!", &user_roles);
    const readonly_id = try security_manager.createUser("readonly", "readonly@example.com", "Password123!", &readonly_roles);

    // 测试管理员权限
    try testing.expect(try security_manager.checkPermission(admin_id, Permission.Admin));
    try testing.expect(try security_manager.checkPermission(admin_id, Permission.Read));
    try testing.expect(try security_manager.checkPermission(admin_id, Permission.Write));
    try testing.expect(try security_manager.checkPermission(admin_id, Permission.Delete));

    // 测试普通用户权限
    try testing.expect(!try security_manager.checkPermission(user_id, Permission.Admin));
    try testing.expect(try security_manager.checkPermission(user_id, Permission.Read));
    try testing.expect(try security_manager.checkPermission(user_id, Permission.Write));
    try testing.expect(!try security_manager.checkPermission(user_id, Permission.Delete));

    // 测试只读用户权限
    try testing.expect(!try security_manager.checkPermission(readonly_id, Permission.Admin));
    try testing.expect(try security_manager.checkPermission(readonly_id, Permission.Read));
    try testing.expect(!try security_manager.checkPermission(readonly_id, Permission.Write));
    try testing.expect(!try security_manager.checkPermission(readonly_id, Permission.Delete));

    print("✓ 权限检查测试通过\n", .{});
}

// 测试账户锁定机制
test "Account lockout mechanism" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var security_manager = MockSecurityManager.init(allocator);
    defer security_manager.deinit();

    // 创建测试用户
    const role_ids = [_][]const u8{"user"};
    _ = try security_manager.createUser("testuser", "test@example.com", "Password123!", &role_ids);

    // 模拟多次失败登录
    var i: u32 = 0;
    while (i < security_manager.max_failed_attempts - 1) : (i += 1) {
        _ = security_manager.authenticate("testuser", "wrongpassword", "127.0.0.1", "test-agent") catch {};
    }

    // 最后一次失败登录应该锁定账户
    try testing.expectError(error.TooManyFailedAttempts, security_manager.authenticate("testuser", "wrongpassword", "127.0.0.1", "test-agent"));

    // 即使密码正确也应该被锁定
    try testing.expectError(error.AccountLocked, security_manager.authenticate("testuser", "Password123!", "127.0.0.1", "test-agent"));

    print("✓ 账户锁定机制测试通过\n", .{});
}

// 测试审计日志
test "Audit logging" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var security_manager = MockSecurityManager.init(allocator);
    defer security_manager.deinit();

    // 创建用户（应该生成审计日志）
    const role_ids = [_][]const u8{"user"};
    _ = try security_manager.createUser("testuser", "test@example.com", "Password123!", &role_ids);

    // 成功登录（应该生成审计日志）
    _ = try security_manager.authenticate("testuser", "Password123!", "127.0.0.1", "test-agent");

    // 失败登录（应该生成审计日志）
    _ = security_manager.authenticate("testuser", "wrongpassword", "127.0.0.1", "test-agent") catch {};

    // 检查审计日志
    try testing.expect(security_manager.audit_logs.items.len >= 3);

    // 检查日志内容
    var found_create = false;
    var found_success_login = false;
    var found_failed_login = false;

    for (security_manager.audit_logs.items) |log| {
        if (std.mem.eql(u8, log.action, "create_user")) {
            found_create = true;
        } else if (std.mem.eql(u8, log.action, "login") and log.success) {
            found_success_login = true;
        } else if (std.mem.eql(u8, log.action, "login") and !log.success) {
            found_failed_login = true;
        }
    }

    try testing.expect(found_create);
    try testing.expect(found_success_login);
    try testing.expect(found_failed_login);

    print("✓ 审计日志测试通过\n", .{});
}
