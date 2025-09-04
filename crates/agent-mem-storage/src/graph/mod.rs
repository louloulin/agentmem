//! 图存储模块
//! 
//! 提供图数据库集成，支持实体关系存储和图查询

pub mod factory;
pub mod neo4j;
pub mod memgraph;

pub use factory::GraphStoreFactory;
pub use neo4j::Neo4jStore;
pub use memgraph::MemgraphStore;

// 重新导出常用类型
pub use agent_mem_traits::{GraphStore, Entity, Relation, Session, GraphResult, Result};
