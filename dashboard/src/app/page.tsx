'use client';

import { useEffect, useState } from 'react';
import { Fingerprint, Users, Orbit, Shield } from 'lucide-react';
import StatsCard from '@/components/StatsCard';
import ProofChart from '@/components/ProofChart';
import ProofTable from '@/components/ProofTable';
import TopologyGraph from '@/components/TopologyGraph';
import { fetchStats, fetchProofs, fetchPeers, fetchWindows } from '@/lib/api';
import type { Stats, Proof, Peer, OrbitalWindow } from '@/lib/types';

export default function DashboardPage() {
  const [stats, setStats] = useState<Stats | null>(null);
  const [proofs, setProofs] = useState<Proof[]>([]);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [windows, setWindows] = useState<OrbitalWindow[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const [s, p, peersData, w] = await Promise.all([
          fetchStats().catch(() => null),
          fetchProofs().catch(() => []),
          fetchPeers().catch(() => []),
          fetchWindows().catch(() => []),
        ]);
        setStats(s);
        setProofs(p);
        setPeers(peersData);
        setWindows(w);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const chartData = [
    { date: 'Mon', count: 12, verified: 10 },
    { date: 'Tue', count: 18, verified: 15 },
    { date: 'Wed', count: 14, verified: 12 },
    { date: 'Thu', count: 22, verified: 20 },
    { date: 'Fri', count: 16, verified: 14 },
    { date: 'Sat', count: 9, verified: 8 },
    { date: 'Sun', count: 15, verified: 13 },
  ];

  const recentProofs = proofs.slice(0, 5);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100">Dashboard</h1>
        <p className="text-surface-500 dark:text-surface-400 mt-1">Overview of the Proof of Contact network</p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatsCard
          title="Total Proofs"
          value={stats?.total_proofs ?? '—'}
          change={stats ? `Verified: ${stats.verified_proofs}` : undefined}
          changeType="positive"
          icon={Fingerprint}
          description="All time proofs generated"
        />
        <StatsCard
          title="Active Peers"
          value={stats?.active_peers ?? peers.length}
          change={peers.length > 0 ? `${peers.length} connected` : undefined}
          changeType="positive"
          icon={Users}
          description="Currently online nodes"
        />
        <StatsCard
          title="Active Windows"
          value={stats?.active_windows ?? windows.length}
          change={windows.length > 0 ? `${windows.length} orbital windows` : undefined}
          changeType={windows.length > 0 ? 'positive' : 'neutral'}
          icon={Orbit}
          description="Open orbital windows"
        />
        <StatsCard
          title="Verification Rate"
          value={stats?.verification_rate != null ? `${(stats.verification_rate * 100).toFixed(0)}%` : '—'}
          change={stats?.avg_difficulty ? `Avg difficulty: ${stats.avg_difficulty}` : undefined}
          changeType="positive"
          icon={Shield}
          description="Proof verification success"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <ProofChart data={chartData} loading={loading} />
        </div>
        <div>
          <TopologyGraph peers={peers} proofs={proofs} loading={loading} />
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <ProofTable proofs={recentProofs} loading={loading} />
        </div>
        <div className="space-y-4">
          <div className="card">
            <div className="card-header">
              <h3 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Network Health</h3>
            </div>
            <div className="card-body space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm text-surface-600 dark:text-surface-400">Total Nodes</span>
                <span className="text-sm font-mono font-medium">{stats?.total_nodes || peers.length || '—'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-surface-600 dark:text-surface-400">Proof Rate</span>
                <span className="text-sm font-mono font-medium">{stats?.proof_rate ? `${stats.proof_rate}/hr` : '—'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-surface-600 dark:text-surface-400">Avg Difficulty</span>
                <span className="text-sm font-mono font-medium">{stats?.avg_difficulty ?? '—'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-surface-600 dark:text-surface-400">Windows Open</span>
                <span className="text-sm font-mono font-medium">{windows.length}</span>
              </div>
              <div className="pt-2">
                <div className="h-2 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
                  <div
                    className="h-full rounded-full bg-primary-500 transition-all duration-500"
                    style={{ width: stats ? `${Math.min(100, (stats.verified_proofs / Math.max(1, stats.total_proofs)) * 100)}%` : '0%' }}
                  />
                </div>
                <p className="text-xs text-surface-400 dark:text-surface-500 mt-1">
                  {stats ? `${((stats.verified_proofs / Math.max(1, stats.total_proofs)) * 100).toFixed(0)}% verification rate` : 'Loading...'}
                </p>
              </div>
            </div>
          </div>

          <div className="card">
            <div className="card-header">
              <h3 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Active Windows</h3>
            </div>
            <div className="card-body space-y-2">
              {windows.length === 0 ? (
                <p className="text-sm text-surface-400 dark:text-surface-500">No active windows</p>
              ) : (
                windows.slice(0, 4).map((w) => (
                  <div key={w.id} className="flex items-center justify-between p-2 rounded-lg bg-surface-50 dark:bg-surface-800">
                    <div>
                      <p className="text-xs font-mono text-surface-700 dark:text-surface-300">{w.id.slice(0, 16)}...</p>
                      <p className="text-xs text-surface-400 dark:text-surface-500">{w.window_type}</p>
                    </div>
                    <span className="badge-success text-xs">{w.status || 'active'}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
