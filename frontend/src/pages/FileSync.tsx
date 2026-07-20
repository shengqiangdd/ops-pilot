import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { FileSyncResult } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';

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
    api.listHosts(token).then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
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
    } finally { setLoading(false); }
  }, [token, hostId, filePath, content]);

  return (
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">File Sync</h2>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}
      {result && (
        <div className="bg-md-primary-container text-md-on-primary-container rounded-md-sm px-4 py-3 text-body-medium">
          File pushed: {result.status} {result.file_path ? `(${result.file_path})` : ''}
        </div>
      )}

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1 space-y-4">
        <h3 className="text-title-medium font-medium text-md-on-surface">Push File to Host</h3>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <div>
            <label className="block text-label-large text-md-on-surface">Target Host</label>
            <select value={hostId} onChange={(e) => setHostId(e.target.value)}
              className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface">
              <option value="">Select host</option>
              {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
            </select>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface">Remote File Path</label>
            <input value={filePath} onChange={(e) => setFilePath(e.target.value)}
              className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
              placeholder="/etc/nginx/nginx.conf" />
          </div>
        </div>
        <div>
          <label className="block text-label-large text-md-on-surface">File Content</label>
          <textarea value={content} onChange={(e) => setContent(e.target.value)} rows={10}
            className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface font-mono"
            placeholder="File content..." />
        </div>
        <div className="flex justify-end">
          <button onClick={handlePush} disabled={loading || !hostId || !filePath}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {loading ? 'Pushing...' : 'Push File'}
          </button>
        </div>
      </div>
    </div>
  );
}
