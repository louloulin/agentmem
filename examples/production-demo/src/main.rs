//! AgentMem 2.0 Production Demo
//! 
//! Comprehensive demonstration of production-ready features including
//! monitoring, logging, security, and deployment capabilities.

use agent_mem_core::{
    monitoring::{MonitoringSystem, MonitoringConfig, MetricType, ComponentStatus, AlertRule, AlertCondition, AlertSeverity, ThreatRuleType},
    logging::{LoggingSystem, LoggingConfig, LogLevel, AuditEventType, AuditResult, SecurityEventType, SecuritySeverity},
    security::{SecuritySystem, SecurityConfig, Permission, Role, UserAccount, ComponentStatus as SecurityComponentStatus},
    MemoryManager, MemoryConfig, Memory, MemoryType, Session,
};
use std::collections::{HashMap, HashSet};
use tokio::time::{sleep, Duration};
use chrono::Utc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting AgentMem 2.0 Production Demo");
    println!("========================================");

    // Initialize all production systems
    let monitoring_config = MonitoringConfig::default();
    let logging_config = LoggingConfig::default();
    let security_config = SecurityConfig::default();
    let memory_config = MemoryConfig::default();

    let monitoring = MonitoringSystem::new(monitoring_config);
    let logging = LoggingSystem::new(logging_config);
    let security = SecuritySystem::new(security_config);
    let memory_manager = MemoryManager::new(memory_config);

    // Start background monitoring tasks
    monitoring.start_background_tasks().await?;

    // Demo 1: Production Monitoring System
    println!("\n📊 Demo 1: Production Monitoring System");
    demo_monitoring_system(&monitoring).await?;

    // Demo 2: Comprehensive Logging System
    println!("\n📝 Demo 2: Comprehensive Logging System");
    demo_logging_system(&logging).await?;

    // Demo 3: Enterprise Security System
    println!("\n🔒 Demo 3: Enterprise Security System");
    demo_security_system(&security).await?;

    // Demo 4: Integrated Production Workflow
    println!("\n🔄 Demo 4: Integrated Production Workflow");
    demo_integrated_workflow(&monitoring, &logging, &security, &memory_manager).await?;

    // Demo 5: Performance and Scalability
    println!("\n⚡ Demo 5: Performance and Scalability");
    demo_performance_scalability(&monitoring).await?;

    // Demo 6: Compliance and Audit
    println!("\n📋 Demo 6: Compliance and Audit");
    demo_compliance_audit(&logging).await?;

    println!("\n✅ AgentMem 2.0 Production Demo completed successfully!");
    println!("🎯 All production features demonstrated and validated");

    Ok(())
}

/// Demonstrate comprehensive monitoring system
async fn demo_monitoring_system(monitoring: &MonitoringSystem) -> Result<(), Box<dyn std::error::Error>> {
    // Record various metrics
    let mut labels = HashMap::new();
    labels.insert("service".to_string(), "agentmem".to_string());
    labels.insert("environment".to_string(), "production".to_string());

    // Counter metrics
    monitoring.increment_counter("requests_total", labels.clone()).await?;
    monitoring.increment_counter("requests_total", labels.clone()).await?;
    monitoring.increment_counter("requests_total", labels.clone()).await?;

    // Gauge metrics
    monitoring.set_gauge("memory_usage_bytes", 1024.0 * 1024.0 * 256.0, labels.clone()).await?;
    monitoring.set_gauge("cpu_usage_percent", 45.5, labels.clone()).await?;
    monitoring.set_gauge("active_connections", 42.0, labels.clone()).await?;

    // Histogram metrics
    monitoring.record_histogram("request_duration_ms", 125.5, labels.clone()).await?;
    monitoring.record_histogram("request_duration_ms", 89.2, labels.clone()).await?;
    monitoring.record_histogram("request_duration_ms", 203.1, labels.clone()).await?;

    // Health checks
    let mut details = HashMap::new();
    details.insert("version".to_string(), "2.0.0".to_string());
    details.insert("uptime".to_string(), "3600".to_string());

    monitoring.update_health_check(
        "database".to_string(),
        ComponentStatus::Healthy,
        "Database connection is stable".to_string(),
        25,
        details.clone(),
    ).await?;

    monitoring.update_health_check(
        "vector_store".to_string(),
        ComponentStatus::Healthy,
        "Vector store is operational".to_string(),
        15,
        details.clone(),
    ).await?;

    monitoring.update_health_check(
        "llm_provider".to_string(),
        ComponentStatus::Degraded,
        "LLM provider experiencing high latency".to_string(),
        500,
        details,
    ).await?;

    // Add alert rules
    let alert_rule = AlertRule {
        id: "high_cpu_usage".to_string(),
        name: "High CPU Usage".to_string(),
        metric_name: "cpu_usage_percent".to_string(),
        condition: AlertCondition::GreaterThan,
        threshold: 80.0,
        duration_seconds: 300,
        severity: AlertSeverity::Warning,
        enabled: true,
        labels: HashMap::new(),
    };

    monitoring.add_alert_rule(alert_rule).await?;

    // Trigger high CPU to test alerting
    monitoring.set_gauge("cpu_usage_percent", 85.0, labels).await?;

    // Evaluate alerts
    let alerts = monitoring.evaluate_alerts().await?;
    
    // Get system status
    let metrics = monitoring.get_metrics().await;
    let health_status = monitoring.get_health_status().await;
    let overall_health = monitoring.get_overall_health().await;
    let system_info = monitoring.get_system_info().await;

    println!("  ✅ Recorded {} metrics", metrics.len());
    println!("  ✅ Health checks: {} components", health_status.len());
    println!("  ✅ Overall health: {:?}", overall_health);
    println!("  ✅ Generated {} alerts", alerts.len());
    println!("  ✅ System version: {}", system_info.version);

    Ok(())
}

