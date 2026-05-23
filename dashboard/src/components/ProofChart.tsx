'use client';

import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Area, AreaChart } from 'recharts';

interface DataPoint {
  date: string;
  count: number;
  verified: number;
}

interface ProofChartProps {
  data: DataPoint[];
  loading?: boolean;
}

export default function ProofChart({ data, loading }: ProofChartProps) {
  if (loading) {
    return (
      <div className="card">
        <div className="card-header">
          <h3 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Proof Creation Rate</h3>
        </div>
        <div className="card-body">
          <div className="h-72 bg-surface-100 dark:bg-surface-700 rounded animate-pulse" />
        </div>
      </div>
    );
  }

  return (
    <div className="card">
      <div className="card-header">
        <h3 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Proof Creation Rate</h3>
        <p className="text-sm text-surface-500 dark:text-surface-400">Last 7 days</p>
      </div>
      <div className="card-body">
        <div className="h-72">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data}>
              <defs>
                <linearGradient id="countGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#6366f1" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#6366f1" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="verifiedGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" className="stroke-surface-200 dark:stroke-surface-700" />
              <XAxis
                dataKey="date"
                className="text-xs text-surface-500"
                tick={{ fill: 'currentColor' }}
                tickLine={false}
              />
              <YAxis className="text-xs text-surface-500" tick={{ fill: 'currentColor' }} tickLine={false} axisLine={false} />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'var(--tooltip-bg, #fff)',
                  border: '1px solid var(--tooltip-border, #e2e8f0)',
                  borderRadius: '8px',
                  fontSize: '13px',
                }}
                labelClassName="font-medium"
              />
              <Area type="monotone" dataKey="count" stroke="#6366f1" fill="url(#countGrad)" strokeWidth={2} name="Total" />
              <Area type="monotone" dataKey="verified" stroke="#10b981" fill="url(#verifiedGrad)" strokeWidth={2} name="Verified" />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}
