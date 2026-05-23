'use client';

import { useEffect, useState } from 'react';
import { Plus, RefreshCw, Clock, Orbit } from 'lucide-react';
import { format, formatDistanceToNow } from 'date-fns';
import CreateWindowModal from '@/components/CreateWindowModal';
import { fetchWindows } from '@/lib/api';
import type { OrbitalWindow } from '@/lib/types';

export default function WindowsPage() {
  const [windows, setWindows] = useState<OrbitalWindow[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const w = await fetchWindows().catch(() => []);
      setWindows(w);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, []);

  const handleCreateWindow = async (data: { start_time: string; end_time: string }) => {
    // Would call API to create window
    await load();
  };

  const now = new Date();
  const activeWindows = windows.filter(
    (w) => new Date(w.start_time) <= now && new Date(w.end_time) >= now
  );
  const upcomingWindows = windows.filter((w) => new Date(w.start_time) > now);

  const getTimeRemaining = (endTime: string) => {
    const end = new Date(endTime);
    if (end <= now) return 'Expired';
    return formatDistanceToNow(end, { addSuffix: true });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100">Orbital Windows</h1>
          <p className="text-surface-500 dark:text-surface-400 mt-1">Manage time-based proof windows</p>
        </div>
        <div className="flex gap-3">
          <button onClick={load} className="btn-secondary">
            <RefreshCw className="h-4 w-4" />
            Refresh
          </button>
          <button onClick={() => setShowCreate(true)} className="btn-primary">
            <Plus className="h-4 w-4" />
            Create Window
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="card p-6">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary-50 dark:bg-primary-900/20 text-primary-600">
              <Orbit className="h-5 w-5" />
            </div>
            <div>
              <p className="text-sm text-surface-500 dark:text-surface-400">Total Windows</p>
              <p className="text-2xl font-bold">{windows.length}</p>
            </div>
          </div>
        </div>
        <div className="card p-6">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-emerald-50 dark:bg-emerald-900/20 text-emerald-600">
              <Clock className="h-5 w-5" />
            </div>
            <div>
              <p className="text-sm text-surface-500 dark:text-surface-400">Active</p>
              <p className="text-2xl font-bold">{activeWindows.length}</p>
            </div>
          </div>
        </div>
        <div className="card p-6">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-blue-50 dark:bg-blue-900/20 text-blue-600">
              <Clock className="h-5 w-5" />
            </div>
            <div>
              <p className="text-sm text-surface-500 dark:text-surface-400">Upcoming</p>
              <p className="text-2xl font-bold">{upcomingWindows.length}</p>
            </div>
          </div>
        </div>
      </div>

      {activeWindows.length > 0 && (
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-semibold flex items-center gap-2">
              <Clock className="h-5 w-5 text-emerald-500" />
              Active Windows
            </h3>
          </div>
          <div className="card-body">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {activeWindows.map((w) => (
                <div key={w.id} className="p-4 rounded-lg border border-emerald-200 dark:border-emerald-900 bg-emerald-50/50 dark:bg-emerald-900/10">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-xs font-mono text-surface-500">{w.id.slice(0, 16)}...</span>
                    <span className="badge-success text-xs">{w.window_type}</span>
                  </div>
                  <p className="text-sm text-surface-700 dark:text-surface-300">
                    Started {formatDistanceToNow(new Date(w.start_time), { addSuffix: true })}
                  </p>
                  <p className="text-sm text-surface-500 dark:text-surface-400">
                    Expires {getTimeRemaining(w.end_time)}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-header">
          <h3 className="text-lg font-semibold">All Windows</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-surface-200 dark:border-surface-700">
                <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">ID</th>
                <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Type</th>
                <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Start Time</th>
                <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">End Time</th>
                <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Duration</th>
                <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-200 dark:divide-surface-700">
              {loading ? (
                Array.from({ length: 3 }).map((_, i) => (
                  <tr key={i}>
                    {Array.from({ length: 6 }).map((_, j) => (
                      <td key={j} className="px-6 py-4">
                        <div className="h-4 bg-surface-100 dark:bg-surface-700 rounded animate-pulse" />
                      </td>
                    ))}
                  </tr>
                ))
              ) : windows.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center text-surface-500 dark:text-surface-400">
                    No windows created yet
                  </td>
                </tr>
              ) : (
                windows.map((w) => {
                  const start = new Date(w.start_time);
                  const end = new Date(w.end_time);
                  const isActive = start <= now && end >= now;
                  const isUpcoming = start > now;
                  const status = isActive ? 'Active' : isUpcoming ? 'Upcoming' : 'Expired';
                  const statusBadge = isActive ? 'badge-success' : isUpcoming ? 'badge-info' : 'badge-warning';
                  return (
                    <tr key={w.id} className="hover:bg-surface-50 dark:hover:bg-surface-800/50">
                      <td className="px-6 py-4 text-sm font-mono text-surface-700 dark:text-surface-300">
                        {w.id.slice(0, 20)}...
                      </td>
                      <td className="px-6 py-4 text-sm capitalize">{w.window_type}</td>
                      <td className="px-6 py-4 text-sm text-surface-600 dark:text-surface-400">
                        {format(start, 'MMM d, HH:mm')}
                      </td>
                      <td className="px-6 py-4 text-sm text-surface-600 dark:text-surface-400">
                        {format(end, 'MMM d, HH:mm')}
                      </td>
                      <td className="px-6 py-4 text-sm text-surface-600 dark:text-surface-400">
                        {formatDistanceToNow(end, { addSuffix: true })}
                      </td>
                      <td className="px-6 py-4">
                        <span className={statusBadge}>{status}</span>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </div>

      <CreateWindowModal
        open={showCreate}
        onClose={() => setShowCreate(false)}
        onSubmit={handleCreateWindow}
      />
    </div>
  );
}
