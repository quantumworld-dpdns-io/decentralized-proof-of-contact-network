#!/usr/bin/env python3
"""
Simple proof-of-contact example using the Python SDK.

Creates a proof, verifies it, and displays the results.

Usage:
    python examples/simple-proof.py
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone

from poi import PoiClient


def main() -> None:
    # Connect to local node
    client = PoiClient(base_url="http://localhost:3000")

    try:
        # 1. Health check
        health = client.health_check()
        print(f"Node: {health.get('node_id', 'unknown')}")
        print(f"Status: {health.get('status', 'unknown')}")
        print()

        # 2. Get node info
        node = client.get_node_info()
        print(f"Node ID: {node.id}")
        print(f"Version: {node.version}")
        print(f"Peers: {node.peer_count}")
        print()

        # 3. Create an orbital window if none active
        active = client.get_active_windows()
        if active:
            window = active[0]
            print(f"Using active window: {window.id} ({window.window_type})")
        else:
            now = datetime.now(timezone.utc)
            window = client.create_window(
                start_time=now.isoformat(),
                end_time=(now + timedelta(hours=2)).isoformat(),
                window_type="standard",
            )
            print(f"Created new window: {window.id}")

        print()

        # 4. Create a proof
        target_node = node.id  # Self-proof for demonstration
        purpose = "simple_example"
        print(f"Creating proof: proving={node.id} target={target_node}")

        proof = client.create_proof(
            target_node=target_node,
            window_id=window.id,
            purpose=purpose,
        )
        print(f"Proof ID:     {proof.id}")
        print(f"Proving Node: {proof.proving_node}")
        print(f"Target Node:  {proof.target_node}")
        print(f"Timestamp:    {proof.timestamp}")
        print(f"Signature:    {proof.signature[:40]}...")
        print()

        # 5. Verify the proof
        report = client.verify_proof(proof.id)
        print("Verification Report:")
        print(f"  Signature:       {'PASS' if report['signature_valid'] else 'FAIL'}")
        print(f"  Timestamp:       {'PASS' if report['timestamp_valid'] else 'FAIL'}")
        print(f"  Orbital Window:  {'PASS' if report['orbital_window_valid'] else 'FAIL'}")
        print(f"  Chain Integrity: {'PASS' if report.get('chain_integrity_valid', True) else 'FAIL'}")
        print(f"  ALL PASSED:      {'YES' if report['all_passed'] else 'NO'}")
        print()

        # 6. List proofs
        proofs = client.list_proofs(limit=5)
        print(f"Recent proofs: {len(proofs)}")
        for p in proofs:
            print(f"  {p.id} [{p.proving_node[:16]}... -> {p.target_node[:16]}...]")

        print()
        print("Success! Proof created and verified.")

    finally:
        client.close()


if __name__ == "__main__":
    main()
