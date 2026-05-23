#!/usr/bin/env python3
"""
AI integration example for the proof-of-contact network.

Demonstrates natural language queries, semantic search, proof analysis,
and anomaly detection.

Prerequisites:
    - Running POI node with AI provider configured (see docs/tutorials/01-setup.md)
    - Ollama running with gemma3:latest and nomic-embed-text:latest

Usage:
    python examples/ai-integration.py
"""

from __future__ import annotations

from poi import PoiClient


def main() -> None:
    client = PoiClient(base_url="http://localhost:3000")

    try:
        health = client.health_check()
        ai_status = health.get("ai_provider", "unknown")
        print(f"AI Provider: {ai_status}")
        print()

        # 1. Basic natural language query
        print("=== Natural Language Queries ===")
        questions = [
            "How many proofs are in the network?",
            "Summarize the current network health",
            "Are there any active orbital windows?",
        ]

        for question in questions:
            print(f"\nQ: {question}")
            try:
                answer = client.ai_query(question)
                print(f"A: {answer}")
            except Exception as e:
                print(f"  (Query failed: {e})")

        print()

        # 2. Semantic search
        print("=== Semantic Search ===")
        try:
            results = client.search_proofs(
                "proofs with high confidence scores",
                limit=5,
            )
            print(f"Found {len(results)} matching proofs:")
            for r in results:
                proof_id = r.get("id", "N/A")[:12]
                confidence = r.get("metadata", {}).get("confidence_score", "N/A")
                purpose = r.get("metadata", {}).get("proof_purpose", "N/A")
                print(f"  {proof_id}... confidence={confidence} purpose={purpose}")
        except Exception as e:
            print(f"  (Search failed: {e})")

        print()

        # 3. Analyze a specific proof
        print("=== Proof Analysis ===")
        try:
            proofs = client.list_proofs(limit=1)
            if proofs:
                proof = proofs[0]
                print(f"Analyzing proof {proof.id}...")
                analysis = client.analyze_proof(proof.id)
                if isinstance(analysis, dict):
                    for key, value in analysis.items():
                        print(f"  {key}: {value}")
                else:
                    print(f"  {analysis}")
            else:
                print("  No proofs to analyze. Create one first with simple-proof.py")
        except Exception as e:
            print(f"  (Analysis failed: {e})")

        print()

        # 4. Anomaly detection
        print("=== Anomaly Detection ===")
        try:
            anomalies = client.detect_anomalies()
            if anomalies:
                print(f"Found {len(anomalies)} anomalies:")
                for anomaly in anomalies:
                    sev = anomaly.get("severity", "unknown")
                    atype = anomaly.get("type", "unknown")
                    desc = anomaly.get("description", anomaly.get("detail", ""))
                    print(f"  [{sev}] {atype}: {desc}")
            else:
                print("  No anomalies detected. Network is healthy.")
        except Exception as e:
            print(f"  (Anomaly detection failed: {e})")

        print()

        # 5. Complex analytical query
        print("=== Complex Analysis ===")
        try:
            answer = client.ai_query(
                "Analyze the proof patterns and tell me if there are "
                "any unusual spikes in proof creation or verification failures"
            )
            print(f"Analysis: {answer}")
        except Exception as e:
            print(f"  (Analysis failed: {e})")

    finally:
        client.close()


if __name__ == "__main__":
    main()
