# System Architecture Overview

## System Context

The decentralized proof-of-contact network is a peer-to-peer system that generates, verifies, and stores digitally signed proofs certifying that two nodes communicated during a specific orbital window. It enables trustless verification of contact events in decentralized environments.

## Goals

- **Trustless proof generation**: Nodes generate cryptographic proofs of contact without a central authority
- **Post-quantum security**: Ed25519 signatures with optional Dilithium5 (ML-DSA) post-quantum overlay
- **Scalable analytics**: Data lakehouse architecture (DuckDB, Iceberg, Parquet) for large-scale querying
- **AI-native**: Built-in RAG pipeline for natural language queries against the proof graph
- **Operator-friendly**: OpenMetrics, structured logging (JSON), Grafana dashboards

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         POI Network Node                            │
│                                                                     │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────────────┐  │
│  │ P2P Network  │  │  Core        │  │  API Server (axum)        │  │
│  │ (poi-net)    │◄─┤  Protocol    │◄─┤  REST / OpenAPI           │  │
│  │ QUIC/TLS     │  │  (poi-core)  │  │  /api/v1/*                │  │
│  └─────────────┘  └──────┬───────┘  └───────────┬───────────────┘  │
│                          │                      │                   │
│  ┌─────────────┐  ┌──────┴───────┐  ┌───────────┴───────────────┐  │
│  │ Vector Store │  │  Analytics   │  │  AI Engine (poi-ai)       │  │
│  │ (poi-vstore) │  │  Lakehouse   │  │  RAG / LLM / Embeddings   │  │
│  │ Chroma/Qdrant│  │  DuckDB/Iceb │  │  Ollama/LM Studio/vLLM   │  │
│  └─────────────┘  └──────────────┘  └───────────────────────────┘  │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Observability Stack                                         │   │
│  │  Prometheus Metrics │ OTLP Tracing │ Structured JSON Logs   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               │
  ┌────────────────────────────┼────────────────────────────────────┐
  │                            │                                     │
  │  ┌────────┐  ┌────────┐  ┌┴────────┐  ┌────────┐  ┌────────┐   │
  │  │ CLI    │  │ Python │  │ MCP     │  │ Web    │  │ Other  │   │
  │  │ (poi)  │  │ SDK    │  │ Server  │  │ Dashbd │  │ Clients│   │
  │  └────────┘  └────────┘  └─────────┘  └────────┘  └────────┘   │
  └────────────────────────────────────────────────────────────────────┘
```

## Component Descriptions

### poi-core
The foundational library containing all protocol types, cryptographic operations, and verification logic. It has no network or storage dependencies — pure protocol logic.

**Key modules:**
- `types.rs` — `ContactProof`, `OrbitalWindow`, `Signature`, `ProofId`, `KeyPair`
- `contact_proof.rs` — Proof lifecycle: creation, signing, serialization (JSON/CBOR)
- `orbital_window.rs` — Window types (Standard, Extended, Emergency) and scheduling
- `verification.rs` — `ProofVerifier`, `FullVerificationReport`, multi-check verification
- `proof_chain.rs` — Chain management, Merkle root computation, integrity verification
- `keypair.rs` — Ed25519 key generation, PEM serialization, PQC (Dilithium5)
- `hashing.rs` — BLAKE3 hashing, SHA-256, `ProofId` derivation

### poi-networking
P2P networking layer using TLS/QUIC for peer discovery, handshake, and proof relay.

### poi-vector-store
Abstracts vector database backends (Chroma, Qdrant, LanceDB, Milvus, Weaviate) for semantic proof search.

### poi-analytics
Data lakehouse engine supporting DuckDB (embedded), Iceberg (S3), DataFusion, and Trino for large-scale analytical queries.

### poi-ai
AI/ML integration supporting multiple inference backends (Ollama, LM Studio, vLLM, SGLang, llama.cpp) for natural language queries, proof analysis, and anomaly detection.

### poi-api
REST API server built on Axum with OpenAPI documentation (utoipa), CORS, rate limiting, and Swagger UI.

### poi-api-client
Rust SDK for programmatic access to the REST API.

### poi-cli
Command-line interface for node operators and developers.

### poi-mcp-server
MCP (Model Context Protocol) server enabling AI agents (Claude Code, etc.) to interact with the network.

### poi-node
The node binary that composes all crates into a runnable service.

## Technology Choices

| Technology | Rationale |
|------------|-----------|
| **Rust** | Memory safety, zero-cost abstractions, strong crypto ecosystem |
| **Axum** | Ergonomic async HTTP, tower middleware, OpenAPI integration |
| **Ed25519 (ed25519-dalek)** | Proven, fast, small signatures (64 bytes) |
| **Dilithium5 (liboqs)** | NIST-standardized post-quantum signature for future-proofing |
| **BLAKE3** | Fastest cryptographic hash, parallelizable, streaming |
| **DuckDB** | Embedded OLAP, zero-config, Iceberg compatibility |
| **Chroma / Qdrant** | Open-source vector databases, self-hostable |
| **Prometheus / OTLP** | Industry-standard observability |
| **MCP** | Universal protocol for AI tool integration |
