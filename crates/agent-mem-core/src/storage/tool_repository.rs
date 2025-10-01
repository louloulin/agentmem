//! Tool repository implementation

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};

use super::models::Tool;
use super::repository::Repository;
use crate::{CoreError, CoreResult};

/// Tool repository
pub struct ToolRepository {
    pool: PgPool,
}

impl ToolRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List tools by organization
    pub async fn list_by_organization(
        &self,
        organization_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<Tool>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Tool>(
            r#"
            SELECT * FROM tools
            WHERE organization_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(organization_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list tools: {}", e)))?;

        Ok(results)
    }

    /// Find tool by name
    pub async fn find_by_name(
        &self,
        organization_id: &str,
        name: &str,
    ) -> CoreResult<Option<Tool>> {
        let result = sqlx::query_as::<_, Tool>(
            r#"
            SELECT * FROM tools
            WHERE organization_id = $1 AND name = $2 AND is_deleted = FALSE
            "#,
        )
        .bind(organization_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to find tool by name: {}", e)))?;

        Ok(result)
    }

    /// List tools by tags
    pub async fn list_by_tags(
        &self,
        organization_id: &str,
        tags: &[String],
        match_all: bool,
    ) -> CoreResult<Vec<Tool>> {
        let query = if match_all {
            // Match all tags (AND)
            r#"
            SELECT * FROM tools
            WHERE organization_id = $1 
              AND is_deleted = FALSE
              AND tags @> $2::jsonb
            ORDER BY created_at DESC
            "#
        } else {
            // Match any tag (OR)
            r#"
            SELECT * FROM tools
            WHERE organization_id = $1 
              AND is_deleted = FALSE
              AND tags ?| $2
            ORDER BY created_at DESC
            "#
        };

        let tags_json = serde_json::to_value(tags).map_err(|e| {
            CoreError::SerializationError(format!("Failed to serialize tags: {}", e))
        })?;

        let results = sqlx::query_as::<_, Tool>(query)
            .bind(organization_id)
            .bind(tags_json)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to list tools by tags: {}", e)))?;

        Ok(results)
    }

    /// Get tools for an agent
    pub async fn list_by_agent(&self, agent_id: &str) -> CoreResult<Vec<Tool>> {
        let results = sqlx::query_as::<_, Tool>(
            r#"
            SELECT t.* FROM tools t
            INNER JOIN tools_agents ta ON t.id = ta.tool_id
            WHERE ta.agent_id = $1 AND t.is_deleted = FALSE
            ORDER BY t.name
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list tools by agent: {}", e)))?;

        Ok(results)
    }

    /// Add a tool to an agent
    pub async fn add_to_agent(&self, tool_id: &str, agent_id: &str) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tools_agents (tool_id, agent_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(tool_id)
        .bind(agent_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to add tool to agent: {}", e)))?;

        Ok(())
    }

    /// Remove a tool from an agent
    pub async fn remove_from_agent(&self, tool_id: &str, agent_id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM tools_agents
            WHERE tool_id = $1 AND agent_id = $2
            "#,
        )
        .bind(tool_id)
        .bind(agent_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to remove tool from agent: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Batch create tools
    pub async fn batch_create(&self, tools: &[Tool]) -> CoreResult<Vec<Tool>> {
        let mut created_tools = Vec::new();

        for tool in tools {
            let created = self.create(tool).await?;
            created_tools.push(created);
        }

        Ok(created_tools)
    }
}

#[async_trait]
impl Repository<Tool> for ToolRepository {
    async fn create(&self, tool: &Tool) -> CoreResult<Tool> {
        let result = sqlx::query_as::<_, Tool>(
            r#"
            INSERT INTO tools (
                id, organization_id, name, description,
                json_schema, source_type, source_code, tags,
                metadata_, created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(&tool.id)
        .bind(&tool.organization_id)
        .bind(&tool.name)
        .bind(&tool.description)
        .bind(&tool.json_schema)
        .bind(&tool.source_type)
        .bind(&tool.source_code)
        .bind(&tool.tags)
        .bind(&tool.metadata_)
        .bind(&tool.created_at)
        .bind(&tool.updated_at)
        .bind(&tool.is_deleted)
        .bind(&tool.created_by_id)
        .bind(&tool.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create tool: {}", e)))?;

        Ok(result)
    }

    async fn read(&self, id: &str) -> CoreResult<Option<Tool>> {
        let result = sqlx::query_as::<_, Tool>(
            r#"
            SELECT * FROM tools
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to read tool: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, tool: &Tool) -> CoreResult<Tool> {
        let result = sqlx::query_as::<_, Tool>(
            r#"
            UPDATE tools
            SET name = $2, description = $3, json_schema = $4,
                source_type = $5, source_code = $6, tags = $7,
                metadata_ = $8, updated_at = $9, last_updated_by_id = $10
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING *
            "#,
        )
        .bind(&tool.id)
        .bind(&tool.name)
        .bind(&tool.description)
        .bind(&tool.json_schema)
        .bind(&tool.source_type)
        .bind(&tool.source_code)
        .bind(&tool.tags)
        .bind(&tool.metadata_)
        .bind(Utc::now())
        .bind(&tool.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update tool: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE tools
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete tool: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn hard_delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM tools WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to hard delete tool: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<Tool>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Tool>(
            r#"
            SELECT * FROM tools
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list tools: {}", e)))?;

        Ok(results)
    }

    async fn count(&self) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM tools
            WHERE is_deleted = FALSE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count tools: {}", e)))?;

        Ok(result.try_get("count").unwrap_or(0))
    }
}
