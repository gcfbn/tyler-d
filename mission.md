# Mission
Build `tyler-d`, an AI-powered, local-first "second brain" that stores, processes, and retrieves user thoughts, documents, and media. 

# Core Objectives
- **Privacy First:** All data processing and storage happens locally. No unprompted internet lookups.
- **Language:** Native support for Polish, leveraging local models like Bielik.
- **Multimodal Ingestion:** Accept raw text streams, PDFs, and images (via OCR).
- **Fact-Based Retrieval:** The system must answer based ONLY on the provided user context. It should not hallucinate or guess missing information.
- **Modular Architecture:** Microservices-based design (Dockerized). The LLM component must be loosely coupled so it can be swapped between local CPU-only models (via Ollama) and external APIs (Gemini/Claude) with minimal configuration changes.

# Technical Constraints
- Target environment lacks a GPU; local LLMs must run on CPU (quantized GGUF formats).
- Prefer statically typed languages (Rust) for core services to minimize memory footprint.
