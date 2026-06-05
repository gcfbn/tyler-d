use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, debug, warn, error};
use tower_http::cors::CorsLayer;
use tonic_web::GrpcWebLayer;

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
    max_context_tokens: usize,
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
        let history = req.history;
        info!("Processing ask request: '{}' (history size: {})", query, history.len());

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
        for res in search_results {
            if let Some(text) = res.payload.get("text").or_else(|| res.payload.get("content")).and_then(|v| v.as_str()) {
                context.push_str(&format!("- {}\n", text));
            }
        }

        if context.is_empty() {
            debug!("No relevant context found in storage for query");
        }

        let system_prompt = "Jesteś Tyler-d, osobistym asystentem i \"drugim mózgiem\" użytkownika. Twoja wiedza pochodzi z prywatnych notatek i dokumentów użytkownika, które są ci udostępniane jako Twoja własna pamięć.
Zasady:
1. Odpowiadaj wyłącznie w języku polskim.
2. Nigdy nie wspominaj użytkownikowi o istnieniu \"dostarczonego kontekstu\", \"fragmentów\" lub \"dokumentów\". Traktuj tę wiedzę jako własną.
3. Jeśli w Twojej pamięci nie ma odpowiedzi na pytanie, odpowiedz uprzejmie, że nie posiadasz takich informacji. Nie próbuj zgadywać ani nie opisuj innych informacji, które widzisz w pamięci, a które nie dotyczą pytania.
4. Nie używaj przypisów ani numeracji źródeł (np. [1]).
5. Bądź zwięzły i pomocny.".to_string();
        let context_str = format!("Pamięć (Twoja wiedza):\n{}\n\nPytanie użytkownika: {}\n\nTwoja odpowiedź:", context, query);

        // Token management using tiktoken
        // We use o200k_base as a high-quality universal tokenizer (GPT-4o) which is a safe proxy for most modern BPEs.
        let enc = tiktoken::get_encoding("o200k_base")
            .ok_or_else(|| Status::internal("Failed to load o200k_base tokenizer"))?;

        let system_tokens = enc.encode_with_special_tokens(&system_prompt).len();
        let context_tokens = enc.encode_with_special_tokens(&context_str).len();
        let mut total_tokens = system_tokens + context_tokens;

        let mut final_history = Vec::new();
        // Prune history from newest to oldest until we hit max_context_tokens
        for msg in history.into_iter().rev() {
            let msg_tokens = enc.encode_with_special_tokens(&msg.content).len();
            if total_tokens + msg_tokens <= self.max_context_tokens {
                total_tokens += msg_tokens;
                final_history.insert(0, msg);
            } else {
                debug!("Pruning older message from history due to token limit ({} tokens)", msg_tokens);
            }
        }

        use crate::llm::llm::Message as LlmMessage;
        let mut messages = vec![
            LlmMessage {
                role: "system".to_string(),
                content: system_prompt,
            }
        ];

        for msg in final_history {
            messages.push(LlmMessage {
                role: msg.role,
                content: msg.content,
            });
        }

        messages.push(LlmMessage {
            role: "user".to_string(),
            content: context_str,
        });

        debug!("Generating answer from LLM gateway (total tokens estimate: {})", total_tokens);
        let answer = self.llm_client.generate_chat(messages)
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
        max_context_tokens: config.max_context_tokens,
    };

    info!("Orchestrator gRPC server listening on {}, max context tokens: {}", config.server_addr, config.max_context_tokens);

    let orchestrator_service = OrchestratorServer::new(orchestrator);

    // Enable gRPC-Web and CORS for browser compatibility
    let cors = CorsLayer::permissive();

    Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(GrpcWebLayer::new())
        .add_service(orchestrator_service)
        .serve(config.server_addr)
        .await?;

    Ok(())
}
