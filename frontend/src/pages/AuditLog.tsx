import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { AuditLogEntry, AuditLogResponse } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

interface AuditFilters {
  user: string;
  action: string;
  from: string;
  to: string;
}

export function AuditLogPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [logs, setLogs] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [perPage] = useState(20);
  const [filters, setFilters] = useState<AuditFilters>({
    user: '',
    action: '',
    from: '',
    to: '',
  });

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const params: Record<string, string> = {
        page: String(page),
        per_page: String(perPage),
      };
      if (filters.user) params.user = filters.user;
      if (filters.action) params.action = filters.action;
      if (filters.from) params.from = filters.from;
      if (filters.to) params.to = filters.to;

      const resp: AuditLogResponse = await api.listAuditLogs(token, params);
      setLogs(resp.data);
      setTotal(resp.total);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load audit logs');
    } finally {
      setLoading(false);
    }
  }, [token, page, perPage, filters]);

  useEffect(() => { load(); }, [load]);

  const handleExport = () => {
    if (!token) return;
    const params = new URLSearchParams();
    if (filters.from) params.set('from', filters.from);
    if (filters.to) params.set('to', filters.to);
    if (filters.user) params.set('user', filters.user);
    if (filters.action) params.set('action', filters.action);

    const url = `/api/audit/export?${params.toString()}`;
    const a = document.createElement('a');
    a.href = url;
    a.download = 'audit_log.csv';
    a.click();
  };

  const handleFilterChange = (key: keyof AuditFilters, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value }));
    setPage(1);
  };

  const handleResetFilters = () => {
    setFilters({ user: '', action: '', from: '', to: '' });
    setPage(1);
  };

  const totalPages = Math.ceil(total / perPage);

  const outcomeColor = (outcome: string) => {
    switch (outcome.toLowerCase()) {
      case 'success': return 'text-green-600 dark:text-green-400';
      case 'denied': case 'failure': case 'error': return 'text-md-error';
      case 'pending': return 'text-amber-600 dark:text-amber-400';
      default: return 'text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.audit')}
        </h2>
        <div className="flex gap-2">
          <button
            onClick={load}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors"
          >
            {loading ? t('audit.loading') : t('audit.reload')}
          </button>
          <button
            onClick={handleExport}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
          >
            {t('audit.export_csv')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Filter panel */}
      <div className="glass-card rounded-md-xl p-4 sm:p-5">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">
              {t('audit.filter.username')}
            </label>
            <input
              type="text"
              value={filters.user}
              onChange={(e) => handleFilterChange('user', e.target.value)}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary focus:ring-1 focus:ring-md-primary/20 outline-none text-md-on-surface"
              placeholder={t('audit.filter.username_placeholder')}
            />
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">
              {t('audit.filter.action')}
            </label>
            <select
              value={filters.action}
              onChange={(e) => handleFilterChange('action', e.target.value)}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface"
            >
              <option value="">{t('audit.filter.all')}</option>
              <option value="ssh.connect">ssh.connect</option>
              <option value="ssh.disconnect">ssh.disconnect</option>
              <option value="auth.login">auth.login</option>
              <option value="auth.logout">auth.logout</option>
              <option value="vault.unlock">vault.unlock</option>
              <option value="host.create">host.create</option>
              <option value="host.delete">host.delete</option>
              <option value="scan.run">scan.run</option>
            </select>
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">
              {t('audit.filter.from')}
            </label>
            <input
              type="date"
              value={filters.from}
              onChange={(e) => handleFilterChange('from', e.target.value)}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface"
            />
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">
              {t('audit.filter.to')}
            </label>
            <input
              type="date"
              value={filters.to}
              onChange={(e) => handleFilterChange('to', e.target.value)}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface"
            />
          </div>
          <div className="flex items-end">
            <button
              onClick={handleResetFilters}
              className="w-full text-sm px-3 py-2 rounded-md-sm border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors"
            >
              {t('audit.filter.reset')}
            </button>
          </div>
        </div>
      </div>

      {/* Table */}
      <div className="bg-md-surface-container-low rounded-md-xl overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('audit.col.timestamp')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('audit.col.user')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('audit.col.action')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('audit.col.resource')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('audit.col.outcome')}</th>
              </tr>
            </thead>
            <tbody>
              {logs.map((log) => (
                <tr key={log.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">
                    {new Date(log.created_at).toLocaleString()}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{log.user}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">{log.action}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">{log.resource}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small">
                    <span className={cn('font-medium', outcomeColor(log.outcome))}>{log.outcome}</span>
                  </td>
                </tr>
              ))}
              {!loading && logs.length === 0 && (
                <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('audit.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between">
          <span className="text-sm text-md-on-surface-variant">
            {t('audit.pagination.total').replace('{total}', String(total))}
          </span>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setPage(p => Math.max(1, p - 1))}
              disabled={page === 1}
              className="px-3 py-1.5 text-sm rounded-md-sm border border-md-outline disabled:opacity-50 hover:bg-md-surface-container-high transition-colors"
            >
              {t('audit.pagination.prev')}
            </button>
            <span className="px-3 py-1.5 text-sm text-md-on-surface-variant">
              {page} / {totalPages}
            </span>
            <button
              onClick={() => setPage(p => Math.min(totalPages, p + 1))}
              disabled={page >= totalPages}
              className="px-3 py-1.5 text-sm rounded-md-sm border border-md-outline disabled:opacity-50 hover:bg-md-surface-container-high transition-colors"
            >
              {t('audit.pagination.next')}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
