import { useCallback, useEffect, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { api } from '../api/client';
import type { ApmService, ApmTrace, ApmError, ApmDashboard } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function APMPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [dashboard, setDashboard] = useState<ApmDashboard | null>(null);
  const [services, setServices] = useState<ApmService[]>([]);
  const [errors, setErrors] = useState<ApmError[]>([]);
  const [selectedService, setSelectedService] = useState<ApmService | null>(null);
  const [traces, setTraces] = useState<ApmTrace[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadAll = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const [dash, svc, errs] = await Promise.all([
        api.getApmDashboard(token),
        api.listApmServices(token),
        api.listRecentErrors(token),
      ]);
      setDashboard(dash);
      setServices(svc);
      setErrors(errs);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadAll(); }, [loadAll]);

  const loadTraces = async (serviceId: string) => {
    try {
      const svc = await api.getApmService(token!, serviceId);
      setSelectedService(svc);
      const traceData = await api.listServiceTraces(token!, serviceId);
      setTraces(traceData);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load traces');
    }
  };

  const handleErrorUpdate = async (errorId: string, status: string) => {
    try {
      await api.updateApmError(token!, errorId, { status });
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to update');
    }
  };

  const healthColor = (health: string) => {
    switch (health) {
      case 'healthy': return 'bg-green-500';
      case 'degraded': return 'bg-amber-500';
      case 'down': return 'bg-red-500';
      default: return 'bg-md-outline';
    }
  };

  const healthTextColor = (health: string) => {
    switch (health) {
      case 'healthy': return 'text-green-600';
      case 'degraded': return 'text-amber-600';
      case 'down': return 'text-red-600';
      default: return 'text-md-on-surface-variant';
    }
  };

  // Mock latency distribution data
  const latencyData = traces.length > 0 ? [
    { range: '0-100ms', count: traces.filter(t => t.duration_ms < 100).length },
    { range: '100-500ms', count: traces.filter(t => t.duration_ms >= 100 && t.duration_ms < 500).length },
    { range: '500ms-1s', count: traces.filter(t => t.duration_ms >= 500 && t.duration_ms < 1000).length },
    { range: '1s+', count: traces.filter(t => t.duration_ms >= 1000).length },
  ] : [];


  if (loading) return <LoadingState skeleton="chart" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.apm')}
        </h2>
        <button onClick={loadAll} disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
          {loading ? t('apm.loading') : t('apm.reload')}
        </button>
      </div>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      {/* Dashboard Stats */}
      {dashboard && (
        <div className="grid grid-cols-2 sm:grid-cols-5 gap-4">
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{dashboard.total_services}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('apm.stats.services')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-green-500">{dashboard.healthy_services}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('apm.stats.healthy')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{dashboard.total_requests.toLocaleString()}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('apm.stats.requests')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className={cn('text-headline-medium font-bold', dashboard.error_rate > 5 ? 'text-red-500' : 'text-md-primary')}>
              {dashboard.error_rate.toFixed(2)}%
            </p>
            <p className="text-label-small text-md-on-surface-variant">{t('apm.stats.error_rate')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{dashboard.avg_latency.toFixed(0)}ms</p>
            <p className="text-label-small text-md-on-surface-variant">{t('apm.stats.avg_latency')}</p>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Service List */}
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="px-4 py-3 border-b border-md-outline-variant">
            <h3 className="text-title-medium font-semibold text-md-on-surface">{t('apm.services')}</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('apm.svc_col.name')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('apm.svc_col.type')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('apm.svc_col.health')}</th>
                </tr>
              </thead>
              <tbody>
                {services.map(svc => (
                  <tr key={svc.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors cursor-pointer"
                      onClick={() => loadTraces(svc.id)}>
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{svc.name}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{svc.service_type}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('inline-flex items-center gap-1.5', healthTextColor(svc.health))}>
                        <span className={cn('h-2 w-2 rounded-full', healthColor(svc.health))} />
                        {svc.health}
                      </span>
                    </td>
                  </tr>
                ))}
                {services.length === 0 && <tr><td colSpan={3} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('apm.no_services')}</td></tr>}
              </tbody>
            </table>
          </div>
        </div>

        {/* Latency Distribution */}
        <div className="glass-card rounded-md-xl p-4">
          <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('apm.latency_dist')}</h3>
          {latencyData.length > 0 ? (
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={latencyData}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--md-sys-color-outline-variant)" />
                <XAxis dataKey="range" tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }} />
                <YAxis tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }} />
                <Tooltip />
                <Bar dataKey="count" fill="var(--md-sys-color-primary)" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[200px] flex items-center justify-center text-body-small text-md-on-surface-variant">{t('apm.no_data')}</div>
          )}
        </div>
      </div>

      {/* Selected Service Traces */}
      {selectedService && (
        <div className="glass-card rounded-md-xl p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-title-medium font-semibold text-md-on-surface">
              {selectedService.name} - {t('apm.traces')}
            </h3>
            <button onClick={() => setSelectedService(null)} className="text-sm text-md-primary hover:underline">{t('apm.close')}</button>
          </div>
          <div className="space-y-2">
            {traces.map(trace => (
              <div key={trace.id} className="flex items-center gap-3 px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
                <span className={cn('h-2 w-2 rounded-full shrink-0', trace.status === 'ok' ? 'bg-green-500' : 'bg-red-500')} />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-body-small font-medium text-md-on-surface">{trace.operation}</span>
                    {trace.http_method && <span className="text-label-small text-md-on-surface-variant font-mono">{trace.http_method}</span>}
                    {trace.http_path && <span className="text-label-small text-md-on-surface-variant font-mono truncate">{trace.http_path}</span>}
                  </div>
                  <div className="flex items-center gap-3 mt-1">
                    <span className="text-label-small text-md-on-surface-variant">{trace.duration_ms.toFixed(0)}ms</span>
                    <span className="text-label-small text-md-on-surface-variant">trace: {trace.trace_id.slice(0, 8)}...</span>
                  </div>
                </div>
                {/* Simple bar visualization */}
                <div className="w-32 h-2 bg-md-surface-container-highest rounded-full overflow-hidden">
                  <div className={cn('h-full rounded-full', trace.status === 'ok' ? 'bg-green-500' : 'bg-red-500')}
                    style={{ width: `${Math.min((trace.duration_ms / 1000) * 100, 100)}%` }} />
                </div>
              </div>
            ))}
            {traces.length === 0 && <p className="text-body-medium text-md-on-surface-variant text-center py-4">{t('apm.no_traces')}</p>}
          </div>
        </div>
      )}

      {/* Recent Errors */}
      <div className="glass-card rounded-md-xl p-4">
        <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">{t('apm.recent_errors')}</h3>
        <div className="space-y-2">
          {errors.map(err => (
            <div key={err.id} className="flex items-center justify-between px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className={cn('text-xs font-medium px-2 py-0.5 rounded-md-sm',
                    err.error_type === 'Exception' ? 'bg-red-500/10 text-red-600' : 'bg-amber-500/10 text-amber-600')}>
                    {err.error_type}
                  </span>
                  <span className="text-body-small font-medium text-md-on-surface truncate">{err.error_message}</span>
                </div>
                <p className="text-label-small text-md-on-surface-variant mt-1">
                  {t('apm.error_count')}: {err.count} · {t('apm.error_last')}: {new Date(err.last_seen).toLocaleString()}
                </p>
              </div>
              {err.status === 'open' && (
                <button onClick={() => handleErrorUpdate(err.id, 'resolved')}
                  className="text-xs px-2 py-1 rounded-md-sm text-green-600 hover:bg-green-500/10 transition-colors">
                  {t('apm.resolve')}
                </button>
              )}
            </div>
          ))}
          {errors.length === 0 && <p className="text-body-medium text-md-on-surface-variant text-center py-4">{t('apm.no_errors')}</p>}
        </div>
      </div>
    </div>
  );
}
