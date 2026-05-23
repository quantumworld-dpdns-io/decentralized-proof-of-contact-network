# Tutorial 2: Creating Your First Proof-of-Contact

This tutorial demonstrates creating, signing, verifying, and querying your first proof-of-contact.

## Prerequisites

- A running POI node (see [Tutorial 1](01-setup.md))
- Python SDK installed (`pip install -e ".[dev]"`)
- At least one active orbital window

## Step 1: Check Active Windows

Proofs must be created within an active orbital window. List available windows:

```python
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")
windows = client.get_active_windows()
for w in windows:
    print(f"Window {w.id}: {w.window_type} [{w.start_time} - {w.end_time}]")
```

If no windows are active, create one:

```python
from datetime import datetime, timedelta, timezone

now = datetime.now(timezone.utc)
window = client.create_window(
    start_time=now.isoformat(),
    end_time=(now + timedelta(hours=2)).isoformat(),
    window_type="standard",
)
print(f"Created window: {window.id}")
```

## Step 2: Get Node Info

Identify your node and pick a target node (you can use a second node or the same node for testing):

```python
node = client.get_node_info()
print(f"My node ID: {node.id}")
```

## Step 3: Create a Proof

```python
target_node = "poi-550e8400-e29b-41d4-a716-446655440001"  # Replace with actual node ID

proof = client.create_proof(
    target_node=target_node,
    window_id=window.id,
    purpose="test_first_proof",
)
print(f"Created proof:")
print(f"  ID:           {proof.id}")
print(f"  Proving:      {proof.proving_node}")
print(f"  Target:       {proof.target_node}")
print(f"  Signature:    {proof.signature[:32]}...")
print(f"  Timestamp:    {proof.timestamp}")
```

## Step 4: Verify the Proof

```python
report = client.verify_proof(proof.id)
print(f"Verification report:")
print(f"  Signature:      {'PASS' if report['signature_valid'] else 'FAIL'}")
print(f"  Timestamp:      {'PASS' if report['timestamp_valid'] else 'FAIL'}")
print(f"  Orbital Window: {'PASS' if report['orbital_window_valid'] else 'FAIL'}")
print(f"  All Passed:     {'YES' if report['all_passed'] else 'NO'}")
```

## Step 5: Query Proofs

```python
# List recent proofs
proofs = client.list_proofs(limit=10)
print(f"Found {len(proofs)} recent proofs")

# Get specific proof
proof = client.get_proof(proof.id)
print(f"Proof details: {proof.model_dump_json(indent=2)}")
```

## Step 6: Using the CLI (Alternative)

```bash
# Create a proof
cargo run -p poi-cli -- proof create \
    --target "poi-550e8400-e29b-41d4-a716-446655440001" \
    --window "6ba7b810-9dad-11d1-80b4-00c04fd430c8"

# Verify a proof
cargo run -p poi-cli -- proof verify "550e8400-e29b-41d4-a716-446655440000"
```

## Step 7: Using the Verification Script

```bash
python skills/proof-verification/scripts/verify.py \
    --api-url http://localhost:3000 \
    --proof-id "550e8400-e29b-41d4-a716-446655440000"
```

## What's Happening Under the Hood

1. The node creates a `ContactProof` with the target node, orbital window, and metadata
2. The proof is signed with the node's Ed25519 private key
3. The canonical bytes are hashed with BLAKE3 to create a deterministic proof hash
4. The signed proof is stored in the local database (DuckDB)
5. Verification recomputes the canonical bytes and checks the Ed25519 signature
6. The orbital window is checked for validity (not expired)
7. The timestamp is checked against the allowed drift (default 5 minutes)

## Next Steps

- [Tutorial 3: Multi-Node Network](03-multi-node.md) — Set up multiple nodes and create proofs between them
- [Tutorial 4: AI Queries](04-ai-queries.md) — Query the network using natural language
