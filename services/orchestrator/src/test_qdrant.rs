use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::CreateCollection;

pub async fn check_inference() {
    let config = QdrantConfig::from_url("http://localhost:6334");
    let client = Qdrant::new(config).unwrap();
    
    // Health check
    let _health = client.health_check().await.unwrap();
    
    // Create collection
    let create_collection = CreateCollection::default();
    let _ = client.create_collection(create_collection).await.unwrap();
}
