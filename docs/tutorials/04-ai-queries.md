# Tutorial 4: Using AI Queries

This tutorial shows how to use natural language queries against the proof-of-contact network using the built-in AI engine.

## Prerequisites

- A running POI node with AI provider configured (see [Tutorial 1](01-setup.md))
- Ollama running with `gemma3:latest` and `nomic-embed-text:latest`
- Some proofs already created in the network

## Step 1: Verify AI Provider

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")

# Quick health check
health = client.health_check()
print(f"AI available: {health.get('ai_provider', 'unknown')}")
```

## Step 2: Basic Natural Language Query

```python
# Ask a simple question
answer = client.ai_query("How many proofs are in the system?")
print(f"Answer: {answer}")
```

## Step 3: Complex Queries

```python
questions = [
    "Which node has the most proofs?",
    "Show me proofs created in the last hour",
    "Are there any anomalies in the network?",
    "What's the average confidence score across all proofs?",
    "Compare proof activity between alpha and beta nodes",
]

for question in questions:
    print(f"\nQ: {question}")
    answer = client.ai_query(question)
    print(f"A: {answer}")
```

## Step 4: Semantic Search

Search proofs by meaning, not just keywords:

```python
results = client.search_proofs(
    "high confidence proofs between peer nodes",
    limit=5,
)
for r in results:
    print(f"  [{r.get('id', 'N/A')[:8]}...] {r.get('metadata', {}).get('proof_purpose', 'N/A')}")
```

## Step 5: Proof Analysis

Get AI-powered analysis of a specific proof:

```python
# Get a proof ID first
proofs = client.list_proofs(limit=1)
if proofs:
    proof_id = proofs[0].id
    analysis = client.analyze_proof(proof_id)
    print(f"Analysis for {proof_id}:")
    print(analysis)
```

## Step 6: Using the AI Query Script

```bash
# Simple query
python skills/ai-query/scripts/query.py --api-url http://localhost:3000 \
    "Summarize the network activity for today"

# Get structured analysis
python skills/ai-query/scripts/query.py --api-url http://localhost:3000 \
    --json "List all active windows"

# Analyze a specific proof
python skills/ai-query/scripts/query.py --api-url http://localhost:3000 \
    --analyze "550e8400-e29b-41d4-a716-446655440000"

# Save results to file
python skills/ai-query/scripts/query.py --api-url http://localhost:3000 \
    --output network-summary.md \
    "Give me a complete network summary"
```

## Step 7: Anomaly Detection

```python
# Detect anomalies
anomalies = client.detect_anomalies()
for anomaly in anomalies:
    print(f"[{anomaly['severity']}] {anomaly['type']}: {anomaly.get('description', '')}")

# Ask AI about anomalies
answer = client.ai_query("Are there any suspicious patterns in the proof data?")
print(f"Analysis: {answer}")
```

## AI Provider Options

### Ollama (Default)

```toml
[ai]
provider = "ollama"
model = "gemma3:latest"          # Any Ollama model
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"
```

### LM Studio

```toml
[ai]
provider = "lm-studio"
model = "local-model"
embedding_model = "local-embeddings"
endpoint = "http://localhost:1234"
```

### vLLM

```toml
[ai]
provider = "vllm"
model = "mistralai/Mistral-7B-Instruct-v0.3"
embedding_model = "BAAI/bge-small-en-v1.5"
endpoint = "http://localhost:8000"
api_key = "${VLLM_API_KEY}"
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "AI provider not available" | Ensure Ollama is running (`ollama serve`) |
| Slow queries | Embedding model may be slow on CPU; consider GPU acceleration |
| Empty responses | Rephrase query to be more specific |
| Model not found | Pull the model: `ollama pull gemma3:latest` |
