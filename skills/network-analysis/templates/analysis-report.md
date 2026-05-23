# Network Analysis Report

**Generated:** {{ generated_at }}

---

## Node Status

| Metric | Value |
|--------|-------|
| Node ID | `{{ node_id }}` |
| Version | {{ version }} |
| Uptime | {{ uptime }} |
| Status | {{ status }} |

## Peer Summary

| Metric | Value |
|--------|-------|
| Total Peers | {{ total_peers }} |
| Connected | {{ connected_peers }} |
| Avg Reputation | {{ avg_reputation }} |
| Avg Uptime | {{ avg_uptime }} |

## Peer Details

| Peer ID | Address | Connected Since | Reputation |
|---------|---------|-----------------|------------|
{% for peer in peers %}
| `{{ peer.id }}` | {{ peer.address }} | {{ peer.connected_since }} | {{ peer.reputation_score }} |
{% endfor %}

## Network Statistics

| Statistic | Value |
|-----------|-------|
| Total Proofs | {{ total_proofs }} |
| Active Nodes | {{ active_nodes }} |
| Messages/sec | {{ messages_per_sec }} |
| Avg Latency | {{ avg_latency }}ms |

## Health Checks

| Check | Status |
|-------|--------|
| API Reachable | {{ api_ok }} |
| Storage | {{ storage_ok }} |
| Vector Store | {{ vector_store_ok }} |
| AI Provider | {{ ai_provider_ok }} |

## Recommendations

{% if total_peers == 0 %}
- Node is isolated. Check bootstrap peers and firewall.
{% endif %}
{% if avg_reputation < 0.5 %}
- Investigate low-reputation peers; they may be misconfigured.
{% endif %}
{% if not storage_ok %}
- Storage backend is unreachable. Check database connectivity.
{% endif %}
