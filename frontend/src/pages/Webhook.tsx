import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { WebhookInfo } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function WebhookPage() {
  const { token } = useAuthStore();
  const [webhooks, setWebhooks] = useState<WebhookInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [name, setName] = useState('');
  const [url, setUrl] = useState('');
  const [secret, setSecret] = useState('');
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listWebhooks(token);
      setWebhooks(data.webhooks || []);
    } catch { setWebhooks([]); } finally { setLoading(false); }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleRegister = useCallback(async () => {
    if (!token || !name || !url) return;
    setSaving(true);
    setError(null);
    try {
      await api.registerWebhook(token, { name, url, secret: secret || undefined, retry_count: 3 });
      setName(''); setUrl(''); setSecret('');
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally { setSaving(false); }
  }, [token, name, url, secret, load]);


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Webhooks</h2>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
        <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Register Webhook</h3>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-4">
          <input value={name} onChange={(e) => setName(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="Name (e.g. slack-alerts)" />
          <input value={url} onChange={(e) => setUrl(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="URL" />
          <input value={secret} onChange={(e) => setSecret(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="Secret (optional)" />
          <button onClick={handleRegister} disabled={saving || !name || !url}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {saving ? 'Registering...' : 'Register'}
          </button>
        </div>
      </div>

      <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1 overflow-hidden">
        <table className="min-w-full">
          <thead>
            <tr className="border-b border-md-outline-variant">
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Name</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">URL</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Secret</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Retries</th>
            </tr>
          </thead>
          <tbody>
            {webhooks.map((w, i) => (
              <tr key={i} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{w.name}</td>
                <td className="px-4 py-3 text-body-medium text-md-on-surface-variant font-mono">{w.url}</td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{w.secret ? '***' : '-'}</td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{w.retry_count}</td>
              </tr>
            ))}
            {webhooks.length === 0 && !loading && (
              <tr><td colSpan={4} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">No webhooks configured</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
