import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { HealthDashboard } from '../HealthDashboard';
import * as api from '../../api/client';
import type { ModuleHealth } from '../../api/types';

vi.mock('../../api/client');

const mockHealthData: ModuleHealth[] = [
  { name: 'mod-rca', status: { Healthy: null }, enabled: true },
  { name: 'mod-core', status: { Degraded: { reason: 'high memory' } }, enabled: true },
  { name: 'mod-finops', status: { Unhealthy: { reason: 'connection refused' } }, enabled: false },
];

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.api.getHealthAll).mockResolvedValue(mockHealthData);
});

describe('HealthDashboard', () => {
  it('renders health data after loading', async () => {
    render(<HealthDashboard />);
    expect(await screen.findByText('mod-rca')).toBeInTheDocument();
    expect(screen.getByText('mod-core')).toBeInTheDocument();
    expect(screen.getByText('mod-finops')).toBeInTheDocument();
  });

  it('shows summary counts', async () => {
    render(<HealthDashboard />);
    await screen.findByText('Healthy');
    const ones = screen.getAllByText('1');
    expect(ones).toHaveLength(3);
  });

  it('highlights unhealthy modules', async () => {
    render(<HealthDashboard />);
    await screen.findByText('mod-finops');
    expect(screen.getByText('connection refused')).toBeInTheDocument();
  });

  it('refreshes on button click', async () => {
    const user = userEvent.setup();
    render(<HealthDashboard />);
    await screen.findByText('Refresh Now');

    await user.click(screen.getByText('Refresh Now'));

    await waitFor(() => {
      expect(api.api.getHealthAll).toHaveBeenCalledTimes(2);
    });
  });

  it('displays error on failure', async () => {
    vi.mocked(api.api.getHealthAll).mockRejectedValue(new Error('Timeout'));

    render(<HealthDashboard />);
    expect(await screen.findByText('Timeout')).toBeInTheDocument();
  });

  it('shows last refresh time', async () => {
    render(<HealthDashboard />);
    expect(await screen.findByText(/Last refreshed/)).toBeInTheDocument();
  });
});
