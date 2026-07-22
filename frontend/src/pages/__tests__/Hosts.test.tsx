import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => mockNavigate,
}));

vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: () => ({
    token: 'test-token',
    canWrite: () => true,
  }),
}));

vi.mock('../../stores/useVaultStore', () => ({
  useVaultStore: () => ({
    isUnlocked: true,
  }),
}));

vi.mock('../../api/client', () => {
  const mockHosts = [
    {
      id: 'host-1',
      name: 'web-server',
      address: '10.0.0.1',
      port: 22,
      username: 'admin',
      auth_method: 'key',
      status: 'online',
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
    },
    {
      id: 'host-2',
      name: 'db-server',
      address: '10.0.0.2',
      port: 22,
      username: 'root',
      auth_method: 'password',
      status: 'offline',
      created_at: '2026-01-02T00:00:00Z',
      updated_at: '2026-01-02T00:00:00Z',
    },
  ];
  return {
    api: {
      listHosts: vi.fn().mockResolvedValue(mockHosts),
      deleteHost: vi.fn().mockResolvedValue(undefined),
    },
  };
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe('HostsPage', () => {
  it('renders hosts list', async () => {
    const { HostsPage } = await import('../Hosts');
    render(<HostsPage />);
    await waitFor(() => {
      expect(screen.getByText('web-server')).toBeInTheDocument();
    });
    expect(screen.getByText('db-server')).toBeInTheDocument();
  });

  it('shows host status', async () => {
    const { HostsPage } = await import('../Hosts');
    render(<HostsPage />);
    await waitFor(() => {
      expect(screen.getByText('online')).toBeInTheDocument();
    });
    expect(screen.getByText('offline')).toBeInTheDocument();
  });

  it('shows host addresses', async () => {
    const { HostsPage } = await import('../Hosts');
    render(<HostsPage />);
    await waitFor(() => {
      expect(screen.getByText('10.0.0.1')).toBeInTheDocument();
    });
    expect(screen.getByText('10.0.0.2')).toBeInTheDocument();
  });

  it('displays empty state when no hosts', async () => {
    const mod = await import('../../api/client');
    (mod.api.listHosts as any).mockResolvedValue([]);
    const { HostsPage } = await import('../Hosts');
    render(<HostsPage />);
    await waitFor(() => {
      expect(screen.getByText('No hosts configured')).toBeInTheDocument();
    });
  });

  it('displays error on load failure', async () => {
    const mod = await import('../../api/client');
    (mod.api.listHosts as any).mockRejectedValue(new Error('Connection failed'));
    const { HostsPage } = await import('../Hosts');
    render(<HostsPage />);
    await waitFor(() => {
      expect(screen.getByText('Connection failed')).toBeInTheDocument();
    });
  });
});
