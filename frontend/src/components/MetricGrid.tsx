import { useState } from 'react';
import { MetricChart } from './MetricChart';
import type { MetricSeries } from '../lib/metrics';
import { calculateStats } from '../lib/metrics';

interface MetricGridProps {
  metrics: MetricSeries[];
  thresholds?: Record<string, number>;
  title?: string;
}

export function MetricGrid({ metrics, thresholds = {}, title }: MetricGridProps) {
  const [fullscreen, setFullscreen] = useState<string | null>(null);

  const fullscreenMetric = metrics.find((m) => m.name === fullscreen);

  return (
    <>
      <div className="space-y-4">
        {title && (
          <div className="flex items-center justify-between">
            <h3 className="text-title-medium font-semibold text-md-on-surface">{title}</h3>
          </div>
        )}

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {metrics.map((metric) => {
            const threshold = thresholds[metric.name];
            return (
              <div
                key={metric.name}
                className="cursor-pointer hover:shadow-md-2 transition-shadow"
                onClick={() => setFullscreen(metric.name)}
              >
                <MetricChart
                  data={metric.data}
                  title={metric.name}
                  color={metric.color}
                  unit={metric.name.includes('%') ? '%' : ' MB/s'}
                  threshold={threshold}
                  thresholdLabel={threshold ? `Alert: ${threshold}` : undefined}
                  height={180}
                />
              </div>
            );
          })}
        </div>
      </div>

      {/* Fullscreen Modal */}
      {fullscreen && fullscreenMetric && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          onClick={() => setFullscreen(null)}
        >
          <div
            className="glass-card rounded-md-2xl p-6 w-full max-w-5xl max-h-[90vh] shadow-md-3 animate-scale-in"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-title-large font-semibold text-md-on-surface">
                {fullscreenMetric.name} — Detailed View
              </h3>
              <button
                onClick={() => setFullscreen(null)}
                className="w-8 h-8 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors"
              >
                <svg className="w-5 h-5 text-md-on-surface-variant" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div style={{ height: '60vh' }}>
              <MetricChart
                data={fullscreenMetric.data}
                title=""
                color={fullscreenMetric.color}
                unit={fullscreenMetric.name.includes('%') ? '%' : ' MB/s'}
                threshold={thresholds[fullscreenMetric.name]}
                thresholdLabel={thresholds[fullscreenMetric.name] ? `Alert: ${thresholds[fullscreenMetric.name]}` : undefined}
                height={400}
                showBrush={true}
              />
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-4 gap-4 mt-4">
              {(() => {
                const stats = calculateStats(fullscreenMetric.data);
                return [
                  { label: 'Current', value: `${stats.current}${fullscreenMetric.name.includes('%') ? '%' : ' MB/s'}` },
                  { label: 'Min', value: `${stats.min}${fullscreenMetric.name.includes('%') ? '%' : ' MB/s'}` },
                  { label: 'Average', value: `${stats.avg}${fullscreenMetric.name.includes('%') ? '%' : ' MB/s'}` },
                  { label: 'Max', value: `${stats.max}${fullscreenMetric.name.includes('%') ? '%' : ' MB/s'}` },
                ].map((s) => (
                  <div key={s.label} className="bg-md-surface-container-highest rounded-md-sm p-3 text-center">
                    <p className="text-label-small text-md-on-surface-variant">{s.label}</p>
                    <p className="text-title-small font-semibold text-md-on-surface">{s.value}</p>
                  </div>
                ));
              })()}
            </div>
          </div>
        </div>
      )}
    </>
  );
}
