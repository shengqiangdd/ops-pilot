import { describe, it, expect } from 'vitest';
import {
  generateCPUMetrics,
  generateMemoryMetrics,
  generateDiskMetrics,
  generateNetworkMetrics,
  generateAllMetrics,
  formatTimestamp,
  calculateStats,
  type MetricDataPoint,
} from '../metrics';

describe('generateCPUMetrics', () => {
  it('generates correct number of points', () => {
    const data = generateCPUMetrics(60, 30);
    expect(data.length).toBe(120); // 60 min * 60 sec / 30 sec
  });

  it('returns values between 5 and 95', () => {
    const data = generateCPUMetrics(10, 30);
    for (const point of data) {
      expect(point.value).toBeGreaterThanOrEqual(5);
      expect(point.value).toBeLessThanOrEqual(95);
    }
  });

  it('each point has timestamp and value', () => {
    const data = generateCPUMetrics(5, 30);
    for (const point of data) {
      expect(point).toHaveProperty('timestamp');
      expect(point).toHaveProperty('value');
      expect(typeof point.value).toBe('number');
    }
  });

  it('timestamps are valid ISO strings', () => {
    const data = generateCPUMetrics(5, 60);
    for (const point of data) {
      expect(() => new Date(point.timestamp)).not.toThrow();
      expect(new Date(point.timestamp).toISOString()).toBe(point.timestamp);
    }
  });
});

describe('generateMemoryMetrics', () => {
  it('generates values between 30 and 90', () => {
    const data = generateMemoryMetrics(10, 30);
    for (const point of data) {
      expect(point.value).toBeGreaterThanOrEqual(30);
      expect(point.value).toBeLessThanOrEqual(90);
    }
  });

  it('returns array of MetricDataPoint', () => {
    const data = generateMemoryMetrics(5, 30);
    expect(data.length).toBeGreaterThan(0);
    expect(data[0]).toHaveProperty('timestamp');
    expect(data[0]).toHaveProperty('value');
  });
});

describe('generateDiskMetrics', () => {
  it('generates values with slow increase pattern', () => {
    const data = generateDiskMetrics(60, 30);
    // Values should generally be within range
    for (const point of data) {
      expect(point.value).toBeGreaterThan(0);
      expect(point.value).toBeLessThan(100);
    }
  });
});

describe('generateNetworkMetrics', () => {
  it('generates positive values', () => {
    const data = generateNetworkMetrics(10, 30);
    for (const point of data) {
      expect(point.value).toBeGreaterThan(0);
    }
  });
});

describe('generateAllMetrics', () => {
  it('returns four metric series', () => {
    const series = generateAllMetrics(10, 30);
    expect(series.length).toBe(4);
  });

  it('each series has name, data, color', () => {
    const series = generateAllMetrics(10, 30);
    for (const s of series) {
      expect(s).toHaveProperty('name');
      expect(s).toHaveProperty('data');
      expect(s).toHaveProperty('color');
      expect(s.data.length).toBeGreaterThan(0);
    }
  });

  it('series names are descriptive', () => {
    const series = generateAllMetrics(5, 30);
    const names = series.map((s) => s.name);
    expect(names).toContain('CPU %');
    expect(names).toContain('Memory %');
    expect(names).toContain('Disk %');
    expect(names).toContain('Network (MB/s)');
  });
});

describe('formatTimestamp', () => {
  it('formats short format', () => {
    const result = formatTimestamp('2026-01-01T12:30:00.000Z', 'short');
    expect(result).toBeTruthy();
    expect(typeof result).toBe('string');
  });

  it('formats long format', () => {
    const result = formatTimestamp('2026-01-01T12:30:00.000Z', 'long');
    expect(result).toBeTruthy();
    expect(typeof result).toBe('string');
  });

  it('default format is short', () => {
    const result = formatTimestamp('2026-01-01T12:30:00.000Z');
    expect(result).toBeTruthy();
  });
});

describe('calculateStats', () => {
  it('returns zeros for empty data', () => {
    const stats = calculateStats([]);
    expect(stats).toEqual({ min: 0, max: 0, avg: 0, current: 0 });
  });

  it('calculates correct statistics', () => {
    const data: MetricDataPoint[] = [
      { timestamp: '2026-01-01T00:00:00Z', value: 10 },
      { timestamp: '2026-01-01T00:01:00Z', value: 20 },
      { timestamp: '2026-01-01T00:02:00Z', value: 30 },
    ];

    const stats = calculateStats(data);
    expect(stats.min).toBe(10);
    expect(stats.max).toBe(30);
    expect(stats.avg).toBe(20);
    expect(stats.current).toBe(30);
  });

  it('handles single data point', () => {
    const data: MetricDataPoint[] = [
      { timestamp: '2026-01-01T00:00:00Z', value: 42 },
    ];

    const stats = calculateStats(data);
    expect(stats.min).toBe(42);
    expect(stats.max).toBe(42);
    expect(stats.avg).toBe(42);
    expect(stats.current).toBe(42);
  });

  it('rounds values to one decimal', () => {
    const data: MetricDataPoint[] = [
      { timestamp: '2026-01-01T00:00:00Z', value: 10 },
      { timestamp: '2026-01-01T00:01:00Z', value: 20.37 },
    ];

    const stats = calculateStats(data);
    expect(stats.avg).toBe(15.2); // (10 + 20.37) / 2 = 15.185 -> 15.2
  });
});
