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
    <div className="space-y-6 animate-slide-up">
      <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">Runbook</h2>

      {error && <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>}

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
          <h3 className="mb-4 text-title-medium font-medium text-md-on-surface">Create Runbook</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-label-large text-md-on-surface">Name</label>
              <input value={rbName} onChange={(e) => setRbName(e.target.value)}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface"
                placeholder="Restart Nginx service" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface">Steps (one per line)</label>
              <textarea value={rbDesc} onChange={(e) => setRbDesc(e.target.value)} rows={5}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface font-mono"
                placeholder={"Check disk space\nRestart nginx service\nVerify service health"} />
            </div>
            <button onClick={handleCreate} disabled={loading || !rbName || !rbDesc}
              className="w-full bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {loading ? 'Creating...' : 'Create Runbook'}
            </button>
          </div>
        </div>

        <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
          <h3 className="mb-4 text-title-medium font-medium text-md-on-surface">Execute Runbook</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-label-large text-md-on-surface">Runbook Name</label>
              <input value={execName} onChange={(e) => setExecName(e.target.value)}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface" />
            </div>
            <div>
              <label className="block text-label-large text-md-on-surface">Target Host (optional)</label>
              <input value={execHost} onChange={(e) => setExecHost(e.target.value)}
                className="mt-1 w-full bg-md-surface-container-highest rounded-md-sm px-4 py-3 border border-md-outline text-body-medium text-md-on-surface"
                placeholder="localhost" />
            </div>
            <button onClick={handleExecute} disabled={loading || !execName}
              className="w-full bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
              {loading ? 'Executing...' : 'Execute'}
            </button>
          </div>
        </div>
      </div>

      {created && (
        <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
          <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Created: {created.name}</h3>
          <p className="mb-3 text-body-medium text-md-on-surface-variant">{created.description}</p>
          <div className="space-y-2">
            {created.steps.map((s) => (
              <div key={s.id} className="flex items-center gap-3 bg-md-surface-container rounded-md-md px-3 py-2">
                <span className="text-label-medium text-md-on-surface-variant font-mono">{s.id}</span>
                <span className="text-body-medium text-md-on-surface">{s.name}</span>
                {s.requires_approval && <span className="rounded-md-full bg-amber-100 dark:bg-amber-900/30 px-2 py-0.5 text-label-medium text-amber-700 dark:text-amber-200">Approval needed</span>}
              </div>
            ))}
          </div>
        </div>
      )}

      {execution && (
        <div className="bg-md-surface-container-low rounded-md-lg p-4 sm:p-6 shadow-md-1">
          <div className="mb-3 flex items-center gap-2">
            <h3 className="text-title-medium font-medium text-md-on-surface">Execution Result</h3>
            <span className={cn('rounded-md-full px-2.5 py-0.5 text-label-medium font-semibold',
              execution.success ? 'bg-md-primary-container text-md-on-primary-container' : 'bg-md-error-container text-md-on-error-container')}>
              {execution.success ? 'Success' : 'Failed'}
            </span>
          </div>
          <div className="space-y-2">
            {execution.steps.map((s) => (
              <div key={s.step_id} className="bg-md-surface-container rounded-md-md px-3 py-2">
                <div className="flex items-center justify-between">
                  <span className="text-body-medium font-medium text-md-on-surface">{s.step_id}</span>
                  <span className="text-label-medium text-md-on-surface-variant">{s.duration_ms}ms</span>
                </div>
                <p className="mt-1 text-body-medium text-md-on-surface-variant">{s.output}</p>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
