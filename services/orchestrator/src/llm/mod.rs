use async_trait::async_trait;
use anyhow::Result;

pub mod ollama;

#[async_trait]
pub trait EmbeddingModel {
    /// Generate an embedding vector for the given text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait CompletionModel {
    /// Generate a completion for the given prompt
    async fn generate(&self, prompt: &str) -> Result<String>;
}
