import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ModuleHealth } from '../api/types';
import { cn } from '../lib/cn';
import { getHealthLabel, getHealthColor } from '../lib/health';
import { useI18n } from '../i18n';
import { Skeleton } from './Skeleton';

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
  const total = modules.length;

  const labelFor = (label: string | null) => {
    if (label === 'Healthy') return t('modules.healthy');
    if (label === 'Degraded') return t('modules.degraded');
    if (label === 'Unhealthy') return t('modules.unhealthy');
    return label ?? '--';
  };

  return (
    <div className="space-y-6 animate-slide-up">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-headline-small font-semibold text-md-on-surface">{t('health.title')}</h2>
          <p className="text-body-medium text-md-on-surface-variant mt-1">所有模块的实时健康状态</p>
        </div>
        <button onClick={load} disabled={loading}
          className="glass-card rounded-md-lg px-5 py-2.5 text-body-medium font-medium text-md-primary hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2">
          <svg className={cn('w-4 h-4', loading && 'animate-spin')} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {loading ? '…' : t('health.refresh')}
        </button>
      </div>

      {error && (
        <div className="glass-card rounded-md-lg px-5 py-4 text-body-medium text-md-error bg-md-error-container/20">
          {error}
        </div>
      )}

      {/* Summary cards */}
      <div className="grid grid-cols-1 sm:grid-cols-4 gap-4">
        {loading ? (
          <>
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="glass-card rounded-md-xl p-5 space-y-3"><Skeleton height="12px" width="60%" /><Skeleton height="28px" width="40%" /></div>
            ))}
          </>
        ) : (
          <>
            <div className="glass-card rounded-md-xl p-5 animate-slide-up">
              <p className="text-label-medium text-md-on-surface-variant mb-1">模块总数</p>
              <p className="text-headline-medium font-bold tabular-nums text-md-primary">{total}</p>
              <div className="mt-3 h-1 rounded-full bg-md-surface-container-highest overflow-hidden">
                <div className="h-full rounded-full bg-md-primary transition-all duration-1000" style={{ width: '100%' }} />
              </div>
            </div>
            <div className="glass-card rounded-md-xl p-5 animate-slide-up" style={{ animationDelay: '60ms' }}>
              <p className="text-label-medium text-md-on-surface-variant mb-1">{t('modules.healthy')}</p>
              <p className="text-headline-medium font-bold tabular-nums text-green-500">{healthy}</p>
              <div className="mt-3 h-1 rounded-full bg-md-surface-container-highest overflow-hidden">
                <div className="h-full rounded-full bg-green-500 transition-all duration-1000" style={{ width: total ? `${(healthy / total) * 100}%` : 0 }} />
              </div>
            </div>
            <div className="glass-card rounded-md-xl p-5 animate-slide-up" style={{ animationDelay: '120ms' }}>
              <p className="text-label-medium text-md-on-surface-variant mb-1">{t('modules.degraded')}</p>
              <p className="text-headline-medium font-bold tabular-nums text-amber-500">{degraded}</p>
              <div className="mt-3 h-1 rounded-full bg-md-surface-container-highest overflow-hidden">
                <div className="h-full rounded-full bg-amber-500 transition-all duration-1000" style={{ width: total ? `${(degraded / total) * 100}%` : 0 }} />
              </div>
            </div>
            <div className="glass-card rounded-md-xl p-5 animate-slide-up" style={{ animationDelay: '180ms' }}>
              <p className="text-label-medium text-md-on-surface-variant mb-1">{t('modules.unhealthy')}</p>
              <p className="text-headline-medium font-bold tabular-nums text-md-error">{unhealthy}</p>
              <div className="mt-3 h-1 rounded-full bg-md-surface-container-highest overflow-hidden">
                <div className="h-full rounded-full bg-md-error transition-all duration-1000" style={{ width: total ? `${(unhealthy / total) * 100}%` : 0 }} />
              </div>
            </div>
          </>
        )}
      </div>

      {/* Module health list (card-based) */}
      <div className="glass-card rounded-md-xl overflow-hidden p-5">
        <h3 className="text-title-medium font-semibold text-md-on-surface mb-4">模块详情</h3>
        {loading ? (
          <div className="space-y-3">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex items-center gap-3">
                <Skeleton circle height="10px" width="10px" />
                <Skeleton height="14px" width="25%" />
                <Skeleton height="12px" width="50px" className="ml-auto" />
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-2">
            {modules.map((m) => {
              const label = getHealthLabel(m.status);
              return (
                <div key={m.name} className="flex items-center justify-between px-4 py-3 rounded-md-lg hover:bg-md-surface-container-high/50 transition-colors">
                  <div className="flex items-center gap-3">
                    <span className={cn('h-2.5 w-2.5 rounded-full shrink-0', getHealthColor(m.status))} />
                    <span className="text-body-medium font-medium text-md-on-surface">{m.name}</span>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className={cn('text-label-medium', label === 'Healthy' ? 'text-green-500' : label === 'Degraded' ? 'text-amber-500' : label === 'Unhealthy' ? 'text-md-error' : 'text-md-outline')}>
                      {labelFor(label)}
                    </span>
                    <span className={cn('h-2 w-2 rounded-full', m.enabled ? 'bg-green-500' : 'bg-md-outline')} />
                  </div>
                </div>
              );
            })}
            {modules.length === 0 && (
              <p className="text-body-medium text-md-on-surface-variant text-center py-8">暂无模块数据</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
