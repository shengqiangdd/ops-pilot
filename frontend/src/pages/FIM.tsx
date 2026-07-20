import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { FimScanResult } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const STATUS_COLORS: Record<string, string> = {
  modified: 'bg-yellow-100 text-yellow-800',
  deleted: 'bg-red-100 text-red-800',
  added: 'bg-blue-100 text-blue-800',
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
      setBaselineStatus(`基线已创建: ${res.files_baselined} 个文件`);
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
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">文件完整性监控</h2>
        <div className="flex gap-2">
          <select value={hostId} onChange={(e) => setHostId(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm">
            <option value="">选择主机</option>
            {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
          </select>
          <button onClick={handleBaseline} disabled={loading || !hostId} className={cn('rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50')}>
            创建基线
          </button>
          <button onClick={handleScan} disabled={loading || !hostId} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {loading ? '扫描中...' : '执行扫描'}
          </button>
        </div>
      </div>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}
      {baselineStatus && <div className="rounded-md bg-green-50 p-3 text-sm text-green-700">{baselineStatus}</div>}

      {scanResult && (
        <div className="rounded-lg border border-gray-200 bg-white shadow-sm">
          <div className="border-b border-gray-200 px-4 py-3">
            <span className="text-sm font-medium text-gray-700">
              扫描结果: {scanResult.total_files} 个文件, {scanResult.changes.length} 个变更
            </span>
          </div>
          {scanResult.changes.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">状态</th>
                    <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">文件路径</th>
                    <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">Hash</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200">
                  {scanResult.changes.map((c, i) => (
                    <tr key={i} className="hover:bg-gray-50">
                      <td className="whitespace-nowrap px-4 py-3">
                        <span className={cn('inline-block rounded-full px-2.5 py-0.5 text-xs font-semibold', STATUS_COLORS[c.status] || 'bg-gray-100 text-gray-800')}>
                          {c.status}
                        </span>
                      </td>
                      <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-900">{c.path}</td>
                      <td className="whitespace-nowrap px-4 py-3 text-xs text-gray-500 font-mono">
                        {c.new_hash || c.hash || c.old_hash || '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="p-6 text-center text-sm text-gray-500">所有文件与基线一致</div>
          )}
        </div>
      )}

      {!scanResult && !loading && (
        <div className="rounded-lg border border-gray-200 bg-white p-8 text-center text-sm text-gray-500 shadow-sm">
          选择主机后点击"创建基线"建立基准，再点击"执行扫描"检测变更
        </div>
      )}
    </div>
  );
}
