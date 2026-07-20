import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { WebhookInfo } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

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
    } catch {
      setWebhooks([]);
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleRegister = useCallback(async () => {
    if (!token || !name || !url) return;
    setSaving(true);
    setError(null);
    try {
      await api.registerWebhook(token, { name, url, secret: secret || undefined, retry_count: 3 });
      setName('');
      setUrl('');
      setSecret('');
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setSaving(false);
    }
  }, [token, name, url, secret, load]);

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold text-gray-900">Webhook 管理</h2>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
        <h3 className="mb-3 text-base font-semibold text-gray-900">注册 Webhook</h3>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-4">
          <input value={name} onChange={(e) => setName(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="名称 (如 slack-alerts)" />
          <input value={url} onChange={(e) => setUrl(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="URL" />
          <input value={secret} onChange={(e) => setSecret(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="Secret (可选)" />
          <button onClick={handleRegister} disabled={saving || !name || !url} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {saving ? '注册中...' : '注册'}
          </button>
        </div>
      </div>

      <div className="rounded-lg border border-gray-200 bg-white shadow-sm overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">名称</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">URL</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">Secret</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">重试次数</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {webhooks.map((w, i) => (
              <tr key={i} className="hover:bg-gray-50">
                <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-gray-900">{w.name}</td>
                <td className="px-4 py-3 text-sm text-gray-600 font-mono">{w.url}</td>
                <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-500">{w.secret ? '***' : '-'}</td>
                <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-500">{w.retry_count}</td>
              </tr>
            ))}
            {webhooks.length === 0 && !loading && (
              <tr><td colSpan={4} className="px-4 py-8 text-center text-sm text-gray-500">暂无 Webhook</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
