# Tutorial 6: OWASP Security Testing

This tutorial covers security testing of the proof-of-contact network using OWASP Top 10 methodology and Robot Framework test suites.

## Prerequisites

- A running POI node (see [Tutorial 1](01-setup.md))
- Robot Framework installed: `pip install robotframework robotframework-requests`
- Basic understanding of web security concepts

## Step 1: Running the OWASP Test Suite

The project includes Robot Framework test suites for OWASP Top 10 testing:

```bash
make test-robot-owasp
```

This runs `tests/robot/test-suites/99-security/owasp-top10.robot` against your node.

## Step 2: Manual Security Testing

### A1: Broken Access Control

```bash
# Try accessing without API key
curl http://localhost:3000/api/v1/proofs
# Expected: 401 Unauthorized or limited access

# Try accessing another node's data
curl http://localhost:3000/api/v1/node
# Should only return local node info
```

### A2: Cryptographic Failures

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")

# Verify signatures are properly validated
proofs = client.list_proofs(limit=5)
for proof in proofs:
    report = client.verify_proof(proof.id)
    # Ensure signature validation is strict
    assert isinstance(report['signature_valid'], bool)
```

### A3: Injection

```python
# SQL injection attempts (DuckDB backend)
try:
    client.search_proofs("'; DROP TABLE proofs; --")
except Exception as e:
    print(f"Injection prevented: {e}")

# NoSQL injection (if using vector store)
try:
    client.search_proofs('{"$gt": ""}')
except Exception as e:
    print(f"JSON injection prevented: {e}")
```

### A4: Insecure Design

```python
# Check for rate limiting
import time
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")
start = time.time()
for i in range(100):
    try:
        client.health_check()
    except Exception as e:
        print(f"Rate limited after {i} requests at {time.time() - start:.1f}s: {e}")
        break
```

### A5: Security Misconfiguration

```bash
# Check for debug endpoints
curl http://localhost:3000/debug
curl http://localhost:3000/.env
curl http://localhost:3000/config
# All should return 404

# Check CORS headers
curl -I -X OPTIONS -H "Origin: https://evil.com" http://localhost:3000/api/v1/health
# Should not reflect arbitrary origins in production
```

### A6: Vulnerable Components

```bash
# Run dependency audits
cargo audit
cargo deny check

# Docker image scan (if using Docker)
make docker-node
docker scan poi-node:latest
```

### A7: Identification and Authentication Failures

```python
# Test weak authentication
from poi import PoiClient

# Try without API key
client = PoiClient(base_url="http://localhost:3000")
try:
    client.get_node_info()  # May work without auth (read-only)
    print("Read-only access allowed")
except Exception as e:
    print(f"Auth required: {e}")
```

### A8: Software and Data Integrity Failures

```bash
# Verify signed commits
git verify-commit HEAD

# Check for unsigned dependencies
cargo deny check sources
```

### A9: Security Logging and Monitoring

```python
# Check that operations are logged
import httpx

# Trigger various operations
operations = [
    ("GET", "/api/v1/health"),
    ("GET", "/api/v1/proofs"),
    ("POST", "/api/v1/proofs"),
    ("GET", "/api/v1/ai/anomalies"),
]

for method, path in operations:
    resp = httpx.request(method, f"http://localhost:3000{path}")
    print(f"{method} {path}: {resp.status_code}")
```

## Step 3: Running the Full Security Audit

```bash
# Full CI pipeline (includes lint, build, test)
make ci

# Security-specific audit
make audit

# OWASP Top 10 tests
make test-robot-owasp
```

## Step 4: Verifying Security Headers

```bash
curl -sI http://localhost:3000/api/v1/health | grep -iE "^(content-security-policy|strict-transport-security|x-content-type-options|x-frame-options)"
```

## Security Configuration Checklist

- [ ] API authentication enabled in production
- [ ] TLS configured for REST API
- [ ] CORS restricted to known origins
- [ ] Rate limiting enabled
- [ ] Auto-prune enabled for storage
- [ ] `enable_pqc` set appropriately
- [ ] `log_level` set to `info` or `warn` (not `debug`) in production
- [ ] Metrics endpoint not exposed publicly
- [ ] Regular `cargo audit` runs in CI

## Reporting Vulnerabilities

If you discover a security vulnerability, follow the process in [SECURITY.md](../../SECURITY.md). Do not open public GitHub issues for security vulnerabilities.
