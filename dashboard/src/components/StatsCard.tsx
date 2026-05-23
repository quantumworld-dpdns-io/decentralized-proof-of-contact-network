'use client';

import { LucideIcon } from 'lucide-react';

interface StatsCardProps {
  title: string;
  value: string | number;
  change?: string;
  changeType?: 'positive' | 'negative' | 'neutral';
  icon: LucideIcon;
  description?: string;
}

export default function StatsCard({ title, value, change, changeType = 'neutral', icon: Icon, description }: StatsCardProps) {
  const changeColors = {
    positive: 'text-emerald-600 dark:text-emerald-400',
    negative: 'text-red-600 dark:text-red-400',
    neutral: 'text-surface-500 dark:text-surface-400',
  };

  return (
    <div className="card p-6">
      <div className="flex items-start justify-between">
        <div className="space-y-1">
          <p className="text-sm text-surface-500 dark:text-surface-400">{title}</p>
          <p className="text-3xl font-bold text-surface-900 dark:text-surface-100">{value}</p>
          {change && (
            <p className={`text-sm font-medium ${changeColors[changeType]}`}>
              {change}
            </p>
          )}
          {description && (
            <p className="text-xs text-surface-400 dark:text-surface-500">{description}</p>
          )}
        </div>
        <div className="p-3 rounded-lg bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400">
          <Icon className="h-6 w-6" />
        </div>
      </div>
    </div>
  );
}
