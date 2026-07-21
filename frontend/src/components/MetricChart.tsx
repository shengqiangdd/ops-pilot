import { useMemo } from 'react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
  Brush,
} from 'recharts';
import type { MetricDataPoint } from '../lib/metrics';
import { formatTimestamp, calculateStats } from '../lib/metrics';

interface MetricChartProps {
  data: MetricDataPoint[];
  title: string;
  color: string;
  unit?: string;
  threshold?: number;
  thresholdLabel?: string;
  height?: number;
  showBrush?: boolean;
  gradient?: boolean;
}

export function MetricChart({
  data,
  title,
  color,
  unit = '%',
  threshold,
  thresholdLabel = 'Threshold',
  height = 200,
  showBrush = false,
  gradient = true,
}: MetricChartProps) {
  const stats = useMemo(() => calculateStats(data), [data]);

  const chartData = useMemo(() => {
    return data.map((d) => ({
      ...d,
      time: formatTimestamp(d.timestamp),
    }));
  }, [data]);

  const gradientId = `gradient-${title.replace(/\s+/g, '-').toLowerCase()}`;

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (!active || !payload || !payload.length) return null;
    return (
      <div className="glass-card rounded-md-lg px-3 py-2 shadow-md-2 text-sm">
        <p className="text-label-medium text-md-on-surface-variant mb-1">{label}</p>
        <p className="text-body-medium font-medium" style={{ color }}>
          {payload[0].value}{unit}
        </p>
      </div>
    );
  };

  return (
    <div className="glass-card rounded-md-xl p-4 h-full flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-title-small font-semibold text-md-on-surface">{title}</h3>
        <div className="flex items-center gap-3 text-label-small text-md-on-surface-variant">
          <span>Min: <span className="font-medium text-md-on-surface">{stats.min}{unit}</span></span>
          <span>Avg: <span className="font-medium text-md-on-surface">{stats.avg}{unit}</span></span>
          <span>Max: <span className="font-medium text-md-on-surface">{stats.max}{unit}</span></span>
        </div>
      </div>

      <div className="flex-1" style={{ minHeight: height }}>
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={chartData} margin={{ top: 5, right: 10, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor={color} stopOpacity={0.3} />
                <stop offset="100%" stopColor={color} stopOpacity={0.05} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--md-sys-color-outline-variant)" />
            <XAxis
              dataKey="time"
              tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }}
              tickLine={false}
              axisLine={{ stroke: 'var(--md-sys-color-outline-variant)' }}
            />
            <YAxis
              tick={{ fontSize: 11, fill: 'var(--md-sys-color-on-surface-variant)' }}
              tickLine={false}
              axisLine={{ stroke: 'var(--md-sys-color-outline-variant)' }}
              domain={[0, 'auto']}
            />
            <Tooltip content={<CustomTooltip />} />
            {threshold !== undefined && (
              <ReferenceLine
                y={threshold}
                stroke="#B3261E"
                strokeDasharray="5 5"
                label={{
                  value: thresholdLabel,
                  position: 'right',
                  fill: '#B3261E',
                  fontSize: 11,
                }}
              />
            )}
            <Area
              type="monotone"
              dataKey="value"
              stroke={color}
              strokeWidth={2}
              fill={gradient ? `url(#${gradientId})` : 'none'}
              dot={false}
              activeDot={{ r: 4, strokeWidth: 2, fill: color }}
            />
            {showBrush && (
              <Brush
                dataKey="time"
                height={30}
                stroke={color}
                fill="var(--md-sys-color-surface-container)"
              />
            )}
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
