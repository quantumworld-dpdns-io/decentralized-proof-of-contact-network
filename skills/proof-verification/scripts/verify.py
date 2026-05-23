#!/usr/bin/env python3
"""
Proof verification script for the decentralized proof-of-contact network.

Usage:
    python verify.py --api-url http://localhost:3000 --proof-id <UUID>

Environment variables:
    POI_API_KEY     Optional API key for authenticated endpoints
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone

import httpx


API_URL = os.environ.get("POI_API_URL", "http://localhost:3000")
API_KEY = os.environ.get("POI_API_KEY")


class VerificationError(Exception):
    pass


def verify_proof(api_url: str, proof_id: str, api_key: str | None = None) -> dict:
    url = f"{api_url.rstrip('/')}/api/v1/proofs/{proof_id}/verify"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    response = httpx.post(url, headers=headers, timeout=30)
    if response.status_code == 404:
        raise VerificationError(f"Proof {proof_id} not found")
    if response.status_code == 401:
        raise VerificationError("Authentication failed")
    if response.status_code >= 400:
        raise VerificationError(f"API error {response.status_code}: {response.text}")
    return response.json()


def get_proof(api_url: str, proof_id: str, api_key: str | None = None) -> dict:
    url = f"{api_url.rstrip('/')}/api/v1/proofs/{proof_id}"
    headers = {"Accept": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    response = httpx.get(url, headers=headers, timeout=30)
    if response.status_code == 404:
        raise VerificationError(f"Proof {proof_id} not found")
    response.raise_for_status()
    return response.json()


def format_report(proof_id: str, proof: dict, report: dict) -> str:
    status = "PASSED" if report.get("all_passed") else "FAILED"
    lines = [
        "=" * 60,
        f"Proof Verification Report",
        "=" * 60,
        f"Proof ID:      {proof_id}",
        f"Status:        {status}",
        f"Generated:     {datetime.now(timezone.utc).isoformat()}",
        "",
        "Checks:",
        f"  Signature:         {'PASS' if report.get('signature_valid') else 'FAIL'}",
        f"  Timestamp:         {'PASS' if report.get('timestamp_valid') else 'FAIL'}",
        f"  Orbital Window:    {'PASS' if report.get('orbital_window_valid') else 'FAIL'}",
        f"  Chain Integrity:   {'PASS' if report.get('chain_integrity_valid') else 'FAIL'}",
        "",
    ]

    if proof:
        meta = proof.get("metadata", {})
        window = proof.get("orbital_window", {})
        lines += [
            "Proof Details:",
            f"  Proving Node:  {proof.get('proving_node', 'N/A')}",
            f"  Target Node:   {proof.get('target_node', 'N/A')}",
            f"  Window Start:  {window.get('start_time', 'N/A')}",
            f"  Window End:    {window.get('end_time', 'N/A')}",
            f"  Window Type:   {window.get('window_type', 'N/A')}",
            f"  Created At:    {proof.get('timestamp', 'N/A')}",
            f"  Chain Pos:     {meta.get('chain_position', 'N/A')}",
            f"  Confidence:    {meta.get('confidence_score', 'N/A')}",
            f"  Protocol Ver:  {meta.get('protocol_version', 'N/A')}",
            "",
        ]

    lines.append("=" * 60)
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description="Verify a proof-of-contact proof")
    parser.add_argument("--api-url", default=API_URL, help="POI API base URL")
    parser.add_argument("--proof-id", required=True, help="UUID of the proof to verify")
    parser.add_argument("--json", action="store_true", help="Output raw JSON")
    parser.add_argument("--api-key", help="API key for authenticated requests")
    args = parser.parse_args()

    api_key = args.api_key or API_KEY

    try:
        proof = get_proof(args.api_url, args.proof_id, api_key)
        report = verify_proof(args.api_url, args.proof_id, api_key)
    except VerificationError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except httpx.HTTPError as e:
        print(f"HTTP error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.json:
        output = {"proof_id": args.proof_id, "proof": proof, "report": report}
        print(json.dumps(output, indent=2, default=str))
    else:
        print(format_report(args.proof_id, proof, report))

    if not report.get("all_passed"):
        sys.exit(2)


if __name__ == "__main__":
    main()
