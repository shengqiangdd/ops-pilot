import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { AlertHistoryEntry } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function AlertHistoryPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [entries, setEntries] = useState<AlertHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');
  const [severityFilter, setSeverityFilter] = useState('');

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const params: Record<string, string> = {};
      if (fromDate) params.from = fromDate;
      if (toDate) params.to = toDate;
      if (severityFilter) params.severity = severityFilter;

      const data = await api.listAlertHistory(token, params);
      setEntries(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load alert history');
    } finally {
      setLoading(false);
    }
  }, [token, fromDate, toDate, severityFilter]);

  useEffect(() => { load(); }, [load]);

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'warning': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'info': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'firing': return 'text-red-500';
      case 'acknowledged': return 'text-amber-500';
      case 'resolved': return 'text-green-500';
      default: return 'text-md-on-surface-variant';
    }
  };


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.alert_history')}
        </h2>
        <button
          onClick={load}
          disabled={loading}
          className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors"
        >
          {loading ? t('alert_history.loading') : t('alert_history.reload')}
        </button>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Filters */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-end gap-3">
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('alert_history.filter.from')}</label>
            <input type="datetime-local" value={fromDate} onChange={(e) => setFromDate(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface" />
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('alert_history.filter.to')}</label>
            <input type="datetime-local" value={toDate} onChange={(e) => setToDate(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface" />
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('alert_history.filter.severity')}</label>
            <select value={severityFilter} onChange={(e) => setSeverityFilter(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface">
              <option value="">{t('alert_history.filter.all')}</option>
              <option value="critical">{t('alert_rules.severity_critical')}</option>
              <option value="warning">{t('alert_rules.severity_warning')}</option>
              <option value="info">{t('alert_rules.severity_info')}</option>
            </select>
          </div>
        </div>
      </div>

      {/* Table */}
      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_history.col.time')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_history.col.rule')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_history.col.severity')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_history.col.message')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('alert_history.col.status')}</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => (
                <tr key={entry.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">
                    {new Date(entry.triggered_at).toLocaleString()}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{entry.rule_name}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small">
                    <span className={cn('inline-block text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(entry.severity))}>
                      {entry.severity}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant max-w-xs truncate">
                    {entry.message}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small">
                    <span className={cn('font-medium', statusColor(entry.status))}>{entry.status}</span>
                  </td>
                </tr>
              ))}
              {!loading && entries.length === 0 && (
                <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('alert_history.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
