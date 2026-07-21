import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import { DiagnosticCard } from '../components/DiagnosticCard';
import type { SystemStatus } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

interface DiagnosticItem {
  check_name: string;
  status: string;
  value: string;
  threshold: string | null;
  message: string;
  suggestion: string;
}

interface DiagnosticCategory {
  name: string;
  status: string;
  score: number;
  items: DiagnosticItem[];
}

interface DiagnosticReport {
  id: string;
  host_id: string;
  timestamp: string;
  overall_status: string;
  overall_score: number;
  categories: DiagnosticCategory[];
}

export function DiagnosticsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [report, setReport] = useState<DiagnosticReport | null>(null);
  const [history, setHistory] = useState<DiagnosticReport[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedHost, setSelectedHost] = useState('');
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'history'>('overview');

  // Load system status
  const loadSystemStatus = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getDiagnosticsStatus(token);
      setSystemStatus(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load status');
    }
  }, [token]);

  // Load hosts
  useEffect(() => {
    if (!token) return;
    api.listHosts(token)
      .then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name }))))
      .catch(() => {});
  }, [token]);

  useEffect(() => { loadSystemStatus(); }, [loadSystemStatus]);

  // Load history
  const loadHistory = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getDiagnosticsHistory(token);
      setHistory(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load history');
    }
  }, [token]);

  useEffect(() => {
    if (activeTab === 'history') loadHistory();
  }, [activeTab, loadHistory]);

  // Run diagnostics
  const handleRunDiagnostics = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.runDiagnostics(token!, {
        host_id: selectedHost || undefined,
        checks: ['cpu', 'memory', 'disk', 'network', 'services', 'security'],
      });
      setReport(data);
      await loadSystemStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to run diagnostics');
    } finally {
      setLoading(false);
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'ok': case 'healthy': return 'text-green-500';
      case 'warning': return 'text-amber-500';
      case 'critical': return 'text-red-500';
      default: return 'text-md-on-surface-variant';
    }
  };

  const statusBg = (status: string) => {
    switch (status) {
      case 'ok': case 'healthy': return 'bg-green-500/10';
      case 'warning': return 'bg-amber-500/10';
      case 'critical': return 'bg-red-500/10';
      default: return 'bg-md-surface-container-high';
    }
  };

  // Overall score circle
  const overallScore = report?.overall_score ?? systemStatus?.overall_health_score ?? 0;
  const radius = 60;
  const circumference = 2 * Math.PI * radius;
  const progress = (overallScore / 100) * circumference;

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.diagnostics')}
        </h2>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium flex items-center justify-between">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-sm underline">{t('diagnostics.dismiss')}</button>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 border-b border-md-outline-variant pb-2">
        <button
          onClick={() => setActiveTab('overview')}
          className={cn(
            'px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
            activeTab === 'overview'
              ? 'bg-md-primary text-md-on-primary'
              : 'text-md-on-surface-variant hover:bg-md-surface-container-high',
          )}
        >
          {t('diagnostics.tab.overview')}
        </button>
        <button
          onClick={() => setActiveTab('history')}
          className={cn(
            'px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
            activeTab === 'history'
              ? 'bg-md-primary text-md-on-primary'
              : 'text-md-on-surface-variant hover:bg-md-surface-container-high',
          )}
        >
          {t('diagnostics.tab.history')}
        </button>
      </div>

      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* System Overview Score */}
          <div className="glass-card rounded-md-xl p-6">
            <div className="flex flex-col md:flex-row items-center gap-8">
              {/* Score Circle */}
              <div className="relative w-36 h-36 shrink-0">
                <svg className="w-36 h-36 -rotate-90" viewBox="0 0 140 140">
                  <circle
                    cx="70" cy="70" r={radius}
                    fill="none"
                    stroke="var(--md-sys-color-surface-container-highest)"
                    strokeWidth="8"
                  />
                  <circle
                    cx="70" cy="70" r={radius}
                    fill="none"
                    className={cn(
                      'transition-all duration-1000',
                      overallScore >= 80 ? 'stroke-green-500' : overallScore >= 60 ? 'stroke-amber-500' : 'stroke-red-500'
                    )}
                    strokeWidth="8"
                    strokeDasharray={circumference}
                    strokeDashoffset={circumference - progress}
                    strokeLinecap="round"
                  />
                </svg>
                <div className="absolute inset-0 flex flex-col items-center justify-center">
                  <span className="text-headline-large font-bold text-md-on-surface">
                    {Math.round(overallScore)}
                  </span>
                  <span className="text-label-small text-md-on-surface-variant">{t('diagnostics.score')}</span>
                </div>
              </div>

              {/* Status Info */}
              <div className="flex-1 space-y-4">
                <div>
                  <h3 className="text-title-large font-semibold text-md-on-surface">
                    {t('diagnostics.system_health')}
                  </h3>
                  <p className="text-body-medium text-md-on-surface-variant mt-1">
                    {overallScore >= 80
                      ? t('diagnostics.status_healthy')
                      : overallScore >= 60
                      ? t('diagnostics.status_warning')
                      : t('diagnostics.status_critical')}
                  </p>
                </div>

                {systemStatus && (
                  <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                    <div>
                      <p className="text-label-small text-md-on-surface-variant">{t('diagnostics.hosts')}</p>
                      <p className="text-body-medium font-medium text-md-on-surface">
                        {systemStatus.online_hosts}/{systemStatus.total_hosts}
                      </p>
                    </div>
                    <div>
                      <p className="text-label-small text-md-on-surface-variant">{t('diagnostics.services')}</p>
                      <p className="text-body-medium font-medium text-md-on-surface">
                        {systemStatus.active_services}/{systemStatus.total_services}
                      </p>
                    </div>
                    <div>
                      <p className="text-label-small text-md-on-surface-variant">{t('diagnostics.alert_rules')}</p>
                      <p className="text-body-medium font-medium text-md-on-surface">
                        {systemStatus.active_alert_rules}/{systemStatus.total_alert_rules}
                      </p>
                    </div>
                    <div>
                      <p className="text-label-small text-md-on-surface-variant">{t('diagnostics.recent_alerts')}</p>
                      <p className="text-body-medium font-medium text-md-on-surface">
                        {systemStatus.recent_alerts}
                      </p>
                    </div>
                  </div>
                )}

                {/* Run Diagnostics */}
                <div className="flex items-center gap-3">
                  <select
                    value={selectedHost}
                    onChange={(e) => setSelectedHost(e.target.value)}
                    className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface"
                  >
                    <option value="">{t('diagnostics.all_hosts')}</option>
                    {hosts.map((h) => (
                      <option key={h.id} value={h.id}>{h.name}</option>
                    ))}
                  </select>
                  <button
                    onClick={handleRunDiagnostics}
                    disabled={loading}
                    className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2"
                  >
                    {loading ? (
                      <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    ) : (
                      <span>🔍</span>
                    )}
                    {loading ? t('diagnostics.running') : t('diagnostics.run')}
                  </button>
                </div>
              </div>
            </div>
          </div>

          {/* Diagnostic Results */}
          {report && (
            <div className="space-y-4">
              <h3 className="text-title-medium font-semibold text-md-on-surface">
                {t('diagnostics.results')}
              </h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {report.categories.map((cat) => (
                  <div key={cat.name}>
                    <DiagnosticCard
                      name={cat.name}
                      status={cat.status}
                      score={cat.score}
                      icon=""
                      onClick={() => setExpandedCategory(expandedCategory === cat.name ? null : cat.name)}
                      expanded={expandedCategory === cat.name}
                    />
                    {expandedCategory === cat.name && (
                      <div className="mt-2 glass-card rounded-md-lg p-3 space-y-2 animate-slide-up">
                        {cat.items.map((item) => (
                          <div key={item.check_name} className="flex items-start gap-3 p-2 rounded-md-sm bg-md-surface-container-highest/50">
                            <span className={cn(
                              'h-5 w-5 rounded-full flex items-center justify-center text-xs shrink-0 mt-0.5',
                              item.status === 'ok' ? 'bg-green-500/20 text-green-600' :
                              item.status === 'warning' ? 'bg-amber-500/20 text-amber-600' :
                              'bg-red-500/20 text-red-600'
                            )}>
                              {item.status === 'ok' ? '✓' : item.status === 'warning' ? '!' : '✗'}
                            </span>
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center justify-between">
                                <span className="text-body-small font-medium text-md-on-surface">{item.check_name}</span>
                                <span className="text-label-small text-md-on-surface-variant">{item.value}</span>
                              </div>
                              <p className="text-label-small text-md-on-surface-variant mt-0.5">{item.suggestion}</p>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {!report && !loading && (
            <div className="glass-card rounded-md-xl p-8 text-center">
              <div className="text-4xl mb-3">🔍</div>
              <p className="text-body-medium text-md-on-surface-variant">
                {t('diagnostics.no_report')}
              </p>
            </div>
          )}
        </div>
      )}

      {activeTab === 'history' && (
        <div className="glass-card rounded-md-xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('diagnostics.history.time')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('diagnostics.history.host')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('diagnostics.history.status')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('diagnostics.history.score')}</th>
                </tr>
              </thead>
              <tbody>
                {history.map((h) => (
                  <tr key={h.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors cursor-pointer"
                      onClick={() => { setReport(h); setActiveTab('overview'); }}>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">
                      {new Date(h.timestamp).toLocaleString()}
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface">{h.host_id}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusBg(h.overall_status), statusColor(h.overall_status))}>
                        {h.overall_status}
                      </span>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">
                      {Math.round(h.overall_score)}
                    </td>
                  </tr>
                ))}
                {history.length === 0 && (
                  <tr><td colSpan={4} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('diagnostics.no_history')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
