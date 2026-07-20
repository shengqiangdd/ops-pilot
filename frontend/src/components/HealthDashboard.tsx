import { useCallback, useEffect, useRef, useState } from 'react';
import { api } from '../api/client';
import type { ModuleHealth } from '../api/types';
import { cn } from '../lib/cn';

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
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [load]);

  const healthy = modules.filter((m) => 'Healthy' in m.status).length;
  const degraded = modules.filter((m) => 'Degraded' in m.status).length;
  const unhealthy = modules.filter((m) => 'Unhealthy' in m.status).length;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-gray-900">Health Dashboard</h2>
          {lastRefresh && (
            <p className="text-xs text-gray-500">
              Last refreshed: {lastRefresh.toLocaleTimeString()}
            </p>
          )}
        </div>
        <button
          onClick={load}
          disabled={loading}
          className={cn(
            'rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white',
            'hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
            'disabled:opacity-50',
          )}
        >
          Refresh Now
        </button>
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>
      )}

      {/* Summary cards */}
      <div className="grid grid-cols-3 gap-4">
        <div className="rounded-lg border border-green-200 bg-green-50 p-4 text-center">
          <div className="text-3xl font-bold text-green-700">{healthy}</div>
          <div className="text-sm text-green-600">Healthy</div>
        </div>
        <div className="rounded-lg border border-yellow-200 bg-yellow-50 p-4 text-center">
          <div className="text-3xl font-bold text-yellow-700">{degraded}</div>
          <div className="text-sm text-yellow-600">Degraded</div>
        </div>
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-center">
          <div className="text-3xl font-bold text-red-700">{unhealthy}</div>
          <div className="text-sm text-red-600">Unhealthy</div>
        </div>
      </div>

      {/* Module list */}
      <div className="overflow-hidden rounded-lg border border-gray-200">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Module
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Status
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Details
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 bg-white">
            {modules.map((m) => {
              const isUnhealthy = 'Unhealthy' in m.status;
              const isDegraded = 'Degraded' in m.status;
              const reason =
                'Degraded' in m.status
                  ? m.status.Degraded.reason
                  : 'Unhealthy' in m.status
                    ? m.status.Unhealthy.reason
                    : null;

              return (
                <tr
                  key={m.name}
                  className={cn(
                    isUnhealthy && 'bg-red-50',
                    isDegraded && 'bg-yellow-50',
                  )}
                >
                  <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-gray-900">
                    {m.name}
                    {!m.enabled && (
                      <span className="ml-2 text-xs text-gray-400">(disabled)</span>
                    )}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-sm">
                    <span className="inline-flex items-center gap-1.5">
                      <span
                        className={cn(
                          'h-2.5 w-2.5 rounded-full',
                          'Healthy' in m.status
                            ? 'bg-green-500'
                            : 'Degraded' in m.status
                              ? 'bg-yellow-500'
                              : 'bg-red-500',
                        )}
                      />
                      {'Healthy' in m.status
                        ? 'Healthy'
                        : 'Degraded' in m.status
                          ? 'Degraded'
                          : 'Unhealthy'}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-500">
                    {reason || '--'}
                  </td>
                </tr>
              );
            })}
            {!loading && modules.length === 0 && (
              <tr>
                <td colSpan={3} className="px-4 py-8 text-center text-sm text-gray-500">
                  No modules to display
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Auto-refresh indicator */}
      <div className="flex items-center gap-2 text-xs text-gray-400">
        <span className="relative flex h-2 w-2">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
          <span className="relative inline-flex h-2 w-2 rounded-full bg-green-500" />
        </span>
        Auto-refreshing every {REFRESH_INTERVAL / 1000}s
      </div>
    </div>
  );
}