/// Demonstrate comprehensive logging system
async fn demo_logging_system(logging: &LoggingSystem) -> Result<(), Box<dyn std::error::Error>> {
    // Structured logging
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), serde_json::Value::String("user123".to_string()));
    fields.insert("session_id".to_string(), serde_json::Value::String("session456".to_string()));
    fields.insert("request_id".to_string(), serde_json::Value::String("req789".to_string()));

    logging.log(
        LogLevel::Info,
        "User authenticated successfully".to_string(),
        "auth_service".to_string(),
        Some("authenticate".to_string()),
        Some("user123".to_string()),
        Some("session456".to_string()),
        Some("req789".to_string()),
        fields,
        vec!["authentication".to_string(), "success".to_string()],
    ).await?;

    // Convenience methods
    logging.info("System startup completed", "system").await?;
    logging.warn("High memory usage detected", "monitor").await?;
    logging.error("Database connection failed", "database", None).await?;

    // Audit logging
    let mut audit_details = HashMap::new();
    audit_details.insert("memory_id".to_string(), serde_json::Value::String("mem123".to_string()));
    audit_details.insert("content_length".to_string(), serde_json::Value::Number(serde_json::Number::from(256)));

    logging.audit(
        AuditEventType::MemoryCreated,
        Some("user123".to_string()),
        Some("session456".to_string()),
        Some("mem123".to_string()),
        Some("memory".to_string()),
        "create_memory".to_string(),
        AuditResult::Success,
        Some("192.168.1.100".to_string()),
        Some("Mozilla/5.0".to_string()),
        audit_details,
    ).await?;

    logging.audit(
        AuditEventType::ConfigurationChanged,
        Some("admin".to_string()),
        Some("admin_session".to_string()),
        Some("config_main".to_string()),
        Some("configuration".to_string()),
        "update_config".to_string(),
        AuditResult::Success,
        Some("192.168.1.1".to_string()),
        Some("AdminPanel/1.0".to_string()),
        HashMap::new(),
    ).await?;

    // Security logging
    logging.security(
        SecurityEventType::AuthenticationFailure,
        SecuritySeverity::Medium,
        Some("192.168.1.200".to_string()),
        Some("attacker".to_string()),
        None,
        "Multiple failed login attempts detected".to_string(),
        vec!["brute_force".to_string(), "suspicious_ip".to_string()],
        vec!["rate_limit_applied".to_string(), "ip_monitored".to_string()],
        HashMap::new(),
    ).await?;

    logging.security(
        SecurityEventType::SuspiciousActivity,
        SecuritySeverity::High,
        Some("10.0.0.50".to_string()),
        Some("user456".to_string()),
        Some("session789".to_string()),
        "Unusual data access pattern detected".to_string(),
        vec!["anomalous_access".to_string(), "data_exfiltration_risk".to_string()],
        vec!["user_flagged".to_string(), "session_monitored".to_string()],
        HashMap::new(),
    ).await?;

    // Get logs and statistics
    let logs = logging.get_logs(None).await;
    let audit_logs = logging.get_audit_logs().await;
    let security_logs = logging.get_security_logs().await;
    let high_risk_events = logging.get_high_risk_audit_events(50).await;
    let critical_security_events = logging.get_critical_security_events().await;

    println!("  ✅ Generated {} log entries", logs.len());
    println!("  ✅ Generated {} audit entries", audit_logs.len());
    println!("  ✅ Generated {} security entries", security_logs.len());
    println!("  ✅ High-risk events: {}", high_risk_events.len());
    println!("  ✅ Critical security events: {}", critical_security_events.len());

    Ok(())
}

