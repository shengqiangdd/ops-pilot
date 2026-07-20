import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api/client';
import { getHealthLabel, getHealthColor } from '../../lib/health';
import { cn } from '../../lib/cn';
import type { ModuleHealth } from '../../api/types';

export function ModuleStatusWidget() {
  const [health, setHealth] = useState<ModuleHealth[]>([]);

  const load = useCallback(async () => {
    try {
      const data = await api.getHealthAll();
      setHealth(data);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => { load(); const iv = setInterval(load, 30000); return () => clearInterval(iv); }, [load]);

  const healthy = health.filter(m => getHealthLabel(m.status) === 'Healthy').length;

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <span className="text-label-medium text-md-on-surface-variant">
          {healthy}/{health.length} 健康
        </span>
      </div>
      <div className="flex-1 space-y-1.5 overflow-auto">
        {health.map((m) => {
          const label = getHealthLabel(m.status);
          const dot = getHealthColor(m.status);
          return (
            <div
              key={m.name}
              className={cn(
                'flex items-center gap-2 px-3 py-2 rounded-md-lg glass-card',
                !m.enabled && 'opacity-50',
              )}
            >
              <span className={cn('h-2 w-2 rounded-full shrink-0', dot)} />
              <span className="flex-1 text-body-small font-medium text-md-on-surface truncate">{m.name}</span>
              <span className={cn('text-label-small', label === 'Healthy' ? 'text-green-500' : label === 'Degraded' ? 'text-amber-500' : 'text-red-500')}>
                {label === 'Healthy' ? '健康' : label === 'Degraded' ? '降级' : '异常'}
              </span>
            </div>
          );
        })}
        {health.length === 0 && (
          <p className="text-body-small text-md-on-surface-variant text-center py-4">暂无模块数据</p>
        )}
      </div>
    </div>
  );
}
