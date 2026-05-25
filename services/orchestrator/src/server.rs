use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, debug, warn, error};

use crate::config::Config;
use crate::storage::{qdrant::QdrantBackend, StorageBackend};
use crate::llm::{LlmGatewayClient, EmbeddingModel, CompletionModel};
use crate::ocr::OcrClient;

pub mod tyler_d {
    tonic::include_proto!("tyler_d");
}

use tyler_d::orchestrator_server::{Orchestrator, OrchestratorServer};
use tyler_d::{IngestRequest, IngestResponse, AskRequest, AskResponse, ListModelsRequest, ListModelsResponse, ModelInfo};

pub struct MyOrchestrator {
    storage: Arc<dyn StorageBackend + Send + Sync>,
    llm_client: Arc<LlmGatewayClient>,
    ocr_client: Mutex<OcrClient>,
}

#[tonic::async_trait]
impl Orchestrator for MyOrchestrator {
    async fn list_models(&self, _request: Request<ListModelsRequest>) -> Result<Response<ListModelsResponse>, Status> {
        debug!("Listing available models");
        let models = self.llm_client.list_models().await
            .map_err(|e| {
                error!("Failed to list models from gateway: {}", e);
                Status::internal(format!("Failed to list models: {}", e))
            })?;
        
        let provider = "gateway"; // Transparent provider

        let models = models.into_iter()
            .map(|id| ModelInfo {
                id,
                provider: provider.to_string(),
            })
            .collect();

        Ok(Response::new(ListModelsResponse { models }))
    }

    async fn ingest(&self, request: Request<IngestRequest>) -> Result<Response<IngestResponse>, Status> {
        let req = request.into_inner();
        
        let (text, file_name) = match req.source {
            Some(tyler_d::ingest_request::Source::Text(t)) => {
                debug!("Ingesting direct text");
                (t, None)
            },
            Some(tyler_d::ingest_request::Source::File(f)) => {
                let name = if f.file_name.is_empty() { None } else { Some(f.file_name.clone()) };
                debug!("Ingesting file: {:?}", name);
                let mut ocr = self.ocr_client.lock().await;
                let ocr_res = ocr.process_document(f.content, &f.file_type)
                    .await
                    .map_err(|e| {
                        error!("OCR processing failed for {:?}: {}", name, e);
                        Status::internal(format!("OCR failed: {}", e))
                    })?;
                (ocr_res.text, name)
            }
            None => {
                warn!("Ingest request received with no source");
                return Err(Status::invalid_argument("No source provided"));
            },
        };

        if text.trim().is_empty() {
             warn!("Ingest request contains empty text");
             return Ok(Response::new(IngestResponse {
                success: true,
                message: "Empty text, nothing to ingest".to_string(),
            }));
        }

        debug!("Generating embedding for ingested text (length: {})", text.len());
        let embedding = self.llm_client.embed(&text)
            .await
            .map_err(|e| {
                error!("Embedding generation failed: {}", e);
                Status::internal(format!("Embedding failed: {}", e))
            })?;

        // Generate a simple ID (hash of text)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let id = hasher.finish();

        let mut payload_map = serde_json::Map::new();
        payload_map.insert("text".to_string(), serde_json::json!(text));
        if let Some(name) = file_name {
            payload_map.insert("file_name".to_string(), serde_json::json!(name));
        }
        let payload = serde_json::Value::Object(payload_map);

        debug!("Upserting point {} to storage", id);
        self.storage.upsert_point(id, embedding, payload)
            .await
            .map_err(|e| {
                error!("Failed to store ingested data: {}", e);
                Status::internal(format!("Storage failed: {}", e))
            })?;

        info!("Successfully ingested data (ID: {})", id);
        Ok(Response::new(IngestResponse {
            success: true,
            message: "Ingested successfully".to_string(),
        }))
    }

    async fn ask(&self, request: Request<AskRequest>) -> Result<Response<AskResponse>, Status> {
        let req = request.into_inner();
        let query = req.query;
        debug!("Processing ask request: '{}'", query);

        debug!("Generating embedding for query");
        let query_embedding = self.llm_client.embed(&query)
            .await
            .map_err(|e| {
                error!("Failed to embed query: {}", e);
                Status::internal(format!("Embedding query failed: {}", e))
            })?;

        debug!("Searching storage for relevant context");
        let search_results = self.storage.search(query_embedding, 3)
            .await
            .map_err(|e| {
                error!("Storage search failed: {}", e);
                Status::internal(format!("Search failed: {}", e))
            })?;

        let mut context = String::new();
        for (i, res) in search_results.iter().enumerate() {
            if let Some(text) = res.payload.get("text").or_else(|| res.payload.get("content")).and_then(|v| v.as_str()) {
                context.push_str(&format!("[{}] {}\n", i + 1, text));
            }
        }

        if context.is_empty() {
            debug!("No relevant context found in storage for query");
        }

        let prompt = format!(
            "Jesteś pomocnym asystentem o nazwie Tyler-d. Odpowiadaj wyłącznie na podstawie dostarczonego kontekstu w języku polskim.\n\nKontekst:\n{}\n\nPytanie: {}\n\nOdpowiedź:",
            context, query
        );

        debug!("Generating answer from LLM gateway");
        let answer = self.llm_client.generate(&prompt)
            .await
            .map_err(|e| {
                error!("LLM generation failed: {}", e);
                Status::internal(format!("Generation failed: {}", e))
            })?;

        info!("Answer generated for query '{}'", query);
        Ok(Response::new(AskResponse {
            answer,
        }))
    }
}

pub async fn run_server(config: Config) -> Result<()> {
    info!("Connecting to Qdrant at: {}", config.qdrant_url);
    let storage = Arc::new(QdrantBackend::new(&config.qdrant_url, "thoughts", config.vector_dimension)?);
    
    // Check Qdrant health
    storage.health_check().await?;
    storage.init().await?;

    info!("Connecting to LLM Gateway at: {}", config.llm_gateway_url);
    let llm_client = Arc::new(LlmGatewayClient::new(&config.llm_gateway_url).await?);

    info!("Connecting to OCR at: {}", config.ocr_url);
    let ocr_client = OcrClient::new(&config.ocr_url).await?;

    let orchestrator = MyOrchestrator {
        storage,
        llm_client,
        ocr_client: Mutex::new(ocr_client),
    };

    info!("Orchestrator gRPC server listening on {}", config.server_addr);

    Server::builder()
        .add_service(OrchestratorServer::new(orchestrator))
        .serve(config.server_addr)
        .await?;

    Ok(())
}
