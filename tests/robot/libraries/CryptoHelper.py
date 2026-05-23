import base64
import hashlib
import json
from typing import Optional


class CryptoHelper:

    def __init__(self):
        self._keypairs = {}

    def generate_keypair(self, key_id="default"):
        import ed25519
        signing_key, verifying_key = ed25519.create_keypair()
        sk_bytes = signing_key.to_bytes()
        vk_bytes = verifying_key.to_bytes()
        keypair = {
            "public_key": base64.b64encode(vk_bytes).decode(),
            "secret_key": base64.b64encode(sk_bytes).decode(),
            "public_key_hex": vk_bytes.hex(),
            "secret_key_hex": sk_bytes.hex(),
            "key_id": key_id,
        }
        self._keypairs[key_id] = keypair
        return keypair

    def get_keypair(self, key_id="default"):
        if key_id not in self._keypairs:
            raise AssertionError(f"Keypair '{key_id}' not found. Generate it first.")
        return self._keypairs[key_id]

    def sign_data(self, data, secret_key_b64):
        import ed25519
        sk_bytes = base64.b64decode(secret_key_b64)
        signing_key = ed25519.SigningKey(sk_bytes)
        if isinstance(data, str):
            data = data.encode()
        signature = signing_key.sign(data)
        return base64.b64encode(signature).decode()

    def verify_signature(self, data, signature_b64, public_key_b64):
        import ed25519
        vk_bytes = base64.b64decode(public_key_b64)
        sig_bytes = base64.b64decode(signature_b64)
        verifying_key = ed25519.VerifyingKey(vk_bytes)
        if isinstance(data, str):
            data = data.encode()
        try:
            verifying_key.verify(sig_bytes, data)
            return True
        except ed25519.BadSignatureError:
            return False

    def hash_blake3(self, data):
        if isinstance(data, str):
            data = data.encode()
        return hashlib.blake2b(data, digest_size=32).hexdigest()

    def hash_sha256(self, data):
        if isinstance(data, str):
            data = data.encode()
        return hashlib.sha256(data).hexdigest()

    def compute_proof_hash(self, proof_json):
        proof = proof_json if isinstance(proof_json, dict) else json.loads(proof_json)
        canonical = (
            proof.get("id", "").encode()
            + proof.get("proving_node", "").encode()
            + proof.get("target_node", "").encode()
            + proof.get("window_id", "").encode()
        )
        return hashlib.blake2b(canonical, digest_size=32).hexdigest()

    def canonical_bytes(self, proof_data):
        parts = [
            proof_data.get("id", ""),
            proof_data.get("proving_node", ""),
            proof_data.get("target_node", ""),
            proof_data.get("window_id", ""),
        ]
        return "|".join(parts).encode()

    def generate_pqc_keypair(self, key_id="pqc-default"):
        keypair = {
            "public_key": base64.b64encode(b"pqc-public-key-placeholder").decode(),
            "secret_key": base64.b64encode(b"pqc-secret-key-placeholder").decode(),
            "algorithm": "dilithium5",
            "key_id": key_id,
        }
        self._keypairs[key_id] = keypair
        return keypair

    def encode_base64(self, data):
        if isinstance(data, str):
            data = data.encode()
        return base64.b64encode(data).decode()

    def decode_base64(self, data):
        return base64.b64decode(data)

    def verify_jwt_structure(self, token):
        parts = token.split(".")
        if len(parts) != 3:
            return False
        try:
            header = json.loads(base64.b64decode(parts[0] + "=="))
            payload = json.loads(base64.b64decode(parts[1] + "=="))
            return "alg" in header and "sub" in payload
        except (json.JSONDecodeError, base64.binascii.Error):
            return False
