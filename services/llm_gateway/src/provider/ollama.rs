use anyhow::{Context, Result};
use async_trait::async_trait;
use crate::provider::{EmbeddingModel, CompletionModel, llm::Message};
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

        let status = response.status();
        let res_body = response.text().await
            .context("Failed to read Ollama response body")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("Ollama returned error ({}): {}", status, res_body));
        }

        let res_json: serde_json::Value = serde_json::from_str(&res_body)
            .context(format!("Failed to parse Ollama embedding response: {}", res_body))?;

        let embedding = res_json["embedding"]
            .as_array()
            .context(format!("No 'embedding' field found in Ollama response. Full response: {}", res_body))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }
}

#[async_trait]
impl CompletionModel for OllamaClient {
    async fn generate(&self, messages: Vec<Message>) -> Result<String> {
        let ollama_messages: Vec<serde_json::Value> = messages.into_iter()
            .map(|m| json!({
                "role": m.role,
                "content": m.content,
            }))
            .collect();

        let response = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&json!({
                "model": self.model_name,
                "messages": ollama_messages,
                "stream": false,
            }))
            .send()
            .await
            .context("Failed to connect to Ollama (chat)")?;

        let status = response.status();
        let res_body = response.text().await
            .context("Failed to read Ollama response body")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("Ollama returned error ({}): {}", status, res_body));
        }

        let res_json: serde_json::Value = serde_json::from_str(&res_body)
            .context(format!("Failed to parse Ollama chat response: {}", res_body))?;

        let text = res_json["message"]["content"]
            .as_str()
            .context(format!("No 'message.content' field found in Ollama response. Full response: {}", res_body))?
            .to_string();

        Ok(text)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .context("Failed to connect to Ollama (tags)")?;

        let res_json: serde_json::Value = response.json().await
            .context("Failed to parse Ollama tags response")?;

        let models = res_json["models"]
            .as_array()
            .context("No models found in Ollama tags response")?
            .iter()
            .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(models)
    }
}
