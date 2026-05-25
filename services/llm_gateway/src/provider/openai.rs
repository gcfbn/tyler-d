use anyhow::{Context, Result};
use async_trait::async_trait;
use async_openai::{
    types::{
        CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, 
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestSystemMessageArgs, CreateEmbeddingRequestArgs
    },
    Client,
    config::OpenAIConfig,
};
use crate::provider::{CompletionModel, EmbeddingModel, llm::Message};

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    model_name: String,
}

impl OpenAIClient {
    pub fn new(api_key: &str, base_url: &str, model_name: &str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);
        
        Self {
            client: Client::with_config(config),
            model_name: model_name.to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingModel for OpenAIClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model_name)
            .input(text)
            .build()
            .context("Failed to build embedding request")?;

        let response = self.client
            .embeddings()
            .create(request)
            .await
            .context("Failed to connect to OpenAI-compatible API (embeddings)")?;

        let embedding = response.data
            .first()
            .map(|data| data.embedding.clone())
            .context("No embedding found in response")?;

        Ok(embedding)
    }
}

#[async_trait]
impl CompletionModel for OpenAIClient {
    async fn generate(&self, messages: Vec<Message>) -> Result<String> {
        let mut chat_messages = Vec::new();
        for m in messages {
            let chat_m = match m.role.as_str() {
                "system" => ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(m.content)
                        .build()?
                ),
                "assistant" => ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(m.content)
                        .build()?
                ),
                _ => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(m.content)
                        .build()?
                ),
            };
            chat_messages.push(chat_m);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model_name)
            .messages(chat_messages)
            .temperature(0.1)
            .build()
            .context("Failed to build chat completion request")?;

        let response = self.client
            .chat()
            .create(request)
            .await
            .context("Failed to connect to OpenAI-compatible API")?;

        let text = response.choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .context("No response text found in completion response")?;

        Ok(text)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .models()
            .list()
            .await
            .context("Failed to list models from OpenAI-compatible API")?;

        let models = response.data
            .into_iter()
            .map(|m| m.id)
            .collect();

        Ok(models)
    }
}
