import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ModuleInfo, HealthStatus } from '../api/types';
import { cn } from '../lib/cn';

interface ModuleRow extends ModuleInfo {
  health?: HealthStatus;
}

export function ModuleBrowser() {
  const [modules, setModules] = useState<ModuleRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [toggling, setToggling] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await api.listModules();
      const withHealth: ModuleRow[] = await Promise.all(
        list.map(async (m) => {
          try {
            const health = await api.getModuleHealth(m.name);
            return { ...m, health };
          } catch {
            return m;
          }
        }),
      );
      setModules(withHealth);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load modules');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const toggle = async (name: string, enabled: boolean) => {
    setToggling(name);
    try {
      if (enabled) {
        await api.disableModule(name);
      } else {
        await api.enableModule(name);
      }
      setModules((prev) =>
        prev.map((m) => (m.name === name ? { ...m, enabled: !enabled } : m)),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Toggle failed');
    } finally {
      setToggling(null);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">Modules</h2>
        <button
          onClick={load}
          disabled={loading}
          className={cn(
            'rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white',
            'hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
            'disabled:opacity-50',
          )}
        >
          {loading ? 'Loading...' : 'Reload'}
        </button>
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>
      )}

      <div className="overflow-hidden rounded-lg border border-gray-200">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Name
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Version
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Description
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                Health
              </th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500">
                Enabled
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 bg-white">
            {modules.map((m) => (
              <tr key={m.name} className={cn(!m.enabled && 'bg-gray-50 opacity-60')}>
                <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-gray-900">
                  {m.name}
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-500">
                  {m.version}
                </td>
                <td className="px-4 py-3 text-sm text-gray-500">{m.description}</td>
                <td className="whitespace-nowrap px-4 py-3 text-sm">
                  {m.health ? (
                    <span className="inline-flex items-center gap-1.5">
                      <span
                        className={cn(
                          'h-2 w-2 rounded-full',
                          'Healthy' in m.health
                            ? 'bg-green-500'
                            : 'Degraded' in m.health
                              ? 'bg-yellow-500'
                              : 'bg-red-500',
                        )}
                      />
                      {'Healthy' in m.health
                        ? 'Healthy'
                        : 'Degraded' in m.health
                          ? 'Degraded'
                          : 'Unhealthy'}
                    </span>
                  ) : (
                    <span className="text-gray-400">--</span>
                  )}
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-right">
                  <button
                    onClick={() => toggle(m.name, m.enabled)}
                    disabled={toggling === m.name}
                    className={cn(
                      'relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent',
                      'transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
                      'disabled:cursor-not-allowed disabled:opacity-50',
                      m.enabled ? 'bg-blue-600' : 'bg-gray-300',
                    )}
                    role="switch"
                    aria-checked={m.enabled}
                    aria-label={`Toggle ${m.name}`}
                  >
                    <span
                      className={cn(
                        'pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow ring-0',
                        'transition duration-200 ease-in-out',
                        m.enabled ? 'translate-x-5' : 'translate-x-0',
                      )}
                    />
                  </button>
                </td>
              </tr>
            ))}
            {!loading && modules.length === 0 && (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-sm text-gray-500">
                  No modules loaded
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
