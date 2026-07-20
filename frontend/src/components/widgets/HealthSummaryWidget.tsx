import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api/client';
import { getHealthLabel } from '../../lib/health';
import type { ModuleHealth } from '../../api/types';

export function HealthSummaryWidget() {
  const [health, setHealth] = useState<ModuleHealth[]>([]);

  const load = useCallback(async () => {
    try {
      const data = await api.getHealthAll();
      setHealth(data);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => { load(); const iv = setInterval(load, 30000); return () => clearInterval(iv); }, [load]);

  const healthy = health.filter(m => getHealthLabel(m.status) === 'Healthy').length;
  const degraded = health.filter(m => getHealthLabel(m.status) === 'Degraded').length;
  const unhealthy = health.filter(m => getHealthLabel(m.status) === 'Unhealthy').length;
  const total = health.length;

  return (
    <div className="grid grid-cols-4 gap-3 h-full items-center">
      {[
        { icon: '📦', label: '模块总数', value: total, color: 'text-md-primary', bg: 'from-primary/20 to-primary/5' },
        { icon: '✅', label: '健康', value: healthy, color: 'text-green-500', bg: 'from-green-500/20 to-green-500/5' },
        { icon: '⚠️', label: '降级', value: degraded, color: 'text-amber-500', bg: 'from-amber-500/20 to-amber-500/5' },
        { icon: '❌', label: '异常', value: unhealthy, color: 'text-red-500', bg: 'from-red-500/20 to-red-500/5' },
      ].map((item) => (
        <div key={item.label} className="text-center">
          <div className={`w-10 h-10 mx-auto rounded-md-lg bg-gradient-to-br ${item.bg} flex items-center justify-center text-lg mb-2`}>
            {item.icon}
          </div>
          <p className={`text-headline-small font-bold tabular-nums ${item.color}`}>{item.value}</p>
          <p className="text-label-small text-md-on-surface-variant">{item.label}</p>
        </div>
      ))}
    </div>
  );
}
