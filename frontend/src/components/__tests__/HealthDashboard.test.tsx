import { render, screen, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
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
  vi.useFakeTimers();
  vi.resetAllMocks();
  vi.mocked(api.api.getHealthAll).mockResolvedValue(mockHealthData);
});

afterEach(() => {
  vi.useRealTimers();
});

describe('HealthDashboard', () => {
  it('renders health data after loading', async () => {
    await act(async () => {
      render(<HealthDashboard />);
    });
    await waitFor(() => {
      expect(screen.getByText('mod-rca')).toBeInTheDocument();
    });
    expect(screen.getByText('mod-core')).toBeInTheDocument();
    expect(screen.getByText('mod-finops')).toBeInTheDocument();
  });

  it('shows summary counts', async () => {
    await act(async () => {
      render(<HealthDashboard />);
    });
    await waitFor(() => {
      expect(screen.getByText('1')).toBeInTheDocument(); // 1 healthy
    });
    // Check degraded and unhealthy counts
    const degradedSection = screen.getByText('Degraded').closest('div')!;
    expect(degradedSection).toHaveTextContent('1');
    const unhealthySection = screen.getByText('Unhealthy').closest('div')!;
    expect(unhealthySection).toHaveTextContent('1');
  });

  it('highlights unhealthy modules', async () => {
    await act(async () => {
      render(<HealthDashboard />);
    });
    await waitFor(() => {
      expect(screen.getByText('mod-finops')).toBeInTheDocument();
    });
    expect(screen.getByText('connection refused')).toBeInTheDocument();
  });

  it('refreshes on button click', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    await act(async () => {
      render(<HealthDashboard />);
    });
    await waitFor(() => {
      expect(screen.getByText('Refresh Now')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Refresh Now'));

    await waitFor(() => {
      expect(api.api.getHealthAll).toHaveBeenCalledTimes(2);
    });
  });

  it('auto-refreshes every 30 seconds', async () => {
    await act(async () => {
      render(<HealthDashboard />);
    });

    await act(async () => {
      vi.advanceTimersByTime(30_000);
    });

    await waitFor(() => {
      expect(api.api.getHealthAll).toHaveBeenCalledTimes(2);
    });
  });

  it('displays error on failure', async () => {
    vi.mocked(api.api.getHealthAll).mockRejectedValue(new Error('Timeout'));

    await act(async () => {
      render(<HealthDashboard />);
    });

    await waitFor(() => {
      expect(screen.getByText('Timeout')).toBeInTheDocument();
    });
  });

  it('shows last refresh time', async () => {
    await act(async () => {
      render(<HealthDashboard />);
    });
    await waitFor(() => {
      expect(screen.getByText(/Last refreshed/)).toBeInTheDocument();
    });
  });
});
