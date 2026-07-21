import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import type { Host, CreateHostInput, BatchExecuteResult } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useVaultStore } from '../stores/useVaultStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

const EMPTY_FORM: CreateHostInput = {
  name: '',
  address: '',
  port: 22,
  username: 'root',
  auth_method: 'key',
};

export function HostsPage() {
  const [hosts, setHosts] = useState<Host[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreateHostInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [deleting, setDeleting] = useState<string | null>(null);
  const { isUnlocked } = useVaultStore();
  const { token, canWrite } = useAuthStore();
  const navigate = useNavigate();
  const { t } = useI18n();

  // Batch selection state
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [showBatchDialog, setShowBatchDialog] = useState(false);
  const [batchCommand, setBatchCommand] = useState('');
  const [batchTimeout, setBatchTimeout] = useState(30);
  const [batchExecuting, setBatchExecuting] = useState(false);
  const [batchResults, setBatchResults] = useState<BatchExecuteResult[] | null>(null);
  const [expandedResults, setExpandedResults] = useState<Set<string>>(new Set());

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const list = await api.listHosts(token);
      setHosts(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load hosts');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.createHost(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create host');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm(t('hosts.delete_confirm'))) return;
    setDeleting(id);
    setError(null);
    try {
      await api.deleteHost(token!, id);
      setSelected(prev => { const n = new Set(prev); n.delete(id); return n; });
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete host');
    } finally {
      setDeleting(null);
    }
  };

  // Selection handlers
  const toggleSelect = (id: string) => {
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selected.size === hosts.length) {
      setSelected(new Set());
    } else {
      setSelected(new Set(hosts.map(h => h.id)));
    }
  };

  const clearSelection = () => setSelected(new Set());

  // Batch operations
  const handleBatchExecute = async () => {
    if (!token || selected.size === 0 || !batchCommand.trim()) return;
    setBatchExecuting(true);
    setError(null);
    try {
      const resp = await api.batchExecute(token, {
        host_ids: Array.from(selected),
        command: batchCommand,
        timeout: batchTimeout,
      });
      setBatchResults(resp.results);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to execute batch command');
    } finally {
      setBatchExecuting(false);
    }
  };

  const handleBatchDelete = async () => {
    if (!token || selected.size === 0) return;
    if (!window.confirm(t('hosts.batch_delete_confirm').replace('{count}', String(selected.size)))) return;
    try {
      for (const id of selected) {
        await api.deleteHost(token, id);
      }
      setSelected(new Set());
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete hosts');
    }
  };

  const toggleResultExpand = (hostId: string) => {
    setExpandedResults(prev => {
      const next = new Set(prev);
      if (next.has(hostId)) next.delete(hostId);
      else next.add(hostId);
      return next;
    });
  };

  const statusColor = (status: Host['status']) => {
    switch (status) {
      case 'online': return 'bg-green-500';
      case 'offline': return 'bg-md-error';
      case 'maintenance': return 'bg-amber-500';
      default: return 'bg-md-outline';
    }
  };

  const closeBatchDialog = () => {
    setShowBatchDialog(false);
    setBatchCommand('');
    setBatchResults(null);
    setExpandedResults(new Set());
  };


  if (loading) return <LoadingState skeleton="list" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">{t('hosts.title')}</h2>
        <div className="flex gap-2">
          <button
            onClick={load}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50"
          >
            {loading ? t('hosts.loading') : t('hosts.reload')}
          </button>
          {canWrite() && (
            <button
              onClick={() => setShowForm(!showForm)}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
            >
              {showForm ? t('hosts.cancel') : t('hosts.add')}
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {!isUnlocked && (
        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-300 dark:border-amber-700 rounded-md-sm px-4 py-3 text-body-medium text-amber-800 dark:text-amber-200">
          {t('hosts.vault_locked')}
        </div>
      )}

      {/* Batch Actions Bar */}
      {selected.size > 0 && (
        <div className="glass-card rounded-md-xl px-4 py-3 flex items-center justify-between animate-slide-up">
          <span className="text-body-medium text-md-on-surface">
            {t('hosts.selected_count').replace('{count}', String(selected.size))}
          </span>
          <div className="flex gap-2">
            {canWrite() && (
              <>
                <button
                  onClick={() => setShowBatchDialog(true)}
                  className="text-sm px-3 py-1.5 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-2 transition-all"
                >
                  {t('hosts.batch_execute')}
                </button>
                <button
                  onClick={handleBatchDelete}
                  className="text-sm px-3 py-1.5 rounded-md-full bg-md-error text-md-on-error hover:shadow-md-2 transition-all"
                >
                  {t('hosts.batch_delete')}
                </button>
              </>
            )}
            <button
              onClick={clearSelection}
              className="text-sm px-3 py-1.5 rounded-md-full border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors"
            >
              {t('hosts.clear_selection')}
            </button>
          </div>
        </div>
      )}

      {/* Create Form */}
      {showForm && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="web-server-1" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.address')}</label>
              <input type="text" required value={form.address} onChange={(e) => setForm({ ...form, address: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="192.168.1.10" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.port')}</label>
              <input type="number" value={form.port ?? ''} onChange={(e) => setForm({ ...form, port: e.target.value ? Number(e.target.value) : undefined })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="22" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.username')}</label>
              <input type="text" required value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="root" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.auth_method')}</label>
              <select value={form.auth_method} onChange={(e) => setForm({ ...form, auth_method: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface">
                <option value="key">SSH Key</option>
                <option value="password">Password</option>
              </select>
            </div>
          </div>
          <div className="flex justify-end">
            <button type="submit" disabled={submitting}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {submitting ? t('hosts.creating') : t('hosts.create_btn')}
            </button>
          </div>
        </form>
      )}

      {/* Hosts Table */}
      <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-md-outline-variant">
                {canWrite() && (
                  <th className="px-4 py-3 text-left">
                    <input
                      type="checkbox"
                      checked={selected.size === hosts.length && hosts.length > 0}
                      onChange={toggleSelectAll}
                      className="w-4 h-4 rounded border-md-outline text-md-primary focus:ring-md-primary"
                    />
                  </th>
                )}
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('hosts.name')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('hosts.address')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('hosts.port')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('hosts.status')}</th>
                <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('hosts.auth')}</th>
                <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('hosts.actions')}</th>
              </tr>
            </thead>
            <tbody>
              {hosts.map((h) => (
                <tr key={h.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                  {canWrite() && (
                    <td className="px-4 py-3">
                      <input
                        type="checkbox"
                        checked={selected.has(h.id)}
                        onChange={() => toggleSelect(h.id)}
                        className="w-4 h-4 rounded border-md-outline text-md-primary focus:ring-md-primary"
                      />
                    </td>
                  )}
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{h.name}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{h.address}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{h.port}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium">
                    <span className="inline-flex items-center gap-1.5">
                      <span className={`h-2 w-2 rounded-full ${statusColor(h.status)}`} />
                      {h.status}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{h.auth_method}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-right">
                    <div className="flex items-center justify-end gap-2">
                      <button
                        onClick={() => navigate(`/terminal/${h.id}`)}
                        className="text-md-primary rounded-md-sm px-2.5 py-1 text-label-large hover:bg-md-primary-container/30 transition-colors"
                      >
                        {t('terminal.ssh')}
                      </button>
                      {canWrite() && (
                        <button onClick={() => handleDelete(h.id)} disabled={deleting === h.id}
                          className="text-md-error rounded-md-sm px-2.5 py-1 text-label-large hover:bg-md-error-container/30 disabled:opacity-50 transition-colors">
                          {deleting === h.id ? t('hosts.deleting') : t('hosts.delete')}
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
              {!loading && hosts.length === 0 && (
                <tr><td colSpan={canWrite() ? 7 : 6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('hosts.empty')}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Batch Execute Dialog */}
      {showBatchDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={closeBatchDialog}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-3xl max-h-[80vh] overflow-auto shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-5">
              <h2 className="text-title-large font-semibold text-md-on-surface">
                {t('hosts.batch_execute_title')}
              </h2>
              <button onClick={closeBatchDialog} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {!batchResults ? (
              <div className="space-y-4">
                <div>
                  <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.batch_target')}</label>
                  <div className="flex flex-wrap gap-2">
                    {Array.from(selected).map(id => {
                      const host = hosts.find(h => h.id === id);
                      return host ? (
                        <span key={id} className="inline-flex items-center gap-1 px-2 py-1 rounded-md-sm bg-md-primary-container/30 text-sm text-md-on-primary-container">
                          {host.name}
                        </span>
                      ) : null;
                    })}
                  </div>
                </div>

                <div>
                  <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.batch_command')}</label>
                  <textarea
                    value={batchCommand}
                    onChange={(e) => setBatchCommand(e.target.value)}
                    rows={3}
                    className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface font-mono text-sm"
                    placeholder="uptime"
                  />
                </div>

                <div>
                  <label className="block text-label-large text-md-on-surface mb-1">{t('hosts.batch_timeout')}</label>
                  <input
                    type="number"
                    value={batchTimeout}
                    onChange={(e) => setBatchTimeout(Number(e.target.value))}
                    min={5}
                    max={300}
                    className="w-32 bg-md-surface-container-highest rounded-md-sm px-4 py-2 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface"
                  />
                  <span className="text-sm text-md-on-surface-variant ml-2">seconds</span>
                </div>

                <div className="flex justify-end gap-2 pt-2">
                  <button onClick={closeBatchDialog}
                    className="px-4 py-2 text-sm font-medium rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
                    {t('hosts.cancel')}
                  </button>
                  <button
                    onClick={handleBatchExecute}
                    disabled={batchExecuting || !batchCommand.trim()}
                    className="px-4 py-2 text-sm font-medium rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all"
                  >
                    {batchExecuting ? t('hosts.batch_executing') : t('hosts.batch_execute_btn')}
                  </button>
                </div>
              </div>
            ) : (
              /* Results View */
              <div className="space-y-3">
                <div className="flex items-center gap-4 text-sm">
                  <span className="text-green-600">{batchResults.filter(r => r.success).length} {t('hosts.batch_succeeded')}</span>
                  <span className="text-md-error">{batchResults.filter(r => !r.success).length} {t('hosts.batch_failed')}</span>
                </div>

                <div className="space-y-2">
                  {batchResults.map((result) => (
                    <div key={result.host_id} className="glass-card rounded-md-lg overflow-hidden">
                      <button
                        onClick={() => toggleResultExpand(result.host_id)}
                        className="w-full flex items-center justify-between px-4 py-3 text-left hover:bg-md-surface-container-high/50 transition-colors"
                      >
                        <div className="flex items-center gap-3">
                          <span className={cn('h-2 w-2 rounded-full', result.success ? 'bg-green-500' : 'bg-md-error')} />
                          <span className="text-body-medium font-medium text-md-on-surface">{result.host_name}</span>
                          <span className="text-body-small text-md-on-surface-variant font-mono">
                            {t('hosts.batch_exit_code')}: {result.exit_code}
                          </span>
                          <span className="text-body-small text-md-on-surface-variant">
                            {result.duration_ms}ms
                          </span>
                        </div>
                        <svg className={cn('w-4 h-4 text-md-on-surface-variant transition-transform', expandedResults.has(result.host_id) && 'rotate-180')} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                          <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
                        </svg>
                      </button>
                      {expandedResults.has(result.host_id) && (
                        <div className="px-4 pb-3 space-y-2 border-t border-md-outline-variant">
                          {result.stdout && (
                            <div>
                              <p className="text-label-small text-md-on-surface-variant mb-1">{t('hosts.batch_stdout')}</p>
                              <pre className="bg-md-surface-container-highest rounded-md-sm p-2 text-body-small font-mono overflow-auto max-h-40">{result.stdout}</pre>
                            </div>
                          )}
                          {result.stderr && (
                            <div>
                              <p className="text-label-small text-md-error mb-1">{t('hosts.batch_stderr')}</p>
                              <pre className="bg-md-error-container/20 rounded-md-sm p-2 text-body-small font-mono overflow-auto max-h-40">{result.stderr}</pre>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  ))}
                </div>

                <div className="flex justify-end pt-2">
                  <button onClick={closeBatchDialog}
                    className="px-4 py-2 text-sm font-medium rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
                    {t('hosts.close')}
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
