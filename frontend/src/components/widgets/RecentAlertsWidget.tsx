import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api/client';
import { useAuthStore } from '../../stores/useAuthStore';
import { cn } from '../../lib/cn';

interface AuditEntry {
  id: string;
  user: string;
  action: string;
  resource: string;
  outcome: string;
  created_at: string;
}

export function RecentAlertsWidget() {
  const { token } = useAuthStore();
  const [logs, setLogs] = useState<AuditEntry[]>([]);

  const load = useCallback(async () => {
    if (!token) return;
    try {
      const resp = await api.listAuditLogs(token, { per_page: '5' });
      setLogs(resp.data);
    } catch { /* ignore */ }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const outcomeColor = (outcome: string) => {
    switch (outcome.toLowerCase()) {
      case 'success': return 'text-green-500';
      case 'denied': case 'failure': case 'error': return 'text-md-error';
      default: return 'text-md-on-surface-variant';
    }
  };

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 space-y-1.5 overflow-auto">
        {logs.map((log) => (
          <div key={log.id} className="flex items-center gap-2 px-3 py-2 rounded-md-lg glass-card">
            <span className={cn('text-label-small font-mono', outcomeColor(log.outcome))}>●</span>
            <span className="flex-1 text-body-small text-md-on-surface truncate">{log.action}</span>
            <span className="text-label-small text-md-on-surface-variant shrink-0">
              {new Date(log.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
            </span>
          </div>
        ))}
        {logs.length === 0 && (
          <p className="text-body-small text-md-on-surface-variant text-center py-4">暂无最近活动</p>
        )}
      </div>
    </div>
  );
}
