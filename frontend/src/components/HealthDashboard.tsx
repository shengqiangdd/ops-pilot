import { useCallback, useEffect, useRef, useState } from 'react';
import { api } from '../api/client';
import type { ModuleHealth } from '../api/types';

const REFRESH_INTERVAL = 30_000;

export function HealthDashboard() {
  const [modules, setModules] = useState<ModuleHealth[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastRefresh, setLastRefresh] = useState<Date | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval>>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      const data = await api.getHealthAll();
      setModules(data);
      setLastRefresh(new Date());
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch health');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
    timerRef.current = setInterval(load, REFRESH_INTERVAL);
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [load]);

  const healthy = modules.filter((m) => 'Healthy' in m.status).length;
  const degraded = modules.filter((m) => 'Degraded' in m.status).length;
  const unhealthy = modules.filter((m) => 'Unhealthy' in m.status).length;

  return (
    <div className="space-y-6 animate-slide-up">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Health Dashboard</h2>
          {lastRefresh && (
            <p className="text-label-medium text-md-on-surface-variant">Last refreshed: {lastRefresh.toLocaleTimeString()}</p>
          )}
        </div>
        <button onClick={load} disabled={loading}
          className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
          Refresh Now
        </button>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="grid grid-cols-3 gap-4">
        <div className="bg-md-primary-container text-md-on-primary-container rounded-md-lg p-4 text-center shadow-md-1">
          <div className="text-headline-medium font-medium">{healthy}</div>
          <div className="text-body-medium">Healthy</div>
        </div>
        <div className="bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-200 rounded-md-lg p-4 text-center shadow-md-1">
          <div className="text-headline-medium font-medium">{degraded}</div>
          <div className="text-body-medium">Degraded</div>
        </div>
        <div className="bg-md-error-container text-md-on-error-container rounded-md-lg p-4 text-center shadow-md-1">
          <div className="text-headline-medium font-medium">{unhealthy}</div>
          <div className="text-body-medium">Unhealthy</div>
        </div>
      </div>

      <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1 overflow-hidden">
        <table className="min-w-full">
          <thead>
            <tr className="border-b border-md-outline-variant">
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Module</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Status</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Details</th>
            </tr>
          </thead>
          <tbody>
            {modules.map((m) => {
              const isUnhealthy = 'Unhealthy' in m.status;
              const isDegraded = 'Degraded' in m.status;
              const reason = 'Degraded' in m.status ? m.status.Degraded.reason
                : 'Unhealthy' in m.status ? m.status.Unhealthy.reason : null;
              const dotColor = 'Healthy' in m.status ? 'bg-green-500'
                : 'Degraded' in m.status ? 'bg-amber-500' : 'bg-md-error';

              return (
                <tr key={m.name}
                  className={`border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors ${isUnhealthy ? 'bg-md-error-container/20' : isDegraded ? 'bg-amber-50 dark:bg-amber-900/10' : ''}`}>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">
                    {m.name}
                    {!m.enabled && <span className="ml-2 text-label-medium text-md-outline">(disabled)</span>}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium">
                    <span className="inline-flex items-center gap-1.5">
                      <span className={`h-2.5 w-2.5 rounded-full ${dotColor}`} />
                      {'Healthy' in m.status ? 'Healthy' : 'Degraded' in m.status ? 'Degraded' : 'Unhealthy'}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-body-medium text-md-on-surface-variant">{reason || '--'}</td>
                </tr>
              );
            })}
            {!loading && modules.length === 0 && (
              <tr><td colSpan={3} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">No modules to display</td></tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="flex items-center gap-2 text-label-medium text-md-outline">
        <span className="relative flex h-2 w-2">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
          <span className="relative inline-flex h-2 w-2 rounded-full bg-green-500" />
        </span>
        Auto-refreshing every {REFRESH_INTERVAL / 1000}s
      </div>
    </div>
  );
}
