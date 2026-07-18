import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ModuleConfigEditor } from '../ModuleConfigEditor';
import * as api from '../../api/client';

vi.mock('../../api/client');

const mockConfig = { enabled: true, timeout: 30, retries: 3 };

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.api.getModuleConfig).mockResolvedValue(mockConfig);
});

describe('ModuleConfigEditor', () => {
  it('loads and displays config on mount', async () => {
    const onClose = vi.fn();
    render(<ModuleConfigEditor moduleName="mod-rca" onClose={onClose} />);

    await waitFor(() => {
      expect(screen.getByText('Configure: mod-rca')).toBeInTheDocument();
    });
    expect(api.api.getModuleConfig).toHaveBeenCalledWith('mod-rca');
    const textarea = screen.getByRole('textbox');
    expect(textarea).toHaveValue(JSON.stringify(mockConfig, null, 2));
  });

  it('shows validation error for invalid JSON', async () => {
    const onClose = vi.fn();
    render(<ModuleConfigEditor moduleName="mod-rca" onClose={onClose} />);

    await waitFor(() => {
      expect(screen.getByRole('textbox')).toHaveValue(JSON.stringify(mockConfig, null, 2));
    });

    const textarea = screen.getByRole('textbox');
    await userEvent.clear(textarea);
    await userEvent.type(textarea, '{ invalid json');

    expect(screen.getByText(/JSON error/)).toBeInTheDocument();
  });

  it('saves valid config', async () => {
    vi.mocked(api.api.saveModuleConfig).mockResolvedValue({ ok: true });
    const onClose = vi.fn();
    render(<ModuleConfigEditor moduleName="mod-rca" onClose={onClose} />);

    await waitFor(() => {
      expect(screen.getByRole('textbox')).toHaveValue(JSON.stringify(mockConfig, null, 2));
    });

    await userEvent.click(screen.getByText('Save'));

    await waitFor(() => {
      expect(api.api.saveModuleConfig).toHaveBeenCalledWith('mod-rca', mockConfig);
    });
    expect(screen.getByText('Configuration saved successfully.')).toBeInTheDocument();
  });

  it('calls onClose when cancel clicked', async () => {
    const onClose = vi.fn();
    render(<ModuleConfigEditor moduleName="mod-rca" onClose={onClose} />);

    await waitFor(() => {
      expect(screen.getByText('Cancel')).toBeInTheDocument();
    });

    await userEvent.click(screen.getByText('Cancel'));
    expect(onClose).toHaveBeenCalled();
  });

  it('calls onClose when close button clicked', async () => {
    const onClose = vi.fn();
    render(<ModuleConfigEditor moduleName="mod-rca" onClose={onClose} />);

    await waitFor(() => {
      expect(screen.getByLabelText('Close')).toBeInTheDocument();
    });

    await userEvent.click(screen.getByLabelText('Close'));
    expect(onClose).toHaveBeenCalled();
  });

  it('does not save when JSON is invalid', async () => {
    const onClose = vi.fn();
    render(<ModuleConfigEditor moduleName="mod-rca" onClose={onClose} />);

    await waitFor(() => {
      expect(screen.getByRole('textbox')).toHaveValue(JSON.stringify(mockConfig, null, 2));
    });

    const textarea = screen.getByRole('textbox');
    await userEvent.clear(textarea);
    await userEvent.type(textarea, 'bad json');

    await userEvent.click(screen.getByText('Save'));
    expect(api.api.saveModuleConfig).not.toHaveBeenCalled();
  });
});
