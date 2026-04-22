use anyhow::{Context, Result};
use async_trait::async_trait;
use crate::llm::{EmbeddingModel, CompletionModel};
use serde_json::json;

pub struct OllamaClient {
    base_url: String,
    model_name: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: &str, model_name: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            model_name: model_name.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl EmbeddingModel for OllamaClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let response = self.client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&json!({
                "model": self.model_name,
                "prompt": text,
            }))
            .send()
            .await
            .context("Failed to connect to Ollama (embeddings)")?;

        let res_json: serde_json::Value = response.json().await
            .context("Failed to parse Ollama embedding response")?;

        let embedding = res_json["embedding"]
            .as_array()
            .context("No embedding found in Ollama response")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }
}

#[async_trait]
impl CompletionModel for OllamaClient {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&json!({
                "model": self.model_name,
                "prompt": prompt,
                "stream": false,
            }))
            .send()
            .await
            .context("Failed to connect to Ollama (generate)")?;

        let res_json: serde_json::Value = response.json().await
            .context("Failed to parse Ollama generation response")?;

        let text = res_json["response"]
            .as_str()
            .context("No response text found in Ollama generation")?
            .to_string();

        Ok(text)
    }
}
