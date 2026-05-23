import type { Proof, Peer, OrbitalWindow, Stats, Anomaly, HealthStatus, VerifyResult, AnalyticsResult } from './types';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE}/api${path}`;
  const res = await fetch(url, {
    headers: { 'Content-Type': 'application/json', ...options?.headers },
    ...options,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => 'Unknown error');
    throw new ApiError(res.status, text);
  }
  return res.json();
}

export async function fetchProofs(): Promise<Proof[]> {
  return request<Proof[]>('/proofs');
}

export async function fetchProof(id: string): Promise<Proof> {
  return request<Proof>(`/proofs/${id}`);
}

export async function verifyProof(id: string): Promise<VerifyResult> {
  return request<VerifyResult>(`/proofs/${id}/verify`, { method: 'POST' });
}

export async function searchProofs(query: string): Promise<Proof[]> {
  return request<Proof[]>(`/proofs/search?q=${encodeURIComponent(query)}`);
}

export async function fetchPeers(): Promise<Peer[]> {
  return request<Peer[]>('/peers');
}

export async function fetchWindows(): Promise<OrbitalWindow[]> {
  return request<OrbitalWindow[]>('/windows');
}

export async function createWindow(start: string, end: string): Promise<OrbitalWindow> {
  return request<OrbitalWindow>('/windows', {
    method: 'POST',
    body: JSON.stringify({ start_time: start, end_time: end }),
  });
}

export async function fetchStats(): Promise<Stats> {
  return request<Stats>('/stats');
}

export async function aiQuery(query: string): Promise<string> {
  const res = await request<{ response: string }>('/ai/query', {
    method: 'POST',
    body: JSON.stringify({ query }),
  });
  return res.response;
}

export async function detectAnomalies(): Promise<Anomaly[]> {
  return request<Anomaly[]>('/ai/anomalies');
}

export async function healthCheck(): Promise<HealthStatus> {
  return request<HealthStatus>('/health');
}

export async function analyzeProof(id: string): Promise<{ analysis: string; score: number }> {
  return request<{ analysis: string; score: number }>(`/ai/analyze/${id}`);
}

export async function runAnalyticsQuery(sql: string): Promise<AnalyticsResult> {
  return request<AnalyticsResult>('/analytics/query', {
    method: 'POST',
    body: JSON.stringify({ query: sql }),
  });
}

export async function fetchPresetQueries(): Promise<{ id: string; name: string; query: string }[]> {
  return request<{ id: string; name: string; query: string }[]>('/analytics/presets');
}
