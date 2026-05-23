from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000", api_key="your-api-key")

results = client.search_proofs(query="conference", limit=20)

print(f"Found {len(results)} proof(s):")
for r in results:
    if isinstance(r, dict):
        pid = r.get("id", r.get("proof_id", "?"))
        prov = r.get("proving_node", "?")
        tgt = r.get("target_node", "?")
        print(f"  {pid}: {prov} -> {tgt}")

client.close()
