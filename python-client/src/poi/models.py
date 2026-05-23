from datetime import datetime

from pydantic import BaseModel, Field


class OrbitalWindow(BaseModel):
    id: str
    start_time: datetime
    end_time: datetime
    window_type: str


class Proof(BaseModel):
    id: str
    proving_node: str
    target_node: str
    orbital_window: OrbitalWindow
    timestamp: datetime
    signature: str
    metadata: dict = Field(default_factory=dict)


class NodeInfo(BaseModel):
    id: str
    public_key: str
    version: str
    uptime_seconds: int
    peer_count: int


class PeerInfo(BaseModel):
    id: str
    address: str
    connected_since: datetime
    reputation_score: float


class ProofStats(BaseModel):
    total_proofs: int
    verified_proofs: int
    pending_proofs: int
    failed_proofs: int
    proofs_per_hour: float
