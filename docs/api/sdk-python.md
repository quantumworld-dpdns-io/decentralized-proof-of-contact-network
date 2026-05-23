# Python SDK Guide

The `poi` Python package provides both synchronous and asynchronous clients for the proof-of-contact network API.

## Installation

```bash
# From source (development)
pip install -e ".[dev,ai,analytics]"

# From PyPI (when published)
pip install poi-client
```

## Quick Start

```python
from poi import PoiClient

# Connect to a local node
client = PoiClient(base_url="http://localhost:3000")

# Health check
health = client.health_check()
print(health)

# Always close the client
client.close()

# Or use as context manager
with PoiClient(base_url="http://localhost:3000") as client:
    health = client.health_check()
```

## Synchronous Client (`PoiClient`)

```python
from poi import PoiClient

client = PoiClient(
    base_url="http://localhost:3000",
    api_key="your-api-key",  # optional
    timeout=30,              # request timeout in seconds
)
```

### Proof Operations

```python
# Create
proof = client.create_proof(
    target_node="poi-xxx",
    window_id="win-uuid",
    purpose="test",
)

# Read
proof = client.get_proof("proof-uuid")

# List
proofs = client.list_proofs(limit=20, offset=0)

# Verify
report = client.verify_proof("proof-uuid")

# Search
results = client.search_proofs("query terms", limit=10)

# Delete
client.delete_proof("proof-uuid")
```

### Node Operations

```python
# Node info
info = client.get_node_info()
print(info.id, info.public_key, info.peer_count)

# Peer management
peers = client.list_peers()
peer = client.connect_peer("192.168.1.2:9090")
client.disconnect_peer("peer-uuid")
```

### Window Operations

```python
# List windows
windows = client.list_windows()

# Active windows
active = client.get_active_windows()

# Create window
from datetime import datetime, timedelta, timezone
window = client.create_window(
    start_time=datetime.now(timezone.utc).isoformat(),
    end_time=(datetime.now(timezone.utc) + timedelta(hours=2)).isoformat(),
    window_type="standard",
)
```

### AI Operations

```python
# Natural language query
answer = client.ai_query("Summarize today's network activity")

# Analyze proof
analysis = client.analyze_proof("proof-uuid")

# Detect anomalies
anomalies = client.detect_anomalies()
```

### Statistics

```python
# Proof stats
stats = client.get_proof_stats()
print(stats.total_proofs, stats.verified_proofs)

# Network stats
net = client.get_network_stats()
```

## Async Client (`AsyncPoiClient`)

```python
import asyncio
from poi import AsyncPoiClient

async def main():
    async with AsyncPoiClient(base_url="http://localhost:3000") as client:
        health = await client.health_check()
        print(health)
        
        proof = await client.create_proof(
            target_node="poi-xxx",
            window_id="win-uuid",
        )
        print(f"Created proof: {proof.id}")

asyncio.run(main())
```

## Data Models

```python
from poi import Proof, NodeInfo, PeerInfo, OrbitalWindow, ProofStats

# Proof
proof = Proof(
    id="uuid",
    proving_node="poi-xxx",
    target_node="poi-yyy",
    orbital_window=OrbitalWindow(...),
    timestamp=datetime.now(timezone.utc),
    signature="base64sig",
    metadata={"protocol_version": "1.0", "confidence_score": 0.95},
)

# NodeInfo
info = NodeInfo(
    id="poi-xxx",
    public_key="base64key",
    version="0.1.0",
    uptime_seconds=3600,
    peer_count=5,
)

# PeerInfo
peer = PeerInfo(
    id="poi-yyy",
    address="192.168.1.2:9090",
    connected_since=datetime.now(timezone.utc),
    reputation_score=0.95,
)

# OrbitalWindow
window = OrbitalWindow(
    id="win-uuid",
    start_time=datetime.now(timezone.utc),
    end_time=datetime.now(timezone.utc) + timedelta(hours=1),
    window_type="standard",
)

# ProofStats
stats = ProofStats(
    total_proofs=1500,
    verified_proofs=1420,
    pending_proofs=50,
    failed_proofs=30,
    proofs_per_hour=12.5,
)
```

## Error Handling

```python
from poi import (
    PoiError,
    ApiError,
    NotFoundError,
    AuthError,
    RateLimitError,
    ValidationError,
    TimeoutError,
)

try:
    proof = client.get_proof("nonexistent-uuid")
except NotFoundError:
    print("Proof not found")
except AuthError:
    print("Invalid API key")
except RateLimitError:
    print("Too many requests")
except ValidationError as e:
    print(f"Validation error: {e}")
except TimeoutError:
    print("Request timed out")
except ApiError as e:
    print(f"API error {e.status_code}: {e.message}")
```

## Examples

See `examples/` directory for complete working examples:

- `examples/simple-proof.py` — Basic proof creation and verification
- `examples/ai-integration.py` — AI query and anomaly detection
- `examples/multi-node-setup/README.md` — Multi-node network setup
