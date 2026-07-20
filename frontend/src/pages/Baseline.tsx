import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { BaselineCheckResult } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const STATUS_ICON: Record<string, string> = { Pass: '✅', Fail: '❌', Warn: '⚠️', Skip: '⏭️', Info: 'ℹ️' };

const STATUS_COLOR: Record<string, string> = {
  Pass: 'bg-md-primary-container text-md-on-primary-container',
  Fail: 'bg-md-error-container text-md-on-error-container',
  Warn: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200',
  Skip: 'bg-md-surface-container-high text-md-on-surface-variant',
  Info: 'bg-md-secondary-container text-md-on-secondary-container',
};

export function BaselinePage() {
  const { token } = useAuthStore();
  const [hostId, setHostId] = useState('');
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [results, setResults] = useState<BaselineCheckResult[]>([]);
  const [score, setScore] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    api.listHosts(token).then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
  }, [token]);

  const runCheck = useCallback(async () => {
    if (!token || !hostId) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.runBaselineCheck(token, hostId);
      setResults(res.results);
      setScore(res.score);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, hostId]);

  const scoreColor = score !== null ? (score >= 80 ? 'text-green-600' : score >= 60 ? 'text-amber-600' : 'text-md-error') : 'text-md-outline';

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Baseline</h2>
        <div className="flex gap-2">
          <select value={hostId} onChange={(e) => setHostId(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-2.5 border border-md-outline text-body-medium text-md-on-surface">
            <option value="">Select host</option>
            {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
          </select>
          <button onClick={runCheck} disabled={loading || !hostId}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {loading ? 'Checking...' : 'Run Check'}
          </button>
        </div>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      {score !== null && (
        <div className="bg-md-surface-container-low rounded-md-lg p-4 shadow-md-1">
          <div className="flex items-center gap-4">
            <div className={cn('text-headline-medium font-medium', scoreColor)}>{score}</div>
            <div>
              <div className="text-body-medium font-medium text-md-on-surface">Compliance Score</div>
              <div className="text-body-medium text-md-on-surface-variant">{results.filter((r) => r.status === 'Pass').length}/{results.length} passed</div>
            </div>
          </div>
        </div>
      )}

      {results.length > 0 && (
        <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1 overflow-hidden">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Status</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Check</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Category</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Details</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Remediation</th>
              </tr>
            </thead>
            <tbody>
              {results.map((r, i) => (
                <tr key={i} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('inline-block rounded-md-full px-2.5 py-0.5 text-label-medium font-semibold', STATUS_COLOR[r.status])}>
                      {STATUS_ICON[r.status]} {r.status}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{r.name}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{r.category}</td>
                  <td className="px-4 py-3 text-body-medium text-md-on-surface-variant">{r.message}</td>
                  <td className="px-4 py-3 text-body-medium text-md-on-surface-variant">{r.remediation || '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {!results.length && !loading && (
        <div className="bg-md-surface-container-low rounded-md-lg p-8 text-center text-body-medium text-md-on-surface-variant shadow-md-1">
          Select a host and click "Run Check" to perform a security baseline audit
        </div>
      )}
    </div>
  );
}
