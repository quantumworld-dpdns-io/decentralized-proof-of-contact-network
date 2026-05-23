'use client';

import { useEffect, useState } from 'react';
import { Plus, RefreshCw } from 'lucide-react';
import ProofTable from '@/components/ProofTable';
import CreateProofModal from '@/components/CreateProofModal';
import { fetchProofs, fetchWindows, createWindow as apiCreateProof } from '@/lib/api';
import type { Proof, OrbitalWindow } from '@/lib/types';

export default function ProofsPage() {
  const [proofs, setProofs] = useState<Proof[]>([]);
  const [windows, setWindows] = useState<OrbitalWindow[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const [p, w] = await Promise.all([
        fetchProofs().catch(() => []),
        fetchWindows().catch(() => []),
      ]);
      setProofs(p);
      setWindows(w);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, []);

  const handleCreateProof = async (data: { proving_node: string; target_node: string; window_id: string }) => {
    // This would call the API to create a proof
    // For now we just refresh
    await load();
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100">Proofs</h1>
          <p className="text-surface-500 dark:text-surface-400 mt-1">
            Browse and manage proof-of-contact proofs
          </p>
        </div>
        <div className="flex gap-3">
          <button onClick={load} className="btn-secondary">
            <RefreshCw className="h-4 w-4" />
            Refresh
          </button>
          <button onClick={() => setShowCreate(true)} className="btn-primary">
            <Plus className="h-4 w-4" />
            Create Proof
          </button>
        </div>
      </div>

      <ProofTable proofs={proofs} loading={loading} />

      <CreateProofModal
        open={showCreate}
        onClose={() => setShowCreate(false)}
        onSubmit={handleCreateProof}
        windows={windows}
      />
    </div>
  );
}
