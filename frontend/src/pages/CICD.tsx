import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { PipelineTemplate, PipelineRun, PipelineRunDetail, Deployment, CreatePipelineTemplateInput } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

type Tab = 'templates' | 'runs' | 'deployments';

const EMPTY_TEMPLATE: CreatePipelineTemplateInput = {
  name: '',
  description: '',
  stages_json: '["build","test","deploy"]',
};

export function CICDPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [activeTab, setActiveTab] = useState<Tab>('runs');
  const [templates, setTemplates] = useState<PipelineTemplate[]>([]);
  const [runs, setRuns] = useState<PipelineRun[]>([]);
  const [deployments, setDeployments] = useState<Deployment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showTemplateForm, setShowTemplateForm] = useState(false);
  const [templateForm, setTemplateForm] = useState<CreatePipelineTemplateInput>(EMPTY_TEMPLATE);
  const [submitting, setSubmitting] = useState(false);
  const [selectedRun, setSelectedRun] = useState<PipelineRunDetail | null>(null);

  const loadTemplates = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.listCICDTemplates(token);
      setTemplates(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load templates');
    }
  }, [token]);

  const loadRuns = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listCICDRuns(token);
      setRuns(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load runs');
    } finally {
      setLoading(false);
    }
  }, [token]);

  const loadDeployments = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listCICDDeployments(token);
      setDeployments(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load deployments');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    if (activeTab === 'templates') loadTemplates();
    else if (activeTab === 'runs') loadRuns();
    else loadDeployments();
  }, [activeTab, loadTemplates, loadRuns, loadDeployments]);

  const handleCreateTemplate = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createCICDTemplate(token!, templateForm);
      setTemplateForm(EMPTY_TEMPLATE);
      setShowTemplateForm(false);
      await loadTemplates();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create template');
    } finally {
      setSubmitting(false);
    }
  };

  const handleTriggerRun = async (templateId: string) => {
    try {
      await api.createCICDRun(token!, { template_id: templateId });
      await loadRuns();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to trigger run');
    }
  };

  const handleViewRun = async (runId: string) => {
    try {
      const detail = await api.getCICDRunDetail(token!, runId);
      setSelectedRun(detail);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load run detail');
    }
  };

  const handleCancelRun = async (runId: string) => {
    try {
      await api.cancelCICDRun(token!, runId);
      await loadRuns();
      if (selectedRun?.run.id === runId) {
        setSelectedRun(null);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to cancel run');
    }
  };

  const handleRollback = async (deploymentId: string) => {
    try {
      await api.rollbackCICDDeployment(token!, deploymentId);
      await loadDeployments();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to rollback');
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'success': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'running': case 'pending': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      case 'failed': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'cancelled': return 'bg-md-surface-container-high text-md-on-surface-variant';
      case 'rollback': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  const envColor = (env: string) => {
    switch (env) {
      case 'production': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'staging': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400';
      case 'dev': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.cicd')}
        </h2>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 border-b border-md-outline-variant pb-2">
        {(['templates', 'runs', 'deployments'] as Tab[]).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={cn(
              'px-4 py-2 text-sm font-medium rounded-md-lg transition-colors',
              activeTab === tab
                ? 'bg-md-primary text-md-on-primary'
                : 'text-md-on-surface-variant hover:bg-md-surface-container-high',
            )}
          >
            {t(`cicd.tab.${tab}`)}
          </button>
        ))}
      </div>

      {/* Templates Tab */}
      {activeTab === 'templates' && (
        <div className="space-y-4">
          <div className="flex justify-end">
            <button
              onClick={() => setShowTemplateForm(!showTemplateForm)}
              className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 transition-all"
            >
              {showTemplateForm ? t('cicd.cancel') : t('cicd.add_template')}
            </button>
          </div>

          {showTemplateForm && (
            <form onSubmit={handleCreateTemplate} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="block text-label-large text-md-on-surface mb-1">{t('cicd.name')}</label>
                  <input type="text" required value={templateForm.name} onChange={(e) => setTemplateForm({ ...templateForm, name: e.target.value })}
                    className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
                </div>
                <div>
                  <label className="block text-label-large text-md-on-surface mb-1">{t('cicd.description')}</label>
                  <input type="text" value={templateForm.description} onChange={(e) => setTemplateForm({ ...templateForm, description: e.target.value })}
                    className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
                </div>
              </div>
              <div>
                <label className="block text-label-large text-md-on-surface mb-1">{t('cicd.stages')}</label>
                <textarea value={templateForm.stages_json} onChange={(e) => setTemplateForm({ ...templateForm, stages_json: e.target.value })}
                  rows={3}
                  className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface font-mono text-sm" />
              </div>
              <div className="flex justify-end gap-2">
                <button type="button" onClick={() => setShowTemplateForm(false)}
                  className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
                  {t('cicd.cancel')}
                </button>
                <button type="submit" disabled={submitting}
                  className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">
                  {submitting ? t('cicd.creating') : t('cicd.create')}
                </button>
              </div>
            </form>
          )}

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {templates.map((tpl) => (
              <div key={tpl.id} className="glass-card rounded-md-xl p-4">
                <div className="flex items-start justify-between mb-2">
                  <h3 className="text-title-small font-semibold text-md-on-surface">{tpl.name}</h3>
                  <button onClick={() => handleTriggerRun(tpl.id)}
                    className="text-xs px-2 py-1 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-1 transition-all">
                    {t('cicd.run')}
                  </button>
                </div>
                <p className="text-body-small text-md-on-surface-variant mb-2">{tpl.description || t('cicd.no_description')}</p>
                <div className="flex flex-wrap gap-1">
                  {JSON.parse(tpl.stages_json || '[]').map((stage: string, i: number) => (
                    <span key={i} className="text-xs px-2 py-0.5 rounded-md-sm bg-md-surface-container-high text-md-on-surface-variant">
                      {stage}
                    </span>
                  ))}
                </div>
              </div>
            ))}
            {!loading && templates.length === 0 && (
              <div className="col-span-full text-center py-8 text-body-medium text-md-on-surface-variant">{t('cicd.no_templates')}</div>
            )}
          </div>
        </div>
      )}

      {/* Runs Tab */}
      {activeTab === 'runs' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.run_col.name')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.run_col.status')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.run_col.branch')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.run_col.time')}</th>
                  <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('cicd.run_col.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {runs.map((run) => (
                  <tr key={run.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{run.name}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(run.status))}>{run.status}</span>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">{run.branch}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">
                      {new Date(run.created_at).toLocaleString()}
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <button onClick={() => handleViewRun(run.id)}
                          className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-2 py-1 rounded-md-sm transition-colors">
                          {t('cicd.view')}
                        </button>
                        {(run.status === 'pending' || run.status === 'running') && (
                          <button onClick={() => handleCancelRun(run.id)}
                            className="text-md-error text-label-large hover:bg-md-error-container/30 px-2 py-1 rounded-md-sm transition-colors">
                            {t('cicd.cancel_run')}
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
                {!loading && runs.length === 0 && (
                  <tr><td colSpan={5} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('cicd.no_runs')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Deployments Tab */}
      {activeTab === 'deployments' && (
        <div className="bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.deploy_col.name')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.deploy_col.env')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.deploy_col.strategy')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.deploy_col.version')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('cicd.deploy_col.status')}</th>
                  <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('cicd.deploy_col.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {deployments.map((dep) => (
                  <tr key={dep.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{dep.name}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', envColor(dep.environment))}>{dep.environment}</span>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">{dep.strategy}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant font-mono">{dep.version}</td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(dep.status))}>{dep.status}</span>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-right">
                      <button onClick={() => handleRollback(dep.id)}
                        className="text-amber-600 text-label-large hover:bg-amber-500/10 px-2 py-1 rounded-md-sm transition-colors">
                        {t('cicd.rollback')}
                      </button>
                    </td>
                  </tr>
                ))}
                {!loading && deployments.length === 0 && (
                  <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('cicd.no_deployments')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Run Detail Modal */}
      {selectedRun && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setSelectedRun(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-2xl max-h-[80vh] overflow-auto shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-title-large font-semibold text-md-on-surface">{selectedRun.run.name}</h3>
              <button onClick={() => setSelectedRun(null)} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="flex items-center gap-3 mb-4">
              <span className={cn('text-sm font-medium px-3 py-1 rounded-full', statusColor(selectedRun.run.status))}>{selectedRun.run.status}</span>
              <span className="text-sm text-md-on-surface-variant">{selectedRun.run.branch}</span>
              {selectedRun.run.duration_ms && (
                <span className="text-sm text-md-on-surface-variant">{selectedRun.run.duration_ms}ms</span>
              )}
            </div>

            <div className="space-y-3">
              {selectedRun.stages.map((stage) => (
                <div key={stage.id} className="glass-card rounded-md-lg p-3">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-body-medium font-medium text-md-on-surface">{stage.stage_name}</span>
                    <span className={cn('text-xs font-medium px-2 py-0.5 rounded-md-sm', statusColor(stage.status))}>{stage.status}</span>
                  </div>
                  {stage.log && (
                    <pre className="bg-md-surface-container-highest rounded-md-sm p-2 text-body-small font-mono overflow-auto max-h-32">{stage.log}</pre>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
