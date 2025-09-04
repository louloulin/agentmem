//! 嵌入提供商实现模块

pub mod anthropic;
pub mod cohere;
pub mod huggingface;
pub mod local;
pub mod openai;

pub use anthropic::AnthropicEmbedder;
pub use cohere::CohereEmbedder;
pub use huggingface::HuggingFaceEmbedder;
pub use local::LocalEmbedder;
pub use openai::OpenAIEmbedder;
