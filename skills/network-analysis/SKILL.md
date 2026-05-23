---
name: Network Analysis
description: Analyze network topology, peer connections, and node health
trigger: When the user asks about network status, peer connections, or topology analysis
---

# Network Analysis Skill

Analyze the peer-to-peer network topology, peer connection health, and overall network state.

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/node` | Local node info |
| `GET /api/v1/node/peers` | List connected peers |
| `GET /api/v1/stats/network` | Network-wide statistics |

## Python SDK Usage

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")

# Get node info
node = client.get_node_info()
print(f"Node ID: {node.id}, Peers: {node.peer_count}")

# List peers
for peer in client.list_peers():
    print(f"Peer {peer.id} at {peer.address} (score: {peer.reputation_score})")

# Network stats
stats = client.get_network_stats()
```

## Using the Analysis Script

```bash
python skills/network-analysis/scripts/analyze.py --api-url http://localhost:3000
```

## Metrics to Monitor

- **Peer count** — Should be stable; sudden drops indicate network partitions
- **Reputation scores** — Low scores may indicate misbehaving peers
- **Connection uptime** — Long-lived connections are healthy
- **Message latency** — High latency may indicate geographic dispersion or network issues

## Troubleshooting

- **Cannot connect to peers**: Check `bootstrap_peers` in config and firewall rules
- **Peer count is 0**: Node may not have discovered the network; check bootstrap nodes
- **High peer churn**: Could indicate network instability or configuration mismatch
