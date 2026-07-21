import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Job, JobRun, JobRunDetail } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

export function JobsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [jobs, setJobs] = useState<Job[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedJob, setSelectedJob] = useState<Job | null>(null);
  const [runs, setRuns] = useState<JobRun[]>([]);
  const [selectedRun, setSelectedRun] = useState<JobRunDetail | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [form, setForm] = useState({ name: '', description: '', steps_json: '[{"type":"ssh_command","command":"echo hello"}]' });
  const [submitting, setSubmitting] = useState(false);

  const loadJobs = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listJobs(token);
      setJobs(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load jobs');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { loadJobs(); }, [loadJobs]);

  const loadRuns = async (jobId: string) => {
    if (!token) return;
    try {
      const data = await api.listJobRuns(token, jobId);
      setRuns(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load runs');
    }
  };

  const handleCreateJob = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createJob(token!, form);
      setForm({ name: '', description: '', steps_json: '[{"type":"ssh_command","command":"echo hello"}]' });
      setShowCreateForm(false);
      await loadJobs();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create job');
    } finally {
      setSubmitting(false);
    }
  };

  const handleExecuteJob = async (jobId: string) => {
    try {
      await api.executeJob(token!, jobId);
      await loadRuns(jobId);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to execute job');
    }
  };

  const handleViewRun = async (runId: string) => {
    try {
      const detail = await api.getJobRunDetail(token!, runId);
      setSelectedRun(detail);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load run detail');
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'success': case 'completed': return 'bg-green-500/10 text-green-600 dark:text-green-400';
      case 'running': case 'pending': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      case 'failed': case 'error': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'cancelled': return 'bg-md-surface-container-high text-md-on-surface-variant';
      default: return 'bg-md-surface-container-high text-md-on-surface-variant';
    }
  };

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.jobs')}
        </h2>
        <div className="flex gap-2">
          <button onClick={loadJobs} disabled={loading}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:bg-md-surface-container-high disabled:opacity-50 transition-colors">
            {loading ? t('jobs.loading') : t('jobs.reload')}
          </button>
          <button onClick={() => setShowCreateForm(!showCreateForm)}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
            {showCreateForm ? t('jobs.cancel') : t('jobs.add')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {showCreateForm && (
        <form onSubmit={handleCreateJob} className="bg-md-surface-container-low rounded-md-lg p-4 space-y-3 animate-slide-up">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('jobs.name')}</label>
              <input type="text" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface mb-1">{t('jobs.description')}</label>
              <input type="text" value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })}
                className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface" />
            </div>
          </div>
          <div>
            <label className="block text-label-large text-md-on-surface mb-1">{t('jobs.steps')}</label>
            <textarea value={form.steps_json} onChange={(e) => setForm({ ...form, steps_json: e.target.value })}
              rows={4}
              className="w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary outline-none text-body-medium text-md-on-surface font-mono text-sm" />
          </div>
          <div className="flex justify-end gap-2">
            <button type="button" onClick={() => setShowCreateForm(false)}
              className="px-4 py-2 text-sm rounded-md-lg border border-md-outline text-md-on-surface-variant hover:bg-md-surface-container-high transition-colors">
              {t('jobs.cancel')}
            </button>
            <button type="submit" disabled={submitting}
              className="px-4 py-2 text-sm rounded-md-lg bg-md-primary text-md-on-primary hover:shadow-md-2 disabled:opacity-50 transition-all">
              {submitting ? t('jobs.creating') : t('jobs.create')}
            </button>
          </div>
        </form>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Jobs List */}
        <div className="lg:col-span-1 bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="p-4 border-b border-md-outline-variant">
            <h3 className="text-title-medium font-semibold text-md-on-surface">{t('jobs.list')}</h3>
          </div>
          <div className="max-h-96 overflow-y-auto">
            {jobs.map((job) => (
              <button
                key={job.id}
                onClick={() => { setSelectedJob(job); loadRuns(job.id); }}
                className={cn(
                  'w-full text-left px-4 py-3 border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors',
                  selectedJob?.id === job.id && 'bg-md-primary-container/20',
                )}
              >
                <div className="flex items-center justify-between">
                  <span className="text-body-medium font-medium text-md-on-surface">{job.name}</span>
                  <button
                    onClick={(e) => { e.stopPropagation(); handleExecuteJob(job.id); }}
                    className="text-xs px-2 py-1 rounded-md-full bg-md-primary text-md-on-primary hover:shadow-md-1 transition-all"
                  >
                    {t('jobs.execute')}
                  </button>
                </div>
                <p className="text-body-small text-md-on-surface-variant mt-1">{job.description || t('jobs.no_description')}</p>
              </button>
            ))}
            {!loading && jobs.length === 0 && (
              <div className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('jobs.empty')}</div>
            )}
          </div>
        </div>

        {/* Runs List */}
        <div className="lg:col-span-2 bg-md-surface-container-low rounded-md-lg overflow-hidden shadow-md-1">
          <div className="p-4 border-b border-md-outline-variant">
            <h3 className="text-title-medium font-semibold text-md-on-surface">
              {selectedJob ? `${selectedJob.name} - ${t('jobs.runs')}` : t('jobs.select_job')}
            </h3>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="border-b border-md-outline-variant">
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('jobs.run_col.status')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('jobs.run_col.time')}</th>
                  <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">{t('jobs.run_col.duration')}</th>
                  <th className="px-4 py-3 text-right text-label-medium text-md-on-surface-variant">{t('jobs.run_col.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {runs.map((run) => (
                  <tr key={run.id} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                    <td className="whitespace-nowrap px-4 py-3">
                      <span className={cn('text-xs font-medium px-2 py-1 rounded-md-sm', statusColor(run.status))}>{run.status}</span>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">
                      {new Date(run.created_at).toLocaleString()}
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-body-small text-md-on-surface-variant">
                      {run.duration_ms ? `${run.duration_ms}ms` : '-'}
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-right">
                      <button onClick={() => handleViewRun(run.id)}
                        className="text-md-primary text-label-large hover:bg-md-primary-container/30 px-2 py-1 rounded-md-sm transition-colors">
                        {t('jobs.view')}
                      </button>
                    </td>
                  </tr>
                ))}
                {selectedJob && runs.length === 0 && (
                  <tr><td colSpan={4} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">{t('jobs.no_runs')}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Run Detail Modal */}
      {selectedRun && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setSelectedRun(null)}>
          <div className="glass-card rounded-md-2xl p-6 w-full max-w-2xl max-h-[80vh] overflow-auto shadow-md-3 animate-scale-in" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-title-large font-semibold text-md-on-surface">{t('jobs.run_detail')}</h3>
              <button onClick={() => setSelectedRun(null)} className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors">
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="flex items-center gap-3 mb-4">
              <span className={cn('text-sm font-medium px-3 py-1 rounded-full', statusColor(selectedRun.run.status))}>{selectedRun.run.status}</span>
              {selectedRun.run.duration_ms && (
                <span className="text-sm text-md-on-surface-variant">{selectedRun.run.duration_ms}ms</span>
              )}
            </div>

            <div className="space-y-3">
              {selectedRun.steps.map((step) => (
                <div key={step.id} className="glass-card rounded-md-lg p-3">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-body-medium font-medium text-md-on-surface">{step.step_name}</span>
                    <span className={cn('text-xs font-medium px-2 py-0.5 rounded-md-sm', statusColor(step.status))}>{step.status}</span>
                  </div>
                  {step.output && (
                    <pre className="bg-md-surface-container-highest rounded-md-sm p-2 text-body-small font-mono overflow-auto max-h-32">{step.output}</pre>
                  )}
                  {step.error && (
                    <pre className="bg-md-error-container/20 rounded-md-sm p-2 text-body-small font-mono overflow-auto max-h-32 text-md-error">{step.error}</pre>
                  )}
                </div>
              ))}
              {selectedRun.steps.length === 0 && (
                <p className="text-body-medium text-md-on-surface-variant text-center py-4">{t('jobs.no_step_data')}</p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