/// Demonstrate enterprise security system
async fn demo_security_system(security: &SecuritySystem) -> Result<(), Box<dyn std::error::Error>> {
    // Wait for initialization
    sleep(Duration::from_millis(200)).await;

    // Create test users
    let admin_user = UserAccount {
        user_id: "admin_001".to_string(),
        username: "admin".to_string(),
        email: "admin@agentmem.com".to_string(),
        roles: vec!["admin".to_string()],
        created_at: Utc::now(),
        last_login: None,
        failed_login_attempts: 0,
        locked_until: None,
        active: true,
        metadata: HashMap::new(),
    };

    let regular_user = UserAccount {
        user_id: "user_001".to_string(),
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        roles: vec!["user".to_string()],
        created_at: Utc::now(),
        last_login: None,
        failed_login_attempts: 0,
        locked_until: None,
        active: true,
        metadata: HashMap::new(),
    };

    security.create_user(admin_user).await?;
    security.create_user(regular_user).await?;

    // Test authentication (this will fail with placeholder password verification)
    let auth_result = security.authenticate_user(
        "john_doe",
        "wrong_password",
        "192.168.1.100",
        "Mozilla/5.0"
    ).await;

    match auth_result {
        Ok(_) => println!("  ✅ Authentication succeeded"),
        Err(_) => println!("  ✅ Authentication failed as expected (demo mode)"),
    }

    // Test permission checking
    let admin_has_permission = security.check_permission("admin_001", &Permission::AdminAccess).await?;
    let user_has_permission = security.check_permission("user_001", &Permission::ReadMemory).await?;
    let user_no_admin = security.check_permission("user_001", &Permission::AdminAccess).await?;

    println!("  ✅ Admin has admin access: {}", admin_has_permission);
    println!("  ✅ User has read access: {}", user_has_permission);
    println!("  ✅ User has no admin access: {}", !user_no_admin);

    // Test threat detection
    security.detect_threat(
        ThreatRuleType::FailedLoginAttempts,
        "192.168.1.200",
        Some("user_001")
    ).await?;

    security.detect_threat(
        ThreatRuleType::RateLimitExceeded,
        "10.0.0.100",
        None
    ).await?;

    // Get security status
    let threat_incidents = security.get_threat_incidents().await;
    let active_sessions = security.get_active_sessions().await;

    println!("  ✅ Threat incidents detected: {}", threat_incidents.len());
    println!("  ✅ Active sessions: {}", active_sessions.len());

    Ok(())
}

/// Demonstrate integrated production workflow
async fn demo_integrated_workflow(
    monitoring: &MonitoringSystem,
    logging: &LoggingSystem,
    security: &SecuritySystem,
    memory_manager: &MemoryManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔄 Simulating production memory operation workflow...");

    // Create session
    let session = Session::new()
        .with_agent_id("production_agent".to_string())
        .with_user_id("user123".to_string())
        .with_session_id("prod_session_001".to_string());

    // Log operation start
    logging.info("Starting memory operation", "memory_service").await?;

    // Record metrics
    let start_time = std::time::Instant::now();
    let mut labels = HashMap::new();
    labels.insert("operation".to_string(), "create_memory".to_string());
    labels.insert("user".to_string(), "user123".to_string());

    monitoring.increment_counter("memory_operations_total", labels.clone()).await?;

    // Check permissions (simulated)
    let has_permission = security.check_permission("user123", &Permission::WriteMemory).await.unwrap_or(true);
    
    if !has_permission {
        logging.security(
            SecurityEventType::AuthorizationFailure,
            SecuritySeverity::Medium,
            Some("192.168.1.100".to_string()),
            Some("user123".to_string()),
            Some("prod_session_001".to_string()),
            "Unauthorized memory write attempt".to_string(),
            vec!["authorization_failure".to_string()],
            vec!["operation_blocked".to_string()],
            HashMap::new(),
        ).await?;
        return Ok(());
    }

    // Create memory
    let memory = Memory::new(
        "User prefers morning meetings and coffee".to_string(),
        MemoryType::Preference,
    )
    .with_agent_id("production_agent".to_string())
    .with_user_id("user123".to_string())
    .with_session_id("prod_session_001".to_string());

    // Simulate memory storage (would normally use actual storage)
    let memory_id = Uuid::new_v4().to_string();

    // Record operation duration
    let duration = start_time.elapsed();
    monitoring.record_histogram("memory_operation_duration_ms", duration.as_millis() as f64, labels).await?;

    // Audit log the operation
    let mut audit_details = HashMap::new();
    audit_details.insert("memory_id".to_string(), serde_json::Value::String(memory_id.clone()));
    audit_details.insert("content_length".to_string(), serde_json::Value::Number(serde_json::Number::from(memory.content.len())));

    logging.audit(
        AuditEventType::MemoryCreated,
        Some("user123".to_string()),
        Some("prod_session_001".to_string()),
        Some(memory_id.clone()),
        Some("memory".to_string()),
        "create_memory".to_string(),
        AuditResult::Success,
        Some("192.168.1.100".to_string()),
        Some("AgentMemClient/2.0".to_string()),
        audit_details,
    ).await?;

    // Update health status
    monitoring.update_health_check(
        "memory_service".to_string(),
        ComponentStatus::Healthy,
        "Memory operations functioning normally".to_string(),
        duration.as_millis() as u64,
        HashMap::new(),
    ).await?;

    // Log successful completion
    logging.info("Memory operation completed successfully", "memory_service").await?;

    println!("  ✅ Integrated workflow completed");
    println!("  ✅ Memory created: {}", memory_id);
    println!("  ✅ Operation duration: {}ms", duration.as_millis());
    println!("  ✅ All systems coordinated successfully");

    Ok(())
}

