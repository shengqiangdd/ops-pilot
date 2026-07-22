import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ThemePicker } from '../ThemePicker';
import { useTheme } from '../ThemeProvider';

vi.mock('../ThemeProvider', () => ({
  useTheme: vi.fn(),
  ALL_THEMES: ['magenta', 'blue', 'green', 'orange', 'purple', 'teal', 'rose', 'neutral'],
}));

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useTheme).mockReturnValue({
    theme: 'magenta',
    isDark: false,
    setTheme: vi.fn(),
    toggleDark: vi.fn(),
  });
});

describe('ThemePicker', () => {
  it('renders theme toggle button', () => {
    render(<ThemePicker />);
    const buttons = screen.getAllByRole('button');
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it('opens theme picker dropdown on click', async () => {
    const user = userEvent.setup();
    render(<ThemePicker />);

    await user.click(screen.getAllByRole('button')[0]);

    // The dropdown shows the theme title and color names
    expect(screen.getByText('theme.title')).toBeInTheDocument();
  });

  it('displays current theme name in dropdown', async () => {
    const user = userEvent.setup();
    render(<ThemePicker />);

    await user.click(screen.getAllByRole('button')[0]);

    expect(screen.getByText('theme.magenta')).toBeInTheDocument();
  });

  it('calls setTheme with new theme when clicking a color', async () => {
    const setTheme = vi.fn();
    vi.mocked(useTheme).mockReturnValue({
      theme: 'magenta',
      isDark: false,
      setTheme,
      toggleDark: vi.fn(),
    });

    const user = userEvent.setup();
    render(<ThemePicker />);

    await user.click(screen.getAllByRole('button')[0]);

    // Click the blue theme button
    const blueButton = screen.getByTitle('theme.blue');
    await user.click(blueButton);

    expect(setTheme).toHaveBeenCalledWith('blue');
  });

  it('closes dropdown when clicking a theme', async () => {
    const setTheme = vi.fn();
    vi.mocked(useTheme).mockReturnValue({
      theme: 'magenta',
      isDark: false,
      setTheme,
      toggleDark: vi.fn(),
    });

    const user = userEvent.setup();
    render(<ThemePicker />);

    await user.click(screen.getAllByRole('button')[0]);
    expect(screen.getByText('theme.title')).toBeInTheDocument();

    // Click a theme to select it
    await user.click(screen.getByTitle('theme.blue'));

    // After clicking a theme, the dropdown should close (setTheme was called and open set to false)
    expect(screen.queryByText('theme.title')).not.toBeInTheDocument();
  });
});
