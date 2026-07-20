import { useCallback, useEffect, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { api } from '../api/client';
import type { HostMetrics, MetricPoint } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { cn } from '../lib/cn';

export function MonitorPage() {
  const { token } = useAuthStore();
  const [hostId, setHostId] = useState('');
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [metrics, setMetrics] = useState<HostMetrics | null>(null);
  const [history, setHistory] = useState<MetricPoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    api.listHosts().then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
  }, [token]);

  const collect = useCallback(async () => {
    if (!token || !hostId) return;
    setLoading(true);
    setError(null);
    try {
      const m = await api.collectMonitorMetrics(token, hostId);
      setMetrics(m);
      const h = await api.getMonitorMetrics(token, hostId);
      setHistory(h);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Collect failed');
    } finally {
      setLoading(false);
    }
  }, [token, hostId]);

  const chartData = history.map((p) => ({
    time: new Date(p.timestamp).toLocaleTimeString(),
    value: p.value,
    metric: p.metric_type,
  }));

  const stats = metrics
    ? [
        { label: 'CPU', value: `${metrics.cpu_percent.toFixed(1)}%`, color: 'text-blue-600' },
        { label: '内存', value: `${metrics.memory_percent.toFixed(1)}%`, color: 'text-green-600' },
        { label: '磁盘', value: `${metrics.disk_percent.toFixed(1)}%`, color: 'text-orange-600' },
        { label: '负载', value: metrics.load_1.toFixed(2), color: 'text-purple-600' },
      ]
    : [];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">性能监控</h2>
        <div className="flex gap-2">
          <select value={hostId} onChange={(e) => setHostId(e.target.value)} className="rounded-md border border-gray-300 px-3 py-1.5 text-sm">
            <option value="">选择主机</option>
            {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
          </select>
          <button onClick={collect} disabled={loading || !hostId} className={cn('rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50')}>
            {loading ? '采集中...' : '采集指标'}
          </button>
        </div>
      </div>

      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      {stats.length > 0 && (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          {stats.map((s) => (
            <div key={s.label} className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm">
              <div className={cn('text-2xl font-bold', s.color)}>{s.value}</div>
              <div className="text-sm text-gray-500">{s.label}</div>
            </div>
          ))}
        </div>
      )}

      {chartData.length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm">
          <h3 className="mb-3 text-sm font-semibold text-gray-700">时序趋势</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="time" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Line type="monotone" dataKey="value" stroke="#3b82f6" dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {!metrics && !loading && (
        <div className="rounded-lg border border-gray-200 bg-white p-8 text-center text-sm text-gray-500 shadow-sm">
          选择主机并点击"采集指标"开始监控
        </div>
      )}
    </div>
  );
}
