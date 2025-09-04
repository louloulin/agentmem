//! 嵌入提供商实现模块

pub mod openai;
pub mod huggingface;
pub mod local;
pub mod anthropic;
pub mod cohere;

pub use openai::OpenAIEmbedder;
pub use huggingface::HuggingFaceEmbedder;
pub use local::LocalEmbedder;
pub use anthropic::AnthropicEmbedder;
pub use cohere::CohereEmbedder;
