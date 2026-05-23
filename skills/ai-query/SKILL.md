---
name: AI Query
description: Natural language queries against the proof-of-contact network data
trigger: When the user asks a question about network data or wants to query using natural language
---

# AI Query Skill

Use natural language to ask questions about the proof-of-contact network, its proofs, peers, and analytics.

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /api/v1/ai/query` | Submit a natural language query |
| `GET /api/v1/ai/analyze/{proof_id}` | Get AI analysis of a specific proof |

## Python SDK Usage

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")

# Ask a question
answer = client.ai_query("Which node has created the most proofs in the last hour?")
print(answer)

# Search proofs semantically
results = client.search_proofs("high confidence proofs between node A and B")
```

## AI Provider Configuration

The AI module supports multiple backends configured in `config.toml`:

```toml
[ai]
provider = "ollama"          # ollama, lm-studio, vllm, sglang, llamacpp
model = "gemma3:latest"      # Model name
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"
```

## Query Examples

| Query | What it does |
|-------|-------------|
| "Show me all proofs from node poi-abc..." | Filters proofs by proving node |
| "What's the current network health?" | Summarizes peer stats and uptime |
| "Are there any anomalies today?" | Lists recent anomalies |
| "Which windows are active?" | Lists currently active orbital windows |
| "Compare proof rates between node A and B" | Comparative analysis |

## Using the Query Script

```bash
python skills/ai-query/scripts/query.py --api-url http://localhost:3000 "Which nodes have the highest proof counts?"
python skills/ai-query/scripts/query.py --api-url http://localhost:3000 --output report.md --template templates/query-results.md "Show network health summary"
```

## Troubleshooting

- **"AI provider not available"**: Ensure the AI provider is running and the endpoint is correctly configured
- **Empty responses**: The query may be too vague; try more specific wording
- **Slow responses**: Vector store queries depend on embedding model performance
- **Model not found**: Verify the model name matches what's available in your AI provider
