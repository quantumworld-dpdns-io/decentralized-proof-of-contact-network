'use client';

import { useState } from 'react';
import { X, Loader2 } from 'lucide-react';

interface CreateWindowModalProps {
  open: boolean;
  onClose: () => void;
  onSubmit: (data: { start_time: string; end_time: string }) => Promise<void>;
}

export default function CreateWindowModal({ open, onClose, onSubmit }: CreateWindowModalProps) {
  const [startTime, setStartTime] = useState('');
  const [endTime, setEndTime] = useState('');
  const [windowType, setWindowType] = useState('standard');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');

  if (!open) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!startTime || !endTime) {
      setError('Both start and end times are required');
      return;
    }
    if (new Date(startTime) >= new Date(endTime)) {
      setError('End time must be after start time');
      return;
    }
    setSubmitting(true);
    setError('');
    try {
      await onSubmit({ start_time: startTime, end_time: endTime });
      setStartTime('');
      setEndTime('');
      setWindowType('standard');
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create window');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-surface-900/50 backdrop-blur-sm" onClick={onClose} />
      <div className="relative w-full max-w-md card">
        <div className="card-header flex items-center justify-between">
          <h2 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Create Orbital Window</h2>
          <button onClick={onClose} className="btn-ghost p-1">
            <X className="h-5 w-5" />
          </button>
        </div>
        <form onSubmit={handleSubmit} className="card-body space-y-4">
          <div>
            <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
              Window Type
            </label>
            <select value={windowType} onChange={(e) => setWindowType(e.target.value)} className="input">
              <option value="standard">Standard</option>
              <option value="rapid">Rapid</option>
              <option value="extended">Extended</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
              Start Time
            </label>
            <input
              type="datetime-local"
              value={startTime}
              onChange={(e) => setStartTime(e.target.value)}
              className="input"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
              End Time
            </label>
            <input
              type="datetime-local"
              value={endTime}
              onChange={(e) => setEndTime(e.target.value)}
              className="input"
              required
            />
          </div>
          {error && (
            <div className="p-3 rounded-lg bg-red-50 dark:bg-red-900/20 text-sm text-red-600 dark:text-red-400">
              {error}
            </div>
          )}
          <div className="flex gap-3 justify-end">
            <button type="button" onClick={onClose} className="btn-secondary">Cancel</button>
            <button type="submit" disabled={submitting} className="btn-primary">
              {submitting && <Loader2 className="h-4 w-4 animate-spin" />}
              {submitting ? 'Creating...' : 'Create Window'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
