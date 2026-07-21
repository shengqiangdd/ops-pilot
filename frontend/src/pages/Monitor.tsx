import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { api } from '../api/client';
import type { HostMetrics, MetricPoint } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';

export function MonitorPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();
  const navigate = useNavigate();
  const [hostId, setHostId] = useState('');
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [metrics, setMetrics] = useState<HostMetrics | null>(null);
  const [history, setHistory] = useState<MetricPoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    api.listHosts(token).then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
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
        { label: 'CPU', value: `${metrics.cpu_percent.toFixed(1)}%`, color: 'text-md-primary' },
        { label: 'Memory', value: `${metrics.memory_percent.toFixed(1)}%`, color: 'text-green-600' },
        { label: 'Disk', value: `${metrics.disk_percent.toFixed(1)}%`, color: 'text-amber-600' },
        { label: 'Load', value: metrics.load_1.toFixed(2), color: 'text-md-tertiary' },
      ]
    : [];

  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">{t('title.monitor')}</h2>
        <div className="flex gap-2">
          <button
            onClick={() => navigate('/metrics')}
            className="border border-md-outline text-md-primary rounded-md-lg px-4 py-2.5 text-sm font-medium hover:bg-md-surface-container-high transition-colors"
          >
            {t('monitor.enhanced_view')}
          </button>
          <select value={hostId} onChange={(e) => setHostId(e.target.value)}
            className="bg-md-surface-container-highest rounded-md-sm px-4 py-2.5 border border-md-outline text-body-medium text-md-on-surface focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none">
            <option value="">{t('monitor.select_host')}</option>
            {hosts.map((h) => (<option key={h.id} value={h.id}>{h.name}</option>))}
          </select>
          <button onClick={collect} disabled={loading || !hostId}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50">
            {loading ? t('monitor.collecting') : t('monitor.collect')}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {stats.length > 0 && (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          {stats.map((s) => (
            <div key={s.label} className="bg-md-surface-container-low rounded-md-lg p-4 shadow-md-1">
              <div className={`text-headline-small font-medium ${s.color}`}>{s.value}</div>
              <div className="text-body-medium text-md-on-surface-variant">{s.label}</div>
            </div>
          ))}
        </div>
      )}

      {chartData.length > 0 && (
        <div className="bg-md-surface-container-low rounded-md-lg p-4 shadow-md-1">
          <h3 className="mb-3 text-title-medium font-medium text-md-on-surface">Time Series</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="time" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Line type="monotone" dataKey="value" stroke="var(--md-sys-color-primary)" dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {!metrics && !loading && (
        <div className="bg-md-surface-container-low rounded-md-lg p-8 text-center text-body-medium text-md-on-surface-variant shadow-md-1">
          Select a host and click "Collect Metrics" to start monitoring
        </div>
      )}
    </div>
  );
}
