#!/usr/bin/env python3
"""
Network analysis script for the decentralized proof-of-contact network.

Usage:
    python analyze.py --api-url http://localhost:3000
    python analyze.py --api-url http://localhost:3000 --json
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


def fetch_json(url: str, api_key: str | None = None) -> dict | list:
    headers = {"Accept": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    response = httpx.get(url, headers=headers, timeout=30)
    response.raise_for_status()
    return response.json()


def analyze_network(api_url: str, api_key: str | None = None) -> dict:
    node_info = fetch_json(f"{api_url}/api/v1/node", api_key)
    peers = fetch_json(f"{api_url}/api/v1/node/peers", api_key)
    stats = fetch_json(f"{api_url}/api/v1/stats/network", api_key)
    health = fetch_json(f"{api_url}/api/v1/health", api_key)

    peer_list = peers if isinstance(peers, list) else peers.get("peers", [])

    total_reputation = sum(p.get("reputation_score", 0) for p in peer_list)
    avg_reputation = total_reputation / len(peer_list) if peer_list else 0.0
    connected_peers = sum(1 for p in peer_list if p.get("connected_since"))
    avg_uptime = (
        sum(p.get("uptime_seconds", 0) for p in peer_list) / len(peer_list)
        if peer_list
        else 0
    )

    return {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "node": {
            "id": node_info.get("id") if isinstance(node_info, dict) else "",
            "version": node_info.get("version") if isinstance(node_info, dict) else "",
            "uptime_seconds": node_info.get("uptime_seconds") if isinstance(node_info, dict) else 0,
            "status": "healthy" if isinstance(health, dict) and health.get("status") == "ok" else "unhealthy",
        },
        "peers": {
            "total": len(peer_list),
            "connected": connected_peers,
            "avg_reputation": round(avg_reputation, 2),
            "avg_uptime_seconds": round(avg_uptime, 0),
        },
        "stats": stats if isinstance(stats, dict) else {},
    }


def format_report(result: dict) -> str:
    node = result["node"]
    peers = result["peers"]
    stats = result.get("stats", {})

    lines = [
        "=" * 60,
        "Network Analysis Report",
        "=" * 60,
        f"Timestamp:     {result['timestamp']}",
        "",
        "Node:",
        f"  ID:           {node['id']}",
        f"  Version:      {node['version']}",
        f"  Uptime:       {node['uptime_seconds']}s",
        f"  Status:       {node['status']}",
        "",
        "Peers:",
        f"  Total:        {peers['total']}",
        f"  Connected:    {peers['connected']}",
        f"  Avg Score:    {peers['avg_reputation']}",
        f"  Avg Uptime:   {peers['avg_uptime_seconds']}s",
        "",
        "Network Stats:",
    ]

    if stats:
        for key, value in stats.items():
            lines.append(f"  {key}: {value}")
    else:
        lines.append("  (no stats available)")

    lines.append("")
    lines.append("=" * 60)
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description="Analyze proof-of-contact network")
    parser.add_argument("--api-url", default=API_URL)
    parser.add_argument("--json", action="store_true", help="Output raw JSON")
    parser.add_argument("--api-key")
    args = parser.parse_args()

    api_key = args.api_key or API_KEY

    try:
        result = analyze_network(args.api_url, api_key)
    except httpx.HTTPError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.json:
        print(json.dumps(result, indent=2, default=str))
    else:
        print(format_report(result))


if __name__ == "__main__":
    main()
