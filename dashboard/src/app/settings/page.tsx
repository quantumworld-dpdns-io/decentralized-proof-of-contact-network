'use client';

import { useState } from 'react';
import { useTheme } from 'next-themes';
import { Save, Sun, Moon, Monitor, Bell, Key, Wifi, Database, RefreshCw } from 'lucide-react';
import { healthCheck } from '@/lib/api';
import type { HealthStatus } from '@/lib/types';

export default function SettingsPage() {
  const { theme, setTheme } = useTheme();
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [checkingHealth, setCheckingHealth] = useState(false);

  const [apiUrl, setApiUrl] = useState(process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000');
  const [nodeId, setNodeId] = useState(process.env.NEXT_PUBLIC_NODE_ID || 'node-1');
  const [nodeAddress, setNodeAddress] = useState(process.env.NEXT_PUBLIC_NODE_ADDRESS || '/ip4/0.0.0.0/tcp/9000');
  const [notifications, setNotifications] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [refreshInterval, setRefreshInterval] = useState('30');

  const handleSave = async () => {
    setSaving(true);
    await new Promise((r) => setTimeout(r, 800));
    setSaving(false);
    setSaved(true);
    setTimeout(() => setSaved(false), 3000);
  };

  const handleHealthCheck = async () => {
    setCheckingHealth(true);
    try {
      const h = await healthCheck();
      setHealth(h);
    } catch {
      setHealth(null);
    } finally {
      setCheckingHealth(false);
    }
  };

  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100">Settings</h1>
        <p className="text-surface-500 dark:text-surface-400 mt-1">Configure your dashboard and node connection</p>
      </div>

      <div className="card">
        <div className="card-header">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Key className="h-5 w-5 text-primary-500" />
            API Configuration
          </h2>
        </div>
        <div className="card-body space-y-4">
          <div>
            <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
              API URL
            </label>
            <input
              type="text"
              value={apiUrl}
              onChange={(e) => setApiUrl(e.target.value)}
              className="input font-mono text-sm"
              placeholder="http://localhost:3000"
            />
            <p className="text-xs text-surface-400 dark:text-surface-500 mt-1">
              The base URL for the POI network API
            </p>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
                Node ID
              </label>
              <input
                type="text"
                value={nodeId}
                onChange={(e) => setNodeId(e.target.value)}
                className="input font-mono text-sm"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
                Node Address
              </label>
              <input
                type="text"
                value={nodeAddress}
                onChange={(e) => setNodeAddress(e.target.value)}
                className="input font-mono text-sm"
              />
            </div>
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            {theme === 'dark' ? <Moon className="h-5 w-5" /> : theme === 'light' ? <Sun className="h-5 w-5" /> : <Monitor className="h-5 w-5" />}
            Theme
          </h2>
        </div>
        <div className="card-body">
          <div className="flex gap-3">
            {[
              { value: 'light', label: 'Light', icon: Sun },
              { value: 'dark', label: 'Dark', icon: Moon },
              { value: 'system', label: 'System', icon: Monitor },
            ].map(({ value, label, icon: Icon }) => (
              <button
                key={value}
                onClick={() => setTheme(value)}
                className={`flex-1 flex items-center justify-center gap-2 p-4 rounded-lg border-2 transition-all ${
                  theme === value
                    ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-300'
                    : 'border-surface-200 dark:border-surface-700 text-surface-600 dark:text-surface-400 hover:border-surface-300 dark:hover:border-surface-600'
                }`}
              >
                <Icon className="h-5 w-5" />
                <span className="text-sm font-medium">{label}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Bell className="h-5 w-5 text-primary-500" />
            Notification Preferences
          </h2>
        </div>
        <div className="card-body space-y-4">
          <label className="flex items-center justify-between">
            <div>
              <span className="text-sm font-medium text-surface-700 dark:text-surface-300">Enable Notifications</span>
              <p className="text-xs text-surface-400 dark:text-surface-500">Receive alerts for important events</p>
            </div>
            <button
              onClick={() => setNotifications(!notifications)}
              className={`relative w-11 h-6 rounded-full transition-colors ${
                notifications ? 'bg-primary-600' : 'bg-surface-300 dark:bg-surface-600'
              }`}
            >
              <span
                className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform ${
                  notifications ? 'translate-x-5' : 'translate-x-0'
                }`}
              />
            </button>
          </label>
          <label className="flex items-center justify-between">
            <div>
              <span className="text-sm font-medium text-surface-700 dark:text-surface-300">Auto Refresh</span>
              <p className="text-xs text-surface-400 dark:text-surface-500">Automatically refresh dashboard data</p>
            </div>
            <button
              onClick={() => setAutoRefresh(!autoRefresh)}
              className={`relative w-11 h-6 rounded-full transition-colors ${
                autoRefresh ? 'bg-primary-600' : 'bg-surface-300 dark:bg-surface-600'
              }`}
            >
              <span
                className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform ${
                  autoRefresh ? 'translate-x-5' : 'translate-x-0'
                }`}
              />
            </button>
          </label>
          {autoRefresh && (
            <div>
              <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
                Refresh Interval (seconds)
              </label>
              <input
                type="number"
                value={refreshInterval}
                onChange={(e) => setRefreshInterval(e.target.value)}
                className="input w-32"
                min="5"
                max="300"
              />
            </div>
          )}
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Database className="h-5 w-5 text-primary-500" />
            Node Connection
          </h2>
        </div>
        <div className="card-body space-y-4">
          <button onClick={handleHealthCheck} disabled={checkingHealth} className="btn-secondary">
            <RefreshCw className={`h-4 w-4 ${checkingHealth ? 'animate-spin' : ''}`} />
            Check Health
          </button>
          {health && (
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-4">
              <div className="p-3 rounded-lg bg-surface-50 dark:bg-surface-800">
                <p className="text-xs text-surface-500 dark:text-surface-400">Status</p>
                <p className={`text-sm font-medium mt-0.5 ${
                  health.status === 'healthy' ? 'text-emerald-600' :
                  health.status === 'degraded' ? 'text-amber-600' : 'text-red-600'
                }`}>
                  {health.status}
                </p>
              </div>
              <div className="p-3 rounded-lg bg-surface-50 dark:bg-surface-800">
                <p className="text-xs text-surface-500 dark:text-surface-400">Version</p>
                <p className="text-sm font-mono font-medium mt-0.5">{health.version}</p>
              </div>
              <div className="p-3 rounded-lg bg-surface-50 dark:bg-surface-800">
                <p className="text-xs text-surface-500 dark:text-surface-400">Uptime</p>
                <p className="text-sm font-mono font-medium mt-0.5">{Math.floor(health.uptime / 3600)}h</p>
              </div>
              <div className="p-3 rounded-lg bg-surface-50 dark:bg-surface-800">
                <p className="text-xs text-surface-500 dark:text-surface-400">Peers</p>
                <p className="text-sm font-mono font-medium mt-0.5">{health.total_peers}</p>
              </div>
              <div className="p-3 rounded-lg bg-surface-50 dark:bg-surface-800">
                <p className="text-xs text-surface-500 dark:text-surface-400">DB Size</p>
                <p className="text-sm font-mono font-medium mt-0.5">{health.database_size}</p>
              </div>
              <div className="p-3 rounded-lg bg-surface-50 dark:bg-surface-800">
                <p className="text-xs text-surface-500 dark:text-surface-400">Memory</p>
                <p className="text-sm font-mono font-medium mt-0.5">{(health.memory_usage * 100).toFixed(0)}%</p>
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="flex justify-end">
        <button onClick={handleSave} disabled={saving} className="btn-primary">
          {saving ? (
            <RefreshCw className="h-4 w-4 animate-spin" />
          ) : saved ? (
            <><Save className="h-4 w-4" /> Saved</>
          ) : (
            <><Save className="h-4 w-4" /> Save Settings</>
          )}
        </button>
      </div>
    </div>
  );
}
