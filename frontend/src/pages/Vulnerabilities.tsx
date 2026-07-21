import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Vulnerability, VulnerabilityStats, CreateVulnerabilityInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

const EMPTY_FORM: CreateVulnerabilityInput = {
  cve_id: '',
  title: '',
  description: '',
  severity: 'medium',
  cvss_score: 5.0,
  affected_host: '',
  affected_service: '',
  notes: '',
};

export function VulnerabilitiesPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [vulns, setVulns] = useState<Vulnerability[]>([]);
  const [stats, setStats] = useState<VulnerabilityStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedVuln, setSelectedVuln] = useState<Vulnerability | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateVulnerabilityInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [severityFilter, setSeverityFilter] = useState('');
  const [statusFilter, setStatusFilter] = useState('');
  const [scanning, setScanning] = useState(false);

  const loadVulns = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const params: Record<string, string> = {};
      if (severityFilter) params.severity = severityFilter;
      if (statusFilter) params.status = statusFilter;
      const data = await api.listVulnerabilities(token, params);
      setVulns(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load vulnerabilities');
    } finally {
      setLoading(false);
    }
  }, [token, severityFilter, statusFilter]);

  const loadStats = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getVulnerabilityStats(token);
      setStats(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load stats');
    }
  }, [token]);

  useEffect(() => { loadVulns(); loadStats(); }, [loadVulns, loadStats]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createVulnerability(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await loadVulns();
      await loadStats();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create vulnerability');
    } finally {
      setSubmitting(false);
    }
  };

  const handleScan = async () => {
    setScanning(true);
    try {
      await api.scanVulnerabilities(token!);
      await loadVulns();
      await loadStats();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to scan');
    } finally {
      setScanning(false);
    }
  };

  const handleUpdateStatus = async (id: string, status: string) => {
    try {
      await api.updateVulnerability(token!, id, { status });
      setSelectedVuln(null);
      await loadVulns();
      await loadStats();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to update');
    }
  };

  const handleVerify = async (id: string) => {
    try {
      await api.verifyVulnerability(token!, id);
      setSelectedVuln(null);
      await loadVulns();
      await loadStats();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to verify');
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

  const statusColor = (status: string) => {
    switch (status) {
      case 'open': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'in_progress': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'fixed': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'verified': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.vulnerabilities')}
        </h2>
        <div className="flex gap-2">
          <button onClick={handleScan} disabled={scanning}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors flex items-center gap-2">
            {scanning ? (
              <div className="h-4 w-4 border-2 border-md-primary border-t-transparent rounded-full animate-spin" />
            ) : '🔍'}
            {scanning ? t('vulns.scanning') : t('vulns.scan')}
          </button>
          <button onClick={() => setShowForm(!showForm)}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
            {showForm ? t('vulns.cancel') : t('vulns.add')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium flex items-center justify-between">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-sm underline">{t('vulns.dismiss')}</button>
        </div>
      )}

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-2 sm:grid-cols-5 gap-4">
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-md-primary">{stats.total}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('vulns.stats.total')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-red-500">{stats.critical}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('vulns.stats.critical')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-orange-500">{stats.high}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('vulns.stats.high')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-amber-500">{stats.medium}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('vulns.stats.medium')}</p>
          </div>
          <div className="glass-card rounded-md-xl p-4 text-center">
            <p className="text-headline-medium font-bold text-blue-500">{stats.low}</p>
            <p className="text-label-small text-md-on-surface-variant">{t('vulns.stats.low')}</p>
          </div>
        </div>
      )}

      {/* Create Form */}
      {showForm && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('vulns.cve_id')}</label>
              <input type="text" required value={form.cve_id} onChange={(e) => setForm({ ...form, cve_id: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                placeholder="CVE-2026-0001" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('vulns.title')}</label>
              <input type="text" required value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('vulns.severity')}</label>
              <select value={form.severity} onChange={(e) => setForm({ ...form, severity: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="critical">{t('vulns.sev_critical')}</option>
                <option value="high">{t('vulns.sev_high')}</option>
                <option value="medium">{t('vulns.sev_medium')}</option>
                <option value="low">{t('vulns.sev_low')}</option>
              </select>
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('vulns.cvss_score')}</label>
              <input type="number" step="0.1" min="0" max="10" value={form.cvss_score} onChange={(e) => setForm({ ...form, cvss_score: Number(e.target.value) })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('vulns.affected_host')}</label>
              <input type="text" value={form.affected_host} onChange={(e) => setForm({ ...form, affected_host: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('vulns.affected_service')}</label>
              <input type="text" value={form.affected_service} onChange={(e) => setForm({ ...form, affected_service: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('vulns.description')}</label>
            <textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })}
              rows={3}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowForm(false)}
              className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('vulns.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">
              {submitting ? t('vulns.creating') : t('vulns.create')}
            </button>
          </div>
        </form>
      )}

      {/* Filters */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-center gap-3">
          <label className="text-label-medium text-md-on-surface-variant">{t('vulns.filter.severity')}</label>
          <div className="flex gap-2">
            {['', 'critical', 'high', 'medium', 'low'].map((sev) => (
              <button key={sev} onClick={() => setSeverityFilter(sev)}
                className={cn('px-3 py-1.5 text-sm rounded-md-full transition-colors',
                  severityFilter === sev ? 'bg-md-primary text-md-on-primary' : 'bg-md-surface-container-high text-md-on-surface-variant'
                )}>
                {sev || t('vulns.filter.all')}
              </button>
            ))}
          </div>
          <label className="text-label-medium text-md-on-surface-variant ml-2">{t('vulns.filter.status')}</label>
          <div className="flex gap-2">
            {['', 'open', 'in_progress', 'fixed', 'verified'].map((st) => (
              <button key={st} onClick={() => setStatusFilter(st)}
                className={cn('px-3 py-1.5 text-sm rounded-md-full transition-colors',
                  statusFilter === st ? 'bg-md-primary text-md-on-primary' : 'bg-md-surface-container-high text-md-on-surface-variant'
                )}>
                {st || t('vulns.filter.all')}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Vulns Table */}
      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('vulns.col.cve')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('vulns.col.title')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('vulns.col.severity')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('vulns.col.cvss')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('vulns.col.host')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('vulns.col.status')}</th>
              </tr>
            </thead>
            <tbody>
              {vulns.map((vuln) => (
                <tr key={vuln.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors cursor-pointer"
                    onClick={() => setSelectedVuln(vuln)}>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small font-mono text-md-on-surface">{vuln.cve_id}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{vuln.title}</td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(vuln.severity))}>{vuln.severity}</span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">{vuln.cvss_score}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{vuln.affected_host || '-'}</td>
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(vuln.status))}>{vuln.status}</span>
                  </td>
                </tr>
              ))}
              {!loading && vulns.length === 0 && (
                <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('vulns.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Detail Modal */}
      {selectedVuln && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setSelectedVuln(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-lg max-h-[80vh] overflow-auto shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-3">
                <h3 className="text-title-large font-semibold text-md-on-surface">{selectedVuln.cve_id}</h3>
                <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', severityColor(selectedVuln.severity))}>{selectedVuln.severity}</span>
              </div>
              <button onClick={() => setSelectedVuln(null)} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <h4 className="text-body-large font-medium text-md-on-surface mb-2">{selectedVuln.title}</h4>
            <p className="text-body-medium text-md-on-surface-variant mb-4">{selectedVuln.description || t('vulns.no_description')}</p>

            <div className="grid grid-cols-2 gap-3 text-sm mb-4">
              <div><span className="text-label-small text-md-on-surface-variant">{t('vulns.col.cvss')}:</span> <span className="font-medium">{selectedVuln.cvss_score}</span></div>
              <div><span className="text-label-small text-md-on-surface-variant">{t('vulns.col.host')}:</span> <span className="font-medium">{selectedVuln.affected_host || '-'}</span></div>
              <div><span className="text-label-small text-md-on-surface-variant">{t('vulns.col.status')}:</span> <span className={cn('font-medium', statusColor(selectedVuln.status).split(' ')[1])}>{selectedVuln.status}</span></div>
              <div><span className="text-label-small text-md-on-surface-variant">{t('vulns.discovered')}:</span> <span className="font-medium">{new Date(selectedVuln.discovered_at).toLocaleDateString()}</span></div>
            </div>

            <div className="flex gap-2">
              {selectedVuln.status === 'open' && (
                <button onClick={() => handleUpdateStatus(selectedVuln.id, 'in_progress')}
                  className="px-3 py-1.5 text-sm rounded-md-full bg-amber-500/10 text-amber-600 hover:bg-amber-500/20 transition-colors">
                  {t('vulns.start_fix')}
                </button>
              )}
              {selectedVuln.status === 'in_progress' && (
                <button onClick={() => handleUpdateStatus(selectedVuln.id, 'fixed')}
                  className="px-3 py-1.5 text-sm rounded-md-full bg-green-500/10 text-green-600 hover:bg-green-500/20 transition-colors">
                  {t('vulns.mark_fixed')}
                </button>
              )}
              {selectedVuln.status === 'fixed' && (
                <button onClick={() => handleVerify(selectedVuln.id)}
                  className="px-3 py-1.5 text-sm rounded-md-full bg-blue-500/10 text-blue-600 hover:bg-blue-500/20 transition-colors">
                  {t('vulns.verify')}
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
