'use client';

import { useState, useMemo } from 'react';
import type { Peer, Proof } from '@/lib/types';

interface TopologyGraphProps {
  peers: Peer[];
  proofs: Proof[];
  loading?: boolean;
}

export default function TopologyGraph({ peers, proofs, loading }: TopologyGraphProps) {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  const connections = useMemo(() => {
    const conns = new Map<string, Set<string>>();
    for (const proof of proofs) {
      if (!conns.has(proof.proving_node)) conns.set(proof.proving_node, new Set());
      if (!conns.has(proof.target_node)) conns.set(proof.target_node, new Set());
      conns.get(proof.proving_node)!.add(proof.target_node);
      conns.get(proof.target_node)!.add(proof.proving_node);
    }
    return conns;
  }, [proofs]);

  const nodePositions = useMemo(() => {
    const allNodes = [...new Set([...peers.map((p) => p.id), ...proofs.flatMap((p) => [p.proving_node, p.target_node])])];
    const radius = 140;
    return allNodes.map((id, i) => {
      const angle = (2 * Math.PI * i) / allNodes.length - Math.PI / 2;
      return { id, x: 180 + radius * Math.cos(angle), y: 180 + radius * Math.sin(angle) };
    });
  }, [peers, proofs]);

  if (loading) {
    return (
      <div className="card">
        <div className="card-header">
          <h3 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Network Topology</h3>
        </div>
        <div className="card-body">
          <div className="h-96 bg-surface-100 dark:bg-surface-700 rounded animate-pulse" />
        </div>
      </div>
    );
  }

  const nodeMap = new Map(nodePositions.map((n) => [n.id, n]));

  return (
    <div className="card">
      <div className="card-header">
        <h3 className="text-lg font-semibold text-surface-900 dark:text-surface-100">Network Topology</h3>
        <p className="text-sm text-surface-500 dark:text-surface-400">{peers.length} peers, {nodePositions.length} nodes</p>
      </div>
      <div className="card-body">
        <svg viewBox="0 0 360 360" className="w-full h-auto max-h-96">
          {nodePositions.map((a) =>
            [...(connections.get(a.id) || [])].map((targetId) => {
              const b = nodeMap.get(targetId);
              if (!b || a.id >= targetId) return null;
              const isActive = selectedNode && (selectedNode === a.id || selectedNode === targetId);
              return (
                <line
                  key={`${a.id}-${targetId}`}
                  x1={a.x}
                  y1={a.y}
                  x2={b.x}
                  y2={b.y}
                  stroke={isActive ? '#6366f1' : '#334155'}
                  strokeWidth={isActive ? 2 : 0.8}
                  className={isActive ? 'opacity-80' : 'opacity-30'}
                />
              );
            })
          )}
          {nodePositions.map((n) => {
            const peer = peers.find((p) => p.id === n.id);
            const isSelected = selectedNode === n.id;
            const radius = peer ? 8 : 5;
            return (
              <g key={n.id} onClick={() => setSelectedNode(isSelected ? null : n.id)} className="cursor-pointer">
                <circle
                  cx={n.x}
                  cy={n.y}
                  r={radius}
                  fill={isSelected ? '#6366f1' : peer ? '#10b981' : '#64748b'}
                  stroke={isSelected ? '#312e81' : 'none'}
                  strokeWidth={2}
                  className="transition-all duration-200"
                />
                <text
                  x={n.x}
                  y={n.y + radius + 12}
                  textAnchor="middle"
                  className="text-[6px] fill-surface-500 dark:fill-surface-400 font-mono"
                >
                  {n.id.slice(0, 8)}
                </text>
              </g>
            );
          })}
        </svg>
        {selectedNode && (
          <div className="mt-4 p-3 rounded-lg bg-surface-50 dark:bg-surface-800 border border-surface-200 dark:border-surface-700">
            <p className="text-sm font-medium text-surface-900 dark:text-surface-100 font-mono">{selectedNode}</p>
            <p className="text-xs text-surface-500 dark:text-surface-400 mt-1">
              {connections.get(selectedNode)?.size || 0} connections
              {peers.find((p) => p.id === selectedNode) ? ' • Peer node' : ''}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
