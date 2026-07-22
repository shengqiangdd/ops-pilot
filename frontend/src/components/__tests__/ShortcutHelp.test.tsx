import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { ShortcutHelp } from '../ShortcutHelp';
import type { Shortcut } from '../../hooks/useKeyboardShortcuts';

const mockShortcuts: Shortcut[] = [
  { key: 'k', ctrl: true, description: '打开命令面板', action: vi.fn() },
  { key: 'h', ctrl: true, shift: true, description: '跳转到主机管理', action: vi.fn() },
  { key: '/', ctrl: true, description: '显示快捷键帮助', action: vi.fn() },
];

describe('ShortcutHelp', () => {
  it('renders shortcuts when open', () => {
    render(<ShortcutHelp shortcuts={mockShortcuts} open={true} onClose={vi.fn()} />);
    expect(screen.getByText('打开命令面板')).toBeInTheDocument();
    expect(screen.getByText('跳转到主机管理')).toBeInTheDocument();
    expect(screen.getByText('显示快捷键帮助')).toBeInTheDocument();
  });

  it('does not render when closed', () => {
    render(<ShortcutHelp shortcuts={mockShortcuts} open={false} onClose={vi.fn()} />);
    expect(screen.queryByText('打开命令面板')).not.toBeInTheDocument();
  });

  it('calls onClose when Escape is pressed', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    render(<ShortcutHelp shortcuts={mockShortcuts} open={true} onClose={onClose} />);

    await user.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('filters shortcuts by search query', async () => {
    const user = userEvent.setup();
    render(<ShortcutHelp shortcuts={mockShortcuts} open={true} onClose={vi.fn()} />);

    const input = screen.getByPlaceholderText('Search shortcuts...');
    await user.type(input, '主机');

    expect(screen.getByText('跳转到主机管理')).toBeInTheDocument();
    expect(screen.queryByText('打开命令面板')).not.toBeInTheDocument();
  });

  it('shows "no matching shortcuts" when search has no results', async () => {
    const user = userEvent.setup();
    render(<ShortcutHelp shortcuts={mockShortcuts} open={true} onClose={vi.fn()} />);

    const input = screen.getByPlaceholderText('Search shortcuts...');
    await user.type(input, 'zzz_nonexistent');

    expect(screen.getByText('No matching shortcuts')).toBeInTheDocument();
  });

  it('renders close button', () => {
    render(<ShortcutHelp shortcuts={mockShortcuts} open={true} onClose={vi.fn()} />);
    const closeButton = screen.getByLabelText('Back');
    expect(closeButton).toBeInTheDocument();
  });
});
