#!/usr/bin/env python3
"""
Orbital window scheduling script for the decentralized proof-of-contact network.

Usage:
    python schedule.py --api-url http://localhost:3000 list
    python schedule.py --api-url http://localhost:3000 list-active
    python schedule.py --api-url http://localhost:3000 create --type extended --duration 4
    python schedule.py --api-url http://localhost:3000 create --start 2026-01-01T00:00:00Z --end 2026-01-01T06:00:00Z
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timedelta, timezone

import httpx


API_URL = os.environ.get("POI_API_URL", "http://localhost:3000")
API_KEY = os.environ.get("POI_API_KEY")


class WindowClient:
    def __init__(self, api_url: str, api_key: str | None = None):
        self.api_url = api_url.rstrip("/")
        self.headers = {"Accept": "application/json", "Content-Type": "application/json"}
        if api_key:
            self.headers["Authorization"] = f"Bearer {api_key}"

    def _get(self, path: str) -> dict | list:
        resp = httpx.get(f"{self.api_url}{path}", headers=self.headers, timeout=30)
        resp.raise_for_status()
        return resp.json()

    def _post(self, path: str, body: dict) -> dict:
        resp = httpx.post(f"{self.api_url}{path}", headers=self.headers, json=body, timeout=30)
        resp.raise_for_status()
        return resp.json()

    def list_windows(self) -> list[dict]:
        data = self._get("/api/v1/windows")
        if isinstance(data, list):
            return data
        return data.get("windows", data.get("data", []))

    def list_active(self) -> list[dict]:
        data = self._get("/api/v1/windows/active")
        if isinstance(data, list):
            return data
        return data.get("windows", data.get("data", []))

    def create(self, start_time: str, end_time: str, window_type: str = "standard") -> dict:
        body = {"start_time": start_time, "end_time": end_time, "window_type": window_type}
        return self._post("/api/v1/windows", body)


def cmd_list(client: WindowClient, args: argparse.Namespace) -> None:
    windows = client.list_windows()
    if not windows:
        print("No windows found.")
        return
    print(f"{'ID':<40} {'Type':<12} {'Start':<28} {'End':<28} {'Active':<8}")
    print("-" * 116)
    now = datetime.now(timezone.utc)
    for w in windows:
        start = w.get("start_time", "")
        end = w.get("end_time", "")
        try:
            active = "yes" if datetime.fromisoformat(start) <= now <= datetime.fromisoformat(end) else "no"
        except (ValueError, TypeError):
            active = "?"
        print(f"{w.get('id', ''):<40} {w.get('window_type', ''):<12} {start:<28} {end:<28} {active:<8}")


def cmd_list_active(client: WindowClient, args: argparse.Namespace) -> None:
    windows = client.list_active()
    if not windows:
        print("No active windows.")
        return
    for w in windows:
        print(f"  {w.get('id')}  {w.get('window_type')}  [{w.get('start_time')} - {w.get('end_time')}]")


def cmd_create(client: WindowClient, args: argparse.Namespace) -> None:
    now = datetime.now(timezone.utc)

    if args.start and args.end:
        start = args.start
        end = args.end
    elif args.duration:
        start = now.isoformat().replace("+00:00", "Z")
        end = (now + timedelta(hours=args.duration)).isoformat().replace("+00:00", "Z")
    else:
        start = now.isoformat().replace("+00:00", "Z")
        end = (now + timedelta(hours=1)).isoformat().replace("+00:00", "Z")

    window = client.create(start, end, args.type)
    print(f"Created window:")
    print(f"  ID:         {window.get('id')}")
    print(f"  Type:       {window.get('window_type')}")
    print(f"  Start:      {window.get('start_time')}")
    print(f"  End:        {window.get('end_time')}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Orbital window scheduler")
    parser.add_argument("--api-url", default=API_URL)
    parser.add_argument("--api-key")

    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("list", help="List all windows")
    sub.add_parser("list-active", help="List active windows")

    create_parser = sub.add_parser("create", help="Create a new window")
    create_parser.add_argument("--type", default="standard", choices=["standard", "extended", "emergency"])
    create_parser.add_argument("--duration", type=int, help="Duration in hours (for extended type)")
    create_parser.add_argument("--start", help="ISO 8601 start time")
    create_parser.add_argument("--end", help="ISO 8601 end time")

    args = parser.parse_args()
    api_key = args.api_key or API_KEY
    client = WindowClient(args.api_url, api_key)

    try:
        if args.command == "list":
            cmd_list(client, args)
        elif args.command == "list-active":
            cmd_list_active(client, args)
        elif args.command == "create":
            cmd_create(client, args)
    except httpx.HTTPError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
