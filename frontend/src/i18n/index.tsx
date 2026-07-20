import { createContext, useContext, useState, type ReactNode } from 'react';
import zh from './zh';
import en from './en';

type Lang = 'zh' | 'en';

const LOCALES: Record<Lang, Record<string, string>> = { zh, en };

interface I18nContextValue {
  lang: Lang;
  t: (key: string) => string;
  setLang: (l: Lang) => void;
}

const I18nContext = createContext<I18nContextValue>({
  lang: 'zh',
  t: (key: string) => key,
  setLang: () => {},
});

const LANG_KEY = 'opspilot-lang';

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(() => {
    try {
      return (localStorage.getItem(LANG_KEY) as Lang) || 'zh';
    } catch {
      return 'zh';
    }
  });

  const setLang = (l: Lang) => {
    setLangState(l);
    try { localStorage.setItem(LANG_KEY, l); } catch { /* ignore */ }
  };

  const t = (key: string): string => {
    return LOCALES[lang]?.[key] ?? LOCALES['zh']?.[key] ?? key;
  };

  return (
    <I18nContext.Provider value={{ lang, t, setLang }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  return useContext(I18nContext);
}

export type { Lang };
