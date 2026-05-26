use anyhow::{Context, Result};
use std::env;
use std::net::SocketAddr;

pub struct Config {
    pub qdrant_url: String,
    pub llm_gateway_url: String,
    pub ocr_url: String,
    pub server_addr: SocketAddr,
    pub vector_dimension: usize,
    pub max_context_tokens: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL must be set")?;
        let llm_gateway_url = env::var("LLM_GATEWAY_URL").context("LLM_GATEWAY_URL must be set")?;
        let ocr_url = env::var("OCR_URL").context("OCR_URL must be set")?;
        
        let server_addr_str = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:50052".to_string());
        let server_addr: SocketAddr = server_addr_str
            .parse()
            .with_context(|| format!("Failed to parse SERVER_ADDR: {}", server_addr_str))?;

        let vector_dimension = env::var("VECTOR_DIMENSION")
            .unwrap_or_else(|_| "1024".to_string())
            .parse()
            .context("Failed to parse VECTOR_DIMENSION")?;

        let max_context_tokens = env::var("MAX_CONTEXT_TOKENS")
            .unwrap_or_else(|_| "4096".to_string())
            .parse()
            .context("Failed to parse MAX_CONTEXT_TOKENS")?;

        Ok(Config {
            qdrant_url,
            llm_gateway_url,
            ocr_url,
            server_addr,
            vector_dimension,
            max_context_tokens,
        })
    }
}
