//! Metrics and monitoring routes

use crate::{error::ServerResult, models::MetricsResponse};
use agent_mem_core::MemoryManager;
use axum::{extract::Extension, response::Json};
use chrono::Utc;
use std::sync::Arc;
use utoipa;

/// Get system metrics
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "health",
    responses(
        (status = 200, description = "Metrics retrieved successfully", body = MetricsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_metrics(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
) -> ServerResult<Json<MetricsResponse>> {
    // Get memory statistics
    let stats = memory_manager
        .get_memory_stats(None)
        .await
        .map_err(|e| crate::error::ServerError::MemoryError(e.to_string()))?;

    let mut metrics = std::collections::HashMap::new();

    // Memory metrics
    metrics.insert("total_memories".to_string(), stats.total_memories as f64);

    // Extract memory counts by type
    let episodic_count = stats
        .memories_by_type
        .get(&agent_mem_core::MemoryType::Episodic)
        .unwrap_or(&0);
    let semantic_count = stats
        .memories_by_type
        .get(&agent_mem_core::MemoryType::Semantic)
        .unwrap_or(&0);
    let procedural_count = stats
        .memories_by_type
        .get(&agent_mem_core::MemoryType::Procedural)
        .unwrap_or(&0);

    metrics.insert("episodic_memories".to_string(), *episodic_count as f64);
    metrics.insert("semantic_memories".to_string(), *semantic_count as f64);
    metrics.insert("procedural_memories".to_string(), *procedural_count as f64);
    metrics.insert(
        "average_importance".to_string(),
        stats.average_importance as f64,
    );

    // System metrics (would be expanded with actual system monitoring)
    metrics.insert("uptime_seconds".to_string(), 0.0); // Placeholder
    metrics.insert("memory_usage_bytes".to_string(), 0.0); // Placeholder
    metrics.insert("cpu_usage_percent".to_string(), 0.0); // Placeholder

    let response = MetricsResponse {
        timestamp: Utc::now(),
        metrics,
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_core::MemoryManager;

    #[tokio::test]
    async fn test_get_metrics() {
        let memory_manager = Arc::new(MemoryManager::new());
        let result = get_metrics(Extension(memory_manager)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.metrics.contains_key("total_memories"));
    }
}
