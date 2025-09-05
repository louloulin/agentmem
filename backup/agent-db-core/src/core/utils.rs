// 工具函数模块
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::AgentDbError;

// 文本处理工具
pub mod text {
    use super::*;

    // 文本清理
    pub fn clean_text(text: &str) -> String {
        text.trim()
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    // 文本分词
    pub fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect()
            })
            .filter(|word: &String| !word.is_empty())
            .collect()
    }

    // 计算文本相似性（Jaccard相似性）
    pub fn jaccard_similarity(text1: &str, text2: &str) -> f32 {
        let tokens1: std::collections::HashSet<String> = tokenize(text1).into_iter().collect();
        let tokens2: std::collections::HashSet<String> = tokenize(text2).into_iter().collect();

        if tokens1.is_empty() && tokens2.is_empty() {
            return 1.0;
        }

        let intersection = tokens1.intersection(&tokens2).count() as f32;
        let union = tokens1.union(&tokens2).count() as f32;

        if union == 0.0 {
            0.0
        } else {
            intersection / union
        }
    }

    // 提取关键词
    pub fn extract_keywords(text: &str, max_keywords: usize) -> Vec<String> {
        let tokens = tokenize(text);
        let mut word_freq: HashMap<String, usize> = HashMap::new();

        for token in tokens {
            if token.len() > 2 { // 过滤短词
                *word_freq.entry(token).or_insert(0) += 1;
            }
        }

        let mut sorted_words: Vec<(String, usize)> = word_freq.into_iter().collect();
        sorted_words.sort_by(|a, b| b.1.cmp(&a.1));

        sorted_words
            .into_iter()
            .take(max_keywords)
            .map(|(word, _)| word)
            .collect()
    }

    // 计算文本摘要（简单版本）
    pub fn summarize(text: &str, max_sentences: usize) -> String {
        let sentences: Vec<&str> = text.split('.').collect();
        if sentences.len() <= max_sentences {
            return text.to_string();
        }

        // 简单地取前几句
        sentences
            .into_iter()
            .take(max_sentences)
            .collect::<Vec<&str>>()
            .join(".")
            + "."
    }
}

// 向量计算工具
pub mod vector {
    use super::*;

    // 向量归一化
    pub fn normalize(vector: &mut [f32]) {
        let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in vector.iter_mut() {
                *x /= norm;
            }
        }
    }

    // 向量点积
    pub fn dot_product(a: &[f32], b: &[f32]) -> Result<f32, AgentDbError> {
        if a.len() != b.len() {
            return Err(AgentDbError::InvalidArgument(
                "Vectors must have the same dimension".to_string(),
            ));
        }
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
    }

    // 向量加法
    pub fn add(a: &[f32], b: &[f32]) -> Result<Vec<f32>, AgentDbError> {
        if a.len() != b.len() {
            return Err(AgentDbError::InvalidArgument(
                "Vectors must have the same dimension".to_string(),
            ));
        }
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x + y).collect())
    }

    // 向量减法
    pub fn subtract(a: &[f32], b: &[f32]) -> Result<Vec<f32>, AgentDbError> {
        if a.len() != b.len() {
            return Err(AgentDbError::InvalidArgument(
                "Vectors must have the same dimension".to_string(),
            ));
        }
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x - y).collect())
    }

    // 向量标量乘法
    pub fn scalar_multiply(vector: &[f32], scalar: f32) -> Vec<f32> {
        vector.iter().map(|x| x * scalar).collect()
    }

    // 计算向量的L2范数
    pub fn l2_norm(vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    // 计算向量的L1范数
    pub fn l1_norm(vector: &[f32]) -> f32 {
        vector.iter().map(|x| x.abs()).sum()
    }

    // 生成随机向量
    pub fn random_vector(dimension: usize) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..dimension).map(|_| rng.gen::<f32>()).collect()
    }

    // 向量量化（简单版本）
    pub fn quantize(vector: &[f32], levels: usize) -> Vec<u8> {
        let min_val = vector.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = vector.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let range = max_val - min_val;

        if range == 0.0 {
            return vec![0; vector.len()];
        }

        vector
            .iter()
            .map(|&x| {
                let normalized = (x - min_val) / range;
                let quantized = (normalized * (levels - 1) as f32).round() as u8;
                quantized.min((levels - 1) as u8)
            })
            .collect()
    }
}

