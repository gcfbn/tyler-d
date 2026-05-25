use anyhow::{Context, Result};
use std::env;
use std::net::SocketAddr;

pub struct Config {
    pub provider: String,
    pub gen_model: String,
    pub emb_model: String,
    pub api_key: Option<String>,
    pub llm_url: Option<String>,
    pub ollama_url: Option<String>,
    pub use_external_embeddings: bool,
    pub server_addr: SocketAddr,
    pub vector_dimension: Option<usize>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let provider = env::var("LLM_PROVIDER").context("LLM_PROVIDER environment variable must be set (e.g., ollama, gemini, openai)")?;
        let gen_model = env::var("LLM_MODEL").context("LLM_MODEL environment variable must be set")?;
        let emb_model = env::var("EMBEDDING_MODEL").context("EMBEDDING_MODEL environment variable must be set")?;

        let api_key = env::var("LLM_API_KEY").ok();
        let llm_url = env::var("LLM_URL").ok();
        let ollama_url = env::var("OLLAMA_URL").ok();
        let use_external_embeddings = env::var("USE_EXTERNAL_EMBEDDINGS").is_ok();
        let vector_dimension = env::var("VECTOR_DIMENSION").ok().and_then(|s| s.parse().ok());

        let server_addr_str = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:50053".to_string());
        let server_addr: SocketAddr = server_addr_str
            .parse()
            .with_context(|| format!("Failed to parse SERVER_ADDR: {}", server_addr_str))?;

        Ok(Config {
            provider,
            gen_model,
            emb_model,
            api_key,
            llm_url,
            ollama_url,
            use_external_embeddings,
            server_addr,
            vector_dimension,
        })
    }
}
