import { useCallback, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { BaselineCheckResult } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

const STATUS_ICON: Record<string, string> = {
  Pass: '✅',
  Fail: '❌',
  Warn: '⚠️',
  Skip: '⏭️',
  Info: 'ℹ️',
};

const STATUS_COLOR: Record<string, string> = {
  Pass: 'bg-green-100 text-green-800',
  Fail: 'bg-red-100 text-red-800',
  Warn: 'bg-yellow-100 text-yellow-800',
  Skip: 'bg-gray-100 text-gray-600',
  Info: 'bg-blue-100 text-blue-800',
};

export function BaselinePage() {
  const { token } = useAuthStore();
  const [hostId, setHostId] = useState('');
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [results, setResults] = useState<BaselineCheckResult[]>([]);
  const [score, setScore] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    api.listHosts().then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
  }, [token]);

  const runCheck = useCallback(async () => {
    if (!token || !hostId) return;
    setLoading(true);
    setError(null);
    try {
      const res = await api.runBaselineCheck(token, hostId);
      setResults(res.results);
      setScore(res.score);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setLoading(false);
    }
  }, [token, hostId]);

  const scoreColor = score !== null ? (score >= 80 ? 'text-green-600' : score >= 60 ? 'text-yellow-600' : 'text-red-600') : 'text-gray-400';

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">安全基线巡检</h2>
        <div className="flex gap-2">
          <select value={hostId} onChange={(e) => setHostId(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm">
            <option value="">选择主机</option>
            {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
          </select>
          <button onClick={runCheck} disabled={loading || !hostId} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {loading ? '检查中...' : '执行检查'}
          </button>
        </div>
      </div>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      {score !== null && (
        <div className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm">
          <div className="flex items-center gap-4">
            <div className={cn('text-4xl font-bold', scoreColor)}>{score}</div>
            <div>
              <div className="text-sm font-medium text-gray-700">合规评分</div>
              <div className="text-xs text-gray-500">{results.filter((r) => r.status === 'Pass').length}/{results.length} 项通过</div>
            </div>
          </div>
        </div>
      )}

      {results.length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white shadow-sm overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">状态</th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">检查项</th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">分类</th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">详情</th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase text-gray-500">修复建议</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {results.map((r, i) => (
                <tr key={i} className="hover:bg-gray-50">
                  <td className="whitespace-nowrap px-4 py-3">
                    <span className={cn('inline-block rounded-full px-2.5 py-0.5 text-xs font-semibold', STATUS_COLOR[r.status])}>
                      {STATUS_ICON[r.status]} {r.status}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-gray-900">{r.name}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-sm text-gray-500">{r.category}</td>
                  <td className="px-4 py-3 text-sm text-gray-600">{r.message}</td>
                  <td className="px-4 py-3 text-xs text-gray-500">{r.remediation || '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {!results.length && !loading && (
        <div className="rounded-lg border border-gray-200 bg-white p-8 text-center text-sm text-gray-500 shadow-sm">
          选择主机后点击"执行检查"进行安全基线巡检
        </div>
      )}
    </div>
  );
}
