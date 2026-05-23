---
name: Anomaly Detection
description: Detect anomalies in contact proof patterns and network behavior
trigger: When the user asks to detect anomalies, check for suspicious activity, or analyze contact patterns
---

# Anomaly Detection Skill

Detect suspicious or anomalous patterns in proof-of-contact creation, network behavior, and peer interactions.

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/ai/anomalies` | List detected anomalies |
| `POST /api/v1/ai/analyze/{proof_id}` | Analyze a specific proof for anomalies |

## Python SDK Usage

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")

# Get all anomalies
anomalies = client.detect_anomalies()
for anomaly in anomalies:
    print(f"[{anomaly['severity']}] {anomaly['type']}: {anomaly['description']}")

# Analyze a specific proof
analysis = client.analyze_proof("550e8400-e29b-41d4-a716-446655440000")
print(analysis)
```

## Anomaly Types

| Type | Description | Severity |
|------|-------------|----------|
| `high_frequency` | Unusually rapid proof creation from a node | Medium |
| `stale_proofs` | Proofs with expired windows being submitted | Low |
| `signature_anomaly` | Proofs with unusual signature patterns | High |
| `unusual_window` | Proofs created outside normal window patterns | Medium |
| `chain_gap` | Missing proofs in a chain sequence | High |
| `new_peer_flood` | Rapid connection/disconnection from new peers | Medium |

## Using the Detection Script

```bash
python skills/anomaly-detection/scripts/detect.py --api-url http://localhost:3000
python skills/anomaly-detection/scripts/detect.py --api-url http://localhost:3000 --severity high --json
```

## Interpreting Results

- **High severity** anomalies should be investigated immediately
- **Medium severity** may indicate misconfiguration or pattern shifts
- **Low severity** are informational (e.g., expired proofs)
- Multiple anomalies from the same node may indicate compromised keys

## Troubleshooting

- **False positives**: Adjust sensitivity parameters in node config
- **Missing anomalies**: Ensure AI provider is configured and reachable
- **Stale anomaly data**: Anomalies are time-bound; check the window range
