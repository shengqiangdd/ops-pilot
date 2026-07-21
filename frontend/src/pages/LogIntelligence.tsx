import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { LogSource, LogPattern, LogAnomaly, LogIntelStats } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

export function LogIntelligencePage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [stats, setStats] = useState<LogIntelStats | null>(null);
  const [sources, setSources] = useState<LogSource[]>([]);
  const [patterns, setPatterns] = useState<LogPattern[]>([]);
  const [anomalies, setAnomalies] = useState<LogAnomaly[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [analyzing, setAnalyzing] = useState(false);
  const [activeTab, setActiveTab] = useState<'sources' | 'patterns' | 'anomalies'>('patterns');

  const loadAll = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const [st, src, pat, anom] = await Promise.all([
        api.getLogIntelStats(token),
        api.listLogSources(token),
        api.listLogPatterns(token),
        api.listLogAnomalies(token),
      ]);
      setStats(st);
      setSources(src);
      setPatterns(pat);
      setAnomalies(anom);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadAll(); }, [loadAll]);

  const handleAnalyze = async () => {
    setAnalyzing(true);
    try {
      await api.analyzeLogs(token!);
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to analyze');
    } finally {
      setAnalyzing(false);
    }
  };

  const handleUpdateAnomaly = async (id: string, status: string) => {
    try {
      await api.updateLogAnomaly(token!, id, { status });
      await loadAll();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to update');
    }
  };

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'high': return 'bg-orange-500/10 text-orange-600 dark:text-orange-400';
      case 'medium': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'low': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.log_intel')}
        </h2>
        <div className="flex gap-2">
          <button onClick={loadAll} disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? t('logintel.loading') : t('logintel.reload')}
          </button>
          <button onClick={handleAnalyze} disabled={analyzing}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2">
            {analyzing ? (
              <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
            ) : '🔍'}
            {analyzing ? t('logintel.analyzing') : t('logintel.analyze')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{stats.total_sources}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('logintel.stats.sources')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-tertiary">{stats.total_patterns}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('logintel.stats.patterns')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-amber-500">{stats.total_anomalies}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('logintel.stats.anomalies')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-red-500">{stats.open_anomalies}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('logintel.stats.open')}</p>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 border-b border-md-outline-variant pb-2">
        {(['sources', 'patterns', 'anomalies'] as const).map((tab) => (
          <button key={tab} onClick={() => setActiveTab(tab)}
            className={cn('px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
              activeTab === tab ? 'bg-md-primary text-md-on-primary' : 'text-md-on-surface-variant hover:bg-md-surface-container-high')}>
            {t(`logintel.tab.${tab}`)}
          </button>
        ))}
      </div>

      {/* Sources */}
      {activeTab === 'sources' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.src_col.name')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.src_col.host')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.src_col.type')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.src_col.path')}</th>
                </tr>
              </thead>
              <tbody>
                {sources.map((src) => (
                  <tr key={src.id} className="border-b border-md-outline-variant last:border-0">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{src.source_name}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{src.host_id}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{src.source_type}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small font-mono text-md-on-surface-variant">{src.log_path}</td>
                  </tr>
                ))}
                {sources.length === 0 && (
                  <tr><td colSpan={4} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('logintel.no_sources')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Patterns */}
      {activeTab === 'patterns' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.pat_col.pattern')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.pat_col.type')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.pat_col.count')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('logintel.pat_col.severity')}</th>
                </tr>
              </thead>
              <tbody>
                {patterns.map((pat) => (
                  <tr key={pat.id} className="border-b border-md-outline-variant last:border-0">
                    <td className="px-4 py-3 text-body-small font-mono text-md-on-surface">{pat.pattern}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{pat.pattern_type}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{pat.count}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(pat.severity))}>{pat.severity}</span>
                    </td>
                  </tr>
                ))}
                {patterns.length === 0 && (
                  <tr><td colSpan={4} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('logintel.no_patterns')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Anomalies */}
      {activeTab === 'anomalies' && (
        <div className="space-y-3">
          {anomalies.map((anom) => (
            <div key={anom.id} className="glass-card rounded-md-xl p-4">
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(anom.severity))}>{anom.severity}</span>
                  <span className="text-body-medium font-medium text-md-on-surface">{anom.anomaly_type}</span>
                </div>
                <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm',
                  anom.status === 'open' ? 'bg-red-500/10 text-red-600' :
                  anom.status === 'investigating' ? 'bg-amber-500/10 text-amber-600' :
                  'bg-green-500/10 text-green-600')}>
                  {anom.status}
                </span>
              </div>
              <p className="text-body-small text-md-on-surface-variant mb-2">{anom.description}</p>
              <div className="flex items-center justify-between">
                <span className="text-label-small text-md-on-surface-variant">
                  🖥️ {anom.host_id} · {new Date(anom.detected_at).toLocaleString()}
                </span>
                {anom.status === 'open' && (
                  <div className="flex gap-1">
                    <button onClick={() => handleUpdateAnomaly(anom.id, 'investigating')}
                      className="text-xs px-2 py-1 rounded-md-sm text-amber-600 hover:bg-amber-500/10 transition-colors">
                      {t('logintel.investigate')}
                    </button>
                    <button onClick={() => handleUpdateAnomaly(anom.id, 'resolved')}
                      className="text-xs px-2 py-1 rounded-md-sm text-green-600 hover:bg-green-500/10 transition-colors">
                      {t('logintel.resolve')}
                    </button>
                  </div>
                )}
              </div>
            </div>
          ))}
          {anomalies.length === 0 && (
            <div className="glass-card rounded-md-xl p-8 text-center">
              <div className="text-4xl mb-3">✅</div>
              <p className="text-body-medium text-md-on-surface-variant">{t('logintel.no_anomalies')}</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
