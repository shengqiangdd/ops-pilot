import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock useNavigate + react-router-dom
const mockNavigate = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => mockNavigate,
}));

// Mock framer-motion to render children directly
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div data-testid="motion-div" {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

import { GlobalSearch } from '../GlobalSearch';

// Mock global fetch
beforeEach(() => {
  vi.clearAllMocks();
  global.fetch = vi.fn();
});

describe('GlobalSearch', () => {
  it('renders search trigger button', () => {
    render(<GlobalSearch />);
    expect(screen.getByText('搜索...')).toBeInTheDocument();
  });

  it('shows cmd+k shortcut hint', () => {
    render(<GlobalSearch />);
    expect(screen.getByText('⌘K')).toBeInTheDocument();
  });

  it('opens search modal on click', async () => {
    const user = userEvent.setup();
    render(<GlobalSearch />);

    await user.click(screen.getByRole('button'));

    expect(screen.getByPlaceholderText('搜索页面、主机、告警、知识库...')).toBeInTheDocument();
  });

  it('shows page results on empty search', async () => {
    const user = userEvent.setup();
    render(<GlobalSearch />);

    await user.click(screen.getByRole('button'));

    await waitFor(() => {
      expect(screen.getByText('总览大屏')).toBeInTheDocument();
      expect(screen.getByText('主机列表')).toBeInTheDocument();
    });
  });

  it('filters results based on search input', async () => {
    const user = userEvent.setup();
    render(<GlobalSearch />);

    await user.click(screen.getByRole('button'));

    const input = screen.getByPlaceholderText('搜索页面、主机、告警、知识库...');
    await user.type(input, '主机');

    await waitFor(() => {
      expect(screen.getByText('主机列表')).toBeInTheDocument();
    });
  });

  it('navigates when selecting a result', async () => {
    const user = userEvent.setup();
    render(<GlobalSearch />);

    await user.click(screen.getByRole('button'));

    // Wait for results to appear
    await waitFor(() => {
      expect(screen.getByText('总览大屏')).toBeInTheDocument();
    });

    // Click the first result
    await user.click(screen.getByText('总览大屏'));

    expect(mockNavigate).toHaveBeenCalledWith('/ops-dashboard');
  });

  it('closes search on Escape key', async () => {
    const user = userEvent.setup();
    render(<GlobalSearch />);

    await user.click(screen.getByRole('button'));
    expect(screen.getByPlaceholderText('搜索页面、主机、告警、知识库...')).toBeInTheDocument();

    await user.keyboard('{Escape}');
    expect(screen.queryByPlaceholderText('搜索页面、主机、告警、知识库...')).not.toBeInTheDocument();
  });

  it('shows "无结果" when no results match', async () => {
    const user = userEvent.setup();
    render(<GlobalSearch />);

    await user.click(screen.getByRole('button'));

    const input = screen.getByPlaceholderText('搜索页面、主机、告警、知识库...');
    await user.type(input, 'zzz_nonexistent');

    await waitFor(() => {
      expect(screen.getByText('无结果')).toBeInTheDocument();
    });
  });
});
