# REST API Reference

Base URL: `http://<node>:3000/api/v1`

## Authentication

Most endpoints accept an optional API key via the `Authorization: Bearer <key>` header.

## Endpoints

### Health

```
GET /api/v1/health
```

Response:
```json
{
  "status": "ok",
  "node_id": "poi-550e8400-e29b-41d4-a716-446655440000",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### Node

#### Get Node Info

```
GET /api/v1/node
```

Response:
```json
{
  "id": "poi-550e8400-e29b-41d4-a716-446655440000",
  "public_key": "base64_public_key",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "peer_count": 5
}
```

#### List Peers

```
GET /api/v1/node/peers
```

Response:
```json
{
  "peers": [
    {
      "id": "poi-peer-uuid",
      "address": "192.168.1.2:9090",
      "connected_since": "2026-05-23T12:00:00Z",
      "reputation_score": 0.95
    }
  ]
}
```

#### Connect Peer

```
POST /api/v1/node/peers
```

Body:
```json
{
  "address": "192.168.1.2:9090"
}
```

#### Disconnect Peer

```
DELETE /api/v1/node/peers/{peer_id}
```

### Proofs

#### Create Proof

```
POST /api/v1/proofs
```

Body:
```json
{
  "target_node": "poi-target-uuid",
  "window_id": "window-uuid",
  "purpose": "node_sync"
}
```

Response: Full `ContactProof` object.

#### Get Proof

```
GET /api/v1/proofs/{proof_id}
```

Returns the `ContactProof` object.

#### List Proofs

```
GET /api/v1/proofs?limit=50&offset=0
```

Response:
```json
{
  "proofs": [...],
  "total": 100,
  "limit": 50,
  "offset": 0
}
```

#### Verify Proof

```
POST /api/v1/proofs/{proof_id}/verify
```

Response:
```json
{
  "signature_valid": true,
  "timestamp_valid": true,
  "orbital_window_valid": true,
  "chain_integrity_valid": true,
  "all_passed": true
}
```

#### Search Proofs

```
GET /api/v1/proofs/search?q=query&limit=10
```

Semantic search using vector embeddings.

#### Delete Proof

```
DELETE /api/v1/proofs/{proof_id}
```

### Windows

#### List Windows

```
GET /api/v1/windows
```

#### Get Active Windows

```
GET /api/v1/windows/active
```

#### Create Window

```
POST /api/v1/windows
```

Body:
```json
{
  "start_time": "2026-05-23T14:00:00Z",
  "end_time": "2026-05-23T15:00:00Z",
  "window_type": "standard"
}
```

### AI

#### Natural Language Query

```
POST /api/v1/ai/query
```

Body:
```json
{
  "question": "How many proofs were created today?"
}
```

Response:
```json
{
  "answer": "15 proofs were created in the last 24 hours.",
  "confidence": 0.92
}
```

#### Analyze Proof

```
GET /api/v1/ai/analyze/{proof_id}
```

#### Detect Anomalies

```
GET /api/v1/ai/anomalies
```

### Stats

#### All Stats

```
GET /api/v1/stats
```

#### Proof Stats

```
GET /api/v1/stats/proofs
```

Response:
```json
{
  "total_proofs": 1500,
  "verified_proofs": 1420,
  "pending_proofs": 50,
  "failed_proofs": 30,
  "proofs_per_hour": 12.5
}
```

#### Network Stats

```
GET /api/v1/stats/network
```

## Error Responses

```json
{
  "error": "description of the error",
  "code": "ERROR_CODE",
  "status": 400
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 400 | Bad request |
| 401 | Unauthorized |
| 404 | Not found |
| 422 | Validation error |
| 429 | Rate limit exceeded |
| 500 | Internal server error |

## Rate Limiting

Default rate limits:
- Read endpoints: 100 requests/minute
- Write endpoints: 20 requests/minute
- AI queries: 10 requests/minute

## OpenAPI Spec

An interactive Swagger UI is available at `http://localhost:3000/swagger-ui/` when running in development mode.
