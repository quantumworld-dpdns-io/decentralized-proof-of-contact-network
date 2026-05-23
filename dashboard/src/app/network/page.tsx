'use client';

import { useEffect, useState } from 'react';
import { RefreshCw, Signal, Wifi, Database, Globe } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import TopologyGraph from '@/components/TopologyGraph';
import { fetchPeers, fetchProofs } from '@/lib/api';
import type { Peer, Proof } from '@/lib/types';

export default function NetworkPage() {
  const [peers, setPeers] = useState<Peer[]>([]);
  const [proofs, setProofs] = useState<Proof[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedPeer, setSelectedPeer] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      const [p, pr] = await Promise.all([
        fetchPeers().catch(() => []),
        fetchProofs().catch(() => []),
      ]);
      setPeers(p);
      setProofs(pr);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, []);

  const avgReputation = peers.length > 0
    ? (peers.reduce((sum, p) => sum + p.reputation_score, 0) / peers.length).toFixed(2)
    : '—';

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100">Network</h1>
          <p className="text-surface-500 dark:text-surface-400 mt-1">Connected peers and network topology</p>
        </div>
        <button onClick={load} className="btn-secondary">
          <RefreshCw className="h-4 w-4" />
          Refresh
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="card p-6">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary-50 dark:bg-primary-900/20 text-primary-600">
              <Wifi className="h-5 w-5" />
            </div>
            <div>
              <p className="text-sm text-surface-500 dark:text-surface-400">Connected Peers</p>
              <p className="text-2xl font-bold text-surface-900 dark:text-surface-100">{peers.length}</p>
            </div>
          </div>
        </div>
        <div className="card p-6">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-emerald-50 dark:bg-emerald-900/20 text-emerald-600">
              <Signal className="h-5 w-5" />
            </div>
            <div>
              <p className="text-sm text-surface-500 dark:text-surface-400">Avg Reputation</p>
              <p className="text-2xl font-bold text-surface-900 dark:text-surface-100">{avgReputation}</p>
            </div>
          </div>
        </div>
        <div className="card p-6">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-amber-50 dark:bg-amber-900/20 text-amber-600">
              <Database className="h-5 w-5" />
            </div>
            <div>
              <p className="text-sm text-surface-500 dark:text-surface-400">Total Connections</p>
              <p className="text-2xl font-bold text-surface-900 dark:text-surface-100">{proofs.length}</p>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <TopologyGraph peers={peers} proofs={proofs} loading={loading} />

        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-semibold flex items-center gap-2">
              <Globe className="h-5 w-5 text-primary-500" />
              Connected Peers
            </h3>
          </div>
          <div className="overflow-y-auto max-h-96">
            <table className="w-full">
              <thead>
                <tr className="border-b border-surface-200 dark:border-surface-700">
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Peer</th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Address</th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Reputation</th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Connected</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-surface-200 dark:divide-surface-700">
                {peers.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-4 py-8 text-center text-surface-500 dark:text-surface-400 text-sm">
                      No peers connected
                    </td>
                  </tr>
                ) : (
                  peers.map((peer) => (
                    <tr
                      key={peer.id}
                      onClick={() => setSelectedPeer(selectedPeer === peer.id ? null : peer.id)}
                      className={`hover:bg-surface-50 dark:hover:bg-surface-800/50 cursor-pointer transition-colors ${
                        selectedPeer === peer.id ? 'bg-primary-50/50 dark:bg-primary-900/10' : ''
                      }`}
                    >
                      <td className="px-4 py-3">
                        <span className="text-sm font-mono text-surface-900 dark:text-surface-100">
                          {peer.id.slice(0, 16)}...
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <span className="text-sm font-mono text-surface-500 dark:text-surface-400">
                          {peer.address.length > 24 ? `${peer.address.slice(0, 24)}...` : peer.address}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <div
                            className="h-1.5 w-16 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden"
                          >
                            <div
                              className={`h-full rounded-full ${
                                peer.reputation_score > 0.7 ? 'bg-emerald-500' : peer.reputation_score > 0.4 ? 'bg-amber-500' : 'bg-red-500'
                              }`}
                              style={{ width: `${peer.reputation_score * 100}%` }}
                            />
                          </div>
                          <span className="text-xs font-mono text-surface-600 dark:text-surface-400">
                            {(peer.reputation_score * 100).toFixed(0)}
                          </span>
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <span className="text-sm text-surface-500 dark:text-surface-400">
                          {formatDistanceToNow(new Date(peer.connected_since), { addSuffix: true })}
                        </span>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {selectedPeer && (
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-semibold font-mono text-sm">Peer: {selectedPeer}</h3>
          </div>
          <div className="card-body">
            <p className="text-sm text-surface-500 dark:text-surface-400">Select a peer from the table to see details here.</p>
          </div>
        </div>
      )}
    </div>
  );
}
