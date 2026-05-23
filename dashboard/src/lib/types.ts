export interface OrbitalWindow {
  id: string;
  start_time: string;
  end_time: string;
  window_type: string;
  status?: string;
  created_at?: string;
}

export interface ProofMetadata {
  proof_type: string;
  difficulty: number;
  distance?: number;
  signal_strength?: number;
  node_version?: string;
  verification_method?: string;
}

export interface Proof {
  id: string;
  proving_node: string;
  target_node: string;
  orbital_window: OrbitalWindow;
  timestamp: string;
  signature: string;
  metadata: ProofMetadata;
  status?: string;
  verified?: boolean;
}

export interface Peer {
  id: string;
  address: string;
  connected_since: string;
  reputation_score: number;
  latitude?: number;
  longitude?: number;
  distance?: number;
  last_seen?: string;
  status?: string;
}

export interface Stats {
  total_proofs: number;
  verified_proofs: number;
  active_peers: number;
  active_windows: number;
  total_nodes?: number;
  proof_rate?: number;
  verification_rate?: number;
  avg_difficulty?: number;
}

export interface Anomaly {
  id: string;
  type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  proof_id?: string;
  node_id?: string;
  detected_at: string;
  resolved: boolean;
}

export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  uptime: number;
  total_peers: number;
  active_windows: number;
  last_block: number;
  version: string;
  database_size: string;
  memory_usage: number;
  cpu_usage: number;
}

export interface VerifyResult {
  valid: boolean;
  verified_by: string[];
  timestamp: string;
  signatures_checked: number;
  errors?: string[];
}

export interface AnalyticsResult {
  columns: string[];
  rows: Record<string, unknown>[];
  query_time_ms: number;
  row_count: number;
}
