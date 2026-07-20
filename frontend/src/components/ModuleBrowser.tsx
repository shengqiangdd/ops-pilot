import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ModuleInfo } from '../api/types';
import { getHealthLabel, getHealthColor } from '../lib/health';
import { useI18n } from '../i18n';
import { Skeleton } from './Skeleton';
import { cn } from '../lib/cn';

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
    <div className="space-y-6 animate-slide-up">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-headline-small font-semibold text-md-on-surface">{t('modules.title')}</h2>
          <p className="text-body-medium text-md-on-surface-variant mt-1">管理 OpsPilot 所有功能模块</p>
        </div>
        <button onClick={load} disabled={loading}
          className="glass-card rounded-md-lg px-5 py-2.5 text-body-medium font-medium text-md-primary hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2">
          <svg className={cn('w-4 h-4', loading && 'animate-spin')} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {loading ? '…' : t('modules.reload')}
        </button>
      </div>

      {error && (
        <div className="glass-card rounded-md-lg px-5 py-4 text-body-medium text-md-error bg-md-error-container/20">
          {error}
        </div>
      )}

      {/* 骨架屏 */}
      {loading && modules.length === 0 && (
        <div className="glass-card rounded-md-xl overflow-hidden">
          <div className="p-5 space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex items-center gap-4">
                <div className="flex-1 space-y-1.5">
                  <Skeleton height="16px" width="30%" />
                  <Skeleton height="12px" width="50%" />
                </div>
                <Skeleton height="28px" width="48px" className="rounded-md-full" />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 模块卡片（替换原来的表格，更现代） */}
      {!loading && modules.length > 0 && (
        <div className="space-y-3">
          {modules.map((m) => {
            const healthLabel = m.health ? getHealthLabel(m.health) : null;
            const dotColor = m.health ? getHealthColor(m.health) : '';
            const labelKey = healthLabel === 'Healthy' ? 'modules.healthy'
              : healthLabel === 'Degraded' ? 'modules.degraded'
              : healthLabel === 'Unhealthy' ? 'modules.unhealthy'
              : null;
            return (
              <div
                key={m.name}
                className={cn(
                  'glass-card rounded-md-xl px-5 py-4 animate-slide-up flex items-center gap-4 transition-all duration-200',
                  !m.enabled && 'opacity-60',
                )}
                style={{ animationDelay: `${modules.indexOf(m) * 40}ms` }}
              >
                {/* 图标 */}
                <div className="w-10 h-10 rounded-md-lg bg-md-primary-container/50 flex items-center justify-center text-lg shrink-0">
                  {m.name.includes('chat') ? '💬' : m.name.includes('ssh') ? '🔌' : m.name.includes('host') ? '🖥️' : m.name.includes('vault') ? '🔑' : m.name.includes('security') ? '🛡️' : m.name.includes('webhook') ? '🔗' : m.name.includes('scheduler') ? '⏰' : m.name.includes('filesync') ? '📁' : m.name.includes('advisor') ? '💡' : '🧩'}
                </div>
                {/* 信息 */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-body-large font-semibold text-md-on-surface truncate">{m.name}</span>
                    <span className="text-label-medium text-md-on-surface-variant px-2 py-0.5 rounded-md-md bg-md-surface-container-higher">{m.version}</span>
                  </div>
                  <p className="text-body-medium text-md-on-surface-variant mt-0.5 truncate">{m.description}</p>
                </div>
                {/* 健康状态 */}
                <div className="shrink-0 text-right">
                  {healthLabel && labelKey ? (
                    <span className="inline-flex items-center gap-1.5 text-label-medium">
                      <span className={cn('h-2 w-2 rounded-full', dotColor)} />
                      {t(labelKey)}
                    </span>
                  ) : <span className="text-label-medium text-md-outline">--</span>}
                </div>
                {/* 开关 */}
                <button
                  onClick={() => toggle(m.name, m.enabled)}
                  disabled={toggling === m.name}
                  className={cn(
                    'relative inline-flex h-7 w-12 shrink-0 cursor-pointer rounded-md-full border-2 border-transparent transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-md-primary focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
                    m.enabled ? 'bg-md-primary' : 'bg-md-surface-container-highest',
                  )}
                  role="switch" aria-checked={m.enabled}
                >
                  <span className={cn(
                    'pointer-events-none inline-block h-6 w-6 rounded-md-full bg-md-on-primary shadow-md-2 ring-0 transition-all duration-200',
                    m.enabled ? 'translate-x-5' : 'translate-x-0',
                  )} />
                </button>
              </div>
            );
          })}
        </div>
      )}

      {!loading && modules.length === 0 && (
        <div className="glass-card rounded-md-xl p-10 text-center">
          <p className="text-body-medium text-md-on-surface-variant">{t('modules.title')}</p>
        </div>
      )}
    </div>
  );
}
