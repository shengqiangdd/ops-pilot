import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { FileSyncResult } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

export function FileSyncPage() {
  const { token } = useAuthStore();
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [hostId, setHostId] = useState('');
  const [filePath, setFilePath] = useState('');
  const [content, setContent] = useState('');
  const [result, setResult] = useState<FileSyncResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    api.listHosts().then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
  }, [token]);

  const handlePush = useCallback(async () => {
    if (!token || !hostId || !filePath) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.fileSyncPush(token, hostId, filePath, content);
      setResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, hostId, filePath, content]);

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold text-gray-900">文件分发</h2>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}
      {result && (
        <div className="rounded-md bg-green-50 p-3 text-sm text-green-700">
          文件已推送: {result.status} {result.file_path ? `(${result.file_path})` : ''}
        </div>
      )}

      <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm space-y-4">
        <h3 className="text-base font-semibold text-gray-900">推送文件到主机</h3>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <div>
            <label className="block text-sm font-medium text-gray-700">目标主机</label>
            <select value={hostId} onChange={(e) => setHostId(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm">
              <option value="">选择主机</option>
              {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700">远程文件路径</label>
            <input value={filePath} onChange={(e) => setFilePath(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="/etc/nginx/nginx.conf" />
          </div>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700">文件内容</label>
          <textarea value={content} onChange={(e) => setContent(e.target.value)} rows={10} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm font-mono focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" placeholder="文件内容..." />
        </div>
        <div className="flex justify-end">
          <button onClick={handlePush} disabled={loading || !hostId || !filePath} className={cn('rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {loading ? '推送中...' : '推送文件'}
          </button>
        </div>
      </div>
    </div>
  );
}
