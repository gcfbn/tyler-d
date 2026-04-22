mod storage;
mod llm;
mod ocr;

use anyhow::Result;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use crate::storage::{qdrant::QdrantBackend, StorageBackend};
use crate::llm::{ollama::OllamaClient, EmbeddingModel, CompletionModel};
use crate::ocr::OcrClient;

pub mod tscherepacha {
    tonic::include_proto!("tscherepacha");
}

use tscherepacha::orchestrator_server::{Orchestrator, OrchestratorServer};
use tscherepacha::{IngestRequest, IngestResponse, AskRequest, AskResponse};

pub struct MyOrchestrator {
    storage: Arc<dyn StorageBackend + Send + Sync>,
    embedder: Arc<dyn EmbeddingModel + Send + Sync>,
    generator: Arc<dyn CompletionModel + Send + Sync>,
    ocr_client: Mutex<OcrClient>,
}

#[tonic::async_trait]
impl Orchestrator for MyOrchestrator {
    async fn ingest(&self, request: Request<IngestRequest>) -> Result<Response<IngestResponse>, Status> {
        let req = request.into_inner();
        
        let (text, file_name) = match req.source {
            Some(tscherepacha::ingest_request::Source::Text(t)) => (t, None),
            Some(tscherepacha::ingest_request::Source::File(f)) => {
                let name = if f.file_name.is_empty() { None } else { Some(f.file_name) };
                let mut ocr = self.ocr_client.lock().await;
                let ocr_res = ocr.process_document(f.content, &f.file_type)
                    .await
                    .map_err(|e| Status::internal(format!("OCR failed: {}", e)))?;
                (ocr_res.text, name)
            }
            None => return Err(Status::invalid_argument("No source provided")),
        };

        if text.trim().is_empty() {
             return Ok(Response::new(IngestResponse {
                success: true,
                message: "Empty text, nothing to ingest".to_string(),
            }));
        }

        let embedding = self.embedder.embed(&text)
            .await
            .map_err(|e| Status::internal(format!("Embedding failed: {}", e)))?;

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

        self.storage.upsert_point(id, embedding, payload)
            .await
            .map_err(|e| Status::internal(format!("Storage failed: {}", e)))?;

        Ok(Response::new(IngestResponse {
            success: true,
            message: "Ingested successfully".to_string(),
        }))
    }

    async fn ask(&self, request: Request<AskRequest>) -> Result<Response<AskResponse>, Status> {
        let req = request.into_inner();
        let query = req.query;

        let query_embedding = self.embedder.embed(&query)
            .await
            .map_err(|e| Status::internal(format!("Embedding query failed: {}", e)))?;

        let search_results = self.storage.search(query_embedding, 3)
            .await
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

        let mut context = String::new();
        for (i, res) in search_results.iter().enumerate() {
            if let Some(text) = res.payload.get("text").or_else(|| res.payload.get("content")).and_then(|v| v.as_str()) {
                context.push_str(&format!("[{}] {}\n", i + 1, text));
            }
        }

        let prompt = format!(
            "Jesteś pomocnym asystentem o nazwie Tscherepacha. Odpowiadaj wyłącznie na podstawie dostarczonego kontekstu w języku polskim.\n\nKontekst:\n{}\n\nPytanie: {}\n\nOdpowiedź:",
            context, query
        );

        let answer = self.generator.generate(&prompt)
            .await
            .map_err(|e| Status::internal(format!("Generation failed: {}", e)))?;

        Ok(Response::new(AskResponse {
            answer,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Tscherepacha Orchestrator starting up...");

    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ocr_url = env::var("OCR_URL").unwrap_or_else(|_| "http://localhost:50051".to_string());

    println!("Connecting to Qdrant at: {}", qdrant_url);
    let storage = Arc::new(QdrantBackend::new(&qdrant_url, "thoughts")?);
    
    // Check Qdrant health
    storage.health_check().await?;
    storage.init().await?;

    println!("Connecting to Ollama at: {} (model: bge-m3 for embeddings)", ollama_url);
    let embedder = Arc::new(OllamaClient::new(&ollama_url, "bge-m3"));

    println!("Connecting to Ollama at: {} (model: bielik for generation)", ollama_url);
    let generator = Arc::new(OllamaClient::new(&ollama_url, "SpeakLeash/bielik-11b-v3.0-instruct:Q4_K_M"));

    println!("Connecting to OCR at: {}", ocr_url);
    let ocr_client = OcrClient::new(&ocr_url).await?;

    let orchestrator = MyOrchestrator {
        storage,
        embedder,
        generator,
        ocr_client: Mutex::new(ocr_client),
    };

    let addr = "0.0.0.0:50052".parse()?;
    println!("Orchestrator gRPC server listening on {}", addr);

    Server::builder()
        .add_service(OrchestratorServer::new(orchestrator))
        .serve(addr)
        .await?;

    Ok(())
}
