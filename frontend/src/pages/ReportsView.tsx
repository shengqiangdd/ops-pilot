import { useState, useEffect } from 'react';

interface OpsReport {
  id: string;
  title: string;
  report_type: string;
  period_start: string;
  period_end: string;
  generated_at: string;
  summary: {
    total_alerts: number;
    critical_alerts: number;
    resolved_incidents: number;
    active_incidents: number;
    total_hosts: number;
    healthy_hosts: number;
    total_vulnerabilities: number;
    sla_achievement: number;
  };
  sections: Record<string, any>;
}

export function ReportsViewPage() {
  const [reports, setReports] = useState<OpsReport[]>([]);
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);

  const fetchReports = async () => {
    try {
      const resp = await fetch('/api/reports/list');
      const data = await resp.json();
      if (data.status === 'ok') {
        setReports(data.reports || []);
      }
    } catch { /* ignore */ }
    setLoading(false);
  };

  const generateReport = async () => {
    setGenerating(true);
    try {
      await fetch('/api/reports/generate', { method: 'POST' });
      await fetchReports();
    } catch { /* ignore */ }
    setGenerating(false);
  };

  const downloadReport = async (id: string) => {
    const resp = await fetch(`/api/reports/download/${id}`);
    const data = await resp.json();
    if (data.status === 'ok') {
      const blob = new Blob([JSON.stringify(data.report, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `report-${id.slice(0, 8)}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  useEffect(() => { fetchReports(); }, []);

  return (
    <div className="p-6 max-w-5xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-md-on-surface mb-1">📊 运维报告</h1>
          <p className="text-md-on-surface-variant">自动生成的运维日报，包含系统健康、告警、资源趋势</p>
        </div>
        <button
          onClick={generateReport}
          disabled={generating}
          className="px-5 py-2.5 bg-md-primary text-md-on-primary rounded-md-lg hover:opacity-90 disabled:opacity-50 font-medium transition-all"
        >
          {generating ? '生成中...' : '📝 生成日报'}
        </button>
      </div>

      {loading ? (
        <div className="text-center py-12 text-md-on-surface-variant">加载中...</div>
      ) : reports.length === 0 ? (
        <div className="glass-card p-12 text-center text-md-on-surface-variant rounded-md-xl">
          <div className="text-4xl mb-3">📊</div>
          <p>暂无报告，点击上方按钮生成第一份日报</p>
        </div>
      ) : (
        <div className="grid gap-4">
          {reports.map(report => (
            <div key={report.id} className="glass-card p-5 rounded-md-xl hover:shadow-md transition-shadow">
              <div className="flex items-start justify-between mb-3">
                <div>
                  <h3 className="text-lg font-semibold text-md-on-surface">{report.title}</h3>
                  <p className="text-sm text-md-on-surface-variant">
                    {new Date(report.generated_at).toLocaleString('zh-CN')} · {report.report_type}
                  </p>
                </div>
                <button
                  onClick={() => downloadReport(report.id)}
                  className="px-3 py-1.5 text-sm bg-md-surface-variant/50 hover:bg-md-surface-variant rounded-md-lg transition-colors"
                >
                  📥 下载
                </button>
              </div>

              {/* Summary cards */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <SummaryCard label="告警总数" value={report.summary.total_alerts} color="text-orange-500" />
                <SummaryCard label="严重告警" value={report.summary.critical_alerts} color="text-red-500" />
                <SummaryCard label="已解决事故" value={report.summary.resolved_incidents} color="text-green-500" />
                <SummaryCard label="在线主机" value={`${report.summary.healthy_hosts}/${report.summary.total_hosts}`} color="text-blue-500" />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function SummaryCard({ label, value, color }: { label: string; value: string | number; color: string }) {
  return (
    <div className="bg-md-surface-variant/20 p-3 rounded-md-lg">
      <p className="text-xs text-md-on-surface-variant mb-1">{label}</p>
      <p className={`text-xl font-bold ${color}`}>{value}</p>
    </div>
  );
}
