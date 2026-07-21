import { useCallback, useEffect, useState } from 'react';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from 'recharts';
import { api } from '../api/client';
import type { ThreatOverview, ThreatIndicator, AffectedAsset } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function ThreatsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [overview, setOverview] = useState<ThreatOverview | null>(null);
  const [indicators, setIndicators] = useState<ThreatIndicator[]>([]);
  const [affectedAssets, setAffectedAssets] = useState<AffectedAsset[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadOverview = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const [ov, ind, assets] = await Promise.all([
        api.getThreatOverview(token),
        api.listThreatIndicators(token),
        api.getAffectedAssets(token),
      ]);
      setOverview(ov);
      setIndicators(ind);
      setAffectedAssets(assets);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load threats');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadOverview(); }, [loadOverview]);

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'high': return 'bg-orange-500/10 text-orange-600 dark:text-orange-400';
      case 'medium': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'low': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  const typeIcon = (type: string) => {
    switch (type) {
      case 'ip': return '🌐';
      case 'domain': return '🔗';
      case 'url': return '🔗';
      case 'hash': return '#️⃣';
      case 'cve': return '🔓';
      default: return '📌';
    }
  };

  const pieData = overview ? [
    { name: 'Critical', value: overview.critical_count, color: '#B3261E' },
    { name: 'High', value: overview.high_count, color: '#E8710A' },
    { name: 'Medium', value: overview.medium_count, color: '#F9A825' },
    { name: 'Low', value: overview.low_count, color: '#2196F3' },
  ].filter(d => d.value > 0) : [];


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.threats')}
        </h2>
        <button onClick={loadOverview} disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
          {loading ? t('threats.loading') : t('threats.reload')}
        </button>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Stats */}
      {overview && (
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{overview.total_indicators}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('threats.stats.total')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-amber-500">{overview.affected_assets}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('threats.stats.affected')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-red-500">{overview.critical_count + overview.high_count}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('threats.stats.high_risk')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-green-500">{overview.today_new}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('threats.stats.today_new')}</p>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Severity Pie Chart */}
        <div className="glass-card rounded-md-xl p-4">
          <h3 className="text-title-small font-semibold text-md-on-surface mb-2">{t('threats.severity_dist')}</h3>
          {pieData.length > 0 ? (
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie data={pieData} cx="50%" cy="50%" outerRadius={80} dataKey="value"
                  label>
                  {pieData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[200px] flex items-center justify-center text-body-small text-md-on-surface-variant">
              {t('threats.no_data')}
            </div>
          )}
        </div>

        {/* Affected Assets */}
        <div className="lg:col-span-2 glass-card rounded-md-xl p-4">
          <h3 className="text-title-small font-semibold text-md-on-surface mb-3">{t('threats.affected_assets')}</h3>
          <div className="space-y-2">
            {affectedAssets.map((asset) => (
              <div key={asset.id} className="flex items-center gap-3 px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
                <span className="text-lg">{typeIcon(asset.indicator_type)}</span>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-body-medium font-medium text-md-on-surface">{asset.host_name}</span>
                    <span className={cn('text-xs font-medium px-2 py-0.5 rounded-md-sm', severityColor(asset.severity))}>{asset.severity}</span>
                  </div>
                  <p className="text-label-small text-md-on-surface-variant truncate">{asset.threat_title}</p>
                </div>
                <span className="text-label-small text-md-on-surface-variant shrink-0">💡 {asset.suggestion}</span>
              </div>
            ))}
            {affectedAssets.length === 0 && (
              <p className="text-body-medium text-md-on-surface-variant text-center py-4">{t('threats.no_affected')}</p>
            )}
          </div>
        </div>
      </div>

      {/* Indicators Table */}
      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="px-4 py-3 border-b border-md-outline-variant">
          <h3 className="text-title-small font-semibold text-md-on-surface">{t('threats.indicators')}</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('threats.col.type')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('threats.col.value')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('threats.col.severity')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('threats.col.title')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('threats.col.first_seen')}</th>
              </tr>
            </thead>
            <tbody>
              {indicators.map((ind) => (
                <tr key={ind.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="whitespace-nowrap px-4 py-3 text-body-small">
                    <span className="flex items-center gap-1">
                      <span>{typeIcon(ind.indicator_type)}</span>
                      <span className="text-md-on-surface-variant">{ind.indicator_type}</span>
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small font-mono text-md-on-surface">{ind.indicator_value}</td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(ind.severity))}>{ind.severity}</span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{ind.title}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{new Date(ind.first_seen).toLocaleDateString()}</td>
                </tr>
              ))}
              {!loading && indicators.length === 0 && (
                <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('threats.no_indicators')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
