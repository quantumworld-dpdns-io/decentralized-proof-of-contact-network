import argparse
import sys
from typing import NoReturn

from .client import PoiClient


def main() -> NoReturn:
    parser = argparse.ArgumentParser(prog="poi", description="Proof-of-Contact CLI")
    parser.add_argument("--url", default="http://localhost:3000", help="API base URL")
    parser.add_argument("--api-key", default=None, help="API key")
    parser.add_argument("--timeout", type=int, default=30, help="Request timeout (seconds)")

    sub = parser.add_subparsers(dest="command", required=True)

    # proof
    p_create = sub.add_parser("create-proof", help="Create a new proof")
    p_create.add_argument("target_node")
    p_create.add_argument("window_id")
    p_create.add_argument("--purpose", default="")

    p_get = sub.add_parser("get-proof", help="Get a proof by ID")
    p_get.add_argument("proof_id")

    p_list = sub.add_parser("list-proofs", help="List proofs")
    p_list.add_argument("--limit", type=int, default=50)
    p_list.add_argument("--offset", type=int, default=0)

    p_verify = sub.add_parser("verify-proof", help="Verify a proof")
    p_verify.add_argument("proof_id")

    p_search = sub.add_parser("search-proofs", help="Search proofs")
    p_search.add_argument("query")
    p_search.add_argument("--limit", type=int, default=10)

    p_delete = sub.add_parser("delete-proof", help="Delete a proof")
    p_delete.add_argument("proof_id")

    # node
    sub.add_parser("node-info", help="Get node info")

    # windows
    sub.add_parser("list-windows", help="List orbital windows")
    sub.add_parser("active-windows", help="Get active windows")

    # stats
    sub.add_parser("stats", help="Get statistics")
    sub.add_parser("proof-stats", help="Get proof statistics")
    sub.add_parser("network-stats", help="Get network statistics")

    # health
    sub.add_parser("health", help="Health check")

    # AI
    p_ai = sub.add_parser("ai-query", help="Ask AI a question")
    p_ai.add_argument("question")
    p_ai.add_argument("--limit", type=int, default=10)

    p_anomalies = sub.add_parser("detect-anomalies", help="Detect anomalies")
    p_analyze = sub.add_parser("analyze-proof", help="Analyze a proof with AI")
    p_analyze.add_argument("proof_id")

    args = parser.parse_args()

    client = PoiClient(base_url=args.url, api_key=args.api_key, timeout=args.timeout)

    try:
        match args.command:
            case "create-proof":
                result = client.create_proof(args.target_node, args.window_id, args.purpose)
                print(result.model_dump_json(indent=2))
            case "get-proof":
                result = client.get_proof(args.proof_id)
                print(result.model_dump_json(indent=2))
            case "list-proofs":
                results = client.list_proofs(args.limit, args.offset)
                print(f"Found {len(results)} proof(s):")
                for p in results:
                    print(f"  {p.id}  {p.proving_node} -> {p.target_node}  [{p.timestamp}]")
            case "verify-proof":
                result = client.verify_proof(args.proof_id)
                import json
                print(json.dumps(result, indent=2))
            case "search-proofs":
                results = client.search_proofs(args.query, args.limit)
                import json
                print(json.dumps(results, indent=2))
            case "delete-proof":
                client.delete_proof(args.proof_id)
                print("Proof deleted.")
            case "node-info":
                result = client.get_node_info()
                print(result.model_dump_json(indent=2))
            case "list-windows":
                results = client.list_windows()
                for w in results:
                    print(f"  {w.id}  {w.window_type}  {w.start_time} -> {w.end_time}")
            case "active-windows":
                results = client.get_active_windows()
                for w in results:
                    print(f"  {w.id}  {w.window_type}  {w.start_time} -> {w.end_time}")
            case "stats":
                result = client.get_stats()
                import json
                print(json.dumps(result, indent=2))
            case "proof-stats":
                result = client.get_proof_stats()
                print(result.model_dump_json(indent=2))
            case "network-stats":
                result = client.get_network_stats()
                import json
                print(json.dumps(result, indent=2))
            case "health":
                result = client.health_check()
                import json
                print(json.dumps(result, indent=2))
            case "ai-query":
                result = client.ai_query(args.question)
                print(result)
            case "analyze-proof":
                result = client.analyze_proof(args.proof_id)
                import json
                print(json.dumps(result, indent=2))
            case "detect-anomalies":
                results = client.detect_anomalies()
                import json
                print(json.dumps(results, indent=2))
    finally:
        client.close()

    sys.exit(0)


if __name__ == "__main__":
    main()
