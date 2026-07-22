import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: () => ({ token: 'test-token' }),
}));

vi.mock('../../api/client', () => {
  const mockEntries = [
    {
      id: 'alert-1',
      rule_id: 'rule-1',
      rule_name: 'CPU High',
      severity: 'critical',
      message: 'CPU usage exceeded 90%',
      status: 'firing',
      triggered_at: '2026-01-01T12:00:00Z',
      acknowledged_at: null,
    },
    {
      id: 'alert-2',
      rule_id: 'rule-2',
      rule_name: 'Memory Low',
      severity: 'warning',
      message: 'Memory below 20%',
      status: 'acknowledged',
      triggered_at: '2026-01-02T08:00:00Z',
      acknowledged_at: '2026-01-02T09:00:00Z',
    },
  ];
  return { api: { listAlertHistory: vi.fn().mockResolvedValue(mockEntries) } };
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe('AlertHistoryPage', () => {
  it('renders alert history entries', async () => {
    const { AlertHistoryPage } = await import('../AlertHistory');
    render(<AlertHistoryPage />);
    await waitFor(() => {
      expect(screen.getByText('CPU High')).toBeInTheDocument();
    });
    expect(screen.getByText('Memory Low')).toBeInTheDocument();
  });

  it('renders severity labels', async () => {
    const { AlertHistoryPage } = await import('../AlertHistory');
    render(<AlertHistoryPage />);
    await waitFor(() => {
      expect(screen.getByText('critical')).toBeInTheDocument();
    });
    expect(screen.getByText('warning')).toBeInTheDocument();
  });

  it('displays empty state when no entries', async () => {
    const mod = await import('../../api/client');
    (mod.api.listAlertHistory as any).mockResolvedValue([]);
    const { AlertHistoryPage } = await import('../AlertHistory');
    render(<AlertHistoryPage />);
    await waitFor(() => {
      expect(screen.getByText('No alerts found')).toBeInTheDocument();
    });
  });

  it('shows filter inputs', async () => {
    const { AlertHistoryPage } = await import('../AlertHistory');
    render(<AlertHistoryPage />);
    await waitFor(() => {
      expect(screen.getByText('CPU High')).toBeInTheDocument();
    });
    expect(screen.getByText('Filter')).toBeInTheDocument();
  });

  it('displays error on load failure', async () => {
    const mod = await import('../../api/client');
    (mod.api.listAlertHistory as any).mockRejectedValue(new Error('Failed to fetch'));
    const { AlertHistoryPage } = await import('../AlertHistory');
    render(<AlertHistoryPage />);
    await waitFor(() => {
      expect(screen.getByText('Failed to fetch')).toBeInTheDocument();
    });
  });
});
