use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;
use crate::provider::{CompletionModel, EmbeddingModel, llm::Message};

pub struct GeminiClient {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
    vector_dimension: Option<usize>,
}

impl GeminiClient {
    pub fn new(api_key: &str, model_name: &str, vector_dimension: Option<usize>) -> Self {
        Self {
            api_key: api_key.to_string(),
            model_name: model_name.to_string(),
            client: reqwest::Client::new(),
            vector_dimension,
        }
    }
}

#[async_trait]
impl EmbeddingModel for GeminiClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
            self.model_name, self.api_key
        );

        let mut body = json!({
            "model": format!("models/{}", self.model_name),
            "content": {
                "parts": [{ "text": text }]
            }
        });

        if let Some(dim) = self.vector_dimension {
            body["outputDimensionality"] = json!(dim);
        }

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Gemini API (embeddings)")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini embedding error: {}", error_text));
        }

        let res_json: serde_json::Value = response.json().await?;
        let embedding = res_json["embedding"]["values"]
            .as_array()
            .context("Invalid embedding response from Gemini")?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }
}

#[async_trait]
impl CompletionModel for GeminiClient {
    async fn generate(&self, messages: Vec<Message>) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name, self.api_key
        );

        let contents: Vec<serde_json::Value> = messages.into_iter()
            .map(|m| {
                let role = match m.role.as_str() {
                    "assistant" | "model" => "model",
                    _ => "user",
                };
                json!({
                    "role": role,
                    "parts": [{ "text": m.content }]
                })
            })
            .collect();

        let response = self.client
            .post(url)
            .json(&json!({
                "contents": contents,
                "generationConfig": {
                    "temperature": 0.1,
                    "topK": 40,
                    "topP": 0.95,
                    "maxOutputTokens": 2048,
                }
            }))
            .send()
            .await
            .context("Failed to connect to Gemini API (generate)")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini generation error: {}", error_text));
        }

        let res_json: serde_json::Value = response.json().await?;
        let text = res_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .context("Invalid generation response from Gemini")?
            .to_string();

        Ok(text)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            self.api_key
        );

        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch models from Gemini API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini list models error: {}", error_text));
        }

        let res_json: serde_json::Value = response.json().await?;
        let models = res_json["models"]
            .as_array()
            .context("Invalid response format: missing 'models' array")?
            .iter()
            .filter_map(|m| {
                let name = m["name"].as_str()?;
                Some(name.strip_prefix("models/").unwrap_or(name).to_string())
            })
            .collect();

        Ok(models)
    }
}
