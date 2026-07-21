import { useState, useEffect } from 'react';

interface AlertTrend {
  date: string;
  count: number;
  critical: number;
  warning: number;
}

interface AnomalyPoint {
  index: number;
  value: number;
  reason: string;
}

interface DetectResult {
  metric: string;
  method: string;
  anomalies: AnomalyPoint[];
  stats: {
    mean: number;
    std_dev: number;
    min: number;
    max: number;
    count: number;
    q1: number;
    q3: number;
    iqr: number;
  };
}

export function AnomalyDetectPage() {
  const [trends, setTrends] = useState<AlertTrend[]>([]);
  const [loading, setLoading] = useState(true);
  const [days, setDays] = useState(7);
  const [customInput, setCustomInput] = useState('');
  const [metricName, setMetricName] = useState('custom');
  const [detectResult, setDetectResult] = useState<DetectResult | null>(null);
  const [detecting, setDetecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchTrends = async () => {
    setLoading(true);
    try {
      const resp = await fetch(`/api/anomaly/alert-trends?days=${days}`);
      const data = await resp.json();
      setTrends(data);
    } catch (e: any) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => { fetchTrends(); }, [days]);

  const handleDetect = async () => {
    setDetecting(true);
    setError(null);
    setDetectResult(null);
    try {
      const values = customInput.split(',').map(v => parseFloat(v.trim())).filter(v => !isNaN(v));
      if (values.length === 0) {
        setError('请输入有效的数值（逗号分隔）');
        setDetecting(false);
        return;
      }
      const resp = await fetch('/api/anomaly/detect', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ metric: metricName, values }),
      });
      if (!resp.ok) {
        const data = await resp.json();
        throw new Error(data.error || '检测失败');
      }
      setDetectResult(await resp.json());
    } catch (e: any) {
      setError(e.message);
    }
    setDetecting(false);
  };

  const maxCount = Math.max(...trends.map(t => t.count), 1);

  return (
    <div className="space-y-6">
      {/* 告警趋势 */}
      <div className="glass-card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-md-on-surface">异常检测</h2>
          <select
            value={days}
            onChange={e => setDays(Number(e.target.value))}
            className="px-3 py-1.5 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none"
          >
            <option value={7}>最近 7 天</option>
            <option value={14}>最近 14 天</option>
            <option value={30}>最近 30 天</option>
          </select>
        </div>

        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">加载中...</div>
        ) : trends.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">暂无告警趋势数据</div>
        ) : (
          <div className="space-y-2">
            {/* 条形图 */}
            <div className="flex items-end gap-1 h-40">
              {trends.map((t, i) => (
                <div key={i} className="flex-1 flex flex-col items-center gap-1">
                  <div className="w-full flex flex-col gap-0.5" style={{ height: `${(t.count / maxCount) * 120}px` }}>
                    <div
                      className="w-full bg-red-400 rounded-t"
                      style={{ height: `${t.count > 0 ? (t.critical / t.count) * 100 : 0}%` }}
                      title={`Critical: ${t.critical}`}
                    />
                    <div
                      className="w-full bg-amber-400 rounded-b"
                      style={{ height: `${t.count > 0 ? (t.warning / t.count) * 100 : 0}%` }}
                      title={`Warning: ${t.warning}`}
                    />
                  </div>
                  <span className="text-[10px] text-md-on-surface-variant font-mono">
                    {t.date.slice(5)}
                  </span>
                </div>
              ))}
            </div>
            {/* 图例 */}
            <div className="flex gap-4 text-xs text-md-on-surface-variant">
              <span><span className="inline-block w-3 h-3 bg-red-400 rounded mr-1" />Critical</span>
              <span><span className="inline-block w-3 h-3 bg-amber-400 rounded mr-1" />Warning</span>
            </div>
          </div>
        )}
      </div>

      {/* 自定义数据检测 */}
      <div className="glass-card p-6">
        <h3 className="text-md font-semibold text-md-on-surface mb-4">自定义数据异常检测</h3>
        <div className="space-y-3">
          <input
            type="text"
            value={metricName}
            onChange={e => setMetricName(e.target.value)}
            placeholder="指标名称"
            className="w-full px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <textarea
            value={customInput}
            onChange={e => setCustomInput(e.target.value)}
            placeholder="输入数值，逗号分隔，例如: 10, 12, 11, 100, 13, 12, 11, 10"
            rows={3}
            className="w-full px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm font-mono focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <button
            onClick={handleDetect}
            disabled={detecting}
            className="px-6 py-2 rounded-md-lg bg-md-primary text-md-on-primary font-medium hover:opacity-90 disabled:opacity-50 transition-all"
          >
            {detecting ? '检测中...' : '开始检测'}
          </button>
        </div>

        {error && (
          <div className="mt-4 p-3 rounded-md-lg bg-red-50 text-red-600 text-sm">❌ {error}</div>
        )}

        {detectResult && (
          <div className="mt-4 space-y-4">
            {/* 统计信息 */}
            <div className="p-3 rounded-md-lg bg-md-surface-container/30 grid grid-cols-4 gap-2 text-xs">
              <div><span className="text-md-on-surface-variant">均值:</span> <span className="text-md-on-surface font-mono">{detectResult.stats.mean.toFixed(2)}</span></div>
              <div><span className="text-md-on-surface-variant">标准差:</span> <span className="text-md-on-surface font-mono">{detectResult.stats.std_dev.toFixed(2)}</span></div>
              <div><span className="text-md-on-surface-variant">IQR:</span> <span className="text-md-on-surface font-mono">{detectResult.stats.iqr.toFixed(2)}</span></div>
              <div><span className="text-md-on-surface-variant">方法:</span> <span className="text-md-on-surface">{detectResult.method}</span></div>
            </div>

            {/* 异常点 */}
            {detectResult.anomalies.length === 0 ? (
              <div className="p-3 rounded-md-lg bg-green-50 text-green-600 text-sm">✅ 未发现异常点</div>
            ) : (
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-md-on-surface">
                  发现 {detectResult.anomalies.length} 个异常点:
                </h4>
                {detectResult.anomalies.map((a, i) => (
                  <div key={i} className="flex items-center gap-3 p-2 rounded-md-lg bg-red-50 border border-red-200">
                    <span className="text-xs text-red-600 font-mono">index: {a.index}</span>
                    <span className="text-sm text-red-700 font-medium">{a.value}</span>
                    <span className="text-xs text-red-600">{a.reason}</span>
                  </div>
                ))}
              </div>
            )}

            {/* 数据可视化 */}
            <div className="p-3 rounded-md-lg bg-md-surface-container/30">
              <h4 className="text-xs font-medium text-md-on-surface-variant mb-2">数据分布:</h4>
              <div className="flex items-end gap-0.5 h-20">
                {customInput.split(',').map((v, i) => {
                  const num = parseFloat(v.trim());
                  if (isNaN(num)) return null;
                  const isAnomaly = detectResult.anomalies.some(a => a.index === i);
                  const height = detectResult.stats.max > detectResult.stats.min
                    ? ((num - detectResult.stats.min) / (detectResult.stats.max - detectResult.stats.min)) * 60 + 10
                    : 30;
                  return (
                    <div
                      key={i}
                      className={`flex-1 rounded-t ${isAnomaly ? 'bg-red-500' : 'bg-md-primary/60'}`}
                      style={{ height: `${height}px` }}
                      title={`${num}`}
                    />
                  );
                })}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