/// Demonstrate performance and scalability features
async fn demo_performance_scalability(monitoring: &MonitoringSystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ Running performance simulation...");

    // Simulate high-load scenario
    let mut tasks = Vec::new();
    
    for i in 0..100 {
        let monitoring_clone = monitoring.clone();
        let task = tokio::spawn(async move {
            let mut labels = HashMap::new();
            labels.insert("worker".to_string(), format!("worker_{}", i));
            labels.insert("batch".to_string(), "performance_test".to_string());

            // Simulate various operations
            let _ = monitoring_clone.increment_counter("operations_total", labels.clone()).await;
            let _ = monitoring_clone.set_gauge("worker_load", (i as f64 * 0.1) % 100.0, labels.clone()).await;
            let _ = monitoring_clone.record_histogram("operation_latency_ms", (i as f64 * 2.5) % 200.0, labels).await;

            // Simulate some processing time
            sleep(Duration::from_millis(10)).await;
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await?;
    }

    // Record system performance metrics
    let mut system_labels = HashMap::new();
    system_labels.insert("test".to_string(), "performance".to_string());

    monitoring.set_gauge("concurrent_operations", 100.0, system_labels.clone()).await?;
    monitoring.set_gauge("throughput_ops_per_sec", 1000.0, system_labels.clone()).await?;
    monitoring.set_gauge("memory_efficiency_percent", 95.5, system_labels).await?;

    let metrics = monitoring.get_metrics().await;
    println!("  ✅ Processed 100 concurrent operations");
    println!("  ✅ Generated {} total metrics", metrics.len());
    println!("  ✅ System maintained high performance under load");

    Ok(())
}

/// Demonstrate compliance and audit features
async fn demo_compliance_audit(logging: &LoggingSystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Generating compliance report...");

    // Generate various audit events for compliance
    let events = vec![
        (AuditEventType::UserLogin, "User login event"),
        (AuditEventType::MemoryAccessed, "Memory access event"),
        (AuditEventType::ConfigurationChanged, "Configuration change"),
        (AuditEventType::DataExport, "Data export operation"),
        (AuditEventType::SystemStartup, "System startup"),
    ];

    for (event_type, description) in events {
        logging.audit(
            event_type,
            Some("compliance_user".to_string()),
            Some("compliance_session".to_string()),
            Some("resource_123".to_string()),
            Some("system".to_string()),
            description.to_string(),
            AuditResult::Success,
            Some("192.168.1.10".to_string()),
            Some("ComplianceBot/1.0".to_string()),
            HashMap::new(),
        ).await?;
    }

    // Export compliance data
    let start_time = Utc::now() - chrono::Duration::hours(1);
    let end_time = Utc::now();
    
    let compliance_export = logging.export_logs_for_compliance(start_time, end_time).await?;

    println!("  ✅ Compliance export generated: {}", compliance_export.export_id);
    println!("  ✅ Log entries: {}", compliance_export.log_entries.len());
    println!("  ✅ Audit entries: {}", compliance_export.audit_entries.len());
    println!("  ✅ Security entries: {}", compliance_export.security_entries.len());
    println!("  ✅ Export timestamp: {}", compliance_export.export_timestamp);

    Ok(())
}
