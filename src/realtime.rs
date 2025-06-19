// Realtime 模块 - 实时数据流处理
// 实时数据流处理系统

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use crate::core::*;

// 流数据类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StreamDataType {
    AgentState,
    Memory,
    Document,
    Vector,
    Event,
    Metric,
}

// 流数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDataItem {
    pub agent_id: u64,
    pub data_type: StreamDataType,
    pub payload: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub timestamp: i64,
    pub priority: u8,
}

impl StreamDataItem {
    pub fn new(agent_id: u64, data_type: StreamDataType, payload: Vec<u8>, metadata: HashMap<String, String>) -> Self {
        Self {
            agent_id,
            data_type,
            payload,
            metadata,
            timestamp: chrono::Utc::now().timestamp(),
            priority: 128, // 默认中等优先级
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn is_high_priority(&self) -> bool {
        self.priority > 200
    }
}

// 流处理配置
#[derive(Debug, Clone)]
pub struct StreamProcessingConfig {
    pub buffer_size: usize,
    pub batch_size: usize,
    pub worker_threads: usize,
    pub batch_timeout: Duration,
    pub max_retries: u32,
}

impl Default for StreamProcessingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            batch_size: 100,
            worker_threads: 4,
            batch_timeout: Duration::from_millis(100),
            max_retries: 3,
        }
    }
}

// 流处理统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProcessingStats {
    pub items_received: u64,
    pub items_processed: u64,
    pub items_dropped: u64,
    pub batches_processed: u64,
    pub avg_latency_ms: f64,
    pub max_latency_ms: u64,
    pub throughput_per_sec: f64,
    pub buffer_utilization: f64,
    pub error_count: u64,
    pub last_update: i64,
}

impl Default for StreamProcessingStats {
    fn default() -> Self {
        Self {
            items_received: 0,
            items_processed: 0,
            items_dropped: 0,
            batches_processed: 0,
            avg_latency_ms: 0.0,
            max_latency_ms: 0,
            throughput_per_sec: 0.0,
            buffer_utilization: 0.0,
            error_count: 0,
            last_update: chrono::Utc::now().timestamp(),
        }
    }
}

// 流数据处理器特征
pub trait StreamProcessor: Send + Sync {
    fn process_item(&self, item: &StreamDataItem) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn process_batch(&self, items: &[StreamDataItem]) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn get_processor_name(&self) -> &str;
}

// 实时流处理器
pub struct RealTimeStreamProcessor {
    config: StreamProcessingConfig,
    processors: HashMap<StreamDataType, Arc<dyn StreamProcessor>>,
    sender: mpsc::Sender<StreamDataItem>,
    receiver: Arc<Mutex<mpsc::Receiver<StreamDataItem>>>,
    buffer: Arc<Mutex<VecDeque<StreamDataItem>>>,
    stats: Arc<RwLock<StreamProcessingStats>>,
    is_running: Arc<AtomicBool>,
    worker_handles: Vec<thread::JoinHandle<()>>,
}