// 时间处理工具
pub mod time {
    use super::*;

    // 获取当前时间戳（秒）
    pub fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    // 获取当前时间戳（毫秒）
    pub fn current_timestamp_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    // 时间戳转换为可读字符串
    pub fn timestamp_to_string(timestamp: i64) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    // 计算时间差（天数）
    pub fn days_between(timestamp1: i64, timestamp2: i64) -> f64 {
        let diff = (timestamp2 - timestamp1).abs() as f64;
        diff / (24.0 * 3600.0)
    }

    // 检查时间戳是否过期
    pub fn is_expired(timestamp: i64, ttl_seconds: i64) -> bool {
        let current = current_timestamp();
        current - timestamp > ttl_seconds
    }
}

// 序列化工具
pub mod serialization {
    use super::*;
    use serde::{Deserialize, Serialize};

    // JSON序列化
    pub fn to_json<T: Serialize>(data: &T) -> Result<String, AgentDbError> {
        serde_json::to_string(data).map_err(AgentDbError::Serde)
    }

    // JSON反序列化
    pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, AgentDbError> {
        serde_json::from_str(json).map_err(AgentDbError::Serde)
    }

    // 二进制序列化（使用JSON作为简单实现）
    pub fn to_binary<T: Serialize>(data: &T) -> Result<Vec<u8>, AgentDbError> {
        let json_str = serde_json::to_string(data).map_err(AgentDbError::Serde)?;
        Ok(json_str.into_bytes())
    }

    // 二进制反序列化
    pub fn from_binary<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T, AgentDbError> {
        let json_str = String::from_utf8(data.to_vec())
            .map_err(|e| AgentDbError::Internal(format!("UTF-8 conversion error: {}", e)))?;
        serde_json::from_str(&json_str).map_err(AgentDbError::Serde)
    }

    // 压缩数据
    pub fn compress(data: &[u8]) -> Result<Vec<u8>, AgentDbError> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| AgentDbError::Internal(format!("Compression error: {}", e)))?;
        encoder.finish()
            .map_err(|e| AgentDbError::Internal(format!("Compression finish error: {}", e)))
    }

    // 解压数据
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>, AgentDbError> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| AgentDbError::Internal(format!("Decompression error: {}", e)))?;
        Ok(decompressed)
    }
}

// 哈希工具
pub mod hash {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    // 计算字符串哈希
    pub fn hash_string(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    // 计算字节数组哈希
    pub fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    // 生成UUID字符串
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    // 生成短ID（基于时间戳和随机数）
    pub fn generate_short_id() -> String {
        use rand::Rng;
        let timestamp = time::current_timestamp_ms();
        let random: u32 = rand::thread_rng().gen();
        format!("{:x}{:x}", timestamp, random)
    }
}

// 验证工具
pub mod validation {
    use super::*;

    // 验证向量维度
    pub fn validate_vector_dimension(vector: &[f32], expected_dim: usize) -> Result<(), AgentDbError> {
        if vector.len() != expected_dim {
            return Err(AgentDbError::InvalidArgument(
                format!("Vector dimension {} does not match expected {}", vector.len(), expected_dim)
            ));
        }
        Ok(())
    }

    // 验证相似性阈值
    pub fn validate_similarity_threshold(threshold: f32) -> Result<(), AgentDbError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(AgentDbError::InvalidArgument(
                "Similarity threshold must be between 0.0 and 1.0".to_string()
            ));
        }
        Ok(())
    }

    // 验证Agent ID
    pub fn validate_agent_id(agent_id: u64) -> Result<(), AgentDbError> {
        if agent_id == 0 {
            return Err(AgentDbError::InvalidArgument(
                "Agent ID must be greater than 0".to_string()
            ));
        }
        Ok(())
    }

    // 验证内容长度
    pub fn validate_content_length(content: &str, max_length: usize) -> Result<(), AgentDbError> {
        if content.len() > max_length {
            return Err(AgentDbError::InvalidArgument(
                format!("Content length {} exceeds maximum {}", content.len(), max_length)
            ));
        }
        Ok(())
    }
}
