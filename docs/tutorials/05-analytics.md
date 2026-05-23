# Tutorial 5: Running Analytics

This tutorial demonstrates using the analytics engine (DuckDB, Iceberg) to run powerful queries against proof data.

## Prerequisites

- A running POI node (see [Tutorial 1](01-setup.md))
- `poi-analytics` features enabled (default)
- Some proof data generated in the system

## Step 1: Verify Analytics Engine

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")

# Check node health includes analytics info
health = client.health_check()
print(f"Analytics engine: {health.get('analytics', 'checking...')}")
```

## Step 2: Get Proof Statistics

```python
stats = client.get_proof_stats()
print(f"Proof Statistics:")
print(f"  Total proofs:    {stats.total_proofs}")
print(f"  Verified:        {stats.verified_proofs}")
print(f"  Pending:         {stats.pending_proofs}")
print(f"  Failed:          {stats.failed_proofs}")
print(f"  Proofs/hour:     {stats.proofs_per_hour:.2f}")
```

## Step 3: Network-Wide Statistics

```python
network_stats = client.get_network_stats()
print(f"Network Stats:")
for key, value in network_stats.items():
    print(f"  {key}: {value}")
```

## Step 4: Using the Analytics Script

```bash
python skills/network-analysis/scripts/analyze.py --api-url http://localhost:3000
```

## Step 5: Advanced Analytics with DuckDB

The embedded DuckDB analytics engine supports SQL queries on proof data:

```bash
# Query the DuckDB database directly (if available)
python -c "
import duckdb
conn = duckdb.connect('./data/analytics.duckdb')
result = conn.execute('''
    SELECT 
        proving_node,
        COUNT(*) as proof_count,
        AVG(confidence_score) as avg_confidence,
        MIN(timestamp) as first_proof,
        MAX(timestamp) as last_proof
    FROM proofs
    GROUP BY proving_node
    ORDER BY proof_count DESC
''').fetchdf()
print(result)
"
```

## Step 6: Time-Series Analysis

```python
# Using the AI query for temporal analysis
client = PoiClient(base_url="http://localhost:3000")

answer = client.ai_query(
    "Show me the proof creation rate over the last 24 hours, broken down by hour"
)
print(answer)
```

## Step 7: Iceberg Integration (Production)

For production deployments, analytics data can be stored in Apache Iceberg format on S3:

```toml
[analytics]
iceberg_warehouse = "s3://poi-analytics/iceberg"
```

Queries can then use any Iceberg-compatible engine (DuckDB, Spark, Trino, Athena).

## Analytics Dashboard

Metrics are exposed via Prometheus:

```bash
curl http://localhost:9091/metrics | grep poi_
```

Key metrics:
- `poi_proofs_created_total` — Total proofs created
- `poi_proofs_verified_total` — Total verifications performed
- `poi_proofs_failed_total` — Failed verifications
- `poi_active_proofs` — Currently active proofs
- `poi_chain_length` — Current chain depth

## Grafana Dashboards

Import the dashboards from `grafana/dashboards/` into your Grafana instance:

```bash
# Dashboards are automatically provisioned if using docker-compose
make docker-up
# Grafana available at http://localhost:3001
```

## Example Analytics Queries

### Top proving nodes

```sql
SELECT proving_node, COUNT(*) as proofs_created
FROM proofs
WHERE timestamp >= NOW() - INTERVAL '7 days'
GROUP BY proving_node
ORDER BY proofs_created DESC
LIMIT 10;
```

### Proof success rate

```sql
SELECT 
    COUNT(*) as total,
    SUM(CASE WHEN verification_status = 'verified' THEN 1 ELSE 0 END) as verified,
    SUM(CASE WHEN verification_status = 'verified' THEN 1 ELSE 0 END)::FLOAT / COUNT(*) as success_rate
FROM proofs;
```

### Active windows

```sql
SELECT id, window_type, start_time, end_time
FROM windows
WHERE start_time <= NOW() AND end_time >= NOW()
ORDER BY end_time ASC;
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| DuckDB file not found | Ensure the node has created at least one proof |
| Zero stats | Analytics may require a restart after first proof creation |
| Iceberg queries slow | Check S3 connectivity; Iceberg works best with AWS S3 or MinIO |
| Prometheus metrics empty | Verify `metrics_enabled = true` in config |
