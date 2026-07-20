import { useCallback, useState } from 'react';
import { api } from '../api/client';
import type { Runbook, RunbookExecution } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

export function RunbookPage() {
  const { token } = useAuthStore();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [rbName, setRbName] = useState('');
  const [rbDesc, setRbDesc] = useState('');
  const [created, setCreated] = useState<Runbook | null>(null);

  const [execName, setExecName] = useState('');
  const [execHost, setExecHost] = useState('');
  const [execution, setExecution] = useState<RunbookExecution | null>(null);

  const handleCreate = useCallback(async () => {
    if (!token || !rbName || !rbDesc) return;
    setLoading(true);
    setError(null);
    try {
      const rb = await api.createRunbook(token, rbName, rbDesc);
      setCreated(rb);
      setExecName(rb.name);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, rbName, rbDesc]);

  const handleExecute = useCallback(async () => {
    if (!token || !execName) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.executeRunbook(token, execName, execHost || undefined);
      setExecution(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, execName, execHost]);

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-gray-900">Runbook 管理</h2>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Create Runbook */}
        <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
          <h3 className="mb-4 text-base font-semibold text-gray-900">创建 Runbook</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700">名称</label>
              <input value={rbName} onChange={(e) => setRbName(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" placeholder="重启 Nginx 服务" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">步骤描述（每行一步）</label>
              <textarea value={rbDesc} onChange={(e) => setRbDesc(e.target.value)} rows={5} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" placeholder={"检查磁盘空间\n重启 nginx 服务\n确认服务健康"} />
            </div>
            <button onClick={handleCreate} disabled={loading || !rbName || !rbDesc} className={cn('w-full rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
              {loading ? '创建中...' : '创建 Runbook'}
            </button>
          </div>
        </div>

        {/* Execute Runbook */}
        <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
          <h3 className="mb-4 text-base font-semibold text-gray-900">执行 Runbook</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700">Runbook 名称</label>
              <input value={execName} onChange={(e) => setExecName(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">目标主机 ID（可选）</label>
              <input value={execHost} onChange={(e) => setExecHost(e.target.value)} className="mt-1 w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm" placeholder="localhost" />
            </div>
            <button onClick={handleExecute} disabled={loading || !execName} className={cn('w-full rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700 disabled:opacity-50')}>
              {loading ? '执行中...' : '执行'}
            </button>
          </div>
        </div>
      </div>

      {/* Created Runbook */}
      {created && (
        <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
          <h3 className="mb-3 text-base font-semibold text-gray-900">已创建: {created.name}</h3>
          <p className="mb-3 text-sm text-gray-600">{created.description}</p>
          <div className="space-y-2">
            {created.steps.map((s) => (
              <div key={s.id} className="flex items-center gap-3 rounded-md border border-gray-100 bg-gray-50 px-3 py-2">
                <span className="text-xs font-mono text-gray-400">{s.id}</span>
                <span className="text-sm text-gray-700">{s.name}</span>
                {s.requires_approval && <span className="rounded-full bg-yellow-100 px-2 py-0.5 text-xs text-yellow-700">需审批</span>}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Execution Result */}
      {execution && (
        <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
          <div className="mb-3 flex items-center gap-2">
            <h3 className="text-base font-semibold text-gray-900">执行结果</h3>
            <span className={cn('rounded-full px-2.5 py-0.5 text-xs font-semibold', execution.success ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800')}>
              {execution.success ? '成功' : '失败'}
            </span>
          </div>
          <div className="space-y-2">
            {execution.steps.map((s) => (
              <div key={s.step_id} className="rounded-md border border-gray-100 bg-gray-50 px-3 py-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-gray-700">{s.step_id}</span>
                  <span className="text-xs text-gray-500">{s.duration_ms}ms</span>
                </div>
                <p className="mt-1 text-xs text-gray-500">{s.output}</p>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
