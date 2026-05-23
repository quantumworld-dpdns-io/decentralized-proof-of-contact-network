# Getting Started

This guide walks you through setting up and running the decentralized proof-of-contact network.

## Prerequisites

- **Rust toolchain**: `rustup` (1.75+)
- **Python**: 3.11+ (for SDK and scripts)
- **Docker** (optional, for vector stores and full stack)

## Installation

### 1. Clone and Build

```bash
git clone https://github.com/quantumworld-dpdns-io/decentralized-proof-of-contact-network.git
cd decentralized-proof-of-contact-network

# Build all workspace crates
cargo build --workspace

# Or use the Makefile
make build
```

### 2. Development Setup

```bash
make dev-setup
```

This installs: `cargo-audit`, `cargo-deny`, `cargo-tarpaulin`, `clippy`, `rustfmt`, and Robot Framework.

### 3. Python SDK

```bash
# Install with dev dependencies
pip install -e ".[dev]"
```

Or install the Python client directly:

```bash
pip install ./python-client
```

### 4. Configuration

Copy the development configuration:

```bash
cp config.dev.toml config.toml
```

Edit `config.toml` to set your node ID and preferences:

```toml
[node]
id = "dev-node-001"
data_dir = "./data"
log_level = "debug"

[network]
listen_addr = "0.0.0.0:9090"
bootstrap_peers = []

[api]
bind_addr = "0.0.0.0:3000"

[ai]
provider = "ollama"
model = "gemma3:latest"
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"
```

### 5. Start the Node

```bash
# Development mode with auto-reload (if configured)
make dev-node

# Or directly:
cargo run -p poi-node -- --config config.toml
```

### 6. Verify It's Running

```bash
curl http://localhost:3000/api/v1/health
```

Expected response:
```json
{
  "status": "ok",
  "node_id": "poi-xxx...",
  "version": "0.1.0",
  "uptime_seconds": 10
}
```

### 7. Create Your First Proof

Using the CLI:

```bash
cargo run -p poi-cli -- proof create --target "poi-target-uuid" --window "window-uuid"
```

Using Python:

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")
proof = client.create_proof(
    target_node="poi-550e8400-e29b-41d4-a716-446655440001",
    window_id="6ba7b810-9dad-11d1-80b4-00c04fd430c8",
)
print(f"Created proof: {proof.id}")
```

## Next Steps

- [Tutorial: Environment Setup](tutorials/01-setup.md)
- [Tutorial: Creating Your First Proof](tutorials/02-first-proof.md)
- [Tutorial: Multi-Node Network](tutorials/03-multi-node.md)
- [API Reference](api/rest-api.md)
- [Python SDK Guide](api/sdk-python.md)
