import { useState, useEffect } from 'react';

interface GitStatus {
  repo_path: string;
  branch: string;
  dirty: boolean;
  ahead: number;
  behind: number;
  last_commit: string;
}

interface SyncResult {
  success: boolean;
  message: string;
  commit_hash: string | null;
}

export function GitOpsPage() {
  const [status, setStatus] = useState<GitStatus | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState<SyncResult | null>(null);
  const [commitMsg, setCommitMsg] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchStatus = async () => {
    setLoading(true);
    setError(null);
    try {
      const resp = await fetch('/api/gitops/status');
      if (!resp.ok) throw new Error('获取状态失败');
      const data = await resp.json();
      setStatus(data);
    } catch (e: any) {
      setError(e.message);
    }
    setLoading(false);
  };

  useEffect(() => { fetchStatus(); }, []);

  const handleSync = async () => {
    setSyncing(true);
    setSyncResult(null);
    setError(null);
    try {
      const resp = await fetch('/api/gitops/sync', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message: commitMsg || 'ops-pilot: config sync' }),
      });
      if (!resp.ok) {
        const data = await resp.json();
        throw new Error(data.error || '同步失败');
      }
      const data = await resp.json();
      setSyncResult(data);
      fetchStatus();
    } catch (e: any) {
      setError(e.message);
    }
    setSyncing(false);
  };

  return (
    <div className="space-y-6">
      {/* Git 状态 */}
      <div className="glass-card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-md-on-surface">GitOps 配置同步</h2>
          <button
            onClick={fetchStatus}
            className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-surface-container text-md-on-surface hover:glass-card transition-all"
          >
            刷新状态
          </button>
        </div>

        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">加载中...</div>
        ) : status ? (
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="p-3 rounded-md-lg bg-md-surface-container/50">
              <span className="text-md-on-surface-variant text-xs">仓库路径</span>
              <p className="text-md-on-surface font-mono mt-1">{status.repo_path}</p>
            </div>
            <div className="p-3 rounded-md-lg bg-md-surface-container/50">
              <span className="text-md-on-surface-variant text-xs">当前分支</span>
              <p className="text-md-on-surface font-medium mt-1">{status.branch}</p>
            </div>
            <div className="p-3 rounded-md-lg bg-md-surface-container/50">
              <span className="text-md-on-surface-variant text-xs">工作树状态</span>
              <p className="mt-1">
                <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${status.dirty ? 'bg-amber-100 text-amber-700' : 'bg-green-100 text-green-700'}`}>
                  {status.dirty ? '有未提交更改' : '干净'}
                </span>
              </p>
            </div>
            <div className="p-3 rounded-md-lg bg-md-surface-container/50">
              <span className="text-md-on-surface-variant text-xs">远程同步</span>
              <p className="text-md-on-surface mt-1">
                {status.ahead > 0 && <span className="text-amber-600">领先 {status.ahead} </span>}
                {status.behind > 0 && <span className="text-red-600">落后 {status.behind} </span>}
                {status.ahead === 0 && status.behind === 0 && <span className="text-green-600">已同步</span>}
              </p>
            </div>
            <div className="col-span-2 p-3 rounded-md-lg bg-md-surface-container/50">
              <span className="text-md-on-surface-variant text-xs">最后提交</span>
              <p className="text-md-on-surface font-mono mt-1">{status.last_commit || '无提交记录'}</p>
            </div>
          </div>
        ) : (
          <div className="text-center py-8 text-md-on-surface-variant">无法获取状态</div>
        )}
      </div>

      {/* 同步操作 */}
      <div className="glass-card p-6">
        <h3 className="text-md font-semibold text-md-on-surface mb-4">同步配置</h3>
        <div className="space-y-3">
          <input
            type="text"
            value={commitMsg}
            onChange={e => setCommitMsg(e.target.value)}
            placeholder="提交信息（可选，默认: ops-pilot: config sync）"
            className="w-full px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface placeholder:text-md-on-surface-variant/50 focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <button
            onClick={handleSync}
            disabled={syncing}
            className="px-6 py-2 rounded-md-lg bg-md-primary text-md-on-primary font-medium hover:opacity-90 disabled:opacity-50 transition-all"
          >
            {syncing ? '同步中...' : '执行同步 (git add -A && commit && push)'}
          </button>
        </div>
      </div>

      {/* 同步结果 */}
      {syncResult && (
        <div className={`glass-card p-4 border-l-4 ${syncResult.success ? 'border-green-500' : 'border-red-500'}`}>
          <p className={`text-sm ${syncResult.success ? 'text-green-600' : 'text-red-600'}`}>
            {syncResult.success ? '✅' : '❌'} {syncResult.message}
          </p>
          {syncResult.commit_hash && (
            <p className="text-xs text-md-on-surface-variant mt-1 font-mono">
              Commit: {syncResult.commit_hash}
            </p>
          )}
        </div>
      )}

      {/* 错误提示 */}
      {error && (
        <div className="glass-card p-4 border-l-4 border-red-500">
          <p className="text-red-600 text-sm">❌ {error}</p>
        </div>
      )}
    </div>
  );
}
