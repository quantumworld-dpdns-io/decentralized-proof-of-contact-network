#!/usr/bin/env python3
"""
Anomaly detection script for the decentralized proof-of-contact network.

Usage:
    python detect.py --api-url http://localhost:3000
    python detect.py --api-url http://localhost:3000 --severity high
    python detect.py --api-url http://localhost:3000 --analyze <proof_id>
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

SEVERITY_ORDER = {"critical": 0, "high": 1, "medium": 2, "low": 3}


def fetch_json(url: str, api_key: str | None = None) -> dict | list:
    headers = {"Accept": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    resp = httpx.get(url, headers=headers, timeout=30)
    resp.raise_for_status()
    return resp.json()


def post_json(url: str, api_key: str | None = None) -> dict | list:
    headers = {"Accept": "application/json", "Content-Type": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    resp = httpx.post(url, headers=headers, timeout=30)
    resp.raise_for_status()
    return resp.json()


def get_anomalies(api_url: str, api_key: str | None = None) -> list[dict]:
    data = fetch_json(f"{api_url}/api/v1/ai/anomalies", api_key)
    if isinstance(data, list):
        return data
    return data.get("anomalies", data.get("results", []))


def analyze_proof(api_url: str, proof_id: str, api_key: str | None = None) -> dict:
    data = post_json(f"{api_url}/api/v1/ai/analyze/{proof_id}", api_key)
    if isinstance(data, dict):
        return data
    return {"analysis": str(data)}


def format_report(anomalies: list[dict], severity_filter: str | None = None) -> str:
    filtered = anomalies
    if severity_filter:
        threshold = SEVERITY_ORDER.get(severity_filter, 99)
        filtered = [
            a for a in anomalies
            if SEVERITY_ORDER.get(a.get("severity", "low").lower(), 99) <= threshold
        ]

    if not filtered:
        return "No anomalies detected."

    lines = [
        "=" * 60,
        "Anomaly Detection Report",
        "=" * 60,
        f"Generated: {datetime.now(timezone.utc).isoformat()}",
        f"Total anomalies: {len(filtered)} (filtered from {len(anomalies)})",
        "",
    ]

    for i, anomaly in enumerate(filtered, 1):
        lines.append(f"--- Anomaly #{i} ---")
        lines.append(f"  Type:       {anomaly.get('type', 'unknown')}")
        lines.append(f"  Severity:   {anomaly.get('severity', 'unknown')}")
        lines.append(f"  Source:     {anomaly.get('source', anomaly.get('node_id', 'N/A'))}")
        lines.append(f"  Time:       {anomaly.get('timestamp', anomaly.get('detected_at', 'N/A'))}")
        lines.append(f"  Detail:     {anomaly.get('description', anomaly.get('detail', ''))}")
        lines.append("")

    lines.append("=" * 60)
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description="Detect anomalies in the proof-of-contact network")
    parser.add_argument("--api-url", default=API_URL)
    parser.add_argument("--api-key")
    parser.add_argument("--json", action="store_true", help="Output raw JSON")
    parser.add_argument("--severity", choices=["critical", "high", "medium", "low"], help="Minimum severity level")
    parser.add_argument("--analyze", metavar="PROOF_ID", help="Analyze a specific proof")
    args = parser.parse_args()

    api_key = args.api_key or API_KEY

    try:
        if args.analyze:
            result = analyze_proof(args.api_url, args.analyze, api_key)
            if args.json:
                print(json.dumps(result, indent=2, default=str))
            else:
                print(f"Analysis for proof {args.analyze}:")
                print(json.dumps(result, indent=2, default=str))
        else:
            anomalies = get_anomalies(args.api_url, api_key)
            if args.json:
                print(json.dumps(anomalies, indent=2, default=str))
            else:
                print(format_report(anomalies, args.severity))
    except httpx.HTTPError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
