use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::CreateCollection;

pub async fn check_inference() {
    let url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let config = QdrantConfig::from_url(&url);
    let client = Qdrant::new(config).unwrap();
    
    // Health check
    let _health = client.health_check().await.unwrap();
    
    // Create collection
    let create_collection = CreateCollection::default();
    let _ = client.create_collection(create_collection).await.unwrap();
}
