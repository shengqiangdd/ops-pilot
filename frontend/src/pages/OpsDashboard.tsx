import { useEffect, useState } from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar, Legend } from 'recharts';
import { api } from '../api/client';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

// Generate mock metrics data
function generateMetrics(points: number = 24): { time: string; cpu: number; memory: number; disk: number }[] {
  const data = [];
  const now = Date.now();
  for (let i = 0; i < points; i++) {
    const t = new Date(now - (points - i) * 60000 * 15); // 15 min intervals
    data.push({
      time: t.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      cpu: 35 + Math.sin(i / 5) * 20 + (Math.random() - 0.5) * 10,
      memory: 55 + Math.sin(i / 3) * 15 + (Math.random() - 0.5) * 8,
      disk: 70 + (Math.random() - 0.5) * 5,
    });
  }
  return data;
}

// Generate mock alert timeline data
function generateAlerts(): { time: string; title: string; severity: string }[] {
  const now = Date.now();
  return [
    { time: new Date(now - 300000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), title: 'CPU spike detected', severity: 'warning' },
    { time: new Date(now - 600000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), title: 'Memory threshold exceeded', severity: 'critical' },
    { time: new Date(now - 900000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), title: 'Service health check passed', severity: 'ok' },
    { time: new Date(now - 1200000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), title: 'Deployment completed', severity: 'ok' },
    { time: new Date(now - 1500000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), title: 'Network latency warning', severity: 'warning' },
  ];
}

// Generate mock audit logs
function generateAuditLogs(): { time: string; action: string; user: string }[] {
  const now = Date.now();
  return [
    { time: new Date(now - 120000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), action: 'SSH connect', user: 'admin' },
    { time: new Date(now - 300000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), action: 'Config update', user: 'devops' },
    { time: new Date(now - 480000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), action: 'Host added', user: 'admin' },
    { time: new Date(now - 600000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }), action: 'Security scan', user: 'system' },
  ];
}

// Mock alert trend data
function generateAlertTrend(): { time: string; alerts: number; resolved: number }[] {
  const data = [];
  const now = Date.now();
  for (let i = 23; i >= 0; i--) {
    const t = new Date(now - i * 3600000);
    data.push({
      time: t.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      alerts: Math.floor(Math.random() * 20 + 5),
      resolved: Math.floor(Math.random() * 15 + 3),
    });
  }
  return data;
}

