import { useCallback, useEffect, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ReferenceLine, Legend } from 'recharts';
import { api } from '../api/client';
import type { PredictionResult, RiskItem } from '../api/types';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

export function PredictionsPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [risks, setRisks] = useState<RiskItem[]>([]);
  const [selectedHost, setSelectedHost] = useState('');
  const [selectedMetric, setSelectedMetric] = useState('cpu');
  const [prediction, setPrediction] = useState<PredictionResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);

  useEffect(() => {
    if (!token) return;
    api.listHosts(token)
      .then((h) => setHosts(h.map((x) => ({ id: x.id, name: x.name }))))
      .catch(() => {});
  }, [token]);

  const loadRisks = useCallback(async () => {
    if (!token) return;
    try {
      const data = await api.getPredictionRisks(token);
      setRisks(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load risks');
    }
  }, [token]);

  useEffect(() => { loadRisks(); }, [loadRisks]);

  const handleAnalyze = async () => {
    if (!token || !selectedHost) return;
    setLoading(true);
    setError(null);
    try {
      const data = await api.analyzePrediction(token, {
        host_id: selectedHost,
        metric_type: selectedMetric,
        forecast_hours: 24,
        threshold: selectedMetric === 'disk' ? 90 : selectedMetric === 'memory' ? 95 : 80,
      });
      setPrediction(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to analyze');
    } finally {
      setLoading(false);
    }
  };

  const trendIcon = (trend: string) => {
    switch (trend) {
      case 'up': return '📈';
      case 'down': return '📉';
      default: return '➡️';
    }
  };

  const riskColor = (level: string) => {
    switch (level) {
      case 'critical': return 'bg-red-500/10 text-red-600 dark:text-red-400 border-red-500/30';
      case 'warning': return 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30';
      default: return 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/30';
    }
  };

  const chartData = prediction?.data_points.map((dp) => ({
    time: new Date(dp.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    actual: dp.actual,
    predicted: dp.predicted,
  })) || [];


  if (loading) return <LoadingState skeleton="chart" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  return (
    <div className="space-y-4 animate-slide-up">
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.predictions')}
        </h2>
      </div>

      {error && (
        <div className="bg-md-error-container text-md-on-error-container rounded-md-sm px-4 py-3 text-body-medium">{error}</div>
      )}

      {/* Analysis Controls */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-end gap-3">
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('predictions.host')}</label>
            <select value={selectedHost} onChange={(e) => setSelectedHost(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface">
              <option value="">{t('predictions.select_host')}</option>
              {hosts.map((h) => <option key={h.id} value={h.id}>{h.name}</option>)}
            </select>
          </div>
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('predictions.metric')}</label>
            <select value={selectedMetric} onChange={(e) => setSelectedMetric(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface">
              <option value="cpu">CPU</option>
              <option value="memory">Memory</option>
              <option value="disk">Disk</option>
              <option value="network">Network</option>
            </select>
          </div>
          <button onClick={handleAnalyze} disabled={loading || !selectedHost}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-4 py-2 text-sm font-medium hover:shadow-md-2 active:scale-[0.97] transition-all disabled:opacity-50 flex items-center gap-2">
            {loading ? (
              <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
            ) : '🔮'}
            {loading ? t('predictions.analyzing') : t('predictions.analyze')}
          </button>
        </div>
      </div>

      {/* Prediction Results */}
      {prediction && (
        <div className="space-y-4">
          {/* Summary Cards */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
            <div className="glass-card rounded-md-xl p-4 text-center">
              <p className="text-headline-medium font-bold text-md-primary">{prediction.current_value.toFixed(1)}%</p>
              <p className="text-label-small text-md-on-surface-variant">{t('predictions.current')}</p>
            </div>
            <div className="glass-card rounded-md-xl p-4 text-center">
              <p className="text-headline-medium font-bold text-md-tertiary">{prediction.predicted_value.toFixed(1)}%</p>
              <p className="text-label-small text-md-on-surface-variant">{t('predictions.predicted')}</p>
            </div>
            <div className="glass-card rounded-md-xl p-4 text-center">
              <p className="text-2xl">{trendIcon(prediction.trend)}</p>
              <p className="text-label-small text-md-on-surface-variant">{t('predictions.trend')}: {prediction.trend}</p>
            </div>
            <div className={cn('glass-card rounded-md-xl p-4 text-center border', riskColor(prediction.risk_level))}>
              <p className="text-body-large font-bold">{prediction.risk_level}</p>
              <p className="text-label-small text-md-on-surface-variant">{t('predictions.risk')}</p>
            </div>
          </div>

          {/* Chart */}
          <div className="glass-card rounded-md-xl p-4">
            <h3 className="text-title-medium font-semibold text-md-on-surface mb-3">
              {t('predictions.trend_chart')}
            </h3>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--md-sys-color-outline-variant)" />
                <XAxis dataKey="time" tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }} />
                <YAxis tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }} />
                <Tooltip
                  contentStyle={{ background: 'var(--md-sys-color-surface-container)', border: '1px solid var(--md-sys-color-outline-variant)', borderRadius: '8px' }}
                />
                <Legend />
                <ReferenceLine y={prediction.risk_level === 'critical' ? 90 : 80} stroke="#B3261E" strokeDasharray="5 5" label={{ value: 'Threshold', fill: '#B3261E', fontSize: 11 }} />
                <Line type="monotone" dataKey="actual" stroke="var(--md-sys-color-primary)" strokeWidth={2} dot={false} name="Actual" />
                <Line type="monotone" dataKey="predicted" stroke="var(--md-sys-color-tertiary)" strokeWidth={2} strokeDasharray="5 5" dot={false} name="Predicted" />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {/* Confidence & Info */}
          <div className="glass-card rounded-md-xl p-4 flex items-center gap-6">
            <div>
              <p className="text-label-small text-md-on-surface-variant">{t('predictions.confidence')}</p>
              <div className="flex items-center gap-2">
                <div className="w-32 h-2 bg-md-surface-container-highest rounded-full overflow-hidden">
                  <div className="h-full bg-md-primary rounded-full" style={{ width: `${prediction.confidence * 100}%` }} />
                </div>
                <span className="text-body-medium font-medium text-md-on-surface">{(prediction.confidence * 100).toFixed(0)}%</span>
              </div>
            </div>
            {prediction.estimated_time_to_threshold_hours !== null && prediction.estimated_time_to_threshold_hours !== undefined && (
              <div>
                <p className="text-label-small text-md-on-surface-variant">{t('predictions.time_to_threshold')}</p>
                <p className="text-body-medium font-medium text-md-on-surface">
                  {prediction.estimated_time_to_threshold_hours < 1
                    ? t('predictions.less_than_hour')
                    : `${prediction.estimated_time_to_threshold_hours.toFixed(1)} ${t('predictions.hours')}`}
                </p>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Risk List */}
      {risks.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-title-medium font-semibold text-md-on-surface">{t('predictions.risks')}</h3>
          {risks.map((risk) => (
            <div key={`${risk.host_id}-${risk.metric_type}`} className={cn('glass-card rounded-md-xl p-4 border', riskColor(risk.risk_level))}>
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  <span className="text-lg">{risk.metric_type === 'disk' ? '💿' : risk.metric_type === 'memory' ? '💾' : '🖥️'}</span>
                  <div>
                    <span className="text-body-medium font-medium text-md-on-surface">{risk.host_name}</span>
                    <span className="text-body-small text-md-on-surface-variant ml-2">· {risk.metric_type}</span>
                  </div>
                </div>
                <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-md-surface-container-highest">{risk.risk_level}</span>
              </div>
              <div className="grid grid-cols-3 gap-4 text-sm mb-2">
                <div>
                  <p className="text-label-small text-md-on-surface-variant">{t('predictions.current')}</p>
                  <p className="text-body-medium font-medium">{risk.current_value.toFixed(1)}%</p>
                </div>
                <div>
                  <p className="text-label-small text-md-on-surface-variant">{t('predictions.predicted')}</p>
                  <p className="text-body-medium font-medium">{risk.predicted_value.toFixed(1)}%</p>
                </div>
                <div>
                  <p className="text-label-small text-md-on-surface-variant">{t('predictions.time_to_threshold')}</p>
                  <p className="text-body-medium font-medium">
                    {risk.estimated_time_hours < 1 ? t('predictions.less_than_hour') : `${risk.estimated_time_hours.toFixed(0)}h`}
                  </p>
                </div>
              </div>
              <p className="text-body-small text-md-on-surface-variant">💡 {risk.suggestion}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