impl RealTimeStreamProcessor {
    pub fn new(config: StreamProcessingConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            config,
            processors: HashMap::new(),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(StreamProcessingStats::default())),
            is_running: Arc::new(AtomicBool::new(false)),
            worker_handles: Vec::new(),
        }
    }

    pub fn register_processor(&mut self, data_type: StreamDataType, processor: Arc<dyn StreamProcessor>) {
        self.processors.insert(data_type, processor);
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err("Stream processor already running".into());
        }

        self.is_running.store(true, Ordering::SeqCst);

        // 启动工作线程
        for worker_id in 0..self.config.worker_threads {
            let receiver = Arc::clone(&self.receiver);
            let buffer = Arc::clone(&self.buffer);
            let stats = Arc::clone(&self.stats);
            let is_running = Arc::clone(&self.is_running);
            let processors = self.processors.clone();
            let config = self.config.clone();

            let handle = thread::spawn(move || {
                Self::worker_thread(worker_id, receiver, buffer, stats, is_running, processors, config);
            });

            self.worker_handles.push(handle);
        }

        println!("Real-time stream processor started with {} workers", self.config.worker_threads);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.is_running.store(false, Ordering::SeqCst);

        // 等待所有工作线程完成
        while let Some(handle) = self.worker_handles.pop() {
            let _ = handle.join();
        }

        Ok(())
    }

    pub fn submit_data(&self, item: StreamDataItem) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err("Stream processor not running".into());
        }

        self.sender.send(item)?;

        // 更新统计
        if let Ok(mut stats) = self.stats.write() {
            stats.items_received += 1;
        }

        Ok(())
    }

    fn worker_thread(
        worker_id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<StreamDataItem>>>,
        buffer: Arc<Mutex<VecDeque<StreamDataItem>>>,
        stats: Arc<RwLock<StreamProcessingStats>>,
        is_running: Arc<AtomicBool>,
        processors: HashMap<StreamDataType, Arc<dyn StreamProcessor>>,
        config: StreamProcessingConfig,
    ) {
        println!("Worker thread {} started", worker_id);

        while is_running.load(Ordering::SeqCst) {
            // 尝试接收数据
            if let Ok(receiver) = receiver.lock() {
                match receiver.try_recv() {
                    Ok(item) => {
                        let start_time = Instant::now();
                        
                        // 处理数据项
                        if let Some(processor) = processors.get(&item.data_type) {
                            match processor.process_item(&item) {
                                Ok(_) => {
                                    // 更新成功统计
                                    if let Ok(mut stats) = stats.write() {
                                        stats.items_processed += 1;
                                        let latency = start_time.elapsed().as_millis() as u64;
                                        if latency > stats.max_latency_ms {
                                            stats.max_latency_ms = latency;
                                        }
                                        // 更新平均延迟
                                        stats.avg_latency_ms = (stats.avg_latency_ms * 0.9) + (latency as f64 * 0.1);
                                    }
                                }
                                Err(_) => {
                                    // 更新错误统计
                                    if let Ok(mut stats) = stats.write() {
                                        stats.error_count += 1;
                                    }
                                }
                            }
                        } else {
                            // 没有对应的处理器，丢弃数据
                            if let Ok(mut stats) = stats.write() {
                                stats.items_dropped += 1;
                            }
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // 没有数据，短暂休眠
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        }

        println!("Worker thread {} stopped", worker_id);
    }

    pub fn get_stats(&self) -> StreamProcessingStats {
        self.stats.read().unwrap().clone()
    }

    pub fn get_buffer_size(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }
}

// 流查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamQueryType {
    VectorSimilarity,
    MemorySearch,
    AgentStateMonitor,
    EventPattern,
    RealTimeStats,
}

// 流查询
#[derive(Debug, Clone)]
pub struct StreamQuery {
    pub id: String,
    pub query_type: StreamQueryType,
    pub parameters: HashMap<String, String>,
    pub callback: String,
    pub created_at: Instant,
    pub last_result: Option<Vec<u8>>,
}

// 流式查询处理器
pub struct StreamQueryProcessor {
    query_cache: Arc<RwLock<HashMap<String, (Vec<u8>, Instant)>>>,
    cache_ttl: Duration,
    active_queries: Arc<RwLock<HashMap<String, StreamQuery>>>,
}

impl StreamQueryProcessor {
    pub fn new() -> Self {
        Self {
            query_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5分钟缓存
            active_queries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_query(&self, query: StreamQuery) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut queries = self.active_queries.write().unwrap();
        queries.insert(query.id.clone(), query);
        Ok(())
    }

    pub fn unregister_query(&self, query_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut queries = self.active_queries.write().unwrap();
        queries.remove(query_id);
        Ok(())
    }

    pub fn process_stream_item(&self, item: &StreamDataItem) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut triggered_queries = Vec::new();
        let queries = self.active_queries.read().unwrap();

        for (query_id, query) in queries.iter() {
            if self.should_trigger_query(query, item) {
                triggered_queries.push(query_id.clone());
            }
        }

        Ok(triggered_queries)
    }

    fn should_trigger_query(&self, query: &StreamQuery, item: &StreamDataItem) -> bool {
        match query.query_type {
            StreamQueryType::AgentStateMonitor => {
                if let Some(target_agent) = query.parameters.get("agent_id") {
                    if let Ok(agent_id) = target_agent.parse::<u64>() {
                        return item.agent_id == agent_id && 
                               matches!(item.data_type, StreamDataType::AgentState);
                    }
                }
                false
            }
            StreamQueryType::EventPattern => {
                matches!(item.data_type, StreamDataType::Event)
            }
            StreamQueryType::VectorSimilarity => {
                matches!(item.data_type, StreamDataType::Vector)
            }
            StreamQueryType::MemorySearch => {
                matches!(item.data_type, StreamDataType::Memory)
            }
            StreamQueryType::RealTimeStats => {
                true // 所有数据都可能触发统计查询
            }
        }
    }

    pub fn get_active_query_count(&self) -> usize {
        self.active_queries.read().unwrap().len()
    }

    pub fn get_cache_size(&self) -> usize {
        self.query_cache.read().unwrap().len()
    }
}
