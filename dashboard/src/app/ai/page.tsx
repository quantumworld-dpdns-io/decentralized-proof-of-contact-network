'use client';

import { useState, useRef, useEffect } from 'react';
import { Send, Brain, AlertTriangle, ShieldCheck, Loader2, Sparkles } from 'lucide-react';
import { aiQuery, detectAnomalies, analyzeProof } from '@/lib/api';
import type { Anomaly, Proof } from '@/lib/types';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  type?: 'analysis' | 'anomaly' | 'default';
  data?: unknown;
}

export default function AIPage() {
  const [messages, setMessages] = useState<Message[]>([
    {
      role: 'assistant',
      content: 'Hello! I can help you analyze the Proof of Contact network. Ask me anything about proofs, nodes, or network health. Try typing "analyze recent proofs" or "check for anomalies".',
      type: 'default',
    },
  ]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [anomalies, setAnomalies] = useState<Anomaly[]>([]);
  const [showAnomalies, setShowAnomalies] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || loading) return;
    const userMsg = input.trim();
    setInput('');
    setMessages((prev) => [...prev, { role: 'user', content: userMsg }]);
    setLoading(true);

    try {
      const response = await aiQuery(userMsg);
      setMessages((prev) => [
        ...prev,
        { role: 'assistant', content: response, type: 'default' },
      ]);
    } catch {
      setMessages((prev) => [
        ...prev,
        {
          role: 'assistant',
          content: 'Sorry, I encountered an error processing your request. Please try again.',
          type: 'default',
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  const handleDetectAnomalies = async () => {
    setLoading(true);
    setShowAnomalies(true);
    try {
      const result = await detectAnomalies();
      setAnomalies(result);
      setMessages((prev) => [
        ...prev,
        {
          role: 'assistant',
          content: `Found ${result.length} anomaly/ies in the network.`,
          type: 'anomaly',
          data: result,
        },
      ]);
    } catch {
      setMessages((prev) => [
        ...prev,
        { role: 'assistant', content: 'Failed to detect anomalies.', type: 'default' },
      ]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <div>
        <h1 className="text-2xl font-bold text-surface-900 dark:text-surface-100 flex items-center gap-2">
          <Brain className="h-6 w-6 text-primary-500" />
          AI Assistant
        </h1>
        <p className="text-surface-500 dark:text-surface-400 mt-1">
          Natural language interface for network analysis
        </p>
      </div>

      <div className="flex gap-3">
        <button onClick={handleDetectAnomalies} disabled={loading} className="btn-secondary">
          <AlertTriangle className="h-4 w-4" />
          Detect Anomalies
        </button>
        <button
          onClick={() => {
            setMessages((prev) => [
              ...prev,
              { role: 'user', content: 'Analyze the network health and recent proof activity' },
            ]);
            setLoading(true);
            aiQuery('Analyze the network health and recent proof activity')
              .then((res) => {
                setMessages((prev) => [
                  ...prev,
                  { role: 'assistant', content: res, type: 'analysis' },
                ]);
              })
              .catch(() => {})
              .finally(() => setLoading(false));
          }}
          disabled={loading}
          className="btn-secondary"
        >
          <ShieldCheck className="h-4 w-4" />
          Network Analysis
        </button>
        <button
          onClick={() => {
            setMessages((prev) => [
              ...prev,
              { role: 'user', content: 'Show me a summary of the network' },
            ]);
            setLoading(true);
            aiQuery('Show me a summary of the network')
              .then((res) => {
                setMessages((prev) => [
                  ...prev,
                  { role: 'assistant', content: res, type: 'default' },
                ]);
              })
              .catch(() => {})
              .finally(() => setLoading(false));
          }}
          disabled={loading}
          className="btn-secondary"
        >
          <Sparkles className="h-4 w-4" />
          Network Summary
        </button>
      </div>

      {showAnomalies && anomalies.length > 0 && (
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-semibold flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-red-500" />
              Detected Anomalies
            </h3>
          </div>
          <div className="card-body space-y-3">
            {anomalies.map((a) => (
              <div
                key={a.id}
                className="p-3 rounded-lg border border-surface-200 dark:border-surface-700 flex items-start justify-between"
              >
                <div>
                  <div className="flex items-center gap-2">
                    <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${
                      a.severity === 'critical' ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400' :
                      a.severity === 'high' ? 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400' :
                      a.severity === 'medium' ? 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400' :
                      'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
                    }`}>
                      {a.severity}
                    </span>
                    <span className="text-sm font-medium text-surface-700 dark:text-surface-300">{a.type}</span>
                  </div>
                  <p className="text-sm text-surface-500 dark:text-surface-400 mt-1">{a.description}</p>
                </div>
                <span className={`text-xs px-2 py-1 rounded ${a.resolved ? 'badge-success' : 'badge-warning'}`}>
                  {a.resolved ? 'Resolved' : 'Active'}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-body p-0">
          <div className="h-[500px] overflow-y-auto p-6 space-y-4 scrollbar-thin">
            {messages.map((msg, i) => (
              <div key={i} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                <div
                  className={`max-w-[80%] rounded-2xl px-4 py-3 ${
                    msg.role === 'user'
                      ? 'bg-primary-600 text-white'
                      : 'bg-surface-100 dark:bg-surface-800 text-surface-900 dark:text-surface-100'
                  }`}
                >
                  {msg.role === 'assistant' && (
                    <div className="flex items-center gap-2 mb-2">
                      <Brain className="h-4 w-4 text-primary-500" />
                      <span className="text-xs font-medium text-surface-500 dark:text-surface-400">AI Assistant</span>
                    </div>
                  )}
                  <p className="text-sm whitespace-pre-wrap leading-relaxed">{msg.content}</p>
                </div>
              </div>
            ))}
            {loading && (
              <div className="flex justify-start">
                <div className="bg-surface-100 dark:bg-surface-800 rounded-2xl px-4 py-3">
                  <div className="flex items-center gap-2">
                    <Loader2 className="h-4 w-4 animate-spin text-primary-500" />
                    <span className="text-sm text-surface-500">Thinking...</span>
                  </div>
                </div>
              </div>
            )}
            <div ref={bottomRef} />
          </div>
        </div>
        <div className="card-header">
          <div className="flex gap-3">
            <textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Ask about the network..."
              className="input min-h-[44px] max-h-[120px] resize-none"
              rows={1}
            />
            <button
              onClick={handleSend}
              disabled={loading || !input.trim()}
              className="btn-primary px-4"
            >
              <Send className="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
