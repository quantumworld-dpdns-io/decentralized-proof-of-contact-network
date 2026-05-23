# Tutorial 3: Setting Up a Multi-Node Network

This tutorial demonstrates running multiple POI nodes and creating verifiable proofs-of-contact between them.

## Prerequisites

- Completed [Tutorial 1: Setup](01-setup.md)
- Basic understanding of P2P networking

## Architecture

```
┌─────────────┐         ┌─────────────┐
│  Node Alpha  │◄───P2P──►│  Node Beta   │
│  :9090       │         │  :9091       │
│  :3001       │         │  :3002       │
└─────────────┘         └─────────────┘
       │                       │
       │       ┌────────┐     │
       └──────►│Bootstrap│◄────┘
               │ :9092   │
               └────────┘
```

## Step 1: Create Configuration Files

### Node Alpha (`config-alpha.toml`)

```toml
[node]
id = "alpha-node"
data_dir = "./data/alpha"
log_level = "debug"

[network]
listen_addr = "0.0.0.0:9090"
external_addr = "127.0.0.1:9090"
bootstrap_peers = ["127.0.0.1:9092"]

[storage]
provider = "duckdb"
duckdb_path = "./data/alpha/proofs.duckdb"

[vector_store]
provider = "chroma"
host = "localhost"
port = 8000
collection = "poi-proofs-alpha"

[api]
bind_addr = "0.0.0.0:3001"
allowed_origins = ["http://localhost:3001"]

[ai]
provider = "ollama"
model = "gemma3:latest"
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"

[analytics]
duckdb_path = "./data/alpha/analytics.duckdb"
```

### Node Beta (`config-beta.toml`)

```toml
[node]
id = "beta-node"
data_dir = "./data/beta"
log_level = "debug"

[network]
listen_addr = "0.0.0.0:9091"
external_addr = "127.0.0.1:9091"
bootstrap_peers = ["127.0.0.1:9092"]

[storage]
provider = "duckdb"
duckdb_path = "./data/beta/proofs.duckdb"

[vector_store]
provider = "chroma"
host = "localhost"
port = 8000
collection = "poi-proofs-beta"

[api]
bind_addr = "0.0.0.0:3002"
allowed_origins = ["http://localhost:3002"]

[ai]
provider = "ollama"
model = "gemma3:latest"
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"

[analytics]
duckdb_path = "./data/beta/analytics.duckdb"
```

### Bootstrap Node (`config-bootstrap.toml`)

```toml
[node]
id = "bootstrap-node"
data_dir = "./data/bootstrap"
log_level = "info"

[network]
listen_addr = "0.0.0.0:9092"
external_addr = "127.0.0.1:9092"
bootstrap_peers = []

[api]
bind_addr = "0.0.0.0:3003"

[ai]
provider = "ollama"
model = "gemma3:latest"
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"

[analytics]
duckdb_path = "./data/bootstrap/analytics.duckdb"
```

## Step 2: Prepare Data Directories

```bash
mkdir -p data/alpha data/beta data/bootstrap
```

## Step 3: Start the Nodes

In separate terminal windows:

```bash
# Terminal 1: Bootstrap node
cargo run -p poi-node -- --config config-bootstrap.toml

# Terminal 2: Node Alpha
cargo run -p poi-node -- --config config-alpha.toml

# Terminal 3: Node Beta
cargo run -p poi-node -- --config config-beta.toml
```

## Step 4: Verify Network Connections

```bash
# Check Alpha's peers
curl http://localhost:3001/api/v1/node/peers

# Check Beta's peers
curl http://localhost:3002/api/v1/node/peers
```

Each node should show the other (and the bootstrap node) in its peer list.

## Step 5: Create Proofs Between Nodes

Using the Python SDK:

```python
from poi import PoiClient

# Connect to Alpha
alpha = PoiClient(base_url="http://localhost:3001")
alpha_node = alpha.get_node_info()
print(f"Alpha node: {alpha_node.id}")

# Connect to Beta
beta = PoiClient(base_url="http://localhost:3002")
beta_node = beta.get_node_info()
print(f"Beta node: {beta_node.id}")

# Create a window on Alpha (if none active)
from datetime import datetime, timedelta, timezone
now = datetime.now(timezone.utc)
window = alpha.create_window(
    start_time=now.isoformat(),
    end_time=(now + timedelta(hours=2)).isoformat(),
    window_type="standard",
)

# Alpha creates a proof about contacting Beta
proof = alpha.create_proof(
    target_node=beta_node.id,
    window_id=window.id,
    purpose="multi_node_test",
)
print(f"Proof created: {proof.id}")

# Verify on Alpha
report = alpha.verify_proof(proof.id)
print(f"Alpha verification: {'PASS' if report['all_passed'] else 'FAIL'}")

# Beta can also verify (proofs are signed, public keys are known)
# Note: Beta needs Alpha's public key to verify
report_beta = beta.verify_proof(proof.id)
print(f"Beta verification: {'PASS' if report_beta['all_passed'] else 'FAIL'}")
```

## Step 6: Cross-Node Verification

Proofs are signed with Ed25519, so any node with the proving node's public key can verify:

```python
# Get Alpha's public key
alpha_info = alpha.get_node_info()
alpha_pubkey = alpha_info.public_key

# Beta can verify Alpha's proof if it has the public key
report = beta.verify_proof(proof.id)
```

## Network Analysis

```bash
python skills/network-analysis/scripts/analyze.py --api-url http://localhost:3001
```

This will show peer connections, reputation scores, and network health.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Nodes can't find each other | Ensure bootstrap node started first; check `listen_addr` doesn't conflict |
| Connection refused | Check firewall; ensure all ports are correct |
| Proof creation fails | Verify an active orbital window exists |
| Verification fails cross-node | Ensure clocks are synchronized (NTP) |
