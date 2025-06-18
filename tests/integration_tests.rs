// 集成测试
use agent_state_db_rust::{
    AgentDB, AgentState, Memory, MemoryType, StateType,
    CacheManager, MonitoringManager, LogLevel, AgentDbConfig,
};
use tempfile::TempDir;

// 测试辅助函数
async fn create_test_db() -> (AgentDB, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let db = AgentDB::new(db_path.to_str().unwrap(), 384).await.unwrap();
    (db, temp_dir)
}

#[tokio::test]
async fn test_basic_integration() {
    let (db, _temp_dir) = create_test_db().await;

    // 1. 创建Agent状态
    let agent_id = 1001u64;
    let state = AgentState::new(agent_id, 1, StateType::WorkingMemory, vec![1, 2, 3, 4, 5]);

    // 保存状态
    db.save_agent_state(&state).await.unwrap();

    // 加载状态
    let loaded_state = db.load_agent_state(agent_id).await.unwrap();
    assert!(loaded_state.is_some());
    assert_eq!(loaded_state.unwrap().data, vec![1, 2, 3, 4, 5]);

    // 2. 添加记忆
    let memory1 = Memory::new(agent_id, MemoryType::Episodic, "重要事件1".to_string(), 0.9);

    db.store_memory(&memory1).await.unwrap();

    // 检索记忆
    let memories = db.get_agent_memories(agent_id, None, 10).await.unwrap();
    assert_eq!(memories.len(), 1);
}

#[tokio::test]
async fn test_config_and_monitoring() {
    let config = AgentDbConfig::default();

    // 测试配置
    assert_eq!(config.vector.dimension, 384);
    assert!(config.validate().is_ok());

    // 测试监控
    let monitor = MonitoringManager::new(config.logging);
    monitor.log(LogLevel::Info, "test", "测试消息", None);

    let logs = monitor.get_logs(None, Some(1));
    assert_eq!(logs.len(), 1);
}





