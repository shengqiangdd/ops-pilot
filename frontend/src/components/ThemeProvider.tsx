import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';

type ThemeName = 'magenta' | 'blue' | 'green' | 'orange' | 'purple' | 'teal' | 'rose' | 'neutral';

interface ThemeContextValue {
  theme: ThemeName;
  isDark: boolean;
  setTheme: (t: ThemeName) => void;
  toggleDark: () => void;
}

const ThemeContext = createContext<ThemeContextValue>({
  theme: 'magenta',
  isDark: false,
  setTheme: () => {},
  toggleDark: () => {},
});

const THEME_KEY = 'opspilot-theme';
const DARK_KEY = 'opspilot-dark';

function applyThemeClass(theme: ThemeName, isDark: boolean) {
  const root = document.documentElement;
  const themes: ThemeName[] = ['magenta', 'blue', 'green', 'orange', 'purple', 'teal', 'rose', 'neutral'];
  themes.forEach(t => root.classList.remove(`theme-${t}`, 'dark'));
  root.classList.add(`theme-${theme}`);
  if (isDark) root.classList.add('dark');
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<ThemeName>(() => {
    try {
      return (localStorage.getItem(THEME_KEY) as ThemeName) || 'magenta';
    } catch {
      return 'magenta';
    }
  });
  const [isDark, setIsDark] = useState<boolean>(() => {
    try {
      const stored = localStorage.getItem(DARK_KEY);
      if (stored !== null) return stored === 'true';
      return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false;
    } catch {
      return false;
    }
  });

  useEffect(() => {
    applyThemeClass(theme, isDark);
    try {
      localStorage.setItem(THEME_KEY, theme);
      localStorage.setItem(DARK_KEY, String(isDark));
    } catch { /* ignore */ }
  }, [theme, isDark]);

  const setTheme = (t: ThemeName) => setThemeState(t);
  const toggleDark = () => setIsDark(d => !d);

  return (
    <ThemeContext.Provider value={{ theme, isDark, setTheme, toggleDark }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext);
}

export type { ThemeName };
export const ALL_THEMES: ThemeName[] = ['magenta', 'blue', 'green', 'orange', 'purple', 'teal', 'rose', 'neutral'];
