import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentChat } from '../AgentChat';
import * as api from '../../api/client';

vi.mock('../../api/client', () => ({
  api: {
    createAgentSession: vi.fn(),
    sendAgentMessage: vi.fn(),
    nlQuery: vi.fn(),
    diagnose: vi.fn(),
  },
}));

vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: vi.fn(),
}));

vi.mock('react-markdown', () => ({
  default: ({ children }: { children: string }) => <div data-testid="markdown">{children}</div>,
}));

import { useAuthStore } from '../../stores/useAuthStore';

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAuthStore).mockImplementation((selector: any) => {
    const state = { token: 'test-token' };
    return selector ? selector(state) : state;
  });
  vi.mocked(api.api.createAgentSession).mockResolvedValue({ session_id: 'test-session' });
});

describe('AgentChat', () => {
  it('shows login required when no token', () => {
    vi.mocked(useAuthStore).mockImplementation((selector: any) => {
      const state = { token: null };
      return selector ? selector(state) : state;
    });

    render(<AgentChat />);
    expect(screen.getByText('chat.login_required')).toBeInTheDocument();
  });

  it('renders chat interface when authenticated', async () => {
    render(<AgentChat />);
    // After token check and session creation
    expect(await screen.findByPlaceholderText('chat.placeholder')).toBeInTheDocument();
    expect(screen.getByText('chat.send')).toBeInTheDocument();
  });

  it('creates agent session on mount', async () => {
    render(<AgentChat />);
    await screen.findByPlaceholderText('chat.placeholder');
    expect(api.api.createAgentSession).toHaveBeenCalledWith('test-token');
  });

  it('shows connecting state while session is being created', () => {
    vi.mocked(api.api.createAgentSession).mockReturnValue(new Promise(() => {}));
    render(<AgentChat />);
    // The input placeholder shows "connecting" while session is null
    const input = screen.getByPlaceholderText('chat.connecting');
    expect(input).toBeInTheDocument();
  });

  it('renders quick action buttons', async () => {
    render(<AgentChat />);
    await screen.findByPlaceholderText('chat.placeholder');
    expect(screen.getByText('chat.quick.diagnose')).toBeInTheDocument();
    expect(screen.getByText('chat.quick.knowledge')).toBeInTheDocument();
    expect(screen.getByText('chat.quick.metrics')).toBeInTheDocument();
    expect(screen.getByText('chat.quick.hosts')).toBeInTheDocument();
  });

  it('shows empty start message', async () => {
    render(<AgentChat />);
    await screen.findByPlaceholderText('chat.placeholder');
    expect(screen.getByText('chat.start')).toBeInTheDocument();
  });

  it('displays error message when session creation fails', async () => {
    vi.mocked(api.api.createAgentSession).mockRejectedValue(new Error('Session error'));
    render(<AgentChat />);
    expect(await screen.findByText('Session error')).toBeInTheDocument();
  });
});
