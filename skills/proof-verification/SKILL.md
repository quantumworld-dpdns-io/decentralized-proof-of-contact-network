---
name: Proof Verification
description: Verify proof-of-contact proofs and analyze their chain integrity
trigger: When the user asks to verify a proof or check proof validity
---

# Proof Verification Skill

Use this skill to verify proof-of-contact proofs, check signatures, validate orbital windows, and analyze chain integrity.

## API Endpoints

### Verify a proof

```
POST /api/v1/proofs/{proof_id}/verify
```

Returns a `FullVerificationReport` with fields:
- `signature_valid` (bool) — Ed25519 signature matches the proving node's public key
- `timestamp_valid` (bool) — Proof timestamp within allowed drift (default 300s)
- `orbital_window_valid` (bool) — Current time falls within the proof's orbital window
- `chain_integrity_valid` (bool) — Chain positions are sequential and unbroken
- `all_passed` (bool) — All enabled checks passed

### Verify via Python SDK

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")
report = client.verify_proof("550e8400-e29b-41d4-a716-446655440000")
print(report)
```

### Verify via CLI

```bash
poi-cli proof verify 550e8400-e29b-41d4-a716-446655440000
```

## Using the Verification Script

```bash
python skills/proof-verification/scripts/verify.py \
    --api-url http://localhost:3000 \
    --proof-id 550e8400-e29b-41d4-a716-446655440000
```

## Interpreting Results

| Status | Meaning |
|--------|---------|
| `all_passed: true` | Proof is fully valid |
| `signature_valid: false` | Signature doesn't match; possible tampering |
| `timestamp_valid: false` | Timestamp drift exceeds max allowed |
| `orbital_window_valid: false` | Window has expired or not yet started |
| `chain_integrity_valid: false` | Chain is broken or positions are non-sequential |

## Troubleshooting

- **"Window expired"**: The orbital window end time has passed. Create a new proof in an active window.
- **"Invalid signature"**: The proving node's keypair may have changed or the proof was tampered with.
- **Chain integrity failure**: A proof in the chain may be missing. Check `chain_position` values are sequential from 0.
- **Timestamp drift**: Ensure the node's system clock is synchronized via NTP.
