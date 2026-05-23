'use client';

import { useEffect, useState } from 'react';
import { Play, Save, BarChart3, Table2 } from 'lucide-react';
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  PieChart, Pie, Cell, Legend
} from 'recharts';
import LoadingSpinner from '@/components/LoadingSpinner';
import { runAnalyticsQuery, fetchPresetQueries } from '@/lib/api';
import type { AnalyticsResult } from '@/lib/types';

const COLORS = ['#6366f1', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4'];

export default function AnalyticsPage() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<AnalyticsResult | null>(null);
  const [presets, setPresets] = useState<{ id: string; name: string; query: string }[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [viewMode, setViewMode] = useState<'table' | 'chart'>('table');

  useEffect(() => {
    fetchPresetQueries()
      .then(setPresets)
      .catch(() => setPresets([
        { id: '1', name: 'Proofs by Hour', query: 'SELECT COUNT(*) as count, strftime(\'%H\', timestamp) as hour FROM proofs GROUP BY hour ORDER BY hour' },
        { id: '2', name: 'Top Proving Nodes', query: 'SELECT proving_node, COUNT(*) as count FROM proofs GROUP BY proving_node ORDER BY count DESC LIMIT 10' },
        { id: '3', name: 'Verification Rate', query: 'SELECT COUNT(*) as total, SUM(CASE WHEN verified THEN 1 ELSE 0 END) as verified FROM proofs' },
        { id: '4', name: 'Active Windows', query: 'SELECT window_type, COUNT(*) as count, AVG(difficulty) as avg_difficulty FROM windows GROUP BY window_type' },
        { id: '5', name: 'Peer Reputation', query: 'SELECT id, reputation_score, connected_since FROM peers ORDER BY reputation_score DESC' },
        { id: '6', name: 'Proofs by Difficulty', query: 'SELECT difficulty, COUNT(*) as count FROM proofs GROUP BY difficulty ORDER BY difficulty' },
      ]));
  }, []);

  const handleRunQuery = async (q?: string) => {
    const sql = q ?? query;
    if (!sql.trim()) return;
    setLoading(true);
    setError('');
    setResults(null);
    try {
      const res = await runAnalyticsQuery(sql);
      setResults(res);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Query failed');
    } finally {
      setLoading(false);
    }
  };

  const isChartable = results && results.columns.length >= 2 && results.rows.length > 0;

  const chartData = results?.rows.map((row) => {
    const entry: Record<string, unknown> = {};
    results.columns.forEach((col) => { entry[col] = row[col]; });
    return entry;
  }) ?? [];

  const firstNumericCol = results?.columns.find(
    (col) => typeof results.rows[0]?.[col] === 'number'
  );
  const firstStringCol = results?.columns.find(
    (col) => typeof results.rows[0]?.[col] === 'string'
  );

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100">Analytics</h1>
        <p className="text-surface-500 dark:text-surface-400 mt-1">Query and visualize network data</p>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        {presets.map((preset) => (
          <button
            key={preset.id}
            onClick={() => { setQuery(preset.query); handleRunQuery(preset.query); }}
            className="card p-3 text-left hover:bg-surface-50 dark:hover:bg-surface-800 transition-colors"
          >
            <p className="text-sm font-medium text-surface-700 dark:text-surface-300 truncate">{preset.name}</p>
            <p className="text-xs text-surface-400 dark:text-surface-500 mt-1 truncate">{preset.query.slice(0, 40)}...</p>
          </button>
        ))}
      </div>

      <div className="card">
        <div className="card-header">
          <h3 className="text-lg font-semibold">SQL Query</h3>
        </div>
        <div className="card-body space-y-4">
          <textarea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="SELECT * FROM proofs LIMIT 100"
            className="input font-mono text-sm min-h-[100px] resize-y"
            spellCheck={false}
          />
          <div className="flex items-center justify-between">
            <div className="flex gap-2">
              <button
                onClick={() => handleRunQuery()}
                disabled={loading || !query.trim()}
                className="btn-primary"
              >
                {loading ? (
                  <LoadingSpinner size="sm" />
                ) : (
                  <Play className="h-4 w-4" />
                )}
                Run Query
              </button>
            </div>
            {results && (
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setViewMode('table')}
                  className={`btn-ghost p-2 ${viewMode === 'table' ? 'text-primary-600' : ''}`}
                >
                  <Table2 className="h-4 w-4" />
                </button>
                <button
                  onClick={() => setViewMode('chart')}
                  className={`btn-ghost p-2 ${viewMode === 'chart' ? 'text-primary-600' : ''}`}
                >
                  <BarChart3 className="h-4 w-4" />
                </button>
              </div>
            )}
          </div>
          {error && (
            <div className="p-3 rounded-lg bg-red-50 dark:bg-red-900/20 text-sm text-red-600 dark:text-red-400">
              {error}
            </div>
          )}
        </div>
      </div>

      {loading && <LoadingSpinner text="Running query..." />}

      {results && !loading && (
        <div className="card">
          <div className="card-header flex items-center justify-between">
            <div>
              <h3 className="text-lg font-semibold">Results</h3>
              <p className="text-sm text-surface-500 dark:text-surface-400">
                {results.row_count} rows returned in {results.query_time_ms}ms
              </p>
            </div>
          </div>
          <div className="card-body">
            {viewMode === 'table' ? (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-surface-200 dark:border-surface-700">
                      {results.columns.map((col) => (
                        <th key={col} className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-surface-500">
                          {col}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-surface-200 dark:divide-surface-700">
                    {chartData.map((row, i) => (
                      <tr key={i} className="hover:bg-surface-50 dark:hover:bg-surface-800/50">
                        {results.columns.map((col) => (
                          <td key={col} className="px-4 py-3 text-sm text-surface-700 dark:text-surface-300 font-mono">
                            {String(row[col] ?? 'NULL')}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : isChartable && firstNumericCol ? (
              results.columns.length === 2 ? (
                <div className="h-80">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={chartData}
                        dataKey={firstNumericCol}
                        nameKey={firstStringCol || results.columns[0]}
                        cx="50%"
                        cy="50%"
                        outerRadius={120}
                        label={({ name, value }) => `${name}: ${value}`}
                      >
                        {chartData.map((_, i) => (
                          <Cell key={i} fill={COLORS[i % COLORS.length]} />
                        ))}
                      </Pie>
                      <Legend />
                      <Tooltip />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              ) : (
                <div className="h-80">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={chartData}>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-surface-200 dark:stroke-surface-700" />
                      <XAxis
                        dataKey={firstStringCol || results.columns[0]}
                        className="text-xs"
                        tick={{ fill: 'currentColor' }}
                      />
                      <YAxis className="text-xs" tick={{ fill: 'currentColor' }} />
                      <Tooltip />
                      <Bar dataKey={firstNumericCol} fill="#6366f1" radius={[4, 4, 0, 0]} />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              )
            ) : (
              <p className="text-sm text-surface-500 dark:text-surface-400">Chart view requires numeric data</p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
