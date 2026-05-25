mod provider;
mod config;

use anyhow::{Result, Context};
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::provider::llm::llm_service_server::{LlmService, LlmServiceServer};
use crate::provider::llm::{
    GenerateRequest, GenerateResponse, EmbedRequest, EmbedResponse, 
    ListModelsRequest, ListModelsResponse, ModelInfo
};
use crate::provider::{
    EmbeddingModel, CompletionModel, 
    ollama::OllamaClient, 
    openai::OpenAIClient,
    gemini::GeminiClient,
};

pub struct MyLlmGateway {
    embedder: Arc<dyn EmbeddingModel + Send + Sync>,
    generator: Arc<dyn CompletionModel + Send + Sync>,
    provider: String,
}

#[tonic::async_trait]
impl LlmService for MyLlmGateway {
    async fn generate(&self, request: Request<GenerateRequest>) -> Result<Response<GenerateResponse>, Status> {
        let req = request.into_inner();
        let text = self.generator.generate(req.messages).await
            .map_err(|e| Status::internal(format!("Generation failed: {}", e)))?;
        
        Ok(Response::new(GenerateResponse { text }))
    }

    async fn embed(&self, request: Request<EmbedRequest>) -> Result<Response<EmbedResponse>, Status> {
        let req = request.into_inner();
        let embedding = self.embedder.embed(&req.text).await
            .map_err(|e| Status::internal(format!("Embedding failed: {}", e)))?;
        
        Ok(Response::new(EmbedResponse { embedding }))
    }

    async fn list_models(&self, _request: Request<ListModelsRequest>) -> Result<Response<ListModelsResponse>, Status> {
        let models_ids = self.generator.list_models().await
            .map_err(|e| Status::internal(format!("Failed to list models: {}", e)))?;
        
        let models = models_ids.into_iter()
            .map(|id| ModelInfo {
                id,
                provider: self.provider.clone(),
            })
            .collect();

        Ok(Response::new(ListModelsResponse { models }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Tyler-d LLM Gateway starting up...");

    let config = Config::from_env()?;

    info!("Selected LLM Provider: {}", config.provider);
    info!("Selected Generation Model: {}", config.gen_model);
    info!("Selected Embedding Model: {}", config.emb_model);

    let (embedder, generator): (Arc<dyn EmbeddingModel + Send + Sync>, Arc<dyn CompletionModel + Send + Sync>) = match config.provider.as_str() {
        "gemini" => {
            let api_key = config.api_key.context("LLM_API_KEY must be set for gemini provider")?;
            info!("Initializing Gemini provider");
            let gen_client = Arc::new(GeminiClient::new(&api_key, &config.gen_model, None));
            let emb_client = Arc::new(GeminiClient::new(&api_key, &config.emb_model, config.vector_dimension));
            (emb_client, gen_client)
        }
        "openai" | "external" => {
            let api_key = config.api_key.context("LLM_API_KEY must be set for openai/external provider")?;
            let url = config.llm_url.context("LLM_URL must be set for openai/external provider (e.g., https://api.groq.com/openai/v1)")?;
            
            info!("Initializing OpenAI-compatible provider at {}", url);
            let gen_client = Arc::new(OpenAIClient::new(&api_key, &url, &config.gen_model));
            
            let emb_client: Arc<dyn EmbeddingModel + Send + Sync> = if config.use_external_embeddings {
                info!("Using External LLM for embeddings");
                Arc::new(OpenAIClient::new(&api_key, &url, &config.emb_model))
            } else {
                let ollama_url = config.ollama_url.context("OLLAMA_URL must be set and point to Ollama for local embeddings when using hybrid openai mode")?;
                info!("Falling back to Ollama for embeddings at {}", ollama_url);
                Arc::new(OllamaClient::new(&ollama_url, &config.emb_model))
            };
            
            (emb_client, gen_client)
        }
        "ollama" => {
            let url = config.ollama_url.context("OLLAMA_URL must be set for ollama provider (e.g., http://ollama:11434)")?;
            
            info!("Initializing Ollama provider at {}", url);
            let gen_client = Arc::new(OllamaClient::new(&url, &config.gen_model));
            let emb_client = Arc::new(OllamaClient::new(&url, &config.emb_model));
            (emb_client, gen_client)
        }
        _ => return Err(anyhow::anyhow!("Unsupported LLM_PROVIDER: {}", config.provider)),
    };

    let gateway = MyLlmGateway {
        embedder,
        generator,
        provider: config.provider,
    };

    info!("LLM Gateway gRPC server listening on {}", config.server_addr);

    Server::builder()
        .add_service(LlmServiceServer::new(gateway))
        .serve(config.server_addr)
        .await?;

    Ok(())
}
