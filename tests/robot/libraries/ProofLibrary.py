import json
import time
import uuid
import base64
import requests
from datetime import datetime, timezone, timedelta
from typing import Optional


class ProofLibrary:

    def __init__(self, api_base_url="http://localhost:3000", auth_token="test-bearer-token-for-development"):
        self.api_base_url = api_base_url
        self.auth_token = auth_token
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {self.auth_token}",
            "Content-Type": "application/json",
        })

    def create_proof(self, target_node, window_id, purpose="test", confidence_score=1.0):
        payload = {
            "target_node": target_node,
            "window_id": window_id,
            "purpose": purpose,
            "confidence_score": confidence_score,
            "protocol_version": "1.0",
        }
        resp = self.session.post(f"{self.api_base_url}/api/v1/proofs", json=payload)
        if resp.status_code >= 400:
            raise AssertionError(f"Create proof failed: {resp.status_code} {resp.text}")
        return resp.json()

    def create_proof_raw(self, payload):
        resp = self.session.post(f"{self.api_base_url}/api/v1/proofs", json=payload)
        return resp

    def verify_proof(self, proof_id, public_key=None):
        params = {}
        if public_key:
            params["public_key"] = public_key
        resp = self.session.get(
            f"{self.api_base_url}/api/v1/proofs/{proof_id}/verify",
            params=params,
        )
        if resp.status_code >= 400:
            raise AssertionError(f"Verify proof failed: {resp.status_code} {resp.text}")
        return resp.json()

    def get_proof(self, proof_id):
        resp = self.session.get(f"{self.api_base_url}/api/v1/proofs/{proof_id}")
        if resp.status_code >= 400:
            raise AssertionError(f"Get proof failed: {resp.status_code} {resp.text}")
        return resp.json()

    def search_proofs(self, query=None, proving_node=None, target_node=None, window_id=None, status=None, limit=50, offset=0):
        params = {"limit": limit, "offset": offset}
        if query:
            params["q"] = query
        if proving_node:
            params["proving_node"] = proving_node
        if target_node:
            params["target_node"] = target_node
        if window_id:
            params["window_id"] = window_id
        if status:
            params["status"] = status
        resp = self.session.get(f"{self.api_base_url}/api/v1/proofs", params=params)
        if resp.status_code >= 400:
            raise AssertionError(f"Search proofs failed: {resp.status_code} {resp.text}")
        return resp.json()

    def delete_proof(self, proof_id):
        resp = self.session.delete(f"{self.api_base_url}/api/v1/proofs/{proof_id}")
        if resp.status_code >= 400:
            raise AssertionError(f"Delete proof failed: {resp.status_code} {resp.text}")
        return resp.status_code == 204

    def create_proof_chain(self, count=3):
        chain = []
        window_id = str(uuid.uuid4())
        for i in range(count):
            target = f"node-{uuid.uuid4().hex[:8]}"
            proof = self.create_proof(target_node=target, window_id=window_id, purpose="chain-test")
            chain.append(proof)
        return chain

    def get_proof_chain(self, proof_id):
        resp = self.session.get(f"{self.api_base_url}/api/v1/proofs/{proof_id}/chain")
        if resp.status_code >= 400:
            raise AssertionError(f"Get proof chain failed: {resp.status_code} {resp.text}")
        return resp.json()

    def verify_chain_integrity(self, proof_id):
        resp = self.session.get(f"{self.api_base_url}/api/v1/proofs/{proof_id}/chain/verify")
        if resp.status_code >= 400:
            raise AssertionError(f"Verify chain integrity failed: {resp.status_code} {resp.text}")
        return resp.json()

    def create_orbital_window(self, start_offset_hours=-1, end_offset_hours=1):
        now = datetime.now(timezone.utc)
        start = now + timedelta(hours=start_offset_hours)
        end = now + timedelta(hours=end_offset_hours)
        payload = {
            "start_time": start.isoformat(),
            "end_time": end.isoformat(),
            "window_type": "Standard",
        }
        resp = self.session.post(f"{self.api_base_url}/api/v1/windows", json=payload)
        if resp.status_code >= 400:
            raise AssertionError(f"Create orbital window failed: {resp.status_code} {resp.text}")
        return resp.json()

    def sign_proof(self, proof_id, secret_key_b64=None):
        payload = {}
        if secret_key_b64:
            payload["secret_key"] = secret_key_b64
        resp = self.session.post(
            f"{self.api_base_url}/api/v1/proofs/{proof_id}/sign",
            json=payload,
        )
        if resp.status_code >= 400:
            raise AssertionError(f"Sign proof failed: {resp.status_code} {resp.text}")
        return resp.json()

    def update_proof_metadata(self, proof_id, metadata):
        resp = self.session.patch(
            f"{self.api_base_url}/api/v1/proofs/{proof_id}",
            json=metadata,
        )
        if resp.status_code >= 400:
            raise AssertionError(f"Update proof metadata failed: {resp.status_code} {resp.text}")
        return resp.json()

    def list_proof_statuses(self):
        return ["pending", "signed", "stored", "gossiped", "verified", "archived", "failed"]

    def generate_keypair(self):
        import ed25519
        signing_key, verifying_key = ed25519.create_keypair()
        return {
            "public_key": base64.b64encode(verifying_key.to_bytes()).decode(),
            "secret_key": base64.b64encode(signing_key.to_bytes()).decode(),
        }
