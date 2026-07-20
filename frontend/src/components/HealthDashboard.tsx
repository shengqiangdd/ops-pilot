import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ModuleHealth } from '../api/types';
import { cn } from '../lib/cn';
import { getHealthLabel, getHealthColor } from '../lib/health';
import { useI18n } from '../i18n';

export function HealthDashboard() {
  const { t } = useI18n();
  const [modules, setModules] = useState<ModuleHealth[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.getHealthAll();
      setModules(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load health');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); const interval = setInterval(load, 30000); return () => clearInterval(interval); }, [load]);

  const healthy = modules.filter((m) => getHealthLabel(m.status) === 'Healthy').length;
  const degraded = modules.filter((m) => getHealthLabel(m.status) === 'Degraded').length;
  const unhealthy = modules.filter((m) => getHealthLabel(m.status) === 'Unhealthy').length;

  const labelFor = (label: string | null) => {
    if (label === 'Healthy') return t('modules.healthy');
    if (label === 'Degraded') return t('modules.degraded');
    if (label === 'Unhealthy') return t('modules.unhealthy');
    return label ?? '--';
  };

  return (
    <div className="space-y-6 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">{t('health.title')}</h2>
        <button onClick={load} disabled={loading}
          className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
          {loading ? '…' : t('health.refresh')}
        </button>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      {/* Summary cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="bg-md-primary-container rounded-md-lg p-4 shadow-md-1 animate-slide-up">
          <p className="text-label-medium text-md-on-primary-container/70">{t('modules.healthy')}</p>
          <p className="text-headline-medium font-medium text-md-on-primary-container">{healthy}</p>
        </div>
        <div className="bg-amber-50 dark:bg-amber-900/30 rounded-md-lg p-4 shadow-md-1 animate-slide-up" style={{ animationDelay: '100ms' }}>
          <p className="text-label-medium text-amber-700 dark:text-amber-200/70">{t('modules.degraded')}</p>
          <p className="text-headline-medium font-medium text-amber-800 dark:text-amber-100">{degraded}</p>
        </div>
        <div className="bg-md-error-container rounded-md-lg p-4 shadow-md-1 animate-slide-up" style={{ animationDelay: '200ms' }}>
          <p className="text-label-medium text-md-on-error-container/70">{t('modules.unhealthy')}</p>
          <p className="text-headline-medium font-medium text-md-on-error-container">{unhealthy}</p>
        </div>
      </div>

      {/* Module health table */}
      <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1 overflow-hidden">
        <table className="min-w-full">
          <thead>
            <tr className="border-b border-md-outline-variant">
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('health.module')}</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('health.status')}</th>
              <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('health.enabled')}</th>
            </tr>
          </thead>
          <tbody>
            {modules.map((m) => {
              const label = getHealthLabel(m.status);
              return (
                <tr key={m.name} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="px-4 py-3 text-body-medium font-medium text-md-on-surface">{m.name}</td>
                  <td className="px-4 py-3">
                    <span className="inline-flex items-center gap-1.5">
                      <span className={cn('h-2 w-2 rounded-full', getHealthColor(m.status))} />
                      {labelFor(label)}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-right">
                    <span className={cn('inline-flex items-center gap-1.5 text-label-medium', m.enabled ? 'text-green-600 dark:text-green-400' : 'text-md-on-surface-variant')}>
                      <span className={cn('h-2 w-2 rounded-full', m.enabled ? 'bg-green-500' : 'bg-md-outline')} />
                      {m.enabled ? '已启用' : '已禁用'}
                    </span>
                  </td>
                </tr>
              );
            })}
            {!loading && modules.length === 0 && (
              <tr><td colSpan={3} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('modules.title')}</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
