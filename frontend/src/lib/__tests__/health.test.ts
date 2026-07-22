import { describe, it, expect } from 'vitest';
import { getHealthLabel, getHealthColor, getHealthReason } from '../health';
import type { HealthStatus } from '../../api/types';

describe('getHealthLabel', () => {
  it('returns "Healthy" for Healthy tagged union', () => {
    const status: HealthStatus = { Healthy: null };
    expect(getHealthLabel(status)).toBe('Healthy');
  });

  it('returns "Degraded" for Degraded tagged union', () => {
    const status: HealthStatus = { Degraded: { reason: 'high memory' } };
    expect(getHealthLabel(status)).toBe('Degraded');
  });

  it('returns "Unhealthy" for Unhealthy tagged union', () => {
    const status: HealthStatus = { Unhealthy: { reason: 'connection refused' } };
    expect(getHealthLabel(status)).toBe('Unhealthy');
  });

  it('returns "Unknown" for null/undefined', () => {
    expect(getHealthLabel(null)).toBe('Unknown');
    expect(getHealthLabel(undefined)).toBe('Unknown');
  });

  it('parses string status values', () => {
    expect(getHealthLabel('Healthy')).toBe('Healthy');
    expect(getHealthLabel('Degraded')).toBe('Degraded');
    expect(getHealthLabel('Unhealthy')).toBe('Unhealthy');
  });

  it('returns "Unknown" for unrecognized strings', () => {
    expect(getHealthLabel('UnknownStatus')).toBe('Unknown');
  });

  it('returns "Unknown" for empty string', () => {
    expect(getHealthLabel('')).toBe('Unknown');
  });
});

describe('getHealthColor', () => {
  it('returns green for Healthy', () => {
    const status: HealthStatus = { Healthy: null };
    expect(getHealthColor(status)).toBe('bg-green-500');
  });

  it('returns amber for Degraded', () => {
    const status: HealthStatus = { Degraded: { reason: 'slow' } };
    expect(getHealthColor(status)).toBe('bg-amber-500');
  });

  it('returns error color for Unhealthy', () => {
    const status: HealthStatus = { Unhealthy: { reason: 'down' } };
    expect(getHealthColor(status)).toBe('bg-md-error');
  });

  it('returns outline color for null', () => {
    expect(getHealthColor(null)).toBe('bg-md-outline');
  });

  it('handles string status values', () => {
    expect(getHealthColor('Healthy')).toBe('bg-green-500');
    expect(getHealthColor('Degraded')).toBe('bg-amber-500');
    expect(getHealthColor('Unhealthy')).toBe('bg-md-error');
  });
});

describe('getHealthReason', () => {
  it('returns reason for Degraded', () => {
    const status: HealthStatus = { Degraded: { reason: 'high latency' } };
    expect(getHealthReason(status)).toBe('high latency');
  });

  it('returns reason for Unhealthy', () => {
    const status: HealthStatus = { Unhealthy: { reason: 'OOM' } };
    expect(getHealthReason(status)).toBe('OOM');
  });

  it('returns null for Healthy', () => {
    const status: HealthStatus = { Healthy: null };
    expect(getHealthReason(status)).toBeNull();
  });

  it('returns null for string status', () => {
    expect(getHealthReason('Healthy')).toBeNull();
  });

  it('returns null for null/undefined', () => {
    expect(getHealthReason(null)).toBeNull();
    expect(getHealthReason(undefined)).toBeNull();
  });
});
