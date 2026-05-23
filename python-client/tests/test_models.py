from datetime import datetime, timezone

from poi.models import NodeInfo, OrbitalWindow, PeerInfo, Proof, ProofStats


class TestOrbitalWindow:
    def test_minimal(self) -> None:
        ow = OrbitalWindow(
            id="win-1",
            start_time=datetime(2026, 1, 1, tzinfo=timezone.utc),
            end_time=datetime(2026, 1, 2, tzinfo=timezone.utc),
            window_type="standard",
        )
        assert ow.id == "win-1"
        assert ow.window_type == "standard"

    def test_serialization(self) -> None:
        ow = OrbitalWindow(
            id="w-42",
            start_time=datetime(2026, 5, 1, 12, 0, 0, tzinfo=timezone.utc),
            end_time=datetime(2026, 5, 1, 13, 0, 0, tzinfo=timezone.utc),
            window_type="extended",
        )
        d = ow.model_dump()
        assert d["id"] == "w-42"
        assert d["window_type"] == "extended"


class TestProof:
    def test_minimal(self) -> None:
        ow = OrbitalWindow(
            id="w-1",
            start_time=datetime(2026, 1, 1, tzinfo=timezone.utc),
            end_time=datetime(2026, 1, 2, tzinfo=timezone.utc),
            window_type="standard",
        )
        proof = Proof(
            id="proof-1",
            proving_node="node-a",
            target_node="node-b",
            orbital_window=ow,
            timestamp=datetime(2026, 5, 23, tzinfo=timezone.utc),
            signature="sig123",
            metadata={"purpose": "test"},
        )
        assert proof.id == "proof-1"
        assert proof.metadata["purpose"] == "test"
        assert proof.orbital_window.window_type == "standard"

    def test_default_metadata(self) -> None:
        ow = OrbitalWindow(
            id="w-1",
            start_time=datetime(2026, 1, 1, tzinfo=timezone.utc),
            end_time=datetime(2026, 1, 2, tzinfo=timezone.utc),
            window_type="standard",
        )
        proof = Proof(
            id="p-1",
            proving_node="a",
            target_node="b",
            orbital_window=ow,
            timestamp=datetime(2026, 5, 23, tzinfo=timezone.utc),
            signature="s",
        )
        assert proof.metadata == {}


class TestNodeInfo:
    def test_fields(self) -> None:
        info = NodeInfo(id="node-1", public_key="pk123", version="0.1.0", uptime_seconds=3600, peer_count=5)
        assert info.id == "node-1"
        assert info.peer_count == 5


class TestPeerInfo:
    def test_fields(self) -> None:
        peer = PeerInfo(
            id="peer-1",
            address="/ip4/1.2.3.4/tcp/9090",
            connected_since=datetime(2026, 5, 23, tzinfo=timezone.utc),
            reputation_score=0.95,
        )
        assert peer.reputation_score == 0.95
        assert peer.id == "peer-1"


class TestProofStats:
    def test_fields(self) -> None:
        stats = ProofStats(total_proofs=100, verified_proofs=80, pending_proofs=15, failed_proofs=5, proofs_per_hour=12.5)
        assert stats.total_proofs == 100
        assert stats.verified_proofs == 80
        assert stats.proofs_per_hour == 12.5
