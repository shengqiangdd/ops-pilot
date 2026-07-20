import { useState, useRef, useEffect } from 'react';
import { useTheme, ALL_THEMES, type ThemeName } from './ThemeProvider';
import { useI18n } from '../i18n';

const THEME_COLORS: Record<ThemeName, string> = {
  magenta: '#6750A4',
  blue: '#1B6EF3',
  green: '#006D3A',
  orange: '#8B5000',
  purple: '#7C4DFF',
  teal: '#006A6A',
  rose: '#9C4146',
  neutral: '#5E5E5E',
};

export function ThemePicker() {
  const { theme, setTheme } = useTheme();
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen(o => !o)}
        className="w-9 h-9 rounded-md-full flex items-center justify-center hover:bg-md-surface-container-high transition-colors"
        title={t('theme.title')}
      >
        <span
          className="w-5 h-5 rounded-md-full border-2 border-md-outline-variant"
          style={{ backgroundColor: THEME_COLORS[theme] }}
        />
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
          <div className="absolute right-0 top-12 z-50 bg-md-surface-container rounded-md-lg shadow-md-3 p-3 animate-scale-in min-w-[180px]">
            <p className="text-label-medium text-md-on-surface-variant px-2 pb-2">{t('theme.title')}</p>
            <div className="grid grid-cols-4 gap-2">
              {ALL_THEMES.map(tm => (
                <button
                  key={tm}
                  onClick={() => { setTheme(tm); setOpen(false); }}
                  className={`w-9 h-9 rounded-md-full flex items-center justify-center transition-all ${
                    tm === theme ? 'ring-2 ring-md-primary ring-offset-2 ring-offset-md-surface-container scale-110' : 'hover:scale-105'
                  }`}
                  title={t('theme.' + tm)}
                >
                  <span
                    className="w-7 h-7 rounded-md-full"
                    style={{ backgroundColor: THEME_COLORS[tm] }}
                  />
                </button>
              ))}
            </div>
            <p className="text-label-medium text-md-on-surface-variant px-2 pt-2 mt-1 border-t border-md-outline-variant">
              {t('theme.' + theme)}
            </p>
          </div>
        </>
      )}
    </div>
  );
}
