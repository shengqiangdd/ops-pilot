import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => mockNavigate,
}));

vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: () => ({ token: 'test-token' }),
}));

const mockHostsList = [
  { id: 'host-1', name: 'web-server' },
  { id: 'host-2', name: 'db-server' },
];

vi.mock('../../api/client', () => ({
  api: {
    listHosts: vi.fn().mockResolvedValue(mockHostsList),
    collectMonitorMetrics: vi.fn().mockResolvedValue({
      host_id: 'host-1',
      cpu_percent: 45.2,
      memory_percent: 62.1,
      disk_percent: 78.5,
      network_in_bytes: 1024000,
      network_out_bytes: 512000,
      load_1: 1.5,
      load_5: 1.2,
      load_15: 0.8,
      memory_total_mb: 16384,
      memory_used_mb: 10175,
      disk_total_gb: 500,
      disk_used_gb: 392,
    }),
    getMonitorMetrics: vi.fn().mockResolvedValue([
      { timestamp: '2026-01-01T12:00:00Z', host_id: 'host-1', metric_type: 'cpu', value: 45.2, unit: '%' },
      { timestamp: '2026-01-01T12:00:30Z', host_id: 'host-1', metric_type: 'cpu', value: 46.1, unit: '%' },
    ]),
  },
}));

beforeEach(() => {
  vi.clearAllMocks();
});

describe('MonitorPage', () => {
  it('renders monitor page title', async () => {
    const { MonitorPage } = await import('../Monitor');
    render(<MonitorPage />);
    await waitFor(() => {
      expect(screen.getByText('Monitor')).toBeInTheDocument();
    });
  });

  it('renders host selector', async () => {
    const { MonitorPage } = await import('../Monitor');
    render(<MonitorPage />);
    await waitFor(() => {
      expect(screen.getByText('web-server')).toBeInTheDocument();
    });
    expect(screen.getByText('db-server')).toBeInTheDocument();
  });

  it('renders collect metrics button', async () => {
    const { MonitorPage } = await import('../Monitor');
    render(<MonitorPage />);
    await waitFor(() => {
      expect(screen.getByText('Collect Metrics')).toBeInTheDocument();
    });
  });

  it('selecting host and collecting shows metrics', async () => {
    const user = userEvent.setup();
    const { MonitorPage } = await import('../Monitor');
    render(<MonitorPage />);
    await waitFor(() => {
      expect(screen.getByText('web-server')).toBeInTheDocument();
    });

    const hostOption = screen.getByText('web-server');
    await user.click(hostOption);
    await user.click(screen.getByText('Collect Metrics'));

    await waitFor(() => {
      expect(screen.getByText('CPU')).toBeInTheDocument();
    });
  });

  it('displays error on collection failure', async () => {
    const mod = await import('../../api/client');
    (mod.api.collectMonitorMetrics as any).mockRejectedValue(new Error('Collection failed'));
    const user = userEvent.setup();
    const { MonitorPage } = await import('../Monitor');
    render(<MonitorPage />);
    await waitFor(() => {
      expect(screen.getByText('web-server')).toBeInTheDocument();
    });

    await user.click(screen.getByText('web-server'));
    await user.click(screen.getByText('Collect Metrics'));

    await waitFor(() => {
      expect(screen.getByText('Collection failed')).toBeInTheDocument();
    });
  });
});
