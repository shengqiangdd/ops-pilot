import { useCallback, useEffect, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Cell } from 'recharts';
import { api } from '../api/client';
import type { ScanResult, ScanStats } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

export function SecretsScanPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [results, setResults] = useState<ScanResult[]>([]);
  const [stats, setStats] = useState<ScanStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [severityFilter, setSeverityFilter] = useState('');
  const [statusFilter, setStatusFilter] = useState('');

  const loadResults = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const params: Record<string, string> = {};
      if (severityFilter) params.severity = severityFilter;
      if (statusFilter) params.status = statusFilter;
      const data = await api.listSecretsResults(token, params);
      setResults(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load results');
    } finally {
      setLoading(false);
    }
  }, [token, severityFilter, statusFilter]);

  const loadStats = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getSecretsStats(token);
      setStats(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load stats');
    }
  }, [token]);

  useEffect(() => { loadResults(); loadStats(); }, [loadResults, loadStats]);

  const handleScan = async () => {
    setScanning(true);
    try {
      await api.runSecretsScan(token!);
      await loadResults();
      await loadStats();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to scan');
    } finally {
      setScanning(false);
    }
  };

  const handleUpdateStatus = async (id: string, status: string) => {
    try {
      await api.updateSecretsResult(token!, id, { status });
      await loadResults();
      await loadStats();
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

  const typeLabel = (type: string) => {
    switch (type) {
      case 'hardcoded_key': return '🔑 Hardcoded Key';
      case 'weak_password': return '🔓 Weak Password';
      case 'api_token': return '🎫 API Token';
      case 'private_key': return '🗝️ Private Key';
      case 'env_leak': return '🌍 Env Leak';
      default: return type;
    }
  };

  const COLORS = ['#B3261E', '#E8710A', '#F9A825', '#4CAF50', '#2196F3'];

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.secrets_scan')}
        </h2>
        <button onClick={handleScan} disabled={scanning}
          className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2">
          {scanning ? (
            <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
          ) : '🔍'}
          {scanning ? t('secrets.scanning') : t('secrets.scan')}
        </button>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Stats & Chart */}
      {stats && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {/* Stats Cards */}
          <div className="grid grid-cols-2 gap-4">
            <div className="glass-card rounded-md-xl p-4 text-center">
              <p className="text-headline-medium font-bold text-md-primary">{stats.total}</p>
              <p className="text-label-small text-md-on-surface-variant">{t('secrets.stats.total')}</p>
            </div>
            <div className="glass-card rounded-md-xl p-4 text-center">
              <p className="text-headline-medium font-bold text-red-500">{stats.by_severity.find(s => s.severity === 'critical')?.count || 0}</p>
              <p className="text-label-small text-md-on-surface-variant">{t('secrets.stats.critical')}</p>
            </div>
          </div>

          {/* Chart */}
          <div className="glass-card rounded-md-xl p-4">
            <h3 className="text-title-small font-semibold text-md-on-surface mb-3">{t('secrets.by_type')}</h3>
            <ResponsiveContainer width="100%" height={150}>
              <BarChart data={stats.by_type} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="var(--md-sys-color-outline-variant)" />
                <XAxis type="number" tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }} />
                <YAxis type="category" dataKey="scan_type" tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }} width={100} />
                <Tooltip />
                <Bar dataKey="count" radius={[0, 4, 4, 0]}>
                  {stats.by_type.map((_, index) => (
                    <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-center gap-3">
          <label className="text-label-medium text-md-on-surface-variant">{t('secrets.filter.severity')}</label>
          <div className="flex gap-2">
            {['', 'critical', 'high', 'medium', 'low'].map((sev) => (
              <button key={sev} onClick={() => setSeverityFilter(sev)}
                className={cn('px-3 py-1.5 text-sm rounded-md-full transition-colors',
                  severityFilter === sev ? 'bg-md-primary text-md-on-primary' : 'bg-md-surface-container-high text-md-on-surface-variant')}>
                {sev || t('secrets.filter.all')}
              </button>
            ))}
          </div>
          <label className="text-label-medium text-md-on-surface-variant ml-2">{t('secrets.filter.status')}</label>
          <div className="flex gap-2">
            {['', 'open', 'false_positive', 'fixed'].map((st) => (
              <button key={st} onClick={() => setStatusFilter(st)}
                className={cn('px-3 py-1.5 text-sm rounded-md-full transition-colors',
                  statusFilter === st ? 'bg-md-primary text-md-on-primary' : 'bg-md-surface-container-high text-md-on-surface-variant')}>
                {st || t('secrets.filter.all')}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Results Table */}
      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('secrets.col.file')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('secrets.col.type')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('secrets.col.severity')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('secrets.col.snippet')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('secrets.col.status')}</th>
                <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('secrets.col.actions')}</th>
              </tr>
            </thead>
            <tbody>
              {results.map((result) => (
                <tr key={result.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="whitespace-nowrap px-4 py-3 text-body-small font-mono text-md-on-surface">{result.file_path}:{result.line_number}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{typeLabel(result.scan_type)}</td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(result.severity))}>{result.severity}</span>
                  </td>
                  <td className="px-4 py-3 max-w-xs">
                    <code className="text-xs bg-md-surface-container-highest px-2 py-1 rounded text-red-500 font-mono break-all">{result.snippet}</code>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm',
                      result.status === 'open' ? 'bg-red-500/10 text-red-600' :
                      result.status === 'fixed' ? 'bg-green-500/10 text-green-600' :
                      'bg-md-surface-container-high text-md-on-surface-variant')}>
                      {result.status}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-right">
                    {result.status === 'open' && (
                      <div className="flex items-center justify-end gap-1">
                        <button onClick={() => handleUpdateStatus(result.id, 'false_positive')}
                          className="text-xs px-2 py-1 rounded-md-sm text-amber-600 hover:bg-amber-500/10 transition-colors">
                          {t('secrets.mark_fp')}
                        </button>
                        <button onClick={() => handleUpdateStatus(result.id, 'fixed')}
                          className="text-xs px-2 py-1 rounded-md-sm text-green-600 hover:bg-green-500/10 transition-colors">
                          {t('secrets.mark_fixed')}
                        </button>
                      </div>
                    )}
                  </td>
                </tr>
              ))}
              {!loading && results.length === 0 && (
                <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('secrets.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
