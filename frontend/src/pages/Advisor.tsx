import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { AdvisorSuggestion } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const SEVERITY_COLORS: Record<string, string> = {
  critical: 'bg-md-error-container text-md-on-error-container',
  warning: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200',
  info: 'bg-md-primary-container text-md-on-primary-container',
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
    } catch { setSuggestions([]); } finally { setLoading(false); }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleAcknowledge = useCallback(async (id: string) => {
    if (!token) return;
    try { await api.acknowledgeSuggestion(token, id); await load(); } catch (e) { setError(e instanceof Error ? e.message : 'Failed'); }
  }, [token, load]);

  const handleDismiss = useCallback(async (id: string) => {
    if (!token) return;
    try { await api.dismissSuggestion(token, id); await load(); } catch (e) { setError(e instanceof Error ? e.message : 'Failed'); }
  }, [token, load]);

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Advisor</h2>
        <button onClick={load} disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high disabled:opacity-50">
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="space-y-3">
        {suggestions.filter((s) => !s.dismissed).map((s) => (
          <div key={s.id} className={cn('bg-md-surface-container-low rounded-md-lg p-4 shadow-md-1', s.acknowledged ? 'opacity-60' : '')}>
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <span className={cn('inline-block rounded-md-full px-2.5 py-0.5 text-label-medium font-semibold uppercase', SEVERITY_COLORS[s.severity] || SEVERITY_COLORS.info)}>
                    {s.severity}
                  </span>
                  <span className="text-label-medium text-md-on-surface-variant">{s.category}</span>
                  {s.acknowledged && <span className="text-label-medium text-green-600">Acknowledged</span>}
                </div>
                <h4 className="mt-2 text-body-large font-medium text-md-on-surface">{s.title}</h4>
                <p className="mt-1 text-body-medium text-md-on-surface-variant">{s.description}</p>
                {s.suggested_action && (
                  <p className="mt-1 text-body-medium text-md-primary">Suggested: {s.suggested_action}</p>
                )}
              </div>
              <div className="ml-4 flex gap-2">
                {!s.acknowledged && (
                  <button onClick={() => handleAcknowledge(s.id)}
                    className="border border-md-primary text-md-primary rounded-md-lg px-4 py-1.5 text-label-large font-medium hover:bg-md-primary-container/20 transition-colors">
                    Acknowledge
                  </button>
                )}
                <button onClick={() => handleDismiss(s.id)}
                  className="border border-md-outline text-md-on-surface-variant rounded-md-lg px-4 py-1.5 text-label-large font-medium hover:bg-md-surface-container-high transition-colors">
                  Dismiss
                </button>
              </div>
            </div>
          </div>
        ))}
        {suggestions.filter((s) => !s.dismissed).length === 0 && !loading && (
          <div className="bg-md-surface-container-low rounded-md-lg p-8 text-center text-body-medium text-md-on-surface-variant shadow-md-1">
            No advisor suggestions
          </div>
        )}
      </div>
    </div>
  );
}
