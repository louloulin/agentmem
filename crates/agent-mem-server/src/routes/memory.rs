//! Memory management routes

use crate::{
    error::{ServerError, ServerResult},
    models::{
        MemoryRequest, MemoryResponse, SearchRequest, SearchResponse,
        BatchRequest, BatchResponse, UpdateMemoryRequest
    },
};
use agent_mem_core::MemoryManager;
use agent_mem_core::MemoryType;
use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use tracing::{info, error};
use utoipa;

/// Add a new memory
#[utoipa::path(
    post,
    path = "/api/v1/memories",
    tag = "memory",
    request_body = MemoryRequest,
    responses(
        (status = 201, description = "Memory created successfully", body = MemoryResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<MemoryRequest>,
) -> ServerResult<(StatusCode, Json<MemoryResponse>)> {
    info!("Adding new memory for agent_id: {:?}, user_id: {:?}", 
          request.agent_id, request.user_id);
    
    let memory_type = request.memory_type.unwrap_or(MemoryType::Episodic);
    let importance = request.importance.unwrap_or(0.5);
    
    let memory_id = memory_manager
        .add_memory(
            request.agent_id,
            request.user_id,
            request.content,
            Some(memory_type),
            Some(importance),
            request.metadata,
        )
        .await
        .map_err(|e| {
            error!("Failed to add memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;
    
    let response = MemoryResponse {
        id: memory_id,
        message: "Memory added successfully".to_string(),
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a memory by ID
#[utoipa::path(
    get,
    path = "/api/v1/memories/{id}",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    responses(
        (status = 200, description = "Memory retrieved successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Getting memory with ID: {}", id);
    
    let memory = memory_manager
        .get_memory(&id)
        .await
        .map_err(|e| {
            error!("Failed to get memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;
    
    match memory {
        Some(mem) => {
            let response = serde_json::json!({
                "id": mem.id,
                "agent_id": mem.agent_id,
                "user_id": mem.user_id,
                "content": mem.content,
                "memory_type": mem.memory_type,
                "importance": mem.importance,
                "created_at": mem.created_at,
                "updated_at": mem.created_at, // Using created_at as updated_at for now
                "metadata": mem.metadata
            });
            Ok(Json(response))
        }
        None => Err(ServerError::NotFound("Memory not found".to_string())),
    }
}

/// Update a memory
#[utoipa::path(
    put,
    path = "/api/v1/memories/{id}",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    request_body = UpdateMemoryRequest,
    responses(
        (status = 200, description = "Memory updated successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateMemoryRequest>,
) -> ServerResult<Json<MemoryResponse>> {
    info!("Updating memory with ID: {}", id);
    
    memory_manager
        .update_memory(&id, request.content, request.importance, None)
        .await
        .map_err(|e| {
            error!("Failed to update memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;
    
    let response = MemoryResponse {
        id,
        message: "Memory updated successfully".to_string(),
    };
    
    Ok(Json(response))
}

/// Delete a memory
#[utoipa::path(
    delete,
    path = "/api/v1/memories/{id}",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    responses(
        (status = 200, description = "Memory deleted successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
) -> ServerResult<Json<MemoryResponse>> {
    info!("Deleting memory with ID: {}", id);
    
    memory_manager
        .delete_memory(&id)
        .await
        .map_err(|e| {
            error!("Failed to delete memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;
    
    let response = MemoryResponse {
        id,
        message: "Memory deleted successfully".to_string(),
    };
    
    Ok(Json(response))
}

/// Search memories
#[utoipa::path(
    post,
    path = "/api/v1/memories/search",
    tag = "memory",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search completed successfully", body = SearchResponse),
        (status = 400, description = "Invalid search request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<SearchRequest>,
) -> ServerResult<Json<SearchResponse>> {
    info!("Searching memories with query: {}", request.query);
    
    let query = agent_mem_core::MemoryQuery {
        agent_id: request.agent_id.unwrap_or_default(),
        user_id: request.user_id,
        text_query: Some(request.query),
        vector_query: None,
        memory_type: request.memory_type,
        min_importance: request.threshold,
        max_age_seconds: None,
        limit: request.limit.unwrap_or(10),
    };
    
    let results = memory_manager
        .search_memories(query)
        .await
        .map_err(|e| {
            error!("Failed to search memories: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;
    
    let total = results.len();
    let response = SearchResponse {
        results: results.into_iter().map(|r| serde_json::json!({
            "id": r.memory.id,
            "content": r.memory.content,
            "score": r.score,
            "agent_id": r.memory.agent_id,
            "user_id": r.memory.user_id,
            "memory_type": r.memory.memory_type,
            "importance": r.memory.importance,
            "created_at": r.memory.created_at,
            "metadata": r.memory.metadata
        })).collect(),
        total,
    };
    
    Ok(Json(response))
}

/// Get memory history
#[utoipa::path(
    get,
    path = "/api/v1/memories/{id}/history",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    responses(
        (status = 200, description = "Memory history retrieved successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_history(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Getting history for memory ID: {}", id);
    
    // TODO: Implement memory history functionality
    let response = serde_json::json!({
        "memory_id": id,
        "history": [],
        "message": "Memory history feature not yet implemented"
    });
    
    Ok(Json(response))
}

/// Batch add memories
#[utoipa::path(
    post,
    path = "/api/v1/memories/batch",
    tag = "batch",
    request_body = BatchRequest,
    responses(
        (status = 201, description = "Batch operation completed", body = BatchResponse),
        (status = 400, description = "Invalid batch request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_add_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<BatchRequest>,
) -> ServerResult<(StatusCode, Json<BatchResponse>)> {
    info!("Batch adding {} memories", request.memories.len());
    
    let mut results = Vec::new();
    let mut errors = Vec::new();
    
    for memory_req in request.memories {
        let memory_type = memory_req.memory_type.unwrap_or(MemoryType::Episodic);
        let importance = memory_req.importance.unwrap_or(0.5);
        
        match memory_manager
            .add_memory(
                memory_req.agent_id,
                memory_req.user_id,
                memory_req.content,
                Some(memory_type),
                Some(importance),
                memory_req.metadata,
            )
            .await
        {
            Ok(id) => results.push(id),
            Err(e) => errors.push(e.to_string()),
        }
    }
    
    let response = BatchResponse {
        successful: results.len(),
        failed: errors.len(),
        results,
        errors,
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Batch delete memories
#[utoipa::path(
    post,
    path = "/api/v1/memories/batch/delete",
    tag = "batch",
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Batch delete completed", body = BatchResponse),
        (status = 400, description = "Invalid batch request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_delete_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(ids): Json<Vec<String>>,
) -> ServerResult<Json<BatchResponse>> {
    info!("Batch deleting {} memories", ids.len());
    
    let mut successful = 0;
    let mut errors = Vec::new();
    
    for id in &ids {
        match memory_manager.delete_memory(id).await {
            Ok(_) => successful += 1,
            Err(e) => errors.push(format!("Failed to delete {}: {}", id, e)),
        }
    }
    
    let response = BatchResponse {
        successful,
        failed: errors.len(),
        results: vec![], // No results for delete operations
        errors,
    };
    
    Ok(Json(response))
}
