import { useCallback, useEffect, useMemo, useState } from 'react';
import { api } from '../api/client';
import { MetricGrid } from '../components/MetricGrid';
import { generateAllMetrics, type MetricSeries } from '../lib/metrics';
import { useAuthStore } from '../stores/useAuthStore';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';
import { LoadingState, ErrorState } from '../lib/pageStates';

type TimeRange = '1h' | '6h' | '24h' | '7d';
type RefreshInterval = 5 | 15 | 30 | 0;

const TIME_RANGES: { value: TimeRange; label: string; minutes: number }[] = [
  { value: '1h', label: '1H', minutes: 60 },
  { value: '6h', label: '6H', minutes: 360 },
  { value: '24h', label: '24H', minutes: 1440 },
  { value: '7d', label: '7D', minutes: 10080 },
];

const REFRESH_INTERVALS: { value: RefreshInterval; label: string }[] = [
  { value: 5, label: '5s' },
  { value: 15, label: '15s' },
  { value: 30, label: '30s' },
  { value: 0, label: 'Off' },
];

export function MetricsVizPage() {
  const { token } = useAuthStore();
  const { t } = useI18n();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hosts, setHosts] = useState<Array<{ id: string; name: string }>>([]);
  const [selectedHost, setSelectedHost] = useState('');
  const [timeRange, setTimeRange] = useState<TimeRange>('1h');
  const [refreshInterval, setRefreshInterval] = useState<RefreshInterval>(15);
  const [metrics, setMetrics] = useState<MetricSeries[]>([]);

  // Load hosts
  useEffect(() => {
    if (!token) return;
    api.listHosts(token)
      .then((h) => { setHosts(h.map((x) => ({ id: x.id, name: x.name }))); setLoading(false); })
      .catch((e) => { setError(e?.message || 'Failed to load'); setLoading(false); });
  }, [token]);

  // Generate metrics data
  const loadMetrics = useCallback(() => {
    const range = TIME_RANGES.find((r) => r.value === timeRange);
    const minutes = range?.minutes || 60;
    // Use interval based on time range
    const interval = minutes <= 60 ? 30 : minutes <= 360 ? 300 : 1800;
    const data = generateAllMetrics(minutes, interval);
    setMetrics(data);
  }, [timeRange]);

  // Initial load and refresh
  useEffect(() => {
    loadMetrics();
  }, [loadMetrics]);

  // Auto-refresh
  useEffect(() => {
    if (refreshInterval === 0) return;
    const interval = setInterval(loadMetrics, refreshInterval * 1000);
    return () => clearInterval(interval);
  }, [refreshInterval, loadMetrics]);

  // Thresholds for alerts
  const thresholds = useMemo(() => ({
    'CPU %': 80,
    'Memory %': 85,
    'Disk %': 90,
  }), []);

  if (loading) return <LoadingState skeleton="chart" />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;

  return (
    <div className="space-y-4 animate-slide-up">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-headline-small md:text-headline-medium font-medium text-md-on-surface">
          {t('title.metrics_viz')}
        </h2>
      </div>

      {/* Controls */}
      <div className="glass-card rounded-md-xl p-4">
        <div className="flex flex-wrap items-center gap-4">
          {/* Host Selector */}
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('metrics.host')}</label>
            <select
              value={selectedHost}
              onChange={(e) => setSelectedHost(e.target.value)}
              className="bg-md-surface-container-highest rounded-md-sm px-3 py-2 text-sm border border-md-outline focus:border-md-primary outline-none text-md-on-surface"
            >
              <option value="">{t('metrics.select_host')}</option>
              {hosts.map((h) => (
                <option key={h.id} value={h.id}>{h.name}</option>
              ))}
            </select>
          </div>

          {/* Time Range */}
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('metrics.time_range')}</label>
            <div className="flex gap-1">
              {TIME_RANGES.map((range) => (
                <button
                  key={range.value}
                  onClick={() => setTimeRange(range.value)}
                  className={cn(
                    'px-3 py-1.5 text-sm rounded-md-sm transition-colors',
                    timeRange === range.value
                      ? 'bg-md-primary text-md-on-primary'
                      : 'bg-md-surface-container-high text-md-on-surface-variant hover:bg-md-surface-container-highest',
                  )}
                >
                  {range.label}
                </button>
              ))}
            </div>
          </div>

          {/* Refresh Interval */}
          <div>
            <label className="block text-label-medium text-md-on-surface-variant mb-1">{t('metrics.refresh')}</label>
            <div className="flex gap-1">
              {REFRESH_INTERVALS.map((interval) => (
                <button
                  key={interval.value}
                  onClick={() => setRefreshInterval(interval.value)}
                  className={cn(
                    'px-3 py-1.5 text-sm rounded-md-sm transition-colors',
                    refreshInterval === interval.value
                      ? 'bg-md-primary text-md-on-primary'
                      : 'bg-md-surface-container-high text-md-on-surface-variant hover:bg-md-surface-container-highest',
                  )}
                >
                  {interval.label}
                </button>
              ))}
            </div>
          </div>

          {/* Refresh Now */}
          <div className="ml-auto">
            <button
              onClick={loadMetrics}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-md-lg border border-md-outline text-md-primary hover:bg-md-surface-container-high transition-colors"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              {t('metrics.refresh_now')}
            </button>
          </div>
        </div>
      </div>

      {/* Metric Grid */}
      {metrics.length > 0 ? (
        <MetricGrid
          metrics={metrics}
          thresholds={thresholds}
          title={t('metrics.dashboard_title')}
        />
      ) : (
        <div className="glass-card rounded-md-xl p-8 text-center text-body-medium text-md-on-surface-variant">
          {t('metrics.no_data')}
        </div>
      )}

      {/* Legend */}
      <div className="glass-card rounded-md-xl p-4">
        <h4 className="text-title-small font-semibold text-md-on-surface mb-2">{t('metrics.thresholds')}</h4>
        <div className="flex flex-wrap gap-4 text-sm">
          <div className="flex items-center gap-2">
            <div className="w-3 h-0.5 bg-red-500" />
            <span className="text-md-on-surface-variant">{t('metrics.threshold_cpu')}: 80%</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-0.5 bg-red-500" />
            <span className="text-md-on-surface-variant">{t('metrics.threshold_memory')}: 85%</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-0.5 bg-red-500" />
            <span className="text-md-on-surface-variant">{t('metrics.threshold_disk')}: 90%</span>
          </div>
        </div>
      </div>
    </div>
  );
}
