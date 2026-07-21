import { useState, useEffect } from 'react';

interface SessionSummary {
  session_id: string;
  host: string;
  user_name: string;
  command_count: number;
  started_at: string;
  last_activity: string;
}

interface SessionRecord {
  id: string;
  session_id: string;
  host: string;
  user_name: string;
  command: string;
  output: string | null;
  exit_code: number | null;
  recorded_at: string;
}

export function SessionReplayPage() {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [selectedSession, setSelectedSession] = useState<string | null>(null);
  const [records, setRecords] = useState<SessionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [replaying, setReplaying] = useState(false);

  const fetchSessions = async () => {
    setLoading(true);
    try {
      const resp = await fetch('/api/sessions');
      const data = await resp.json();
      setSessions(data);
    } catch (e: any) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => { fetchSessions(); }, []);

  const handleReplay = async (sessionId: string) => {
    setReplaying(true);
    setSelectedSession(sessionId);
    try {
      const resp = await fetch(`/api/sessions/${sessionId}/replay`);
      const data = await resp.json();
      setRecords(data);
    } catch (e: any) {
      console.error(e);
    }
    setReplaying(false);
  };

  return (
    <div className="space-y-6">
      {/* Session 列表 */}
      <div className="glass-card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-md-on-surface">终端操作回放</h2>
          <button
            onClick={fetchSessions}
            className="px-4 py-1.5 rounded-md-lg text-sm font-medium bg-md-surface-container text-md-on-surface hover:glass-card transition-all"
          >
            刷新
          </button>
        </div>

        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">加载中...</div>
        ) : sessions.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">暂无会话记录</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-md-outline-variant/50">
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">会话 ID</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">主机</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">用户</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">命令数</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">开始时间</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">最后活动</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">操作</th>
                </tr>
              </thead>
              <tbody>
                {sessions.map(s => (
                  <tr key={s.session_id} className="border-b border-md-outline-variant/20 hover:bg-md-surface-container/30">
                    <td className="px-4 py-2.5 font-mono text-xs text-md-on-surface">{s.session_id.slice(0, 8)}...</td>
                    <td className="px-4 py-2.5 text-md-on-surface">{s.host}</td>
                    <td className="px-4 py-2.5 text-md-on-surface">{s.user_name}</td>
                    <td className="px-4 py-2.5 text-md-on-surface">{s.command_count}</td>
                    <td className="px-4 py-2.5 text-xs text-md-on-surface-variant font-mono">{s.started_at}</td>
                    <td className="px-4 py-2.5 text-xs text-md-on-surface-variant font-mono">{s.last_activity}</td>
                    <td className="px-4 py-2.5">
                      <button
                        onClick={() => handleReplay(s.session_id)}
                        className="px-3 py-1 rounded-md text-xs font-medium bg-md-primary text-md-on-primary hover:opacity-90"
                      >
                        回放
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 回放视图 */}
      {selectedSession && (
        <div className="glass-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-md font-semibold text-md-on-surface">
              会话回放: {selectedSession.slice(0, 8)}...
            </h3>
            <button
              onClick={() => { setSelectedSession(null); setRecords([]); }}
              className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface hover:glass-card"
            >
              关闭
            </button>
          </div>

          {replaying ? (
            <div className="text-center py-8 text-md-on-surface-variant">加载中...</div>
          ) : records.length === 0 ? (
            <div className="text-center py-8 text-md-on-surface-variant">无记录</div>
          ) : (
            <div className="space-y-3">
              {records.map((r) => (
                <div key={r.id} className="border border-md-outline-variant/30 rounded-md-lg overflow-hidden">
                  {/* 命令行 */}
                  <div className="flex items-center gap-2 px-4 py-2 bg-md-surface-container/50 border-b border-md-outline-variant/20">
                    <span className="text-xs text-md-on-surface-variant font-mono">{r.recorded_at}</span>
                    <span className="text-xs text-green-600 font-mono">$</span>
                    <span className="text-sm text-md-on-surface font-mono">{r.command}</span>
                    {r.exit_code !== null && (
                      <span className={`text-xs font-mono ml-auto ${r.exit_code === 0 ? 'text-green-600' : 'text-red-600'}`}>
                        exit: {r.exit_code}
                      </span>
                    )}
                  </div>
                  {/* 输出 */}
                  {r.output && (
                    <pre className="px-4 py-3 text-xs text-md-on-surface font-mono bg-md-surface overflow-x-auto max-h-48">
                      {r.output}
                    </pre>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
