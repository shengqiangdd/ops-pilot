import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { AdvisorSuggestion } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const SEVERITY_COLORS: Record<string, string> = {
  critical: 'bg-red-100 text-red-800 border-red-300',
  warning: 'bg-yellow-100 text-yellow-800 border-yellow-300',
  info: 'bg-blue-100 text-blue-800 border-blue-300',
};

export function AdvisorPage() {
  const { token } = useAuthStore();
  const [suggestions, setSuggestions] = useState<AdvisorSuggestion[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listAdvisorSuggestions(token);
      setSuggestions(data.suggestions || []);
    } catch {
      setSuggestions([]);
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleAcknowledge = useCallback(async (id: string) => {
    if (!token) return;
    try {
      await api.acknowledgeSuggestion(token, id);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    }
  }, [token, load]);

  const handleDismiss = useCallback(async (id: string) => {
    if (!token) return;
    try {
      await api.dismissSuggestion(token, id);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    }
  }, [token, load]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">运维建议</h2>
        <button onClick={load} disabled={loading} className={cn('rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50')}>
          {loading ? '加载中...' : '刷新'}
        </button>
      </div>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      <div className="space-y-3">
        {suggestions.filter((s) => !s.dismissed).map((s) => (
          <div key={s.id} className={cn('rounded-lg border bg-white p-4 shadow-sm', s.acknowledged ? 'opacity-60' : '')}>
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <span className={cn('inline-block rounded-full border px-2.5 py-0.5 text-xs font-semibold uppercase', SEVERITY_COLORS[s.severity] || SEVERITY_COLORS.info)}>
                    {s.severity}
                  </span>
                  <span className="text-xs text-gray-500">{s.category}</span>
                  {s.acknowledged && <span className="text-xs text-green-600">已确认</span>}
                </div>
                <h4 className="mt-2 text-sm font-semibold text-gray-900">{s.title}</h4>
                <p className="mt-1 text-sm text-gray-600">{s.description}</p>
                {s.suggested_action && (
                  <p className="mt-1 text-xs text-blue-600">建议操作: {s.suggested_action}</p>
                )}
              </div>
              <div className="ml-4 flex gap-2">
                {!s.acknowledged && (
                  <button onClick={() => handleAcknowledge(s.id)} className="rounded-md border border-green-300 bg-green-50 px-2.5 py-1 text-xs font-medium text-green-700 hover:bg-green-100">
                    确认
                  </button>
                )}
                <button onClick={() => handleDismiss(s.id)} className="rounded-md border border-gray-300 px-2.5 py-1 text-xs font-medium text-gray-600 hover:bg-gray-50">
                  忽略
                </button>
              </div>
            </div>
          </div>
        ))}
        {suggestions.filter((s) => !s.dismissed).length === 0 && !loading && (
          <div className="rounded-lg border border-gray-200 bg-white p-8 text-center text-sm text-gray-500 shadow-sm">
            暂无运维建议
          </div>
        )}
      </div>
    </div>
  );
}
