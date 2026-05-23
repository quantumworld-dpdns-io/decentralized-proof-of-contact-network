# Protocol Specification

## Proof-of-Contact Data Structure

A `ContactProof` certifies that the `proving_node` communicated with `target_node` during a specific orbital window.

### Wire Format (JSON)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "proving_node": "poi-550e8400-e29b-41d4-a716-446655440000",
  "target_node": "poi-550e8400-e29b-41d4-a716-446655440001",
  "orbital_window": {
    "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "start_time": "2026-05-23T14:00:00Z",
    "end_time": "2026-05-23T15:00:00Z",
    "window_type": "Standard"
  },
  "timestamp": "2026-05-23T14:30:00Z",
  "signature": "base64_ed25519_signature_here",
  "pqc_signature": "base64_dilithium5_signature_here_or_null",
  "metadata": {
    "protocol_version": "1.0",
    "chain_position": 42,
    "confidence_score": 0.95,
    "proof_purpose": "node_sync"
  }
}
```

### CBOR Binary Format

For efficient P2P transmission, proofs are serialized using CBOR (RFC 8949). The CBOR encoding is deterministic — maps are sorted by key and floats are encoded as 64-bit IEEE 754.

### Canonical Bytes (for signing)

The canonical byte representation is constructed deterministically by concatenating:

1. `proof.id` (16 bytes as UUID bytes)
2. `proving_node.0` (UTF-8 bytes)
3. `target_node.0` (UTF-8 bytes)
4. `orbital_window.id` (16 bytes as UUID bytes)
5. `orbital_window.start_time` (8 bytes big-endian i64 timestamp)
6. `orbital_window.end_time` (8 bytes big-endian i64 timestamp)
7. `proof.timestamp` (8 bytes big-endian i64 timestamp)
8. `metadata.protocol_version` (UTF-8 bytes)
9. `metadata.chain_position` (8 bytes big-endian u64, if present)
10. `metadata.proof_purpose` (UTF-8 bytes)

**Important**: The canonical bytes must match exactly on all platforms. Timestamps use big-endian encoding to ensure cross-platform determinism.

## Signing Protocol

### Ed25519 (Primary)

1. Generate keypair: `KeyPair::generate()` uses `OsRng` for secure randomness
2. Serialize proof to canonical bytes: `proof.canonical_bytes()`
3. Sign: `SigningKey::sign(&canonical_bytes)` → 64-byte Ed25519 signature
4. Encode: base64-encode the signature bytes → stored in `proof.signature`
5. Verify: `VerifyingKey::verify(&canonical_bytes, &signature)` → boolean

### Dilithium5 (Post-Quantum, Optional)

Gated behind the `pqc` feature flag. Uses the `liboqs` Rust bindings:

1. Sign: `Sig::new(Dilithium5).sign(&canonical_bytes)` → variable-length signature
2. Store: base64-encoded in `proof.pqc_signature`
3. Verify: `Sig::new(Dilithium5).verify(&canonical_bytes, &signature)` → boolean

When PQC is enabled, proofs carry both an Ed25519 and a Dilithium5 signature, providing defense in depth against both classical and quantum adversaries.

### PEM Key Format

```
-----BEGIN POI PRIVATE KEY-----
base64_encoded_32_byte_secret_key_split_across_multiple_lines
-----END POI PRIVATE KEY-----
```

## Orbital Window Calculation

### Window Types

| Type | Typical Duration | Purpose |
|------|-----------------|---------|
| `Standard` | 1 hour (hourly) or 24 hours (daily) | Routine contact proofs at regular intervals |
| `Extended` | Configurable (N hours) | Flexible/longer windows for relaxed schedules |
| `Emergency` | 30 minutes | Urgent proofs outside normal schedule |

### Helper Functions

- `hourly_window()` — Aligned to the current hour boundary, 1-hour duration
- `daily_window()` — Aligned to midnight UTC, 24-hour duration
- `extended_window(hours)` — N-hour window starting now
- `emergency_window(minutes)` — N-minute window starting now

### Active Window Determination

A window is active if: `window.start_time <= now <= window.end_time`

A proof is valid for a window if: `window.contains(proof.timestamp)`

## Verification Rules

The `ProofVerifier` performs up to four checks:

### 1. Signature Verification (`verify_signature`)
- Decode the base64 public key (must be exactly 32 bytes)
- Decode the base64 signature (must be exactly 64 bytes for Ed25519)
- Recompute canonical bytes and verify against the public key
- **Fails if**: Signature doesn't match, key is invalid format, or signature is empty

### 2. Timestamp Verification (`verify_timestamp`)
- Compute `now - proof.timestamp` as absolute drift
- Compare against `max_time_drift` (default 300 seconds)
- **Fails if**: Drift exceeds limit or timestamp is in the future

### 3. Orbital Window Verification (`verify_orbital_window`)
- Check `window.end_time > window.start_time`
- Check `now >= window.start_time` (window has started)
- Check `now <= window.end_time` (window has not expired)
- **Fails if**: Window is malformed, not yet active, or expired

### 4. Chain Integrity Verification (`verify_chain_integrity`)
- Verify proof has a `chain_position`
- Confirm the proof at `chain[chain_position]` matches this proof's ID
- Verify all chain positions are sequential: `pos[i] + 1 == pos[i+1]`
- **Fails if**: Positions are non-sequential, missing, or mismatched

## Network Message Formats

### Peer Handshake

```
Message Type: 0x01 (Handshake)
Payload:
  - node_id: String (UTF-8)
  - public_key: [u8; 32]
  - protocol_version: String
  - timestamp: i64 (Unix timestamp)
  - signature: [u8; 64] (Ed25519 sig over (node_id + public_key + timestamp))
```

### Proof Relay

```
Message Type: 0x02 (ProofRelay)
Payload:
  - proof: ContactProof (CBOR encoded)
  - ttl: u8 (time-to-live, decremented each hop)
  - origin: NodeId
```

### Window Announcement

```
Message Type: 0x03 (WindowAnnounce)
Payload:
  - window: OrbitalWindow (CBOR encoded)
  - announcing_node: NodeId
  - signature: [u8; 64]
```

## Protocol Versioning

The protocol version is specified in `ProofMetadata.protocol_version` (format: `"MAJOR.MINOR"`).

- **Major version changes**: Breaking changes (e.g., canonical byte format changes)
- **Minor version changes**: Backward-compatible additions (e.g., new metadata fields)
- Nodes should reject proofs with incompatible major versions
