//! Chroma向量存储实现

use agent_mem_traits::{
    AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Chroma API请求结构
#[derive(Debug, Serialize)]
struct ChromaAddRequest {
    ids: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    metadatas: Option<Vec<serde_json::Value>>,
}

/// Chroma查询请求结构
#[derive(Debug, Serialize)]
struct ChromaQueryRequest {
    query_embeddings: Vec<Vec<f32>>,
    n_results: usize,
    #[serde(rename = "where")]
    where_clause: Option<serde_json::Value>,
}

/// Chroma查询响应结构
#[derive(Debug, Deserialize)]
struct ChromaQueryResponse {
    ids: Vec<Vec<String>>,
    embeddings: Option<Vec<Vec<Vec<f32>>>>,
    metadatas: Option<Vec<Vec<Option<serde_json::Value>>>>,
    distances: Vec<Vec<f32>>,
}

/// Chroma向量存储实现
pub struct ChromaStore {
    config: VectorStoreConfig,
    client: Client,
    base_url: String,
    collection_name: String,
}

impl ChromaStore {
    /// 创建新的Chroma存储实例
    pub async fn new(config: VectorStoreConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {}", e))
            })?;

        let base_url = config
            .url
            .clone()
            .unwrap_or_else(|| "http://localhost:8000".to_string());

        let collection_name = config
            .collection_name
            .clone()
            .unwrap_or_else(|| "default".to_string());

        Ok(Self {
            config,
            client,
            base_url,
            collection_name,
        })
    }

    /// 获取集合URL
    fn get_collection_url(&self) -> String {
        format!(
            "{}/api/v1/collections/{}",
            self.base_url, self.collection_name
        )
    }
}

#[async_trait]
impl VectorStore for ChromaStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = vectors.iter().map(|v| v.vector.clone()).collect();
        let metadatas: Vec<serde_json::Value> = vectors
            .iter()
            .map(|v| serde_json::to_value(&v.metadata).unwrap_or(serde_json::Value::Null))
            .collect();

        let request = ChromaAddRequest {
            ids: ids.clone(),
            embeddings,
            metadatas: Some(metadatas),
        };

        let url = format!("{}/add", self.get_collection_url());
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {}: {}",
                status, error_text
            )));
        }

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        _threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let request = ChromaQueryRequest {
            query_embeddings: vec![query_vector],
            n_results: limit,
            where_clause: None,
        };

        let url = format!("{}/query", self.get_collection_url());
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {}: {}",
                status, error_text
            )));
        }

        let chroma_response: ChromaQueryResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {}", e))
        })?;

        let mut results = Vec::new();

        if let (Some(ids), Some(distances)) =
            (chroma_response.ids.get(0), chroma_response.distances.get(0))
        {
            for (i, (id, distance)) in ids.iter().zip(distances.iter()).enumerate() {
                let vector = chroma_response
                    .embeddings
                    .as_ref()
                    .and_then(|embs| embs.get(0))
                    .and_then(|emb| emb.get(i))
                    .cloned()
                    .unwrap_or_default();

                let metadata = chroma_response
                    .metadatas
                    .as_ref()
                    .and_then(|metas| metas.get(0))
                    .and_then(|meta| meta.get(i))
                    .and_then(|m| m.as_ref())
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                // Chroma返回距离，我们需要转换为相似度
                let similarity = 1.0 / (1.0 + distance);

                results.push(VectorSearchResult {
                    id: id.clone(),
                    vector,
                    metadata,
                    similarity,
                    distance: *distance,
                });
            }
        }

        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        let url = format!("{}/delete", self.get_collection_url());
        let request = serde_json::json!({
            "ids": ids
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {}: {}",
                status, error_text
            )));
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = vectors.iter().map(|v| v.vector.clone()).collect();
        let metadatas: Vec<serde_json::Value> = vectors
            .iter()
            .map(|v| serde_json::to_value(&v.metadata).unwrap_or(serde_json::Value::Null))
            .collect();

        let request = serde_json::json!({
            "ids": ids,
            "embeddings": embeddings,
            "metadatas": metadatas
        });

        let url = format!("{}/update", self.get_collection_url());
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {}: {}",
                status, error_text
            )));
        }

        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let url = format!("{}/get", self.get_collection_url());
        let request = serde_json::json!({
            "ids": [id],
            "include": ["embeddings", "metadatas"]
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {}: {}",
                status, error_text
            )));
        }

        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {}", e))
        })?;

        // 解析响应并构建VectorData
        if let (Some(ids), Some(embeddings)) = (
            response_data["ids"].as_array(),
            response_data["embeddings"].as_array(),
        ) {
            if !ids.is_empty() && !embeddings.is_empty() {
                let vector: Vec<f32> = embeddings[0]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                let metadata = response_data["metadatas"]
                    .as_array()
                    .and_then(|metas| metas.get(0))
                    .and_then(|meta| serde_json::from_value(meta.clone()).ok())
                    .unwrap_or_default();

                return Ok(Some(VectorData {
                    id: id.to_string(),
                    vector,
                    metadata,
                }));
            }
        }

        Ok(None)
    }

    async fn count_vectors(&self) -> Result<usize> {
        let url = format!("{}/count", self.get_collection_url());
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {}: {}",
                status, error_text
            )));
        }

        let count: usize = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {}", e))
        })?;

        Ok(count)
    }

    async fn clear(&self) -> Result<()> {
        // Chroma没有直接的清空API，我们需要删除整个集合然后重新创建
        Err(AgentMemError::llm_error(
            "Clear operation not supported for Chroma",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chroma_store_creation() {
        let config = VectorStoreConfig {
            provider: "chroma".to_string(),
            url: Some("http://localhost:8000".to_string()),
            collection_name: Some("test".to_string()),
            ..Default::default()
        };

        let store = ChromaStore::new(config).await;
        assert!(store.is_ok());
    }

    #[test]
    fn test_get_collection_url() {
        let config = VectorStoreConfig {
            provider: "chroma".to_string(),
            url: Some("http://localhost:8000".to_string()),
            collection_name: Some("test".to_string()),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let store = rt.block_on(ChromaStore::new(config)).unwrap();

        assert_eq!(
            store.get_collection_url(),
            "http://localhost:8000/api/v1/collections/test"
        );
    }
}
