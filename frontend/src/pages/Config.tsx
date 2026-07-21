import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { LoadingState, ErrorState } from '../lib/pageStates';

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


  if (loading) return <LoadingState skeleton="detail" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Configuration</h2>
        <button onClick={load} disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high disabled:opacity-50">
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
        <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Add/Update Config</h3>
        <div className="flex gap-2">
          <input value={newKey} onChange={(e) => setNewKey(e.target.value)}
            className="w-1/3 bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="Config key (e.g. ssh.host1)" />
          <input value={newValue} onChange={(e) => setNewValue(e.target.value)}
            className="flex-1 bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder='Config value (e.g. "10.0.0.1" or {"port": 22})' />
          <button onClick={handleSave} disabled={saving || !newKey}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>

      <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1 overflow-hidden">
        <table className="min-w-full">
          <thead>
            <tr className="border-b border-md-outline-variant">
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Key</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Value</th>
            </tr>
          </thead>
          <tbody>
            {Object.entries(configs).map(([key, val]) => (
              <tr key={key} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface font-mono">{key}</td>
                <td className="px-4 py-3 text-body-medium text-md-on-surface-variant font-mono">{typeof val === 'string' ? val : JSON.stringify(val)}</td>
              </tr>
            ))}
            {Object.keys(configs).length === 0 && !loading && (
              <tr><td colSpan={2} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">No config entries</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
