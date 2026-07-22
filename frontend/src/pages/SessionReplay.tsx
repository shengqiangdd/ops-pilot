import { useState, useEffect } from 'react';
import { useI18n } from '../i18n';

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
  const { t } = useI18n();
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [selectedSession, setSelectedSession] = useState<string | null>(null);
  const [records, setRecords] = useState<SessionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [replaying, setReplaying] = useState(false);

  const [searchHost, setSearchHost] = useState('');
  const [searchUser, setSearchUser] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  const [page, setPage] = useState(0);
  const pageSize = 20;

  const fetchSessions = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      if (searchHost) params.set('host', searchHost);
      if (searchUser) params.set('user', searchUser);
      const resp = await fetch(`/api/sessions?${params}`);
      const data = await resp.json();
      setSessions(data);
    } catch (e: any) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => { fetchSessions(); }, [searchHost, searchUser]);

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

  const filteredSessions = sessions.filter(s => {
    if (startDate && s.started_at < startDate) return false;
    if (endDate && s.last_activity > endDate) return false;
    return true;
  });

  const totalPages = Math.ceil(filteredSessions.length / pageSize);
  const pagedSessions = filteredSessions.slice(page * pageSize, (page + 1) * pageSize);

  return (
    <div className="space-y-6">
      {/* Filter area */}
      <div className="glass-card p-6">
        <h2 className="text-lg font-semibold text-md-on-surface mb-4">{t('session_replay.title')}</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <input
            type="text"
            value={searchHost}
            onChange={e => { setSearchHost(e.target.value); setPage(0); }}
            placeholder={t('session_replay.filter_host')}
            className="px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <input
            type="text"
            value={searchUser}
            onChange={e => { setSearchUser(e.target.value); setPage(0); }}
            placeholder={t('session_replay.filter_user')}
            className="px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <input
            type="date"
            value={startDate}
            onChange={e => { setStartDate(e.target.value); setPage(0); }}
            className="px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <input
            type="date"
            value={endDate}
            onChange={e => { setEndDate(e.target.value); setPage(0); }}
            className="px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
        </div>
        <div className="mt-3 flex items-center justify-between">
          <span className="text-xs text-md-on-surface-variant">
            {t('session_replay.total_sessions').replace('{count}', String(filteredSessions.length))}
          </span>
          <button
            onClick={() => { setSearchHost(''); setSearchUser(''); setStartDate(''); setEndDate(''); setPage(0); }}
            className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface hover:glass-card"
          >
            {t('session_replay.clear_filters')}
          </button>
        </div>
      </div>

      {/* Session list */}
      <div className="glass-card p-6">
        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('session_replay.loading')}</div>
        ) : pagedSessions.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('session_replay.empty')}</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-md-outline-variant/50">
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('session_replay.col_session_id')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('session_replay.col_host')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('session_replay.col_user')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('session_replay.col_commands')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('session_replay.col_start')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('session_replay.col_last_activity')}</th>
                  <th className="text-left px-4 py-3 text-xs font-semibold text-md-on-surface-variant uppercase">{t('session_replay.col_actions')}</th>
                </tr>
              </thead>
              <tbody>
                {pagedSessions.map(s => (
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
                        {t('session_replay.replay')}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-center gap-2 mt-4 pt-4 border-t border-md-outline-variant/30">
            <button
              onClick={() => setPage(p => Math.max(0, p - 1))}
              disabled={page === 0}
              className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface disabled:opacity-50"
            >
              {t('session_replay.prev')}
            </button>
            <span className="text-xs text-md-on-surface-variant">
              {t('session_replay.page_info').replace('{page}', String(page + 1)).replace('{total}', String(totalPages))}
            </span>
            <button
              onClick={() => setPage(p => Math.min(totalPages - 1, p + 1))}
              disabled={page >= totalPages - 1}
              className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface disabled:opacity-50"
            >
              {t('session_replay.next')}
            </button>
          </div>
        )}
      </div>

      {/* Replay view */}
      {selectedSession && (
        <div className="glass-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-md font-semibold text-md-on-surface">
              {t('session_replay.replay_view')}: {selectedSession.slice(0, 8)}...
            </h3>
            <button
              onClick={() => { setSelectedSession(null); setRecords([]); }}
              className="px-3 py-1 rounded-md text-xs font-medium bg-md-surface-container text-md-on-surface hover:glass-card"
            >
              {t('session_replay.close')}
            </button>
          </div>

          {replaying ? (
            <div className="text-center py-8 text-md-on-surface-variant">{t('session_replay.loading')}</div>
          ) : records.length === 0 ? (
            <div className="text-center py-8 text-md-on-surface-variant">{t('session_replay.no_records')}</div>
          ) : (
            <div className="space-y-3">
              {records.map((r) => (
                <div key={r.id} className="border border-md-outline-variant/30 rounded-md-lg overflow-hidden">
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
