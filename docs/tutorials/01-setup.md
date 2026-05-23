# Tutorial 1: Environment Setup

This tutorial walks through setting up a complete development environment for the proof-of-contact network, including the AI backend and vector store.

## Objectives

By the end of this tutorial, you will have:
- A running POI node with API access
- Ollama running with a language model
- A Chroma vector store for embeddings
- The Python SDK configured

## Step 1: Install System Dependencies

### macOS

```bash
# Install Homebrew if not present
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install rustup-init python@3.12 ollama
```

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev python3.12 python3.12-venv
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Step 2: Install Rust Toolchain

```bash
rustup install stable
rustup default stable
rustup component add clippy rustfmt
```

## Step 3: Clone and Build

```bash
git clone https://github.com/quantumworld-dpdns-io/decentralized-proof-of-contact-network.git
cd decentralized-proof-of-contact-network
cargo build --workspace
```

## Step 4: Set Up AI Backend (Ollama)

```bash
# Start Ollama
ollama serve

# In another terminal, pull models
ollama pull gemma3:latest          # Main LLM
ollama pull nomic-embed-text:latest # Embeddings
```

Verify Ollama is running:
```bash
curl http://localhost:11434/api/tags
```

## Step 5: Set Up Vector Store (Chroma)

```bash
# Using Docker
docker run -d --name chroma -p 8000:8000 chromadb/chroma:latest
```

Or install the Python package:
```bash
pip install chromadb
chroma run --path ./data/chroma
```

## Step 6: Configure the Node

Create `config.toml` from the dev template:

```bash
cp config.dev.toml config.toml
```

Edit the AI section to match your setup:

```toml
[ai]
provider = "ollama"
model = "gemma3:latest"
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"
```

## Step 7: Install Python SDK

```bash
pip install -e ".[dev,ai,analytics]"
```

Verify:
```bash
python -c "from poi import PoiClient; print('SDK installed successfully')"
```

## Step 8: Start the Node

```bash
make dev-node
```

You should see:
```
2026-05-23T12:00:00Z INFO poi_node: Starting node id=poi-xxx...
2026-05-23T12:00:00Z INFO poi_api: API server listening on 0.0.0.0:3000
2026-05-23T12:00:00Z INFO poi_node: Node startup complete
```

## Step 9: Run Health Check

```bash
curl http://localhost:3000/api/v1/health
```

Expected:
```json
{"status":"ok","node_id":"poi-xxx...","version":"0.1.0","uptime_seconds":5}
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `cargo build` fails | Ensure `libssl-dev` (Linux) or `openssl` (macOS via brew) is installed |
| Ollama connection refused | Start `ollama serve` before the node |
| Chroma not reachable | Check Docker: `docker ps` — ensure Chroma is running on port 8000 |
| Port 3000 in use | Change `bind_addr` in config.toml to a different port |

## Verification

Run the verification script:

```bash
python skills/network-analysis/scripts/analyze.py --api-url http://localhost:3000
```

You should see a network analysis report showing your running node.
