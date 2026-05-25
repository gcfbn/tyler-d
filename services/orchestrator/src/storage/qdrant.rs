use anyhow::{Context, Result};
use async_trait::async_trait;
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams, VectorsConfig, vectors_config, PointStruct, UpsertPoints, SearchPoints};
use crate::storage::{StorageBackend, SearchResult};
use tracing::{info, debug, error};

pub struct QdrantBackend {
    client: Qdrant,
    collection_name: String,
    vector_dimension: u64,
}

impl QdrantBackend {
    pub fn new(url: &str, collection_name: &str, vector_dimension: usize) -> Result<Self> {
        let config = QdrantConfig::from_url(url);
        let client = Qdrant::new(config)?;
        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
            vector_dimension: vector_dimension as u64,
        })
    }
}

#[async_trait]
impl StorageBackend for QdrantBackend {
    async fn health_check(&self) -> Result<()> {
        let _ = self.client.health_check().await
            .map_err(|e| {
                error!("Qdrant health check failed: {}", e);
                e
            })
            .context("Qdrant health check failed")?;
        Ok(())
    }

    async fn upsert_point(&self, id: u64, vector: Vec<f32>, payload: serde_json::Value) -> Result<()> {
        debug!("Upserting point {} with vector length {}", id, vector.len());
        let payload_map: serde_json::Map<String, serde_json::Value> = payload
            .as_object()
            .context("Payload must be a JSON object")?
            .clone();
        
        let point = PointStruct::new(
            id,
            vector,
            payload_map,
        );

        let upsert_request = UpsertPoints {
            collection_name: self.collection_name.clone(),
            points: vec![point],
            ..Default::default()
        };

        self.client
            .upsert_points(upsert_request)
            .await
            .map_err(|e| {
                error!("Failed to upsert point {}: {}", id, e);
                e
            })
            .context("Failed to upsert point to Qdrant")?;

        Ok(())
    }

    async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        debug!("Searching for top {} nearest neighbors", limit);
        let search_request = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector,
            limit: limit as u64,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        let response = self.client
            .search_points(search_request)
            .await
            .map_err(|e| {
                error!("Search query failed: {}", e);
                e
            })
            .context("Failed to search points in Qdrant")?;

        debug!("Search returned {} results", response.result.len());
        let results = response.result
            .into_iter()
            .map(|point| {
                let id = match point.id.unwrap().point_id_options.unwrap() {
                    qdrant_client::qdrant::point_id::PointIdOptions::Num(n) => n,
                    _ => 0,
                };
                
                // Convert back to JSON for the backend-agnostic trait
                let payload = serde_json::to_value(point.payload).unwrap_or(serde_json::Value::Null);

                SearchResult {
                    id,
                    score: point.score,
                    payload,
                }
            })
            .collect();

        Ok(results)
    }

    async fn init(&self) -> Result<()> {
        debug!("Initializing Qdrant backend for collection: {}", self.collection_name);
        
        if !self.client.collection_exists(&self.collection_name).await? {
            info!("Creating Qdrant collection '{}' with dimension: {}", self.collection_name, self.vector_dimension);

            let vectors_config = VectorsConfig {
                config: Some(vectors_config::Config::Params(VectorParams {
                    size: self.vector_dimension,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            };

            let create_collection = CreateCollection {
                collection_name: self.collection_name.clone(),
                vectors_config: Some(vectors_config),
                ..Default::default()
            };

            self.client
                .create_collection(create_collection)
                .await
                .map_err(|e| {
                    error!("Failed to create collection '{}': {}", self.collection_name, e);
                    e
                })
                .context("Failed to create Qdrant collection")?;
            
            info!("Collection '{}' created successfully.", self.collection_name);
        } else {
            debug!("Collection '{}' already exists in Qdrant.", self.collection_name);
        }
        
        Ok(())
    }
}
