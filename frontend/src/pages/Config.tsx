import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

export function ConfigPage() {
  const { token } = useAuthStore();
  const [configs, setConfigs] = useState<Record<string, unknown>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listConfig(token);
      setConfigs(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load config');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleSave = useCallback(async () => {
    if (!token || !newKey) return;
    setSaving(true);
    setError(null);
    try {
      let parsed: unknown = newValue;
      try { parsed = JSON.parse(newValue); } catch { /* keep as string */ }
      await api.setConfigValue(token, newKey, parsed);
      setNewKey('');
      setNewValue('');
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save');
    } finally {
      setSaving(false);
    }
  }, [token, newKey, newValue, load]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">配置管理</h2>
        <button onClick={load} disabled={loading} className={cn('rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50')}>
          {loading ? '加载中...' : '刷新'}
        </button>
      </div>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
        <h3 className="mb-3 text-base font-semibold text-gray-900">添加/更新配置</h3>
        <div className="flex gap-2">
          <input value={newKey} onChange={(e) => setNewKey(e.target.value)} className="w-1/3 rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="配置键 (如 ssh.host1)" />
          <input value={newValue} onChange={(e) => setNewValue(e.target.value)} className="flex-1 rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder='配置值 (如 "10.0.0.1" 或 {"port": 22})' />
          <button onClick={handleSave} disabled={saving || !newKey} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {saving ? '保存中...' : '保存'}
          </button>
        </div>
      </div>

      <div className="rounded-lg border border-gray-200 bg-white shadow-sm overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">键</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">值</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {Object.entries(configs).map(([key, val]) => (
              <tr key={key} className="hover:bg-gray-50">
                <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-gray-900 font-mono">{key}</td>
                <td className="px-4 py-3 text-sm text-gray-600 font-mono">{typeof val === 'string' ? val : JSON.stringify(val)}</td>
              </tr>
            ))}
            {Object.keys(configs).length === 0 && !loading && (
              <tr><td colSpan={2} className="px-4 py-8 text-center text-sm text-gray-500">暂无配置项</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
