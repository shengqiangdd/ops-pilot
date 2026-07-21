import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Playbook, Execution, ExecutionDetail, CreatePlaybookInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

const EMPTY_FORM: CreatePlaybookInput = {
  name: '',
  description: '',
  trigger_type: 'manual',
  steps_json: '[{"type":"notify","message":"Test notification"}]',
};

export function SOARPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [playbooks, setPlaybooks] = useState<Playbook[]>([]);
  const [executions, setExecutions] = useState<Execution[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<CreatePlaybookInput>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [selectedExecution, setSelectedExecution] = useState<ExecutionDetail | null>(null);
  const [activeTab, setActiveTab] = useState<'playbooks' | 'executions'>('playbooks');

  const loadPlaybooks = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listPlaybooks(token);
      setPlaybooks(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load playbooks');
    } finally {
      setLoading(false);
    }
  }, [token]);

  const loadExecutions = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.listSoarExecutions(token);
      setExecutions(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load executions');
    }
  }, [token]);

  useEffect(() => {
    if (activeTab === 'playbooks') loadPlaybooks();
    else loadExecutions();
  }, [activeTab, loadPlaybooks, loadExecutions]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createPlaybook(token!, form);
      setForm(EMPTY_FORM);
      setShowForm(false);
      await loadPlaybooks();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create playbook');
    } finally {
      setSubmitting(false);
    }
  };

  const handleExecute = async (playbookId: string) => {
    try {
      await api.executePlaybook(token!, playbookId);
      await loadExecutions();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to execute playbook');
    }
  };

  const handleViewExecution = async (execId: string) => {
    try {
      const detail = await api.getSoarExecution(token!, execId);
      setSelectedExecution(detail);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load execution');
    }
  };

  const handleDeletePlaybook = async (id: string) => {
    if (!window.confirm(t('soar.delete_confirm'))) return;
    try {
      await api.deletePlaybook(token!, id);
      await loadPlaybooks();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete playbook');
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'running': case 'pending': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      case 'failed': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.soar')}
        </h2>
        <div className="flex gap-2">
          <button onClick={() => activeTab === 'playbooks' ? loadPlaybooks() : loadExecutions()}
            disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? t('soar.loading') : t('soar.reload')}
          </button>
          {activeTab === 'playbooks' && (
            <button onClick={() => setShowForm(!showForm)}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
              {showForm ? t('soar.cancel') : t('soar.add')}
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 border-b border-md-outline-variant pb-2">
        <button onClick={() => setActiveTab('playbooks')}
          className={cn('px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
            activeTab === 'playbooks' ? 'bg-md-primary text-md-on-primary' : 'text-md-on-surface-variant hover:bg-md-surface-container-high')}>
          {t('soar.tab.playbooks')}
        </button>
        <button onClick={() => setActiveTab('executions')}
          className={cn('px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
            activeTab === 'executions' ? 'bg-md-primary text-md-on-primary' : 'text-md-on-surface-variant hover:bg-md-surface-container-high')}>
          {t('soar.tab.executions')}
        </button>
      </div>

      {/* Create Form */}
      {showForm && activeTab === 'playbooks' && (
        <form onSubmit={handleSubmit} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('soar.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('soar.trigger_type')}</label>
              <select value={form.trigger_type} onChange={(e) => setForm({ ...form, trigger_type: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface">
                <option value="manual">{t('soar.trigger_manual')}</option>
                <option value="alert">{t('soar.trigger_alert')}</option>
                <option value="incident">{t('soar.trigger_incident')}</option>
                <option value="vulnerability">{t('soar.trigger_vulnerability')}</option>
              </select>
            </div>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('soar.description')}</label>
            <input type="text" value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('soar.steps')}</label>
            <textarea value={form.steps_json} onChange={(e) => setForm({ ...form, steps_json: e.target.value })}
              rows={4}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface font-mono text-sm" />
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowForm(false)}
              className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('soar.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">
              {submitting ? t('soar.creating') : t('soar.create')}
            </button>
          </div>
        </form>
      )}

      {/* Playbooks List */}
      {activeTab === 'playbooks' && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {playbooks.map((pb) => (
            <div key={pb.id} className="glass-card rounded-md-xl p-4">
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  <span className={cn(
                    'text-xs font-medium px-2 py-0.5 rounded-md-sm',
                    pb.trigger_type === 'manual' ? 'bg-blue-500/10 text-blue-600' :
                    pb.trigger_type === 'alert' ? 'bg-amber-500/10 text-amber-600' :
                    pb.trigger_type === 'incident' ? 'bg-red-500/10 text-red-600' :
                    'bg-purple-500/10 text-purple-600',
                  )}>
                    {t(`soar.trigger_${pb.trigger_type}`)}
                  </span>
                  <span className={cn('h-2 w-2 rounded-full', pb.enabled ? 'bg-green-500' : 'bg-md-outline')} />
                </div>
                <div className="flex gap-1">
                  <button onClick={() => handleExecute(pb.id)}
                    className="text-xs px-2 py-1 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-1 transition-all">
                    {t('soar.execute')}
                  </button>
                  <button onClick={() => handleDeletePlaybook(pb.id)}
                    className="text-xs px-2 py-1 rounded-md-full text-md-error hover:bg-md-error-container/30 transition-colors">
                    ×
                  </button>
                </div>
              </div>
              <h3 className="text-body-medium font-medium text-md-on-surface mb-1">{pb.name}</h3>
              <p className="text-body-small text-md-on-surface-variant line-clamp-2">{pb.description || t('soar.no_description')}</p>
            </div>
          ))}
          {!loading && playbooks.length === 0 && (
            <div className="col-span-full glass-card rounded-md-xl p-8 text-center">
              <div className="text-4xl mb-3">📋</div>
              <p className="text-body-medium text-md-on-surface-variant">{t('soar.no_playbooks')}</p>
            </div>
          )}
        </div>
      )}

      {/* Executions List */}
      {activeTab === 'executions' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('soar.exec_col.playbook')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('soar.exec_col.trigger')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('soar.exec_col.status')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('soar.exec_col.time')}</th>
                  <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('soar.exec_col.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {executions.map((exec) => (
                  <tr key={exec.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface font-mono">{exec.playbook_id}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{exec.trigger_source}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(exec.status))}>{exec.status}</span>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{exec.started_at || '-'}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-right">
                      <button onClick={() => handleViewExecution(exec.id)}
                        className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-2 py-1 rounded-md-sm transition-colors">
                        {t('soar.view')}
                      </button>
                    </td>
                  </tr>
                ))}
                {executions.length === 0 && (
                  <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('soar.no_executions')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Execution Detail Modal */}
      {selectedExecution && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setSelectedExecution(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-lg max-h-[80vh] overflow-auto shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-title-large font-semibold text-md-on-surface">{t('soar.execution_detail')}</h3>
              <button onClick={() => setSelectedExecution(null)} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="flex items-center gap-3 mb-4">
              <span className={cn('text-sm font-medium px-3 py-1 rounded-full', statusColor(selectedExecution.execution.status))}>
                {selectedExecution.execution.status}
              </span>
              <span className="text-sm text-md-on-surface-variant">{selectedExecution.playbook_name}</span>
            </div>

            <div className="space-y-2">
              {selectedExecution.steps.map((step, i) => (
                <div key={i} className="flex items-center gap-3 px-3 py-2 rounded-md-lg bg-md-surface-container-highest/50">
                  <span className={cn(
                    'h-5 w-5 rounded-full flex items-center justify-center text-xs shrink-0',
                    step.status === 'completed' ? 'bg-green-500/20 text-green-600' :
                    step.status === 'failed' ? 'bg-red-500/20 text-red-600' :
                    'bg-blue-500/20 text-blue-600',
                  )}>
                    {step.status === 'completed' ? '✓' : step.status === 'failed' ? '✗' : '○'}
                  </span>
                  <div className="flex-1">
                    <span className="text-body-small font-medium text-md-on-surface">{step.step_type}</span>
                    <p className="text-label-small text-md-on-surface-variant">{step.message}</p>
                  </div>
                  {step.duration_ms && (
                    <span className="text-label-small text-md-on-surface-variant">{step.duration_ms}ms</span>
                  )}
                </div>
              ))}
              {selectedExecution.steps.length === 0 && (
                <p className="text-body-medium text-md-on-surface-variant text-center py-4">{t('soar.no_steps')}</p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
