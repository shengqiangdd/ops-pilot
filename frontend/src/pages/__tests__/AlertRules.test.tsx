import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: () => ({
    token: 'test-token',
    canWrite: () => true,
  }),
}));

vi.mock('../../api/client', () => {
  const mockRules = [
    {
      id: 'rule-1',
      name: 'CPU High',
      metric: 'cpu_percent',
      condition: '>',
      threshold: 90,
      severity: 'critical',
      silence_minutes: 5,
      enabled: true,
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
    },
    {
      id: 'rule-2',
      name: 'Memory Low',
      metric: 'memory_percent',
      condition: '<',
      threshold: 20,
      severity: 'warning',
      silence_minutes: 10,
      enabled: false,
      created_at: '2026-01-02T00:00:00Z',
      updated_at: '2026-01-02T00:00:00Z',
    },
  ];
  return {
    api: {
      listAlertRules: vi.fn().mockResolvedValue(mockRules),
      createAlertRule: vi.fn().mockResolvedValue({
        ...mockRules[0],
        id: 'new-rule',
        name: 'New Rule',
        created_at: '2026-01-03T00:00:00Z',
        updated_at: '2026-01-03T00:00:00Z',
      }),
      deleteAlertRule: vi.fn().mockResolvedValue(undefined),
    },
  };
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe('AlertRulesPage', () => {
  it('renders alert rules list', async () => {
    const { AlertRulesPage } = await import('../AlertRules');
    render(<AlertRulesPage />);
    await waitFor(() => {
      expect(screen.getByText('CPU High')).toBeInTheDocument();
    });
    expect(screen.getByText('Memory Low')).toBeInTheDocument();
  });

  it('displays rule severity', async () => {
    const { AlertRulesPage } = await import('../AlertRules');
    render(<AlertRulesPage />);
    await waitFor(() => {
      expect(screen.getByText('critical')).toBeInTheDocument();
    });
    expect(screen.getByText('warning')).toBeInTheDocument();
  });

  it('shows empty state when no rules', async () => {
    const mod = await import('../../api/client');
    (mod.api.listAlertRules as any).mockResolvedValue([]);
    const { AlertRulesPage } = await import('../AlertRules');
    render(<AlertRulesPage />);
    await waitFor(() => {
      expect(screen.getByText('No alert rules configured')).toBeInTheDocument();
    });
  });

  it('shows form when add button is clicked', async () => {
    const user = userEvent.setup();
    const { AlertRulesPage } = await import('../AlertRules');
    render(<AlertRulesPage />);
    await waitFor(() => {
      expect(screen.getByText('CPU High')).toBeInTheDocument();
    });
    await user.click(screen.getByText('Add Rule'));
    expect(screen.getByText('Create Alert Rule')).toBeInTheDocument();
  });

  it('displays error on load failure', async () => {
    const mod = await import('../../api/client');
    (mod.api.listAlertRules as any).mockRejectedValue(new Error('Network error'));
    const { AlertRulesPage } = await import('../AlertRules');
    render(<AlertRulesPage />);
    await waitFor(() => {
      expect(screen.getByText('Network error')).toBeInTheDocument();
    });
  });

  it('creates a rule via form submission', async () => {
    const user = userEvent.setup();
    const { AlertRulesPage } = await import('../AlertRules');
    render(<AlertRulesPage />);
    await waitFor(() => {
      expect(screen.getByText('CPU High')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Add Rule'));

    const nameInput = screen.getByPlaceholderText('e.g. CPU > 90%');
    await user.type(nameInput, 'Disk Full');

    await user.click(screen.getByText('Save'));

    await waitFor(() => {
      const mod = require('../../api/client');
      expect(mod.api.createAlertRule).toHaveBeenCalled();
    });
  });
});
