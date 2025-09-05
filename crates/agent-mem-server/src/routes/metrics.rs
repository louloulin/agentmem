//! Metrics and monitoring routes

use crate::{error::ServerResult, models::MetricsResponse};
use crate::routes::memory::MemoryManager;
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

    // Memory metrics - extract from JSON response
    let total_memories = stats.get("total_memories")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as f64;
    metrics.insert("total_memories".to_string(), total_memories);

    // Extract memory counts by type from nested object
    if let Some(memory_types) = stats.get("memory_types").and_then(|v| v.as_object()) {
        let episodic_count = memory_types.get("episodic")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as f64;
        let semantic_count = memory_types.get("semantic")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as f64;
        let procedural_count = memory_types.get("procedural")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as f64;

        metrics.insert("episodic_memories".to_string(), episodic_count);
        metrics.insert("semantic_memories".to_string(), semantic_count);
        metrics.insert("procedural_memories".to_string(), procedural_count);
    }

    let average_importance = stats.get("average_importance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    metrics.insert("average_importance".to_string(), average_importance);

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
    use crate::routes::memory::MemoryManager;

    #[tokio::test]
    async fn test_get_metrics() {
        let memory_manager = Arc::new(MemoryManager::new());
        let result = get_metrics(Extension(memory_manager)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.metrics.contains_key("total_memories"));
    }
}
