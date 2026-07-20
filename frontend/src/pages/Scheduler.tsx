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
    } catch {
      setJobs([]);
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const handleCreate = useCallback(async () => {
    if (!token || !jobName || !cronExpr) return;
    setSaving(true);
    setError(null);
    try {
      await api.createSchedulerJob(token, { name: jobName, cron_expr: cronExpr, action });
      setJobName('');
      setCronExpr('');
      setAction('');
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setSaving(false);
    }
  }, [token, jobName, cronExpr, action, load]);

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold text-gray-900">定时任务管理</h2>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
        <h3 className="mb-3 text-base font-semibold text-gray-900">创建定时任务</h3>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-4">
          <input value={jobName} onChange={(e) => setJobName(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="任务名称" />
          <input value={cronExpr} onChange={(e) => setCronExpr(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="Cron 表达式 (如 */5 * * * *)" />
          <input value={action} onChange={(e) => setAction(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="执行动作" />
          <button onClick={handleCreate} disabled={saving || !jobName || !cronExpr} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {saving ? '创建中...' : '创建'}
          </button>
        </div>
      </div>

      <div className="rounded-lg border border-gray-200 bg-white shadow-sm overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">名称</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">Cron</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">动作</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">状态</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">上次运行</th>
              <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">下次运行</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {jobs.map((j, i) => (
              <tr key={i} className="hover:bg-gray-50">
                <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-gray-900">{j.name}</td>
                <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-600 font-mono">{j.cron_expr}</td>
                <td className="px-4 py-3 text-sm text-gray-600">{j.action}</td>
                <td className="whitespace-nowrap px-4 py-3">
                  <span className={cn('inline-block rounded-full px-2 py-0.5 text-xs font-semibold', j.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600')}>
                    {j.enabled ? '启用' : '禁用'}
                  </span>
                </td>
                <td className="whitespace-nowrap px-4 py-3 text-xs text-gray-500">{j.last_run_at || '-'}</td>
                <td className="whitespace-nowrap px-4 py-3 text-xs text-gray-500">{j.next_run_at || '-'}</td>
              </tr>
            ))}
            {jobs.length === 0 && !loading && (
              <tr><td colSpan={6} className="px-4 py-8 text-center text-sm text-gray-500">暂无定时任务</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
