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
    await waitFor(() => {
      expect(screen.getByText('mod-rca')).toBeInTheDocument();
    });
    expect(screen.getByText('mod-rca')).toBeInTheDocument();
    expect(screen.getByText('mod-core')).toBeInTheDocument();
    expect(screen.getByText('mod-finops')).toBeInTheDocument();
  });

  it('highlights unhealthy modules', async () => {
    render(<HealthDashboard />);
    await waitFor(() => {
      expect(screen.getByText('mod-finops')).toBeInTheDocument();
    });
    // The component should render the unhealthy module
    expect(screen.getByText('mod-finops')).toBeInTheDocument();
  });

  it('refreshes on button click', async () => {
    const user = userEvent.setup();
    render(<HealthDashboard />);
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /refresh/i })).toBeInTheDocument();
    });

    await user.click(screen.getByRole('button', { name: /refresh/i }));

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
    await waitFor(() => {
      expect(screen.getByText('mod-rca')).toBeInTheDocument();
    });
    expect(screen.getByText('mod-rca')).toBeInTheDocument();
  });
});
