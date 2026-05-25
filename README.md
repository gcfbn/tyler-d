# Tyler-d

AI-powered local-first "second brain" optimized for Polish and privacy.

## Project Structure
- `services/orchestrator`: Core logic (Rust).
- `services/ocr`: Document processing service (Python/gRPC).
- `protos`: Shared gRPC definitions.
- `docker-compose.yml`: Local infrastructure (Ollama, Qdrant).

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

## License
Non-commercial / Personal use.
