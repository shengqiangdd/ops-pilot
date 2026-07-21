/**
 * Mock metrics data generator for development and demo purposes.
 * In production, this would be replaced with real Prometheus/Grafana data sources.
 */

export interface MetricDataPoint {
  timestamp: string;
  value: number;
}

export interface MetricSeries {
  name: string;
  data: MetricDataPoint[];
  color: string;
}

// Generate realistic CPU metrics with random fluctuations and periodic patterns
export function generateCPUMetrics(
  durationMinutes: number = 60,
  intervalSeconds: number = 30,
): MetricDataPoint[] {
  const points: MetricDataPoint[] = [];
  const now = Date.now();
  const totalPoints = Math.floor((durationMinutes * 60) / intervalSeconds);

  for (let i = 0; i < totalPoints; i++) {
    const timestamp = new Date(now - (totalPoints - i) * intervalSeconds * 1000);
    // Base CPU with periodic pattern (higher during business hours)
    const hour = timestamp.getHours();
    const businessHourFactor = hour >= 9 && hour <= 17 ? 1.3 : 0.8;
    const base = 35 * businessHourFactor;
    const noise = (Math.random() - 0.5) * 20;
    const spike = Math.random() > 0.95 ? 30 : 0; // occasional spikes
    const value = Math.max(5, Math.min(95, base + noise + spike));

    points.push({
      timestamp: timestamp.toISOString(),
      value: Math.round(value * 10) / 10,
    });
  }
  return points;
}

// Generate memory metrics with gradual increase pattern
export function generateMemoryMetrics(
  durationMinutes: number = 60,
  intervalSeconds: number = 30,
): MetricDataPoint[] {
  const points: MetricDataPoint[] = [];
  const now = Date.now();
  const totalPoints = Math.floor((durationMinutes * 60) / intervalSeconds);

  let base = 45;
  for (let i = 0; i < totalPoints; i++) {
    const timestamp = new Date(now - (totalPoints - i) * intervalSeconds * 1000);
    // Gradual increase with occasional GC drops
    base += (Math.random() - 0.45) * 2;
    if (base > 75 && Math.random() > 0.8) base -= 15; // GC event
    base = Math.max(30, Math.min(90, base));
    const noise = (Math.random() - 0.5) * 5;

    points.push({
      timestamp: timestamp.toISOString(),
      value: Math.round((base + noise) * 10) / 10,
    });
  }
  return points;
}

// Generate disk metrics with slow linear increase
export function generateDiskMetrics(
  durationMinutes: number = 60,
  intervalSeconds: number = 30,
): MetricDataPoint[] {
  const points: MetricDataPoint[] = [];
  const now = Date.now();
  const totalPoints = Math.floor((durationMinutes * 60) / intervalSeconds);

  let base = 55;
  for (let i = 0; i < totalPoints; i++) {
    const timestamp = new Date(now - (totalPoints - i) * intervalSeconds * 1000);
    // Slow linear increase (disk fills up over time)
    base += 0.02;
    if (base > 85 && Math.random() > 0.7) base -= 5; // log rotation
    const noise = (Math.random() - 0.5) * 1;

    points.push({
      timestamp: timestamp.toISOString(),
      value: Math.round((base + noise) * 10) / 10,
    });
  }
  return points;
}

// Generate network metrics with burst patterns
export function generateNetworkMetrics(
  durationMinutes: number = 60,
  intervalSeconds: number = 30,
): MetricDataPoint[] {
  const points: MetricDataPoint[] = [];
  const now = Date.now();
  const totalPoints = Math.floor((durationMinutes * 60) / intervalSeconds);

  for (let i = 0; i < totalPoints; i++) {
    const timestamp = new Date(now - (totalPoints - i) * intervalSeconds * 1000);
    const hour = timestamp.getHours();
    // Network usage follows business hours with bursts
    const businessFactor = hour >= 9 && hour <= 17 ? 1.5 : 0.5;
    const base = 200 * businessFactor;
    const burst = Math.random() > 0.9 ? 500 : 0;
    const noise = (Math.random() - 0.5) * 100;

    points.push({
      timestamp: timestamp.toISOString(),
      value: Math.round(Math.max(10, base + noise + burst)),
    });
  }
  return points;
}

// Generate all metrics for a dashboard view
export function generateAllMetrics(
  durationMinutes: number = 60,
  intervalSeconds: number = 30,
): MetricSeries[] {
  return [
    {
      name: 'CPU %',
      data: generateCPUMetrics(durationMinutes, intervalSeconds),
      color: '#6750A4',
    },
    {
      name: 'Memory %',
      data: generateMemoryMetrics(durationMinutes, intervalSeconds),
      color: '#7D5260',
    },
    {
      name: 'Disk %',
      data: generateDiskMetrics(durationMinutes, intervalSeconds),
      color: '#B3261E',
    },
    {
      name: 'Network (MB/s)',
      data: generateNetworkMetrics(durationMinutes, intervalSeconds),
      color: '#386A20',
    },
  ];
}

// Format timestamp for display
export function formatTimestamp(iso: string, format: 'short' | 'long' = 'short'): string {
  const date = new Date(iso);
  if (format === 'short') {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleString();
}

// Calculate statistics for a metric series
export function calculateStats(data: MetricDataPoint[]): {
  min: number;
  max: number;
  avg: number;
  current: number;
} {
  if (data.length === 0) {
    return { min: 0, max: 0, avg: 0, current: 0 };
  }

  const values = data.map((d) => d.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const avg = values.reduce((a, b) => a + b, 0) / values.length;
  const current = values[values.length - 1];

  return {
    min: Math.round(min * 10) / 10,
    max: Math.round(max * 10) / 10,
    avg: Math.round(avg * 10) / 10,
    current: Math.round(current * 10) / 10,
  };
}
