# Security Architecture

## Threat Model (STRIDE)

### Spoofing

| Threat | Vector | Mitigation |
|--------|--------|------------|
| Fake node identity | Peer connects claiming a different NodeId | Ed25519 keypairs: NodeId derived from public key, handshake requires signature |
| Proof forgery | Attacker creates proof with another node's identity | Each proof signed by proving node's private key |
| Replay attack | Attacker replays an old proof | Timestamp verification with max drift (300s default) |

### Tampering

| Threat | Vector | Mitigation |
|--------|--------|------------|
| Modify proof fields | Man-in-the-middle alters proof in transit | Ed25519 signature covers canonical bytes; TLS encrypts P2P traffic |
| Chain manipulation | Attacker inserts/deletes proofs in chain | Chain integrity verification checks sequential positions |
| Window forgery | Attacker creates fake orbital window | Window announcements signed by announcing node |

### Repudiation

| Threat | Vector | Mitigation |
|--------|--------|------------|
| Node denies creating proof | "I never said I contacted that node" | Signed proof chain with timestamps provides non-repudiation |
| Node denies receiving proof | Recipient claims never received | Both sides create proofs (mutual verification) |

### Information Disclosure

| Threat | Vector | Mitigation |
|--------|--------|------------|
| Private key leak | Memory dumps, swap files, core dumps | `zeroize` on drop for `SecretKey`, no secret data in logs |
| Metadata leakage | Proof metadata reveals sensitive info | `proof_purpose` field is opaque; node IDs are pseudorandom UUIDs |
| Traffic analysis | Eavesdropping on P2P messages | TLS 1.3 for all P2P connections (optional QUIC) |

### Denial of Service

| Threat | Vector | Mitigation |
|--------|--------|------------|
| Proof flooding | Attacker creates millions of invalid proofs | Rate limiting on API endpoints; proof verification cost |
| Peer connection flood | Attacker opens many peer connections | Max peers config (default 50); handshake timeout (10s) |
| Storage exhaustion | Unlimited proof storage | Auto-prune with max entries (default 1M) |

### Elevation of Privilege

| Threat | Vector | Mitigation |
|--------|--------|------------|
| Unauthorized API access | Attacker calls admin endpoints | API key authentication; RBAC-ready endpoint design |
| Node compromise | Attacker gains shell access | Secret key encrypted at rest; key rotation support |

## Security Controls

### Cryptographic Controls

| Control | Implementation |
|---------|---------------|
| Key generation | `OsRng` (system entropy), never deterministic |
| Signature algorithm | Ed25519 (ed25519-dalek v2.1.1) |
| Post-quantum overlay | Dilithium5 (liboqs, behind `pqc` feature) |
| Hashing | BLAKE3 (primary), SHA-256 (API compatibility) |
| Key serialization | PEM format with `"-----BEGIN POI PRIVATE KEY-----"` |
| Memory security | `zeroize` clears secret keys on drop |

### Network Security Controls

| Control | Implementation |
|---------|---------------|
| P2P transport | TLS 1.3 (rustls), optional QUIC (quinn) |
| Peer authentication | Ed25519 handshake signature |
| Rate limiting | Tower middleware on API server |
| CORS | Configurable allowed origins |
| TLS termination | Optional for REST API (config.dev/prod) |

### Storage Security Controls

| Control | Implementation |
|---------|---------------|
| Data at rest | DuckDB file with filesystem permissions |
| Auto-prune | Removes proofs older than configured threshold |
| Chain integrity | Merkle root verification detects tampering |

## PQC Integration

The post-quantum cryptography (PQC) layer provides defense in depth against future quantum adversaries:

### Architecture

```
Proof Signing (ed25519) ──── canonical_bytes ──── Proof Verification
                         │
                    (optional)
                         │
Proof Signing (Dilithium5) ── canonical_bytes ──── PQC Verification
```

### Key Management

- PQC keys are derived from the same seed material as Ed25519 keys
- Both signatures coexist in `ContactProof` (`signature` + `pqc_signature`)
- Verification can check either or both signatures
- Feature-gated: `cargo build --features pqc`

### Risks

- Dilithium5 signatures are larger than Ed25519 (~2.5KB vs 64 bytes)
- liboqs is a C library via FFI — memory safety depends on correct usage
- Not yet standardized by NIST at ML-DSA final (though very close — FIPS 204)

## Security Configuration

### config.prod.toml (production)

```toml
[node]
# Node ID should be set via environment, not hardcoded
id = "${NODE_ID}"

[api]
# TLS recommended for production
tls_cert = "/etc/poi/tls.crt"
tls_key = "/etc/poi/tls.key"
# Strict CORS in production
allowed_origins = ["${DASHBOARD_URL}"]

[storage]
# Regular backups recommended
auto_prune = true

[observability]
# Metrics should not expose internal details
prometheus_port = 9091
```

## Vulnerability Disclosure

See [SECURITY.md](../../SECURITY.md) for the vulnerability disclosure process, supported versions, and PGP key for secure communication.
