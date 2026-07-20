import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { HealthBadge } from '../HealthBadge';
import type { HealthStatus } from '../../api/types';

describe('HealthBadge', () => {
  it('renders Healthy status', () => {
    const status: HealthStatus = { Healthy: null };
    render(<HealthBadge status={status} />);
    expect(screen.getByText('Healthy')).toBeInTheDocument();
  });

  it('renders Degraded status', () => {
    const status: HealthStatus = { Degraded: { reason: 'high latency' } };
    render(<HealthBadge status={status} />);
    expect(screen.getByText('Degraded')).toBeInTheDocument();
  });

  it('renders Unhealthy status', () => {
    const status: HealthStatus = { Unhealthy: { reason: 'connection refused' } };
    render(<HealthBadge status={status} />);
    expect(screen.getByText('Unhealthy')).toBeInTheDocument();
  });
});
