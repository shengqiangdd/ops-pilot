import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ModuleInfo } from '../api/types';
import { getHealthLabel, getHealthColor } from '../lib/health';
import { useI18n } from '../i18n';

interface ModuleRow extends ModuleInfo {
  health?: import('../api/types').HealthStatus;
}

export function ModuleBrowser() {
  const { t } = useI18n();
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
          } catch { return m; }
        }),
      );
      setModules(withHealth);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load modules');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const toggle = async (name: string, enabled: boolean) => {
    setToggling(name);
    try {
      if (enabled) { await api.disableModule(name); } else { await api.enableModule(name); }
      setModules((prev) => prev.map((m) => (m.name === name ? { ...m, enabled: !enabled } : m)));
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Toggle failed');
    } finally { setToggling(null); }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">{t('modules.title')}</h2>
        <button onClick={load} disabled={loading}
          className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
          {loading ? '…' : t('modules.reload')}
        </button>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1 overflow-hidden">
        <table className="min-w-full">
          <thead>
            <tr className="border-b border-md-outline-variant">
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('modules.name')}</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('modules.version')}</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('modules.description')}</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('modules.health')}</th>
              <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('modules.enabled')}</th>
            </tr>
          </thead>
          <tbody>
            {modules.map((m) => {
              const healthLabel = m.health ? getHealthLabel(m.health) : null;
              const dotColor = m.health ? getHealthColor(m.health) : '';
              const labelKey = healthLabel === 'Healthy' ? 'modules.healthy'
                : healthLabel === 'Degraded' ? 'modules.degraded'
                : healthLabel === 'Unhealthy' ? 'modules.unhealthy'
                : null;
              return (
                <tr key={m.name} className={`border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors ${!m.enabled ? 'opacity-60' : ''}`}>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{m.name}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{m.version}</td>
                  <td className="px-4 py-3 text-body-medium text-md-on-surface-variant">{m.description}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium">
                    {healthLabel && labelKey ? (
                      <span className="inline-flex items-center gap-1.5">
                        <span className={`h-2 w-2 rounded-full ${dotColor}`} />
                        {t(labelKey)}
                      </span>
                    ) : <span className="text-md-outline">--</span>}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-right">
                    <button onClick={() => toggle(m.name, m.enabled)} disabled={toggling === m.name}
                      className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-md-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-md-primary focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 ${m.enabled ? 'bg-md-primary' : 'bg-md-surface-container-highest'}`}
                      role="switch" aria-checked={m.enabled} aria-label={`Toggle ${m.name}`}>
                      <span className={`pointer-events-none inline-block h-5 w-5 rounded-md-full bg-md-on-primary shadow ring-0 transition duration-200 ease-in-out ${m.enabled ? 'translate-x-5' : 'translate-x-0'}`} />
                    </button>
                  </td>
                </tr>
              );
            })}
            {!loading && modules.length === 0 && (
              <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('modules.title')}</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