export function OpsDashboard() {
  const { t: _t } = useI18n();
  const [hosts, setHosts] = useState<any[]>([]);
  const [_loading, setLoading] = useState(true);

  // Mock data
  const metricData = generateMetrics(24);
  const alerts = generateAlerts();
  const auditLogs = generateAuditLogs();
  const alertTrend = generateAlertTrend();

  // Load hosts
  useEffect(() => {
    const token = useAuthStore.getState().token;
    if (token) {
      api.listHosts(token)
        .then((h) => setHosts(h))
        .catch(() => {});
    }
  }, []);

  // Simulate loading
  useEffect(() => {
    const timer = setTimeout(() => setLoading(false), 500);
    return () => clearTimeout(timer);
  }, []);

  // Mock stats
  const totalHosts = hosts.length || 5;
  const onlineHosts = hosts.filter(h => h.status === 'online').length || 4;
  const onlineRate = totalHosts > 0 ? Math.round((onlineHosts / totalHosts) * 100) : 95;
  const alertCount = 12;
  const changesToday = 8;

  const statCards = [
    { label: '总主机数', value: totalHosts, icon: '🖥️', color: 'from-blue-500/20 to-blue-600/5', textColor: 'text-blue-500' },
    { label: '在线率', value: `${onlineRate}%`, icon: '✅', color: 'from-green-500/20 to-green-600/5', textColor: 'text-green-500' },
    { label: '活跃告警', value: alertCount, icon: '🔔', color: 'from-amber-500/20 to-amber-600/5', textColor: 'text-amber-500' },
    { label: '今日变更', value: changesToday, icon: '📝', color: 'from-purple-500/20 to-purple-600/5', textColor: 'text-purple-500' },
  ];

  const severityColors: Record<string, string> = {
    ok: 'bg-green-500',
    warning: 'bg-amber-500',
    critical: 'bg-red-500',
  };

  return (
    <div className="min-h-[calc(100vh-4rem)] p-6" style={{ background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' }}>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white">
            <span className="bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-purple-500">OpsPilot</span>
            <span className="text-gray-400 text-lg ml-3 font-normal">总览大屏</span>
          </h1>
          <p className="text-gray-500 text-sm mt-1">实时运维监控 · 智能分析 · 一键运维</p>
        </div>
        <div className="flex items-center gap-2 text-gray-400 text-sm">
          <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
          <span>系统正常运行</span>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        {statCards.map((stat) => (
          <div key={stat.label} className="rounded-xl p-5 border border-gray-700/50 backdrop-blur-sm" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
            <div className="flex items-start justify-between">
              <div>
                <p className="text-gray-400 text-sm mb-1">{stat.label}</p>
                <p className={cn('text-3xl font-bold', stat.textColor)}>{stat.value}</p>
              </div>
              <div className={cn('w-12 h-12 rounded-xl flex items-center justify-center text-2xl bg-gradient-to-br', stat.color)}>
                {stat.icon}
              </div>
            </div>
            <div className="mt-3 h-1 bg-gray-700 rounded-full overflow-hidden">
              <div
                className={cn('h-full rounded-full transition-all duration-1000', stat.textColor.replace('text', 'bg'))}
                style={{ width: typeof stat.value === 'string' ? stat.value : `${Math.min((stat.value as number / 10) * 100, 100)}%` }}
              />
            </div>
          </div>
        ))}
      </div>

      {/* Charts + Alerts Timeline */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Trend Charts - 60% */}
        <div className="lg:col-span-2 space-y-4">
          {/* CPU/Memory/Disk Trend */}
          <div className="rounded-xl p-5 border border-gray-700/50 backdrop-blur-sm" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
            <h3 className="text-white font-semibold mb-4">资源趋势 (24h)</h3>
            <ResponsiveContainer width="100%" height={280}>
              <AreaChart data={metricData}>
                <defs>
                  <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#6750A4" stopOpacity={0.4} />
                    <stop offset="100%" stopColor="#6750A4" stopOpacity={0.05} />
                  </linearGradient>
                  <linearGradient id="memGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#7D5260" stopOpacity={0.4} />
                    <stop offset="100%" stopColor="#7D5260" stopOpacity={0.05} />
                  </linearGradient>
                  <linearGradient id="diskGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#B3261E" stopOpacity={0.4} />
                    <stop offset="100%" stopColor="#B3261E" stopOpacity={0.05} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="time" tick={{ fontSize: 11, fill: '#9CA3AF' }} />
                <YAxis tick={{ fontSize: 11, fill: '#9CA3AF' }} domain={[0, 100]} />
                <Tooltip
                  contentStyle={{ background: '#1e293b', border: '1px solid #374151', borderRadius: '8px', color: '#e5e7eb' }}
                />
                <Area type="monotone" dataKey="cpu" stroke="#6750A4" fill="url(#cpuGradient)" strokeWidth={2} name="CPU %" />
                <Area type="monotone" dataKey="memory" stroke="#7D5260" fill="url(#memGradient)" strokeWidth={2} name="内存 %" />
                <Area type="monotone" dataKey="disk" stroke="#B3261E" fill="url(#diskGradient)" strokeWidth={2} name="磁盘 %" />
                <Legend />
              </AreaChart>
            </ResponsiveContainer>
          </div>

          {/* Alert Trend */}
          <div className="rounded-xl p-5 border border-gray-700/50 backdrop-blur-sm" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
            <h3 className="text-white font-semibold mb-4">告警趋势 (24h)</h3>
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={alertTrend}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="time" tick={{ fontSize: 11, fill: '#9CA3AF' }} />
                <YAxis tick={{ fontSize: 11, fill: '#9CA3AF' }} />
                <Tooltip contentStyle={{ background: '#1e293b', border: '1px solid #374151', borderRadius: '8px', color: '#e5e7eb' }} />
                <Bar dataKey="alerts" fill="#B3261E" radius={[4, 4, 0, 0]} name="告警" />
                <Bar dataKey="resolved" fill="#386A20" radius={[4, 4, 0, 0]} name="已解决" />
                <Legend />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Right side - Alerts + Audit Logs - 40% */}
        <div className="space-y-4">
          {/* Alert Timeline */}
          <div className="rounded-xl p-5 border border-gray-700/50 backdrop-blur-sm" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
            <h3 className="text-white font-semibold mb-4">告警时间线</h3>
            <div className="space-y-3">
              {alerts.map((alert, i) => (
                <div key={i} className="flex items-start gap-3">
                  <div className={cn('h-2.5 w-2.5 rounded-full mt-1.5 shrink-0', severityColors[alert.severity] || 'bg-gray-500')} />
                  <div className="flex-1">
                    <p className="text-gray-200 text-sm">{alert.title}</p>
                    <p className="text-gray-500 text-xs">{alert.time}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Audit Logs */}
          <div className="rounded-xl p-5 border border-gray-700/50 backdrop-blur-sm" style={{ background: 'rgba(30, 41, 59, 0.8)' }}>
            <h3 className="text-white font-semibold mb-4">最近操作</h3>
            <div className="space-y-3">
              {auditLogs.map((log, i) => (
                <div key={i} className="flex items-center gap-3 text-sm">
                  <span className="text-gray-500 w-16 shrink-0">{log.time}</span>
                  <span className="text-gray-300 flex-1 truncate">{log.action}</span>
                  <span className="text-gray-500">{log.user}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
