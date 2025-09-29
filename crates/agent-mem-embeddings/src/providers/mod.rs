//! 嵌入提供商实现模块

pub mod cohere;
pub mod huggingface;
pub mod local;
pub mod openai;

#[cfg(test)]
mod local_test;

pub use cohere::CohereEmbedder;
pub use huggingface::HuggingFaceEmbedder;
pub use local::LocalEmbedder;
pub use openai::OpenAIEmbedder;
