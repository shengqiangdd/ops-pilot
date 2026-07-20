import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { FimScanResult } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const STATUS_COLORS: Record<string, string> = {
  modified: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200',
  deleted: 'bg-md-error-container text-md-on-error-container',
  added: 'bg-md-primary-container text-md-on-primary-container',
};

export function FIMPage() {
  const { token } = useAuthStore();
  const [hostId, setHostId] = useState('');
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [scanResult, setScanResult] = useState<FimScanResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [baselineStatus, setBaselineStatus] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    api.listHosts().then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
  }, [token]);

  const handleBaseline = useCallback(async () => {
    if (!token || !hostId) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.createFimBaseline(token, hostId);
      setBaselineStatus(`Baseline created: ${res.files_baselined} files`);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, hostId]);

  const handleScan = useCallback(async () => {
    if (!token || !hostId) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.fimScan(token, hostId);
      setScanResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, hostId]);

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">File Integrity</h2>
        <div className="flex gap-2">
          <select value={hostId} onChange={(e) => setHostId(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-2.5 border border-md-outline text-body-medium text-md-on-surface">
            <option value="">Select host</option>
            {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
          </select>
          <button onClick={handleBaseline} disabled={loading || !hostId}
            className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high disabled:opacity-50">
            Create Baseline
          </button>
          <button onClick={handleScan} disabled={loading || !hostId}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {loading ? 'Scanning...' : 'Run Scan'}
          </button>
        </div>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}
      {baselineStatus && <div className="bg-md-primary-container text-md-on-primary-container rounded-md-sm px-4 py-3 text-body-medium">{baselineStatus}</div>}

      {scanResult && (
        <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1">
          <div className="border-b border-md-outline-variant px-4 py-3">
            <span className="text-body-medium font-medium text-md-on-surface">
              Scan results: {scanResult.total_files} files, {scanResult.changes.length} changes
            </span>
          </div>
          {scanResult.changes.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="min-w-full">
                <thead>
                  <tr className="border-b border-md-outline-variant">
                    <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Status</th>
                    <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">File Path</th>
                    <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Hash</th>
                  </tr>
                </thead>
                <tbody>
                  {scanResult.changes.map((c, i) => (
                    <tr key={i} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                      <td className="whitespace-nowrap px-4 py-3">
                        <span className={cn('inline-block rounded-md-full px-2.5 py-0.5 text-label-medium font-semibold', STATUS_COLORS[c.status] || 'bg-md-surface-container-high text-md-on-surface-variant')}>
                          {c.status}
                        </span>
                      </td>
                      <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface">{c.path}</td>
                      <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant font-mono">
                        {c.new_hash || c.hash || c.old_hash || '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="p-6 text-center text-body-medium text-md-on-surface-variant">All files match baseline</div>
          )}
        </div>
      )}

      {!scanResult && !loading && (
        <div className="bg-md-surface-container-low rounded-md-lg p-8 text-center text-body-medium text-md-on-surface-variant shadow-md-1">
          Select a host, create a baseline, then run a scan to detect changes
        </div>
      )}
    </div>
  );
}
