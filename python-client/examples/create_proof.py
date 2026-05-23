from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000", api_key="your-api-key")

proof = client.create_proof(
    target_node="node-b",
    window_id="win-001",
    purpose="conference-meetup",
)

print(f"Created proof: {proof.id}")
print(f"  Proving node: {proof.proving_node}")
print(f"  Target node:  {proof.target_node}")
print(f"  Window:       {proof.orbital_window.id}")
print(f"  Timestamp:    {proof.timestamp}")
print(f"  Signature:    {proof.signature[:16]}...")

client.close()
