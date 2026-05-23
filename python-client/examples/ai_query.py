from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000", api_key="your-api-key")

question = "What patterns do you see in the recent proofs?"
answer = client.ai_query(question)

print(f"Q: {question}")
print(f"A: {answer}")

client.close()
