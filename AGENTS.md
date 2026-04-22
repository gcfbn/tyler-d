# Context & Architecture
`tscherepacha` is a microservices-based AI second brain.

## Architecture Map
- **Orchestrator:** Rust (Actix/Axum). The central hub managing data flow.
- **LLM Engine:** Ollama running `Bielik` (GGUF, CPU-optimized). Exposes an OpenAI-compatible API.
- **Memory (Vector DB):** Qdrant. Stores embeddings for semantic search.
- **OCR Service:** Python/Rust container for extracting text from PDFs and Images.
- **Frontend:** To be determined (CLI or lightweight GUI).
- **Communication:** gRPC for internal services, REST for external UI.

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
