# Tyler-d

AI-powered local-first "second brain" optimized for Polish and privacy.

## Project Structure
- `services/orchestrator`: Core logic and RAG management (Rust).
- `services/ocr`: Document processing service (Python/gRPC).
- `services/gui`: Web-based user interface (Vue 3 / Nuxt UI v4).
- `services/cli`: Command-line interface for headless operations (Rust).
- `services/llm_gateway`: Unified API for local and remote LLM providers (Rust).
- `protos`: Shared gRPC definitions.
- `docker-compose.yml`: Local infrastructure (Ollama, Qdrant).

## Interfaces

### GUI
Modern, accessible web interface built with **Nuxt UI v4** and **Tailwind CSS v4**. 
- Features: Chat with knowledge base, file ingestion (PDF/Images), and text entry.
- Development: `cd services/gui && npm install && npm run dev`

### CLI
Fast and lightweight command-line tool for ingestion and querying.
- Commands: `ingest`, `ask`, `models`.
- Development: `cd services/cli && cargo run -- --help`

## Getting Started

### 1. Start Infrastructure
```bash
docker compose up -d
```

### 2. Prepare Local LLM
Ensure Ollama is running, then pull the models (Bielik for text, bge-m3 for embeddings):
```bash
docker exec -it tyler-d-ollama ollama pull speakleash/bielik-v3-4.5b-instruct:q4_k_m
docker exec -it tyler-d-ollama ollama pull bge-m3
```

### 3. Development
- **Orchestrator:** `cd services/orchestrator && cargo build`
- **OCR:** `cd services/ocr && pip install -r requirements.txt`
- **GUI:** `cd services/gui && npm install && npm run dev`
- **CLI:** `cd services/cli && cargo build`

## License
Non-commercial / Personal use.
