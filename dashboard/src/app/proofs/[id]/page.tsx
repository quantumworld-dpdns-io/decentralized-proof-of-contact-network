'use client';

import { useEffect, useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import {
  ArrowLeft, Shield, ShieldCheck, Fingerprint, Clock, Hash, Brain, AlertTriangle,
  ExternalLink, Copy, Check, Loader2
} from 'lucide-react';
import { format } from 'date-fns';
import LoadingSpinner from '@/components/LoadingSpinner';
import { fetchProof, verifyProof, analyzeProof } from '@/lib/api';
import type { Proof } from '@/lib/types';

export default function ProofDetailPage() {
  const params = useParams();
  const router = useRouter();
  const [proof, setProof] = useState<Proof | null>(null);
  const [analysis, setAnalysis] = useState<{ analysis: string; score: number } | null>(null);
  const [loading, setLoading] = useState(true);
  const [verifying, setVerifying] = useState(false);
  const [verifyResult, setVerifyResult] = useState<{ valid: boolean; verified_by: string[] } | null>(null);
  const [analyzing, setAnalyzing] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const load = async () => {
      try {
        const p = await fetchProof(params.id as string);
        setProof(p);
      } catch (err) {
        console.error('Failed to load proof', err);
      } finally {
        setLoading(false);
      }
    };
    if (params.id) load();
  }, [params.id]);

  const handleVerify = async () => {
    if (!proof) return;
    setVerifying(true);
    try {
      const result = await verifyProof(proof.id);
      setVerifyResult(result);
    } catch (err) {
      console.error('Verification failed', err);
    } finally {
      setVerifying(false);
    }
  };

  const handleAnalyze = async () => {
    if (!proof) return;
    setAnalyzing(true);
    try {
      const result = await analyzeProof(proof.id);
      setAnalysis(result);
    } catch (err) {
      console.error('Analysis failed', err);
    } finally {
      setAnalyzing(false);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (loading) return <LoadingSpinner text="Loading proof..." />;

  if (!proof) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <AlertTriangle className="h-12 w-12 text-surface-400" />
        <h2 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Proof not found</h2>
        <p className="text-sm text-surface-500 dark:text-surface-400">The proof you are looking for does not exist.</p>
        <button onClick={() => router.push('/proofs')} className="btn-primary">
          <ArrowLeft className="h-4 w-4" /> Back to Proofs
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-4xl">
      <div className="flex items-center gap-4">
        <button onClick={() => router.push('/proofs')} className="btn-ghost p-1">
          <ArrowLeft className="h-5 w-5" />
        </button>
        <div>
          <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100 font-mono text-sm">
            Proof {proof.id.slice(0, 24)}...
          </h1>
          <p className="text-surface-500 dark:text-surface-400 mt-1">Detailed proof information</p>
        </div>
        <button
          onClick={() => copyToClipboard(proof.id)}
          className="btn-ghost p-1.5 ml-auto"
          title="Copy ID"
        >
          {copied ? <Check className="h-4 w-4 text-emerald-500" /> : <Copy className="h-4 w-4" />}
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          <div className="card">
            <div className="card-header">
              <h2 className="text-lg font-semibold flex items-center gap-2">
                <Fingerprint className="h-5 w-5 text-primary-500" />
                Proof Data
              </h2>
            </div>
            <div className="card-body space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Proving Node</label>
                  <p className="mt-1 text-sm font-mono text-surface-900 dark:text-surface-100 break-all">{proof.proving_node}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Target Node</label>
                  <p className="mt-1 text-sm font-mono text-surface-900 dark:text-surface-100 break-all">{proof.target_node}</p>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Timestamp</label>
                  <p className="mt-1 text-sm text-surface-700 dark:text-surface-300">
                    {format(new Date(proof.timestamp), 'PPpp')}
                  </p>
                </div>
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Status</label>
                  <p className="mt-1">
                    <span className={proof.verified ? 'badge-success' : 'badge-warning'}>
                      {proof.verified ? 'Verified' : 'Pending Verification'}
                    </span>
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div className="card">
            <div className="card-header">
              <h2 className="text-lg font-semibold flex items-center gap-2">
                <Hash className="h-5 w-5 text-primary-500" />
                Signature Details
              </h2>
            </div>
            <div className="card-body space-y-4">
              <div>
                <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Signature</label>
                <div className="mt-1 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 font-mono text-xs text-surface-700 dark:text-surface-300 break-all">
                  {proof.signature}
                </div>
              </div>
              <div className="grid grid-cols-3 gap-4">
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Difficulty</label>
                  <p className="mt-1 text-sm font-mono">{proof.metadata.difficulty}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Proof Type</label>
                  <p className="mt-1 text-sm">{proof.metadata.proof_type}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Distance</label>
                  <p className="mt-1 text-sm font-mono">{proof.metadata.distance ?? 'N/A'}</p>
                </div>
              </div>
              {proof.metadata.signal_strength != null && (
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Signal Strength</label>
                  <div className="mt-1 flex items-center gap-2">
                    <div className="flex-1 h-2 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
                      <div
                        className="h-full rounded-full bg-primary-500"
                        style={{ width: `${Math.min(100, proof.metadata.signal_strength * 100)}%` }}
                      />
                    </div>
                    <span className="text-sm font-mono text-surface-600 dark:text-surface-400">
                      {(proof.metadata.signal_strength * 100).toFixed(0)}%
                    </span>
                  </div>
                </div>
              )}
            </div>
          </div>

          <div className="card">
            <div className="card-header">
              <h2 className="text-lg font-semibold flex items-center gap-2">
                <Clock className="h-5 w-5 text-primary-500" />
                Orbital Window
              </h2>
            </div>
            <div className="card-body space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Window ID</label>
                  <p className="mt-1 text-sm font-mono">{proof.orbital_window?.id || 'N/A'}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Window Type</label>
                  <p className="mt-1 text-sm capitalize">{proof.orbital_window?.window_type || 'N/A'}</p>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Start Time</label>
                  <p className="mt-1 text-sm">
                    {proof.orbital_window?.start_time ? format(new Date(proof.orbital_window.start_time), 'PPpp') : 'N/A'}
                  </p>
                </div>
                <div>
                  <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">End Time</label>
                  <p className="mt-1 text-sm">
                    {proof.orbital_window?.end_time ? format(new Date(proof.orbital_window.end_time), 'PPpp') : 'N/A'}
                  </p>
                </div>
              </div>
            </div>
          </div>

          {analysis && (
            <div className="card">
              <div className="card-header">
                <h2 className="text-lg font-semibold flex items-center gap-2">
                  <Brain className="h-5 w-5 text-primary-500" />
                  AI Analysis
                </h2>
              </div>
              <div className="card-body">
                <div className="flex items-center gap-3 mb-4">
                  <span className="text-sm font-medium text-surface-700 dark:text-surface-300">Trust Score</span>
                  <div className="flex-1 h-2 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
                    <div
                      className={`h-full rounded-full ${analysis.score > 0.7 ? 'bg-emerald-500' : analysis.score > 0.4 ? 'bg-amber-500' : 'bg-red-500'}`}
                      style={{ width: `${analysis.score * 100}%` }}
                    />
                  </div>
                  <span className="text-sm font-mono font-medium">{(analysis.score * 100).toFixed(0)}%</span>
                </div>
                <p className="text-sm text-surface-600 dark:text-surface-400 leading-relaxed whitespace-pre-wrap">
                  {analysis.analysis}
                </p>
              </div>
            </div>
          )}
        </div>

        <div className="space-y-6">
          <div className="card">
            <div className="card-header">
              <h3 className="font-semibold flex items-center gap-2">
                <Shield className="h-5 w-5 text-primary-500" />
                Verification
              </h3>
            </div>
            <div className="card-body space-y-4">
              {verifyResult ? (
                <div className="space-y-3">
                  <div className={`p-3 rounded-lg text-sm flex items-center gap-2 ${verifyResult.valid ? 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700 dark:text-emerald-400' : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400'}`}>
                    {verifyResult.valid ? (
                      <><ShieldCheck className="h-5 w-5" /> Proof is valid — {verifyResult.verified_by.length} signatures confirmed</>
                    ) : (
                      <><Shield className="h-5 w-5" /> Verification failed</>
                    )}
                  </div>
                  {verifyResult.verified_by.length > 0 && (
                    <div>
                      <label className="text-xs font-medium text-surface-500 dark:text-surface-400 uppercase tracking-wider">Verified By</label>
                      <div className="mt-1 space-y-1">
                        {verifyResult.verified_by.map((v) => (
                          <p key={v} className="text-xs font-mono text-surface-600 dark:text-surface-400">{v}</p>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                <button
                  onClick={handleVerify}
                  disabled={verifying}
                  className="btn-primary w-full"
                >
                  {verifying && <Loader2 className="h-4 w-4 animate-spin" />}
                  {verifying ? 'Verifying...' : 'Verify Proof'}
                </button>
              )}
            </div>
          </div>

          <div className="card">
            <div className="card-header">
              <h3 className="font-semibold flex items-center gap-2">
                <Brain className="h-5 w-5 text-primary-500" />
                AI Analysis
              </h3>
            </div>
            <div className="card-body">
              {analysis ? (
                <div className="flex items-center gap-2 text-sm text-emerald-600 dark:text-emerald-400">
                  <Check className="h-4 w-4" />
                  Analysis complete
                </div>
              ) : (
                <button
                  onClick={handleAnalyze}
                  disabled={analyzing}
                  className="btn-secondary w-full"
                >
                  {analyzing && <Loader2 className="h-4 w-4 animate-spin" />}
                  {analyzing ? 'Analyzing...' : 'Run Analysis'}
                </button>
              )}
            </div>
          </div>

          <div className="card">
            <div className="card-header">
              <h3 className="font-semibold">Metadata</h3>
            </div>
            <div className="card-body space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-surface-500 dark:text-surface-400">Version</span>
                <span className="font-mono">{proof.metadata.node_version || 'N/A'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-surface-500 dark:text-surface-400">Verification Method</span>
                <span>{proof.metadata.verification_method || 'standard'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-surface-500 dark:text-surface-400">Difficulty</span>
                <span className="font-mono">{proof.metadata.difficulty}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
