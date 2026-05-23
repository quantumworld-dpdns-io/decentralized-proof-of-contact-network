#!/usr/bin/env python3
"""
AI natural language query script for the proof-of-contact network.

Usage:
    python query.py --api-url http://localhost:3000 "How many proofs were created today?"
    python query.py --api-url http://localhost:3000 --json "List all active windows"
    python query.py --api-url http://localhost:3000 --analyze <proof_id>
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


def ai_query(api_url: str, question: str, api_key: str | None = None) -> str:
    url = f"{api_url.rstrip('/')}/api/v1/ai/query"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    resp = httpx.post(url, headers=headers, json={"question": question}, timeout=60)
    resp.raise_for_status()
    data = resp.json()
    if isinstance(data, dict):
        return data.get("answer", data.get("response", json.dumps(data)))
    return str(data)


def search_proofs(api_url: str, query: str, limit: int = 10, api_key: str | None = None) -> list[dict]:
    url = f"{api_url.rstrip('/')}/api/v1/proofs/search"
    headers = {"Accept": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    resp = httpx.get(url, headers=headers, params={"q": query, "limit": limit}, timeout=30)
    resp.raise_for_status()
    data = resp.json()
    if isinstance(data, list):
        return data
    return data.get("results", data.get("proofs", data.get("data", [])))


def format_response(question: str, answer: str) -> str:
    lines = [
        "=" * 60,
        "AI Query Result",
        "=" * 60,
        f"Question: {question}",
        f"Answered: {datetime.now(timezone.utc).isoformat()}",
        "",
        answer,
        "",
        "=" * 60,
    ]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description="Query the proof-of-contact network using natural language")
    parser.add_argument("--api-url", default=API_URL)
    parser.add_argument("--api-key")
    parser.add_argument("--json", action="store_true", help="Output raw JSON")
    parser.add_argument("--output", help="Write output to file")
    parser.add_argument("--analyze", metavar="PROOF_ID", help="Analyze a proof instead of querying")
    parser.add_argument("question", nargs="?", help="Natural language question")
    args = parser.parse_args()

    api_key = args.api_key or API_KEY

    try:
        if args.analyze:
            url = f"{args.api_url.rstrip('/')}/api/v1/ai/analyze/{args.analyze}"
            headers = {"Accept": "application/json"}
            if api_key:
                headers["Authorization"] = f"Bearer {api_key}"
            resp = httpx.get(url, headers=headers, timeout=30)
            resp.raise_for_status()
            result = json.dumps(resp.json(), indent=2, default=str)
            output = f"Analysis for proof {args.analyze}:\n{result}"
        elif args.question:
            answer = ai_query(args.api_url, args.question, api_key)
            if args.json:
                output = json.dumps({"question": args.question, "answer": answer}, indent=2)
            else:
                output = format_response(args.question, answer)
        else:
            parser.print_help()
            sys.exit(1)
    except httpx.HTTPError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
        print(f"Output written to {args.output}")
    else:
        print(output)


if __name__ == "__main__":
    main()
