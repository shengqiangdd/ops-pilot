import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: () => ({ token: 'test-token' }),
}));

vi.mock('../../api/client', () => {
  const mockChannels = [
    {
      id: 'ch-1',
      name: 'Slack Alerts',
      channel_type: 'webhook',
      config: '{"url":"https://hooks.slack.com/xxx"}',
      enabled: true,
      created_at: '2026-01-01T00:00:00Z',
    },
    {
      id: 'ch-2',
      name: 'Email Notifications',
      channel_type: 'smtp',
      config: '{"host":"smtp.example.com"}',
      enabled: false,
      created_at: '2026-01-02T00:00:00Z',
    },
  ];
  return {
    api: {
      listNotificationChannels: vi.fn().mockResolvedValue(mockChannels),
      createNotificationChannel: vi.fn().mockResolvedValue({}),
      testNotificationChannel: vi.fn().mockResolvedValue({ status: 'ok' }),
    },
  };
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe('NotificationChannelsPage', () => {
  it('renders notification channels list', async () => {
    const { NotificationChannelsPage } = await import('../NotificationChannels');
    render(<NotificationChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText('Slack Alerts')).toBeInTheDocument();
    });
    expect(screen.getByText('Email Notifications')).toBeInTheDocument();
  });

  it('shows channel type labels', async () => {
    const { NotificationChannelsPage } = await import('../NotificationChannels');
    render(<NotificationChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText('Slack Alerts')).toBeInTheDocument();
    });
    expect(screen.getByText('webhook')).toBeInTheDocument();
    expect(screen.getByText('smtp')).toBeInTheDocument();
  });

  it('displays empty state when no channels', async () => {
    const mod = await import('../../api/client');
    (mod.api.listNotificationChannels as any).mockResolvedValue([]);
    const { NotificationChannelsPage } = await import('../NotificationChannels');
    render(<NotificationChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText('No notification channels configured')).toBeInTheDocument();
    });
  });

  it('shows add channel form on button click', async () => {
    const user = userEvent.setup();
    const { NotificationChannelsPage } = await import('../NotificationChannels');
    render(<NotificationChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText('Slack Alerts')).toBeInTheDocument();
    });
    await user.click(screen.getByText('Add Channel'));
    expect(screen.getByText('Create Notification Channel')).toBeInTheDocument();
  });

  it('displays error on load failure', async () => {
    const mod = await import('../../api/client');
    (mod.api.listNotificationChannels as any).mockRejectedValue(new Error('API error'));
    const { NotificationChannelsPage } = await import('../NotificationChannels');
    render(<NotificationChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText('API error')).toBeInTheDocument();
    });
  });
});
