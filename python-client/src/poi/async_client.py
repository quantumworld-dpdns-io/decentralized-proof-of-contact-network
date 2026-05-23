from __future__ import annotations

from typing import Any

import httpx

from .errors import ApiError, AuthError, NotFoundError, RateLimitError, TimeoutError, ValidationError
from .models import NodeInfo, PeerInfo, Proof, ProofStats, OrbitalWindow


class AsyncPoiClient:
    def __init__(
        self,
        base_url: str,
        api_key: str | None = None,
        timeout: int = 30,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self._client = httpx.AsyncClient(timeout=httpx.Timeout(timeout))

    async def _request(self, method: str, path: str, **kwargs: Any) -> Any:
        url = f"{self.base_url}{path}"
        headers = kwargs.pop("headers", {})
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        headers.setdefault("Content-Type", "application/json")
        headers.setdefault("Accept", "application/json")

        try:
            response = await self._client.request(method, url, headers=headers, **kwargs)
        except httpx.TimeoutException as e:
            raise TimeoutError(f"Request timed out: {e}") from e
        except httpx.HTTPError as e:
            raise ApiError(0, f"HTTP error: {e}") from e

        if response.status_code == 401:
            raise AuthError()
        if response.status_code == 404:
            raise NotFoundError()
        if response.status_code == 429:
            raise RateLimitError()
        if 400 <= response.status_code < 500:
            body = self._try_decode(response)
            msg = body.get("error", body.get("message", response.text)) if isinstance(body, dict) else response.text
            raise ValidationError(msg) if response.status_code == 422 else ApiError(response.status_code, str(msg))
        if response.status_code >= 500:
            body = self._try_decode(response)
            msg = body.get("error", body.get("message", response.text)) if isinstance(body, dict) else response.text
            raise ApiError(response.status_code, str(msg))

        return response.json()

    @staticmethod
    def _try_decode(response: httpx.Response) -> Any:
        try:
            return response.json()
        except Exception:
            return response.text

    def _parse_proof(self, data: dict) -> Proof:
        if "orbital_window" in data:
            data["orbital_window"] = OrbitalWindow(**data["orbital_window"])
        return Proof(**data)

    # ── Proof operations ──────────────────────────────────────────

    async def create_proof(
        self,
        target_node: str,
        window_id: str,
        purpose: str = "",
    ) -> Proof:
        payload: dict[str, Any] = {"target_node": target_node, "window_id": window_id}
        if purpose:
            payload["purpose"] = purpose
        data = await self._request("POST", "/api/v1/proofs", json=payload)
        return self._parse_proof(data)

    async def get_proof(self, proof_id: str) -> Proof:
        data = await self._request("GET", f"/api/v1/proofs/{proof_id}")
        return self._parse_proof(data)

    async def list_proofs(self, limit: int = 50, offset: int = 0) -> list[Proof]:
        data = await self._request("GET", "/api/v1/proofs", params={"limit": limit, "offset": offset})
        if isinstance(data, list):
            return [self._parse_proof(item) for item in data]
        items = data if isinstance(data, dict) else {}
        return [self._parse_proof(item) for item in items.get("proofs", items.get("data", []))]

    async def verify_proof(self, proof_id: str) -> dict:
        return await self._request("POST", f"/api/v1/proofs/{proof_id}/verify")

    async def search_proofs(self, query: str, limit: int = 10) -> list[dict]:
        data = await self._request("GET", "/api/v1/proofs/search", params={"q": query, "limit": limit})
        if isinstance(data, list):
            return data
        return data.get("results", data.get("proofs", data.get("data", [])))

    async def delete_proof(self, proof_id: str) -> None:
        await self._request("DELETE", f"/api/v1/proofs/{proof_id}")

    # ── Node operations ───────────────────────────────────────────

    async def get_node_info(self) -> NodeInfo:
        data = await self._request("GET", "/api/v1/node")
        return NodeInfo(**data)

    async def list_peers(self) -> list[PeerInfo]:
        data = await self._request("GET", "/api/v1/node/peers")
        if isinstance(data, list):
            return [PeerInfo(**item) for item in data]
        raw = data if isinstance(data, dict) else {}
        return [PeerInfo(**item) for item in raw.get("peers", raw.get("data", []))]

    async def connect_peer(self, address: str) -> PeerInfo:
        data = await self._request("POST", "/api/v1/node/peers", json={"address": address})
        return PeerInfo(**data)

    async def disconnect_peer(self, peer_id: str) -> None:
        await self._request("DELETE", f"/api/v1/node/peers/{peer_id}")

    # ── Window operations ─────────────────────────────────────────

    async def list_windows(self) -> list[OrbitalWindow]:
        data = await self._request("GET", "/api/v1/windows")
        if isinstance(data, list):
            return [OrbitalWindow(**item) for item in data]
        raw = data if isinstance(data, dict) else {}
        return [OrbitalWindow(**item) for item in raw.get("windows", raw.get("data", []))]

    async def create_window(
        self,
        start_time: str,
        end_time: str,
        window_type: str = "standard",
    ) -> OrbitalWindow:
        payload: dict[str, Any] = {"start_time": start_time, "end_time": end_time, "window_type": window_type}
        data = await self._request("POST", "/api/v1/windows", json=payload)
        return OrbitalWindow(**data)

    async def get_active_windows(self) -> list[OrbitalWindow]:
        data = await self._request("GET", "/api/v1/windows/active")
        if isinstance(data, list):
            return [OrbitalWindow(**item) for item in data]
        raw = data if isinstance(data, dict) else {}
        return [OrbitalWindow(**item) for item in raw.get("windows", raw.get("data", []))]

    # ── Stats ─────────────────────────────────────────────────────

    async def get_stats(self) -> dict:
        return await self._request("GET", "/api/v1/stats")

    async def get_proof_stats(self) -> ProofStats:
        data = await self._request("GET", "/api/v1/stats/proofs")
        return ProofStats(**data)

    async def get_network_stats(self) -> dict:
        return await self._request("GET", "/api/v1/stats/network")

    # ── AI ────────────────────────────────────────────────────────

    async def ai_query(self, question: str) -> str:
        data = await self._request("POST", "/api/v1/ai/query", json={"question": question})
        if isinstance(data, dict):
            return data.get("answer", data.get("response", str(data)))
        return str(data)

    async def analyze_proof(self, proof_id: str) -> dict:
        return await self._request("GET", f"/api/v1/ai/analyze/{proof_id}")

    async def detect_anomalies(self) -> list[dict]:
        data = await self._request("GET", "/api/v1/ai/anomalies")
        if isinstance(data, list):
            return data
        return data.get("anomalies", data.get("results", []))

    # ── Health ────────────────────────────────────────────────────

    async def health_check(self) -> dict:
        return await self._request("GET", "/api/v1/health")

    async def close(self) -> None:
        await self._client.aclose()

    async def __aenter__(self) -> AsyncPoiClient:
        return self

    async def __aexit__(self, *args: Any) -> None:
        await self.close()
