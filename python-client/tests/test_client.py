from __future__ import annotations

from datetime import datetime, timezone
from typing import Any
from unittest.mock import ANY

import httpx
import pytest
import respx

from poi.client import PoiClient
from poi.errors import AuthError, NotFoundError, RateLimitError, ValidationError
from poi.models import NodeInfo, OrbitalWindow, PeerInfo, Proof, ProofStats


@pytest.fixture
def client() -> PoiClient:
    return PoiClient(base_url="http://localhost:3000", api_key="test-key")


@pytest.fixture
def mock_api(respx_mock: respx.MockRouter) -> respx.MockRouter:
    return respx_mock


SAMPLE_PROOF: dict[str, Any] = {
    "id": "proof-1",
    "proving_node": "node-a",
    "target_node": "node-b",
    "orbital_window": {
        "id": "win-1",
        "start_time": "2026-01-01T00:00:00+00:00",
        "end_time": "2026-01-02T00:00:00+00:00",
        "window_type": "standard",
    },
    "timestamp": "2026-05-23T00:00:00+00:00",
    "signature": "abc123",
    "metadata": {"purpose": "test"},
}

SAMPLE_NODE: dict[str, Any] = {
    "id": "node-1",
    "public_key": "pk123",
    "version": "0.1.0",
    "uptime_seconds": 3600,
    "peer_count": 5,
}

SAMPLE_PEER: dict[str, Any] = {
    "id": "peer-1",
    "address": "/ip4/1.2.3.4/tcp/9090",
    "connected_since": "2026-05-23T00:00:00+00:00",
    "reputation_score": 0.95,
}

SAMPLE_WINDOW: dict[str, Any] = {
    "id": "win-1",
    "start_time": "2026-01-01T00:00:00+00:00",
    "end_time": "2026-01-02T00:00:00+00:00",
    "window_type": "standard",
}


class TestCreateProof:
    def test_creates_proof(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        route = mock_api.post("/api/v1/proofs").respond(201, json=SAMPLE_PROOF)
        proof = client.create_proof("node-b", "win-1", purpose="meetup")
        assert isinstance(proof, Proof)
        assert proof.id == "proof-1"
        assert route.called
        import json
        assert json.loads(route.calls[0].request.content) == {"target_node": "node-b", "window_id": "win-1", "purpose": "meetup"}


class TestGetProof:
    def test_gets_proof(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/proofs/proof-1").respond(200, json=SAMPLE_PROOF)
        proof = client.get_proof("proof-1")
        assert isinstance(proof, Proof)
        assert proof.id == "proof-1"

    def test_raises_404(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/proofs/unknown").respond(404)
        with pytest.raises(NotFoundError):
            client.get_proof("unknown")


class TestListProofs:
    def test_lists_proofs(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/proofs").respond(200, json=[SAMPLE_PROOF])
        proofs = client.list_proofs()
        assert len(proofs) == 1
        assert isinstance(proofs[0], Proof)


class TestVerifyProof:
    def test_verify(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.post("/api/v1/proofs/proof-1/verify").respond(200, json={"valid": True, "status": "verified"})
        result = client.verify_proof("proof-1")
        assert result["valid"] is True


class TestSearchProofs:
    def test_search(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/proofs/search?q=test&limit=10").respond(200, json={"results": [SAMPLE_PROOF]})
        results = client.search_proofs("test")
        assert len(results) == 1


class TestDeleteProof:
    def test_delete(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        route = mock_api.delete("/api/v1/proofs/proof-1").respond(204)
        client.delete_proof("proof-1")
        assert route.called


class TestNodeInfo:
    def test_get_node_info(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/node").respond(200, json=SAMPLE_NODE)
        info = client.get_node_info()
        assert isinstance(info, NodeInfo)
        assert info.id == "node-1"


class TestListPeers:
    def test_list_peers(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/node/peers").respond(200, json=[SAMPLE_PEER])
        peers = client.list_peers()
        assert len(peers) == 1
        assert isinstance(peers[0], PeerInfo)


class TestConnectPeer:
    def test_connect(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        route = mock_api.post("/api/v1/node/peers").respond(201, json=SAMPLE_PEER)
        peer = client.connect_peer("/ip4/1.2.3.4/tcp/9090")
        assert isinstance(peer, PeerInfo)
        import json
        assert json.loads(route.calls[0].request.content) == {"address": "/ip4/1.2.3.4/tcp/9090"}


class TestDisconnectPeer:
    def test_disconnect(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        route = mock_api.delete("/api/v1/node/peers/peer-1").respond(204)
        client.disconnect_peer("peer-1")
        assert route.called


class TestWindows:
    def test_list_windows(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/windows").respond(200, json=[SAMPLE_WINDOW])
        windows = client.list_windows()
        assert len(windows) == 1
        assert isinstance(windows[0], OrbitalWindow)

    def test_create_window(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        route = mock_api.post("/api/v1/windows").respond(201, json=SAMPLE_WINDOW)
        win = client.create_window("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z", "extended")
        assert isinstance(win, OrbitalWindow)
        import json
        assert json.loads(route.calls[0].request.content) == {
            "start_time": "2026-01-01T00:00:00Z",
            "end_time": "2026-01-02T00:00:00Z",
            "window_type": "extended",
        }

    def test_get_active_windows(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/windows/active").respond(200, json=[SAMPLE_WINDOW])
        windows = client.get_active_windows()
        assert len(windows) == 1


class TestStats:
    def test_get_stats(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/stats").respond(200, json={"proofs": 100, "peers": 5})
        stats = client.get_stats()
        assert stats["proofs"] == 100

    def test_get_proof_stats(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        data = {"total_proofs": 100, "verified_proofs": 80, "pending_proofs": 15, "failed_proofs": 5, "proofs_per_hour": 12.5}
        mock_api.get("/api/v1/stats/proofs").respond(200, json=data)
        stats = client.get_proof_stats()
        assert isinstance(stats, ProofStats)
        assert stats.total_proofs == 100

    def test_get_network_stats(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/stats/network").respond(200, json={"peers": 10})
        stats = client.get_network_stats()
        assert stats["peers"] == 10


class TestAI:
    def test_ai_query(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        route = mock_api.post("/api/v1/ai/query").respond(200, json={"answer": "42"})
        answer = client.ai_query("life")
        assert answer == "42"
        import json
        assert json.loads(route.calls[0].request.content) == {"question": "life"}

    def test_analyze_proof(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/ai/analyze/proof-1").respond(200, json={"analysis": "ok"})
        result = client.analyze_proof("proof-1")
        assert result["analysis"] == "ok"

    def test_detect_anomalies(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/ai/anomalies").respond(200, json={"anomalies": [{"type": "suspicious"}]})
        results = client.detect_anomalies()
        assert len(results) == 1


class TestHealth:
    def test_health_check(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/health").respond(200, json={"status": "ok"})
        result = client.health_check()
        assert result["status"] == "ok"


class TestErrors:
    def test_auth_error(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/node").respond(401)
        with pytest.raises(AuthError):
            client.get_node_info()

    def test_rate_limit(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.get("/api/v1/node").respond(429)
        with pytest.raises(RateLimitError):
            client.get_node_info()

    def test_validation_error(self, client: PoiClient, mock_api: respx.MockRouter) -> None:
        mock_api.post("/api/v1/proofs").respond(422, json={"error": "Invalid window_id"})
        with pytest.raises(ValidationError):
            client.create_proof("node-b", "")


class TestContextManager:
    def test_context_manager(self) -> None:
        with PoiClient("http://localhost:3000") as client:
            assert isinstance(client, PoiClient)
