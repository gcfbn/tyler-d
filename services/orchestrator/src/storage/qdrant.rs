use anyhow::{Context, Result};
use async_trait::async_trait;
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams, VectorsConfig, vectors_config, PointStruct, UpsertPoints, SearchPoints};
use crate::storage::{StorageBackend, SearchResult};

pub struct QdrantBackend {
    client: Qdrant,
    collection_name: String,
}

impl QdrantBackend {
    pub fn new(url: &str, collection_name: &str) -> Result<Self> {
        let config = QdrantConfig::from_url(url);
        let client = Qdrant::new(config)?;
        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
        })
    }
}

#[async_trait]
impl StorageBackend for QdrantBackend {
    async fn health_check(&self) -> Result<()> {
        let _ = self.client.health_check().await
            .context("Qdrant health check failed")?;
        Ok(())
    }

    async fn upsert_point(&self, id: u64, vector: Vec<f32>, payload: serde_json::Value) -> Result<()> {
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
            .context("Failed to upsert point to Qdrant")?;

        Ok(())
    }

    async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
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
            .context("Failed to search points in Qdrant")?;

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
        println!("Initializing Qdrant collection: {}...", self.collection_name);
        
        if !self.client.collection_exists(&self.collection_name).await? {
            let vectors_config = VectorsConfig {
                config: Some(vectors_config::Config::Params(VectorParams {
                    size: 1024, // Matches BGE-M3
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
                .context("Failed to create Qdrant collection")?;
            
            println!("Collection '{}' created successfully.", self.collection_name);
        } else {
            println!("Collection '{}' already exists.", self.collection_name);
        }
        
        Ok(())
    }
}
