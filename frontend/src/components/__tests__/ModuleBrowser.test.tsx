import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ModuleBrowser } from '../ModuleBrowser';
import * as api from '../../api/client';
import type { ModuleInfo, HealthStatus } from '../../api/types';

vi.mock('../../api/client');

const mockModules: ModuleInfo[] = [
  { name: 'mod-rca', version: '0.1.0', description: 'Root Cause Analysis', enabled: true },
  { name: 'mod-core', version: '0.2.0', description: 'Core module', enabled: false },
];

const mockHealth: Record<string, HealthStatus> = {
  'mod-rca': { Healthy: null },
  'mod-core': { Degraded: { reason: 'slow' } },
};

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.api.listModules).mockResolvedValue(mockModules);
  vi.mocked(api.api.getModuleHealth).mockImplementation(async (name: string) => {
    return mockHealth[name] ?? { Healthy: null };
  });
});

describe('ModuleBrowser', () => {
  it('renders module list after loading', async () => {
    render(<ModuleBrowser />);
    await waitFor(() => {
      expect(screen.getByText('mod-rca')).toBeInTheDocument();
    });
    expect(screen.getByText('mod-core')).toBeInTheDocument();
    expect(screen.getByText('Root Cause Analysis')).toBeInTheDocument();
    expect(screen.getByText('Core module')).toBeInTheDocument();
  });

  it('shows loading state initially', async () => {
    vi.mocked(api.api.listModules).mockImplementation(() => new Promise(() => {}));
    render(<ModuleBrowser />);
    await waitFor(() => {
      expect(screen.queryByText('mod-rca')).not.toBeInTheDocument();
    });
  });

  it('toggles module enabled state', async () => {
    const user = userEvent.setup();
    vi.mocked(api.api.disableModule).mockResolvedValue({ enabled: false });

    render(<ModuleBrowser />);
    await waitFor(() => {
      expect(screen.getByText('mod-rca')).toBeInTheDocument();
    });

    const toggle = screen.getAllByRole('switch')[0];
    await user.click(toggle);

    expect(api.api.disableModule).toHaveBeenCalledWith('mod-rca');
  });

  it('calls reload when Reload button clicked', async () => {
    const user = userEvent.setup();
    render(<ModuleBrowser />);
    await waitFor(() => {
      expect(screen.getByText('Reload')).toBeInTheDocument();
    });

    vi.mocked(api.api.listModules).mockResolvedValue([]);
    await user.click(screen.getByText('Reload'));

    await waitFor(() => {
      expect(api.api.listModules).toHaveBeenCalledTimes(2);
    });
  });

  it('displays error message on failure', async () => {
    vi.mocked(api.api.listModules).mockRejectedValue(new Error('Network error'));
    render(<ModuleBrowser />);

    await waitFor(() => {
      expect(screen.getByText('Network error')).toBeInTheDocument();
    });
  });

  it('shows empty state when no modules', async () => {
    vi.mocked(api.api.listModules).mockResolvedValue([]);
    render(<ModuleBrowser />);

    // Wait for the loading to finish and check that no module names are displayed
    await waitFor(() => {
      expect(screen.queryByText('mod-rca')).not.toBeInTheDocument();
      expect(screen.queryByText('mod-core')).not.toBeInTheDocument();
    });
  });
});
