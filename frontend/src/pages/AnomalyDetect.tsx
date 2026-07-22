import { useState, useEffect } from 'react';
import { useI18n } from '../i18n';

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
  const { t } = useI18n();
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
        setError(t('anomaly_detect.invalid_input'));
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
        throw new Error(data.error || t('anomaly_detect.detect_failed'));
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
      {/* Alert trends */}
      <div className="glass-card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-md-on-surface">{t('anomaly_detect.title')}</h2>
          <select
            value={days}
            onChange={e => setDays(Number(e.target.value))}
            className="px-3 py-1.5 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none"
          >
            <option value={7}>{t('anomaly_detect.days_7')}</option>
            <option value={14}>{t('anomaly_detect.days_14')}</option>
            <option value={30}>{t('anomaly_detect.days_30')}</option>
          </select>
        </div>

        {loading ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('anomaly_detect.loading')}</div>
        ) : trends.length === 0 ? (
          <div className="text-center py-8 text-md-on-surface-variant">{t('anomaly_detect.no_trend')}</div>
        ) : (
          <div className="space-y-2">
            {/* Bar chart */}
            <div className="flex items-end gap-1 h-40">
              {trends.map((tr, i) => (
                <div key={i} className="flex-1 flex flex-col items-center gap-1">
                  <div className="w-full flex flex-col gap-0.5" style={{ height: `${(tr.count / maxCount) * 120}px` }}>
                    <div
                      className="w-full bg-red-400 rounded-t"
                      style={{ height: `${tr.count > 0 ? (tr.critical / tr.count) * 100 : 0}%` }}
                      title={`${t('anomaly_detect.legend_critical')}: ${tr.critical}`}
                    />
                    <div
                      className="w-full bg-amber-400 rounded-b"
                      style={{ height: `${tr.count > 0 ? (tr.warning / tr.count) * 100 : 0}%` }}
                      title={`${t('anomaly_detect.legend_warning')}: ${tr.warning}`}
                    />
                  </div>
                  <span className="text-[10px] text-md-on-surface-variant font-mono">
                    {tr.date.slice(5)}
                  </span>
                </div>
              ))}
            </div>
            {/* Legend */}
            <div className="flex gap-4 text-xs text-md-on-surface-variant">
              <span><span className="inline-block w-3 h-3 bg-red-400 rounded mr-1" />{t('anomaly_detect.legend_critical')}</span>
              <span><span className="inline-block w-3 h-3 bg-amber-400 rounded mr-1" />{t('anomaly_detect.legend_warning')}</span>
            </div>
          </div>
        )}
      </div>

      {/* Custom data detection */}
      <div className="glass-card p-6">
        <h3 className="text-md font-semibold text-md-on-surface mb-4">{t('anomaly_detect.custom_title')}</h3>
        <div className="space-y-3">
          <input
            type="text"
            value={metricName}
            onChange={e => setMetricName(e.target.value)}
            placeholder={t('anomaly_detect.metric_name')}
            className="w-full px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <textarea
            value={customInput}
            onChange={e => setCustomInput(e.target.value)}
            placeholder={t('anomaly_detect.input_hint')}
            rows={3}
            className="w-full px-4 py-2 rounded-md-lg bg-md-surface border border-md-outline-variant text-md-on-surface text-sm font-mono focus:outline-none focus:ring-2 focus:ring-md-primary/50"
          />
          <button
            onClick={handleDetect}
            disabled={detecting}
            className="px-6 py-2 rounded-md-lg bg-md-primary text-md-on-primary font-medium hover:opacity-90 disabled:opacity-50 transition-all"
          >
            {detecting ? t('anomaly_detect.detecting') : t('anomaly_detect.detect_btn')}
          </button>
        </div>

        {error && (
          <div className="mt-4 p-3 rounded-md-lg bg-red-50 text-red-600 text-sm">{t('anomaly_detect.detect_failed')}: {error}</div>
        )}

        {detectResult && (
          <div className="mt-4 space-y-4">
            {/* Statistics */}
            <div className="p-3 rounded-md-lg bg-md-surface-container/30 grid grid-cols-4 gap-2 text-xs">
              <div><span className="text-md-on-surface-variant">{t('anomaly_detect.mean')}:</span> <span className="text-md-on-surface font-mono">{detectResult.stats.mean.toFixed(2)}</span></div>
              <div><span className="text-md-on-surface-variant">{t('anomaly_detect.std_dev')}:</span> <span className="text-md-on-surface font-mono">{detectResult.stats.std_dev.toFixed(2)}</span></div>
              <div><span className="text-md-on-surface-variant">{t('anomaly_detect.iqr')}:</span> <span className="text-md-on-surface font-mono">{detectResult.stats.iqr.toFixed(2)}</span></div>
              <div><span className="text-md-on-surface-variant">{t('anomaly_detect.method')}:</span> <span className="text-md-on-surface">{detectResult.method}</span></div>
            </div>

            {/* Anomaly points */}
            {detectResult.anomalies.length === 0 ? (
              <div className="p-3 rounded-md-lg bg-green-50 text-green-600 text-sm">{t('anomaly_detect.no_anomalies')}</div>
            ) : (
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-md-on-surface">
                  {t('anomaly_detect.found_anomalies').replace('{count}', String(detectResult.anomalies.length))}:
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

            {/* Data visualization */}
            <div className="p-3 rounded-md-lg bg-md-surface-container/30">
              <h4 className="text-xs font-medium text-md-on-surface-variant mb-2">{t('anomaly_detect.data_dist')}:</h4>
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
