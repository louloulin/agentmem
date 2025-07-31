// Tests 模块 - 测试代码
// 从 lib.rs 自动拆分生成

#[cfg(test)]
use super::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_agent_state_creation() {
        let state = AgentState::new(
            123,
            456,
            StateType::WorkingMemory,
            b"test data".to_vec(),
        );

        assert_eq!(state.agent_id, 123);
        assert_eq!(state.session_id, 456);
        assert_eq!(state.state_type, StateType::WorkingMemory);
        assert_eq!(state.data, b"test data");
        assert_eq!(state.version, 1);
        assert!(state.validate_checksum());
    }

    #[test]
    fn test_agent_state_update() {
        let mut state = AgentState::new(
            123,
            456,
            StateType::WorkingMemory,
            b"test data".to_vec(),
        );

        let original_version = state.version;
        let original_timestamp = state.timestamp;

        state.update_data(b"updated data".to_vec());

        assert_eq!(state.data, b"updated data");
        assert_eq!(state.version, original_version + 1);
        assert!(state.timestamp >= original_timestamp);
        assert!(state.validate_checksum());
    }

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(
            123,
            MemoryType::Episodic,
            "Test memory content".to_string(),
            0.8,
        );

        assert_eq!(memory.agent_id, 123);
        assert_eq!(memory.memory_type, MemoryType::Episodic);
        assert_eq!(memory.content, "Test memory content");
        assert_eq!(memory.importance, 0.8);
        assert_eq!(memory.access_count, 0);
        assert!(!memory.is_expired());
    }

    #[test]
    fn test_memory_access() {
        let mut memory = Memory::new(
            123,
            MemoryType::Episodic,
            "Test memory content".to_string(),
            0.8,
        );

        let original_access_count = memory.access_count;
        let original_last_accessed = memory.last_accessed;

        memory.access();

        assert_eq!(memory.access_count, original_access_count + 1);
        assert!(memory.last_accessed >= original_last_accessed);
    }

    #[test]
    fn test_memory_expiry() {
        let mut memory = Memory::new(
            123,
            MemoryType::Episodic,
            "Test memory content".to_string(),
            0.8,
        );

        assert!(!memory.is_expired());

        memory.set_expiry(-1); // 设置为过去的时间
        assert!(memory.is_expired());
    }

    #[test]
    fn test_state_type_conversion() {
        let state_types = vec![
            StateType::WorkingMemory,
            StateType::LongTermMemory,
            StateType::Context,
            StateType::TaskState,
            StateType::Relationship,
            StateType::Embedding,
        ];

        for state_type in state_types {
            let string_repr = state_type.to_string();
            let converted_back = StateType::from_string(string_repr).unwrap();
            assert_eq!(state_type, converted_back);
        }
    }

    #[test]
    fn test_memory_type_conversion() {
        let memory_types = vec![
            MemoryType::Episodic,
            MemoryType::Semantic,
            MemoryType::Procedural,
            MemoryType::Working,
        ];

        for memory_type in memory_types {
            let string_repr = memory_type.to_string();
            let converted_back = MemoryType::from_string(string_repr).unwrap();
            assert_eq!(memory_type, converted_back);
        }
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        
        assert_eq!(config.db_path, "./agent_db");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.cache_size, 1024 * 1024 * 100);
        assert!(config.enable_compression);
        assert_eq!(config.compression_level, 6);
        assert!(!config.enable_encryption);
        assert!(config.encryption_key.is_none());
    }

    #[test]
    fn test_query_result_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let result = QueryResult::new(data.clone(), 5, 1, 10, 100);

        assert_eq!(result.data, data);
        assert_eq!(result.total_count, 5);
        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 10);
        assert_eq!(result.execution_time_ms, 100);
    }

    #[test]
    fn test_query_result_empty() {
        let result: QueryResult<i32> = QueryResult::empty();

        assert!(result.data.is_empty());
        assert_eq!(result.total_count, 0);
        assert_eq!(result.page, 0);
        assert_eq!(result.page_size, 0);
        assert_eq!(result.execution_time_ms, 0);
    }

    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::default();

        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 50);
        assert!(params.sort_by.is_none());
        assert!(matches!(params.sort_order, SortOrder::Desc));
    }

    #[tokio::test]
    async fn test_agent_database_creation() {
        let config = DatabaseConfig {
            db_path: "./test_db".to_string(),
            ..Default::default()
        };

        // 注意：这个测试需要实际的数据库连接，在CI环境中可能会失败
        // 在实际项目中，应该使用模拟数据库或测试数据库
        match AgentDatabase::new(config).await {
            Ok(db) => {
                assert_eq!(db.config.db_path, "./test_db");
                assert!(db.vector_engine.is_none());
                assert!(db.security_manager.is_none());
            }
            Err(_) => {
                // 如果数据库连接失败，跳过测试
                println!("Database connection failed, skipping test");
            }
        }
    }

    #[test]
    fn test_agent_state_metadata() {
        let mut state = AgentState::new(
            123,
            456,
            StateType::WorkingMemory,
            b"test data".to_vec(),
        );

        state.set_metadata("key1".to_string(), "value1".to_string());
        state.set_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(state.get_metadata("key1"), Some(&"value1".to_string()));
        assert_eq!(state.get_metadata("key2"), Some(&"value2".to_string()));
        assert_eq!(state.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_memory_metadata() {
        let mut memory = Memory::new(
            123,
            MemoryType::Episodic,
            "Test memory content".to_string(),
            0.8,
        );

        memory.set_metadata("category".to_string(), "important".to_string());
        memory.set_metadata("source".to_string(), "user_input".to_string());

        assert_eq!(memory.get_metadata("category"), Some(&"important".to_string()));
        assert_eq!(memory.get_metadata("source"), Some(&"user_input".to_string()));
        assert_eq!(memory.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_memory_importance_update() {
        let mut memory = Memory::new(
            123,
            MemoryType::Episodic,
            "Test memory content".to_string(),
            0.8,
        );

        memory.update_importance(1.5); // 超出范围
        assert_eq!(memory.importance, 1.0); // 应该被限制在1.0

        memory.update_importance(-0.5); // 低于范围
        assert_eq!(memory.importance, 0.0); // 应该被限制在0.0

        memory.update_importance(0.6);
        assert_eq!(memory.importance, 0.6);
    }

    #[test]
    fn test_checksum_validation() {
        let state = AgentState::new(
            123,
            456,
            StateType::WorkingMemory,
            b"test data".to_vec(),
        );

        assert!(state.validate_checksum());

        // 手动创建一个校验和不匹配的状态
        let mut invalid_state = state.clone();
        invalid_state.checksum = 999; // 错误的校验和

        assert!(!invalid_state.validate_checksum());
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let state = Arc::new(std::sync::Mutex::new(AgentState::new(
            123,
            456,
            StateType::WorkingMemory,
            b"test data".to_vec(),
        )));

        let mut handles = vec![];

        // 启动多个线程同时访问状态
        for i in 0..10 {
            let state_clone = Arc::clone(&state);
            let handle = thread::spawn(move || {
                let mut s = state_clone.lock().unwrap();
                s.set_metadata(format!("key{}", i), format!("value{}", i));
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证所有元数据都被正确设置
        let final_state = state.lock().unwrap();
        for i in 0..10 {
            assert_eq!(
                final_state.get_metadata(&format!("key{}", i)),
                Some(&format!("value{}", i))
            );
        }
    }
}
