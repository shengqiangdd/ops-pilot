import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api/client';
import { useAuthStore } from '../../stores/useAuthStore';
import type { Host } from '../../api/types';

export function ResourceUsageWidget() {
  const { token } = useAuthStore();
  const [hosts, setHosts] = useState<Host[]>([]);

  const load = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.listHosts(token);
      setHosts(data);
    } catch { /* ignore */ }
  }, [token]);

  useEffect(() => { load(); }, [load]);

  const online = hosts.filter(h => h.status === 'online').length;
  const total = hosts.length;

  return (
    <div className="flex items-center justify-around h-full">
      <div className="text-center">
        <p className="text-headline-small font-bold text-md-primary">{total}</p>
        <p className="text-label-small text-md-on-surface-variant">主机总数</p>
      </div>
      <div className="w-px h-10 bg-md-outline-variant" />
      <div className="text-center">
        <p className="text-headline-small font-bold text-green-500">{online}</p>
        <p className="text-label-small text-md-on-surface-variant">在线</p>
      </div>
      <div className="w-px h-10 bg-md-outline-variant" />
      <div className="text-center">
        <p className="text-headline-small font-bold text-md-error">{total - online}</p>
        <p className="text-label-small text-md-on-surface-variant">离线</p>
      </div>
    </div>
  );
}
