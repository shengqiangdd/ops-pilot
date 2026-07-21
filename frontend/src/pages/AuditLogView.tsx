import { useState, useEffect } from 'react';

interface AuditLogEntry {
  id: string;
  actor: string;
  action: string;
  resource: string;
  detail: string;
  created_at: string;
}

interface SlowQueryEntry {
  id: string;
  query_text: string;
  duration_ms: number;
  created_at: string;
}

export function AuditLogViewPage() {
  const [logs, setLogs] = useState<AuditLogEntry[]>([]);
  const [slowQueries, setSlowQueries] = useState<SlowQueryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'logs' | 'slow'>('logs');
  const [page, setPage] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const fetchLogs = async () => {
    setLoading(true);
    try {
      const resp = await fetch(`/api/audit/logs?limit=50&offset=${page * 50}`);
      const data = await resp.json();
      setLogs(data);
    } catch (e: any) {
      setError(e.message);
    }
    setLoading(false);
  };

  const fetchSlowQueries = async () => {
    try {
      const resp = await fetch('/api/audit/slow-queries?limit=50');
      const data = await resp.json();
      setSlowQueries(data);
    } catch (e: any) {
      setError(e.message);
    }
  };

  useEffect(() => { fetchLogs(); }, [page]);
  useEffect(() => { if (activeTab === 'slow') fetchSlowQueries(); }, [activeTab]);

  return (
    <div className="space-y-6">
      {/* Tab 切换 */}
      <div className="glass-card p-4">
        <div className="flex gap-2">
          <button
            onClick={() => setActiveTab('logs')}
            className={`px-4 py-2 rounded-md-lg text-sm font-medium transition-all ${
              activeTab === 'logs'
                ? 'bg-md-primary text-md-on-primary'
                : 'bg-md-surface-container text-md-on-surface hover:glass-card'
            }`}
          >
            操作审计日志
          </button>
          <button
            onClick={() => setActiveTab('slow')}
            className={`px-4 py-2 rounded-md-lg text-sm font-medium transition-all ${
              activeTab === 'slow'
                ? 'bg-md-primary text-md-on-primary'
                : 'bg-md-surface-container text-md-on-surface hover:glass-card'
            }`}
          >
            慢查询检测
          </button>
        </div>
      </div>

      {error && (
        <div className="glass-card p-3 border-l-4 border-red-500">
          <p className="text-red-600 text-sm">❌ {error}</p>
        </div>
      )}

      {/* 审计日志表格 */}
      {activeTab === 'logs' && (
        <div className="glass-card overflow-hidden">
          {loading ? (
            <div className="text-center py-8 text-md-on-surface-variant">加载中...</div>
          ) : logs.length === 0 ? (
            <div className="text-center py-8 text-md-on-surface-variant">暂无审计日志</div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-md-outline-variant/50">
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">时间</th>
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">操作人</th>
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">操作</th>
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">资源</th>
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">详情</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.map(log => (
                    <tr key={log.id} className="border-b border-md-outline-variant/20 hover:bg-md-surface-container/30">
                      <td className="px-4 py-2.5 text-xs text-md-on-surface-variant font-mono whitespace-nowrap">{log.created_at}</td>
                      <td className="px-4 py-2.5 text-md-on-surface font-medium">{log.actor}</td>
                      <td className="px-4 py-2.5">
                        <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-700">{log.action}</span>
                      </td>
                      <td className="px-4 py-2.5 text-md-on-surface font-mono text-xs">{log.resource}</td>
                      <td className="px-4 py-2.5 text-md-on-surface-variant text-xs max-w-xs truncate">{log.detail || '-'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          {/* 分页 */}
          <div className="flex items-center justify-between px-4 py-3 border-t border-md-outline-variant/30">
            <span className="text-xs text-md-on-surface-variant">第 {page + 1} 页</span>
            <div className="flex gap-2">
              <button
                onClick={() => setPage(p => Math.max(0, p - 1))}
                disabled={page === 0}
                className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface disabled:opacity-50"
              >
                上一页
              </button>
              <button
                onClick={() => setPage(p => p + 1)}
                disabled={logs.length < 50}
                className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface disabled:opacity-50"
              >
                下一页
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 慢查询表格 */}
      {activeTab === 'slow' && (
        <div className="glass-card overflow-hidden">
          {slowQueries.length === 0 ? (
            <div className="text-center py-8 text-md-on-surface-variant">暂无慢查询记录</div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-md-outline-variant/50">
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">时间</th>
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">耗时 (ms)</th>
                    <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">查询语句</th>
                  </tr>
                </thead>
                <tbody>
                  {slowQueries.map(q => (
                    <tr key={q.id} className="border-b border-md-outline-variant/20 hover:bg-md-surface-container/30">
                      <td className="px-4 py-2.5 text-xs text-md-on-surface-variant font-mono whitespace-nowrap">{q.created_at}</td>
                      <td className="px-4 py-2.5">
                        <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${q.duration_ms > 1000 ? 'bg-red-100 text-red-700' : 'bg-amber-100 text-amber-700'}`}>
                          {q.duration_ms}
                        </span>
                      </td>
                      <td className="px-4 py-2.5 text-md-on-surface font-mono text-xs max-w-lg truncate">{q.query_text}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
