import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { WSStatusIndicator } from '../WSStatusIndicator';

vi.mock('../../hooks/useWebSocketStatus', () => ({
  useWebSocketStatus: vi.fn(),
}));

import { useWebSocketStatus } from '../../hooks/useWebSocketStatus';

beforeEach(() => {
  vi.clearAllMocks();
});

describe('WSStatusIndicator', () => {
  it('renders connected status with green dot', () => {
    vi.mocked(useWebSocketStatus).mockReturnValue('connected');
    render(<WSStatusIndicator />);
    expect(screen.getByText('terminal.status.connected')).toBeInTheDocument();
  });

  it('renders connecting status with amber dot', () => {
    vi.mocked(useWebSocketStatus).mockReturnValue('connecting');
    render(<WSStatusIndicator />);
    expect(screen.getByText('terminal.status.connecting')).toBeInTheDocument();
  });

  it('renders disconnected status with red dot', () => {
    vi.mocked(useWebSocketStatus).mockReturnValue('disconnected');
    render(<WSStatusIndicator />);
    expect(screen.getByText('terminal.status.disconnected')).toBeInTheDocument();
  });

  it('shows correct tooltip', () => {
    vi.mocked(useWebSocketStatus).mockReturnValue('connected');
    render(<WSStatusIndicator />);
    const container = screen.getByText('terminal.status.connected').closest('[title]');
    expect(container?.getAttribute('title')).toContain('terminal.status.connected');
  });

  it('has animated pulse when connecting', () => {
    vi.mocked(useWebSocketStatus).mockReturnValue('connecting');
    const { container } = render(<WSStatusIndicator />);
    const dot = container.querySelector('.animate-pulse');
    expect(dot).toBeInTheDocument();
  });
});
