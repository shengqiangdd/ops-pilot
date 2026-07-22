import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockBackupData = { version: '2.0', tables: {} };

beforeEach(() => {
  vi.clearAllMocks();
  global.fetch = vi.fn();
  global.URL.createObjectURL = vi.fn(() => 'blob:test');
  global.URL.revokeObjectURL = vi.fn();
});

describe('BackupRestorePage', () => {
  it('renders page title', async () => {
    const { BackupRestorePage } = await import('../BackupRestore');
    render(<BackupRestorePage />);
    expect(screen.getByText('Backup & Restore')).toBeInTheDocument();
  });

  it('renders export and import buttons', async () => {
    const { BackupRestorePage } = await import('../BackupRestore');
    render(<BackupRestorePage />);
    expect(screen.getByText('Export Backup')).toBeInTheDocument();
    expect(screen.getByText('Import Backup')).toBeInTheDocument();
  });

  it('shows exporting state on export click', async () => {
    const user = userEvent.setup();
    const { BackupRestorePage } = await import('../BackupRestore');
    render(<BackupRestorePage />);

    await user.click(screen.getByText('Export Backup'));
    expect(screen.getByText(/Exporting/)).toBeInTheDocument();
  });

  it('shows success message after export', async () => {
    (global.fetch as any).mockResolvedValueOnce({
      json: () => Promise.resolve({ status: 'ok', data: mockBackupData }),
    });

    const user = userEvent.setup();
    const { BackupRestorePage } = await import('../BackupRestore');
    render(<BackupRestorePage />);

    await user.click(screen.getByText('Export Backup'));

    await waitFor(() => {
      expect(screen.getByText(/备份已下载/)).toBeInTheDocument();
    });
  });

  it('shows error on export failure', async () => {
    (global.fetch as any).mockRejectedValueOnce(new Error('Export failed'));

    const user = userEvent.setup();
    const { BackupRestorePage } = await import('../BackupRestore');
    render(<BackupRestorePage />);

    await user.click(screen.getByText('Export Backup'));

    await waitFor(() => {
      expect(screen.getByText(/Export failed/)).toBeInTheDocument();
    });
  });

  it('renders description text', async () => {
    const { BackupRestorePage } = await import('../BackupRestore');
    render(<BackupRestorePage />);
    expect(screen.getByText(/Export or import/)).toBeInTheDocument();
  });
});
