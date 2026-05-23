'use client';

import Link from 'next/link';
import { useState, useMemo } from 'react';
import { Search, ChevronDown, ChevronUp, ArrowUpDown } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import type { Proof } from '@/lib/types';

interface ProofTableProps {
  proofs: Proof[];
  loading?: boolean;
}

type SortKey = 'timestamp' | 'proving_node' | 'target_node' | 'status';

export default function ProofTable({ proofs, loading }: ProofTableProps) {
  const [search, setSearch] = useState('');
  const [sortKey, setSortKey] = useState<SortKey>('timestamp');
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('desc');
  const [page, setPage] = useState(0);
  const pageSize = 10;

  const filtered = useMemo(() => {
    if (!search) return proofs;
    const q = search.toLowerCase();
    return proofs.filter(
      (p) =>
        p.id.toLowerCase().includes(q) ||
        p.proving_node.toLowerCase().includes(q) ||
        p.target_node.toLowerCase().includes(q)
    );
  }, [proofs, search]);

  const sorted = useMemo(() => {
    const arr = [...filtered];
    arr.sort((a, b) => {
      const aVal = a[sortKey] || '';
      const bVal = b[sortKey] || '';
      const cmp = typeof aVal === 'string' ? aVal.localeCompare(String(bVal)) : Number(aVal) - Number(bVal);
      return sortDir === 'asc' ? cmp : -cmp;
    });
    return arr;
  }, [filtered, sortKey, sortDir]);

  const totalPages = Math.ceil(sorted.length / pageSize);
  const paged = sorted.slice(page * pageSize, (page + 1) * pageSize);

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    else { setSortKey(key); setSortDir('desc'); }
  };

  const SortHeader = ({ label, sortKey: sk }: { label: string; sortKey: SortKey }) => (
    <button
      onClick={() => toggleSort(sk)}
      className="flex items-center gap-1 text-xs font-medium uppercase tracking-wider text-surface-500 dark:text-surface-400 hover:text-surface-700 dark:hover:text-surface-200"
    >
      {label}
      {sortKey === sk ? (
        sortDir === 'asc' ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />
      ) : (
        <ArrowUpDown className="h-3 w-3" />
      )}
    </button>
  );

  if (loading) {
    return (
      <div className="card overflow-hidden">
        <div className="p-6 space-y-4">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="h-12 bg-surface-100 dark:bg-surface-700 rounded animate-pulse" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="card overflow-hidden">
      <div className="card-header flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <h3 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Proofs</h3>
        <div className="relative w-full sm:w-64">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-surface-400" />
          <input
            type="text"
            placeholder="Search proofs..."
            value={search}
            onChange={(e) => { setSearch(e.target.value); setPage(0); }}
            className="input pl-9"
          />
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-surface-200 dark:border-surface-700">
              <th className="px-6 py-3 text-left">
                <SortHeader label="ID" sortKey="proving_node" />
              </th>
              <th className="px-6 py-3 text-left">
                <SortHeader label="Proving Node" sortKey="proving_node" />
              </th>
              <th className="px-6 py-3 text-left">
                <SortHeader label="Target Node" sortKey="target_node" />
              </th>
              <th className="px-6 py-3 text-left">
                <SortHeader label="Window" sortKey="status" />
              </th>
              <th className="px-6 py-3 text-left">
                <SortHeader label="Timestamp" sortKey="timestamp" />
              </th>
              <th className="px-6 py-3 text-left">
                <SortHeader label="Status" sortKey="status" />
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-surface-200 dark:divide-surface-700">
            {paged.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-6 py-12 text-center text-surface-500 dark:text-surface-400">
                  No proofs found
                </td>
              </tr>
            ) : (
              paged.map((proof) => (
                <tr
                  key={proof.id}
                  className="hover:bg-surface-50 dark:hover:bg-surface-800/50 transition-colors"
                >
                  <td className="px-6 py-4">
                    <Link
                      href={`/proofs/${proof.id}`}
                      className="text-sm font-mono text-primary-600 dark:text-primary-400 hover:underline"
                    >
                      {proof.id.slice(0, 16)}...
                    </Link>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm font-mono text-surface-700 dark:text-surface-300">
                      {proof.proving_node.slice(0, 20)}...
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm font-mono text-surface-700 dark:text-surface-300">
                      {proof.target_node.slice(0, 20)}...
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm text-surface-600 dark:text-surface-400">
                      {proof.orbital_window?.window_type || 'standard'}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm text-surface-500 dark:text-surface-400">
                      {formatDistanceToNow(new Date(proof.timestamp), { addSuffix: true })}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span className={proof.verified ? 'badge-success' : 'badge-warning'}>
                      {proof.verified ? 'Verified' : 'Pending'}
                    </span>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="card-header flex items-center justify-between">
          <p className="text-sm text-surface-500 dark:text-surface-400">
            Showing {page * pageSize + 1} to {Math.min((page + 1) * pageSize, sorted.length)} of {sorted.length}
          </p>
          <div className="flex gap-2">
            <button
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              disabled={page === 0}
              className="btn-secondary text-sm"
            >
              Previous
            </button>
            <button
              onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
              disabled={page >= totalPages - 1}
              className="btn-secondary text-sm"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
