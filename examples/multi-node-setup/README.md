# Multi-Node Setup Example

This directory contains configurations and scripts for running a three-node proof-of-contact network.

## Architecture

```
┌─────────────┐    P2P     ┌─────────────┐
│  Node Alpha  │◄─────────►│  Node Beta   │
│  Port 3001   │           │  Port 3002   │
└──────┬───────┘           └──────┬───────┘
       │                          │
       │         ┌────────┐      │
       └────────►│Bootstrap│◄────┘
                 │ Port 3003│
                 └──────────┘
```

## Files

| File | Purpose |
|------|---------|
| `config-alpha.toml` | Configuration for Node Alpha |
| `config-beta.toml` | Configuration for Node Beta |
| `config-bootstrap.toml` | Bootstrap node configuration |
| `run.sh` | Script to start all three nodes |
| `test-proofs.py` | Script to create cross-node proofs |

## Quick Start

```bash
# 1. Make the run script executable
chmod +x run.sh

# 2. Start all three nodes
./run.sh

# 3. In another terminal, run the test script
python test-proofs.py
```

## Manual Setup

```bash
# Terminal 1: Bootstrap node
cargo run -p poi-node -- --config examples/multi-node-setup/config-bootstrap.toml

# Terminal 2: Node Alpha
cargo run -p poi-node -- --config examples/multi-node-setup/config-alpha.toml

# Terminal 3: Node Beta
cargo run -p poi-node -- --config examples/multi-node-setup/config-beta.toml
```

## Testing Cross-Node Proofs

```bash
# Create a proof from Alpha (port 3001)
curl -X POST http://localhost:3001/api/v1/proofs \
  -H "Content-Type: application/json" \
  -d '{"target_node":"beta-node","window_id":"'$(curl -s http://localhost:3001/api/v1/windows/active | python3 -c "import sys,json; w=json.load(sys.stdin); print(w[0]['id'] if isinstance(w,list) else w['windows'][0]['id'])")'","purpose":"cross_node_test"}'
```

Or use the Python script:

```bash
python examples/multi-node-setup/test-proofs.py
```

## Verification

```bash
# Check peer connections
curl http://localhost:3001/api/v1/node/peers | python3 -m json.tool
curl http://localhost:3002/api/v1/node/peers | python3 -m json.tool

# Network analysis
python skills/network-analysis/scripts/analyze.py --api-url http://localhost:3001
```
