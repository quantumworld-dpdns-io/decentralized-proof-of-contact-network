'use client';

import { useState } from 'react';
import { X, Loader2 } from 'lucide-react';

interface CreateProofModalProps {
  open: boolean;
  onClose: () => void;
  onSubmit: (data: { proving_node: string; target_node: string; window_id: string }) => Promise<void>;
  windows: { id: string; window_type: string }[];
}

export default function CreateProofModal({ open, onClose, onSubmit, windows }: CreateProofModalProps) {
  const [provingNode, setProvingNode] = useState('');
  const [targetNode, setTargetNode] = useState('');
  const [windowId, setWindowId] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');

  if (!open) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!provingNode || !targetNode) {
      setError('Both proving and target nodes are required');
      return;
    }
    setSubmitting(true);
    setError('');
    try {
      await onSubmit({ proving_node: provingNode, target_node: targetNode, window_id: windowId });
      setProvingNode('');
      setTargetNode('');
      setWindowId('');
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create proof');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-surface-900/50 backdrop-blur-sm" onClick={onClose} />
      <div className="relative w-full max-w-md card">
        <div className="card-header flex items-center justify-between">
          <h2 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Create Proof</h2>
          <button onClick={onClose} className="btn-ghost p-1">
            <X className="h-5 w-5" />
          </button>
        </div>
        <form onSubmit={handleSubmit} className="card-body space-y-4">
          <div>
            <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
              Proving Node
            </label>
            <input
              type="text"
              value={provingNode}
              onChange={(e) => setProvingNode(e.target.value)}
              placeholder="Enter proving node ID"
              className="input font-mono text-sm"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
              Target Node
            </label>
            <input
              type="text"
              value={targetNode}
              onChange={(e) => setTargetNode(e.target.value)}
              placeholder="Enter target node ID"
              className="input font-mono text-sm"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
              Orbital Window (optional)
            </label>
            <select
              value={windowId}
              onChange={(e) => setWindowId(e.target.value)}
              className="input"
            >
              <option value="">No window</option>
              {windows.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.window_type} — {w.id.slice(0, 12)}...
                </option>
              ))}
            </select>
          </div>
          {error && (
            <div className="p-3 rounded-lg bg-red-50 dark:bg-red-900/20 text-sm text-red-600 dark:text-red-400">
              {error}
            </div>
          )}
          <div className="flex gap-3 justify-end">
            <button type="button" onClick={onClose} className="btn-secondary">
              Cancel
            </button>
            <button type="submit" disabled={submitting} className="btn-primary">
              {submitting && <Loader2 className="h-4 w-4 animate-spin" />}
              {submitting ? 'Creating...' : 'Create Proof'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
