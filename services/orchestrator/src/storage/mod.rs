use async_trait::async_trait;
use anyhow::Result;

pub mod qdrant;

#[async_trait]
pub trait StorageBackend {
    /// Initialize the storage (e.g., create collections/tables)
    async fn init(&self) -> Result<()>;
    
    /// Health check for the storage service
    async fn health_check(&self) -> Result<()>;
    
    /// Upsert a point (vector + payload) into the storage
    async fn upsert_point(&self, id: u64, vector: Vec<f32>, payload: serde_json::Value) -> Result<()>;

    /// Search for similar points
    async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub payload: serde_json::Value,
}
