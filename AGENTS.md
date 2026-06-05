# Context & Architecture
`tyler-d` is a microservices-based AI second brain.

## Architecture Map
- **Orchestrator:** Rust (Tonic). The central hub managing data flow and RAG logic.
- **LLM Engine:** Ollama running `Bielik` (GGUF, CPU-optimized) or remote providers via Gateway.
- **Memory (Vector DB):** Qdrant. Stores embeddings for semantic search.
- **OCR Service:** Python (gRPC). Extracts text from PDFs and Images.
- **Frontend:** Vue 3 / Nuxt UI v4 (Web) and Rust (CLI).
- **Communication:** gRPC for all internal and external communication (using gRPC-Web for GUI).

## Coding Style & Rules
- Strictly adhere to the rules in `SOUL.md` (concise, bullet points, no emojis).
- Prioritize memory efficiency (vital for CPU-only local LLM hosts).
- Write loosely-coupled interfaces. The LLM integration must use generic API clients (like `async-openai` in Rust) to easily swap between Ollama and external providers.
- Never disable compiler warnings or type checks in Rust. Use explicit error handling (`Result`/`Option`).
- All services must be accompanied by a `Dockerfile`.

## Workflow
1. For every complex task, propose a plan in `artifacts/plan_[task_id].md`.
2. Wait for user confirmation before executing cross-file architectural changes.
3. **Commit often:** Commit changes incrementally in small, logical steps.
4. **Branching:** NEVER commit to `master`. Always use feature branches (e.g., `feat/something` or `fix/something`).
5. Test locally and provide reproduction steps.
6. **Docker:** Do NOT use `sudo` for docker or docker-compose commands. The environment is configured for non-root access.
