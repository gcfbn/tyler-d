use async_trait::async_trait;
use anyhow::Result;

pub mod ollama;
pub mod openai;
pub mod gemini;

pub mod llm {
    tonic::include_proto!("llm");
}

use llm::Message;

#[async_trait]
pub trait EmbeddingModel {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait CompletionModel {
    async fn generate(&self, messages: Vec<Message>) -> Result<String>;
    async fn list_models(&self) -> Result<Vec<String>>;
}
