import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { SchedulerJob } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

export function SchedulerPage() {
  const { token } = useAuthStore();
  const [jobs, setJobs] = useState<SchedulerJob[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [jobName, setJobName] = useState('');
  const [cronExpr, setCronExpr] = useState('');
  const [action, setAction] = useState('');
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api.listSchedulerJobs(token);
      setJobs(data.jobs || []);
    } catch { setJobs([]); } finally { setLoading(false); }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleCreate = useCallback(async () => {
    if (!token || !jobName || !cronExpr) return;
    setSaving(true);
    setError(null);
    try {
      await api.createSchedulerJob(token, { name: jobName, cron_expr: cronExpr, action });
      setJobName(''); setCronExpr(''); setAction('');
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally { setSaving(false); }
  }, [token, jobName, cronExpr, action, load]);

  return (
    <div className="space-y-4 animate-slide-up">
      <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Scheduler</h2>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
        <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Create Scheduled Job</h3>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-4">
          <input value={jobName} onChange={(e) => setJobName(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="Job name" />
          <input value={cronExpr} onChange={(e) => setCronExpr(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="Cron (e.g. */5 * * * *)" />
          <input value={action} onChange={(e) => setAction(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
            placeholder="Action" />
          <button onClick={handleCreate} disabled={saving || !jobName || !cronExpr}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {saving ? 'Creating...' : 'Create'}
          </button>
        </div>
      </div>

      <div className="bg-md-surface-container-low rounded-md-lg shadow-md-1 overflow-hidden">
        <table className="min-w-full">
          <thead>
            <tr className="border-b border-md-outline-variant">
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Name</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Cron</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Action</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Status</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Last Run</th>
              <th className="px-4 py-3 text-left text-label-medium text-md-on-surface-variant">Next Run</th>
            </tr>
          </thead>
          <tbody>
            {jobs.map((j, i) => (
              <tr key={i} className="border-b border-md-outline-variant last:border-0 hover:bg-md-surface-container-high/50 transition-colors">
                <td className="whitespace-nowrap px-4 py-3 text-body-medium font-medium text-md-on-surface">{j.name}</td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant font-mono">{j.cron_expr}</td>
                <td className="px-4 py-3 text-body-medium text-md-on-surface-variant">{j.action}</td>
                <td className="whitespace-nowrap px-4 py-3">
                  <span className={cn('inline-block rounded-md-full px-2 py-0.5 text-label-medium font-semibold',
                    j.enabled ? 'bg-md-primary-container text-md-on-primary-container' : 'bg-md-surface-container-high text-md-on-surface-variant')}>
                    {j.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{j.last_run_at || '-'}</td>
                <td className="whitespace-nowrap px-4 py-3 text-body-medium text-md-on-surface-variant">{j.next_run_at || '-'}</td>
              </tr>
            ))}
            {jobs.length === 0 && !loading && (
              <tr><td colSpan={6} className="px-4 py-8 text-center text-body-medium text-md-on-surface-variant">No scheduled jobs</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
