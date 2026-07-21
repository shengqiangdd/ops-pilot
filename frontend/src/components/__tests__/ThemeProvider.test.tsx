import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach } from 'vitest';
import { ThemeProvider, useTheme } from '../ThemeProvider';

const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
  };
})();

beforeEach(() => {
  Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock, writable: true });
  localStorageMock.clear();
});

function TestConsumer() {
  const { theme, isDark, toggleDark } = useTheme();
  return (
    <div>
      <span data-testid="theme">{theme}</span>
      <span data-testid="dark">{String(isDark)}</span>
      <button onClick={toggleDark}>toggle</button>
    </div>
  );
}

describe('ThemeProvider', () => {
  it('renders children', () => {
    render(
      <ThemeProvider>
        <div data-testid="child">Hello</div>
      </ThemeProvider>
    );
    expect(screen.getByTestId('child')).toBeTruthy();
  });

  it('provides theme context with defaults', () => {
    render(
      <ThemeProvider>
        <TestConsumer />
      </ThemeProvider>
    );
    expect(screen.getByTestId('theme').textContent).toBe('magenta');
    expect(screen.getByTestId('dark').textContent).toBe('false');
  });

  it('toggles dark mode', async () => {
    const user = userEvent.setup();
    render(
      <ThemeProvider>
        <TestConsumer />
      </ThemeProvider>
    );
    expect(screen.getByTestId('dark').textContent).toBe('false');

    await user.click(screen.getByText('toggle'));
    await waitFor(() => {
      expect(screen.getByTestId('dark').textContent).toBe('true');
    });

    await user.click(screen.getByText('toggle'));
    await waitFor(() => {
      expect(screen.getByTestId('dark').textContent).toBe('false');
    });
  });
});
