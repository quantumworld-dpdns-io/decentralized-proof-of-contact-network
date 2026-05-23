# Monitoring Setup

## Metrics Overview

The POI node exposes Prometheus-compatible metrics on port `9091` (configurable).

## Metrics Endpoint

```bash
curl http://localhost:9091/metrics
```

## Key Metrics

### Proof Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `poi_proofs_created_total` | Counter | Total proofs created |
| `poi_proofs_verified_total` | Counter | Total successful verifications |
| `poi_proofs_failed_total` | Counter | Total failed verifications |
| `poi_proofs_rejected_total` | Counter | Total proofs rejected |
| `poi_active_proofs` | Gauge | Currently active proofs |
| `poi_chain_length` | Gauge | Current proof chain length |

### Network Metrics (exposed by networking crate)

| Metric | Description |
|--------|-------------|
| `poi_peers_connected` | Number of connected peers |
| `poi_network_messages_sent` | Messages sent |
| `poi_network_messages_received` | Messages received |
| `poi_network_bytes_sent` | Bytes sent |
| `poi_network_bytes_received` | Bytes received |

## Prometheus Configuration

```yaml
# docker/prometheus/prometheus.yml
scrape_configs:
  - job_name: "poi-node"
    scrape_interval: 15s
    static_configs:
      - targets:
          - "node1:9091"
          - "node2:9091"
          - "node3:9091"
```

## Grafana Dashboards

Dashboards are located in `grafana/dashboards/` and auto-provisioned when using Docker Compose.

### Available Dashboards

- **POI Node Overview** — Node health, peer count, proof rates
- **Proof Analytics** — Proof creation/verification trends, chain depth
- **Network Health** — Peer connections, message throughput, latency

### Importing Dashboards Manually

1. Open Grafana (http://localhost:3001, default admin/admin)
2. Configuration → Data Sources → Add Prometheus
3. Set URL to `http://prometheus:9090`
4. Create → Import → Upload dashboard JSON from `grafana/dashboards/`

## Structured Logging

The node outputs structured JSON logs:

```json
{
  "timestamp": "2026-05-23T12:00:00.123456Z",
  "level": "INFO",
  "target": "poi_node",
  "message": "Proof created",
  "proof_id": "550e8400-e29b-41d4-a716-446655440000",
  "proving_node": "poi-xxx",
  "target_node": "poi-yyy",
  "window_id": "win-uuid"
}
```

Configure log level in `config.toml`:

```toml
[node]
log_level = "info"  # trace, debug, info, warn, error
```

## OpenTelemetry Tracing

The node supports OpenTelemetry for distributed tracing:

```toml
[observability]
otlp_endpoint = "http://otel-collector:4317"
```

Traces can be exported to Jaeger, Tempo, or any OTLP-compatible backend.

## Alerting Rules

### Prometheus Alert Rules

```yaml
groups:
  - name: poi-alerts
    rules:
      - alert: NodeDown
        expr: up{job="poi-node"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "POI node {{ $labels.instance }} is down"

      - alert: NoPeersConnected
        expr: poi_peers_connected == 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Node {{ $labels.instance }} has no connected peers"

      - alert: HighProofFailureRate
        expr: rate(poi_proofs_failed_total[5m]) / rate(poi_proofs_created_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High proof failure rate on {{ $labels.instance }}"

      - alert: ChainIntegrityFailure
        expr: poi_chain_integrity_failures > 0
        labels:
          severity: critical
        annotations:
          summary: "Chain integrity failure detected on {{ $labels.instance }}"
```

## Log Aggregation

### Filebeat Configuration

```yaml
filebeat.inputs:
  - type: log
    paths:
      - /var/lib/poi/logs/*.log
    json.keys_under_root: true
    json.overwrite_keys: true

output.elasticsearch:
  hosts: ["http://elasticsearch:9200"]
  index: "poi-logs-%{+yyyy.MM.dd}"
```

## Recommended Monitoring Stack

| Component | Tool | Purpose |
|-----------|------|---------|
| Metrics | Prometheus | Time-series metric collection |
| Dashboards | Grafana | Visualization and alerting |
| Logs | Elasticsearch + Filebeat + Kibana | Log aggregation and search |
| Tracing | OpenTelemetry + Jaeger/Tempo | Distributed tracing |
| Uptime | Uptime Kuma / Checkmk | External health monitoring |
