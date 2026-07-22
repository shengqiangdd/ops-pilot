import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { I18nProvider } from '../../i18n';

const mockSetAuth = vi.fn();

vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: () => ({
    setAuth: mockSetAuth,
  }),
}));

vi.mock('../../api/client', () => ({
  api: {
    login: vi.fn().mockResolvedValue({ token: 'jwt-token', role: 'admin' }),
    register: vi.fn().mockResolvedValue({ id: 'user-1' }),
  },
}));

function Wrapper({ children }: { children: React.ReactNode }) {
  return <I18nProvider>{children}</I18nProvider>;
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe('LoginPage', () => {
  it('renders login form by default', async () => {
    const { LoginPage } = await import('../Login');
    render(<LoginPage />, { wrapper: Wrapper });
    expect(screen.getByText(/OpsPilot/)).toBeInTheDocument();
    expect(screen.getByText(/AI 驱动/)).toBeInTheDocument();
  });

  it('has username field', async () => {
    const { LoginPage } = await import('../Login');
    render(<LoginPage />, { wrapper: Wrapper });
    expect(screen.getByPlaceholderText('登录')).toBeInTheDocument();
  });

  it('switches to register mode', async () => {
    const user = userEvent.setup();
    const { LoginPage } = await import('../Login');
    render(<LoginPage />, { wrapper: Wrapper });
    await user.click(screen.getByText('创建账号'));
    expect(screen.getByPlaceholderText('you@example.com')).toBeInTheDocument();
  });

  it('calls api.login on submit', async () => {
    const user = userEvent.setup();
    const { LoginPage } = await import('../Login');
    render(<LoginPage />, { wrapper: Wrapper });

    const usernameInput = screen.getByPlaceholderText('登录');
    const passwordInput = screen.getByPlaceholderText('密码');

    await user.type(usernameInput, 'testuser');
    await user.type(passwordInput, 'password123');

    await user.click(screen.getByText('登录'));

    await waitFor(() => {
      const mod = require('../../api/client');
      expect(mod.api.login).toHaveBeenCalledWith('testuser', 'password123');
    });
  });

  it('shows error on login failure', async () => {
    const mod = await import('../../api/client');
    (mod.api.login as any).mockRejectedValue(new Error('Invalid credentials'));

    const user = userEvent.setup();
    const { LoginPage } = await import('../Login');
    render(<LoginPage />, { wrapper: Wrapper });

    await user.type(screen.getByPlaceholderText('登录'), 'testuser');
    await user.type(screen.getByPlaceholderText('密码'), 'wrong');

    await user.click(screen.getByText('登录'));

    await waitFor(() => {
      expect(screen.getByText('Invalid credentials')).toBeInTheDocument();
    });
  });

  it('displays OAuth2 provider buttons', async () => {
    const { LoginPage } = await import('../Login');
    render(<LoginPage />, { wrapper: Wrapper });
    expect(screen.getByText('GitHub')).toBeInTheDocument();
    expect(screen.getByText('GitLab')).toBeInTheDocument();
    expect(screen.getByText('Google')).toBeInTheDocument();
  });
});
