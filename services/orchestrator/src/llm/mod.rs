use async_trait::async_trait;
use anyhow::Result;
use tonic::transport::Channel;
use tracing::{debug, error};

pub mod llm {
    tonic::include_proto!("llm");
}

use llm::llm_service_client::LlmServiceClient;
use llm::{GenerateRequest, EmbedRequest, ListModelsRequest, Message};

#[async_trait]
pub trait EmbeddingModel {
    /// Generate an embedding vector for the given text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait CompletionModel {
    /// Generate a completion for a list of messages
    async fn generate_chat(&self, messages: Vec<Message>) -> Result<String>;

    /// List available models from the provider
    async fn list_models(&self) -> Result<Vec<String>>;
}

pub struct LlmGatewayClient {
    client: LlmServiceClient<Channel>,
}

impl LlmGatewayClient {
    pub async fn new(addr: &str) -> Result<Self> {
        let client = LlmServiceClient::connect(addr.to_string()).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl EmbeddingModel for LlmGatewayClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Calling LLM Gateway for embedding (text length: {})", text.len());
        let mut client = self.client.clone();
        let request = tonic::Request::new(EmbedRequest {
            text: text.to_string(),
            model: String::new(),
        });
        let response = client.embed(request).await
            .map_err(|e| {
                error!("LLM Gateway embedding call failed: {}", e);
                e
            })?;
        Ok(response.into_inner().embedding)
    }
}

#[async_trait]
impl CompletionModel for LlmGatewayClient {
    async fn generate_chat(&self, messages: Vec<Message>) -> Result<String> {
        debug!("Calling LLM Gateway for generation (messages: {})", messages.len());
        let mut client = self.client.clone();
        let request = tonic::Request::new(GenerateRequest {
            messages,
            model: String::new(),
            temperature: 0.1,
        });
        let response = client.generate(request).await
            .map_err(|e| {
                error!("LLM Gateway generation call failed: {}", e);
                e
            })?;
        Ok(response.into_inner().text)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        debug!("Calling LLM Gateway to list models");
        let mut client = self.client.clone();
        let request = tonic::Request::new(ListModelsRequest {});
        let response = client.list_models(request).await
            .map_err(|e| {
                error!("LLM Gateway list_models call failed: {}", e);
                e
            })?;
        let models = response.into_inner().models
            .into_iter()
            .map(|m| m.id)
            .collect();
        Ok(models)
    }
}
