import { useState } from 'react';
import { useAuthStore } from '../stores/useAuthStore';
import { api } from '../api/client';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

type Mode = 'login' | 'register';

export function LoginPage() {
  const [mode, setMode] = useState<Mode>('login');
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const { setAuth } = useAuthStore();
  const { t } = useI18n();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      if (mode === 'login') {
        const resp = await api.login(username, password);
        setAuth(resp.token, username, resp.role);
      } else {
        await api.register(username, email, password);
        const resp = await api.login(username, password);
        setAuth(resp.token, username, resp.role);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Operation failed');
    } finally {
      setLoading(false);
    }
  };

  const switchMode = () => {
    setMode(mode === 'login' ? 'register' : 'login');
    setError(null);
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-md-background p-4 relative overflow-hidden">
      {/* 科技感背景网格 */}
      <div className="absolute inset-0 opacity-[0.03] pointer-events-none" style={{
        backgroundImage: `radial-gradient(circle at 1px 1px, var(--md-sys-color-on-background) 1px, transparent 0)`,
        backgroundSize: '40px 40px',
      }} />

      {/* 光晕 */}
      <div className="absolute top-1/4 -left-32 w-96 h-96 rounded-full bg-md-primary/10 blur-3xl pointer-events-none" />
      <div className="absolute bottom-1/4 -right-32 w-96 h-96 rounded-full bg-md-tertiary/10 blur-3xl pointer-events-none" />

      <div className="w-full max-w-md relative z-10">
        {/* 卡片 */}
        <div className="glass-card rounded-md-2xl p-8 shadow-md-3 animate-scale-in">
          {/* Logo 区域 */}
          <div className="mb-8 text-center">
            <div className="mx-auto mb-5 glow-border rounded-md-2xl">
              <div className="w-16 h-16 rounded-md-2xl bg-gradient-to-br from-md-primary to-md-tertiary flex items-center justify-center shadow-md-3 relative">
                <span className="text-2xl font-extrabold text-md-on-primary tracking-tight">OP</span>
              </div>
            </div>
            <h1 className="text-headline-large font-bold gradient-text">{t('app.name')}</h1>
            <p className="mt-2 text-body-medium text-md-on-surface-variant">
              {t('app.tagline')}
            </p>
          </div>

          {/* 表单 */}
          <form onSubmit={handleSubmit} className="space-y-5">
            <div>
              <label className="block text-label-large text-md-on-surface mb-1.5">{t('login.username')}</label>
              <div className="relative">
                <span className="absolute left-3.5 top-1/2 -translate-y-1/2 text-md-on-surface-variant text-lg">👤</span>
                <input
                  type="text"
                  required
                  autoFocus
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="block w-full bg-md-surface-container-highest/50 rounded-md-lg pl-10 pr-4 py-3 border border-md-outline-variant focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface transition-all duration-200 glass-card"
                  placeholder={t('login.username')}
                />
              </div>
            </div>

            {mode === 'register' && (
              <div>
                <label className="block text-label-large text-md-on-surface mb-1.5">{t('login.email')}</label>
                <div className="relative">
                  <span className="absolute left-3.5 top-1/2 -translate-y-1/2 text-md-on-surface-variant text-lg">📧</span>
                  <input
                    type="email"
                    required
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="block w-full bg-md-surface-container-highest/50 rounded-md-lg pl-10 pr-4 py-3 border border-md-outline-variant focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface transition-all duration-200 glass-card"
                    placeholder="you@example.com"
                  />
                </div>
              </div>
            )}

            <div>
              <label className="block text-label-large text-md-on-surface mb-1.5">{t('login.password')}</label>
              <div className="relative">
                <span className="absolute left-3.5 top-1/2 -translate-y-1/2 text-md-on-surface-variant text-lg">🔒</span>
                <input
                  type="password"
                  required
                  minLength={6}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="block w-full bg-md-surface-container-highest/50 rounded-md-lg pl-10 pr-4 py-3 border border-md-outline-variant focus:border-md-primary focus:ring-2 focus:ring-md-primary/20 outline-none text-body-medium text-md-on-surface transition-all duration-200 glass-card"
                  placeholder={mode === 'register' ? '至少6个字符' : t('login.password')}
                />
              </div>
            </div>

            {error && (
              <div className="glass-card rounded-md-lg px-5 py-3 text-body-medium text-md-error bg-md-error-container/20">
                {error}
              </div>
            )}

            <button
              type="submit"
              disabled={loading}
              className={cn(
                'w-full rounded-md-lg px-6 py-3.5 font-semibold text-body-large transition-all duration-200',
                'bg-gradient-to-r from-md-primary to-md-tertiary text-md-on-primary',
                'hover:shadow-md-3 hover:shadow-md-primary/25 active:scale-[0.97]',
                'disabled:opacity-50 disabled:cursor-not-allowed',
              )}
            >
              {loading ? (
                <span className="flex items-center justify-center gap-2">
                  <svg className="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
                  </svg>
                  {mode === 'login' ? '登录中…' : '创建中…'}
                </span>
              ) : (
                mode === 'login' ? t('login.signin') : t('login.create_account')
              )}
            </button>
          </form>

          <div className="mt-8 pt-6 border-t border-md-outline-variant/30">
            <div className="text-center text-body-medium text-md-on-surface-variant mb-4">
              {mode === 'login' ? (
                <>
                  {t('login.no_account')}{' '}
                  <button onClick={switchMode} className="font-semibold text-md-primary hover:underline transition-all">
                    {t('login.create')}
                  </button>
                </>
              ) : (
                <>
                  {t('login.has_account')}{' '}
                  <button onClick={switchMode} className="font-semibold text-md-primary hover:underline transition-all">
                    {t('login.sign_in')}
                  </button>
                </>
              )}
            </div>

            {/* OAuth2 divider */}
            <div className="flex items-center gap-3 my-4">
              <div className="flex-1 h-px bg-md-outline-variant/30" />
              <span className="text-label-medium text-md-on-surface-variant/60">OR</span>
              <div className="flex-1 h-px bg-md-outline-variant/30" />
            </div>

            {/* OAuth2 buttons */}
            <div className="grid grid-cols-3 gap-3">
              <a href="/api/auth/oauth2/github"
                className="flex items-center justify-center gap-2 px-3 py-2.5 rounded-md-lg border border-md-outline-variant hover:bg-md-surface-container-high transition-colors text-sm font-medium text-md-on-surface">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                GitHub
              </a>
              <a href="/api/auth/oauth2/gitlab"
                className="flex items-center justify-center gap-2 px-3 py-2.5 rounded-md-lg border border-md-outline-variant hover:bg-md-surface-container-high transition-colors text-sm font-medium text-md-on-surface">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M23.955 13.587l-1.342-4.135-2.664-8.189a.455.455 0 00-.867 0L16.418 9.45H7.582L4.918 1.262a.455.455 0 00-.867 0L1.387 9.451.045 13.587a.924.924 0 00.331 1.023L12 23.054l11.624-8.443a.92.92 0 00.331-1.024"/>
                </svg>
                GitLab
              </a>
              <a href="/api/auth/oauth2/google"
                className="flex items-center justify-center gap-2 px-3 py-2.5 rounded-md-lg border border-md-outline-variant hover:bg-md-surface-container-high transition-colors text-sm font-medium text-md-on-surface">
                <svg className="w-5 h-5" viewBox="0 0 24 24">
                  <path fill="#4285F4" d="M23.745 12.27c0-.79-.07-1.54-.19-2.27h-11.3v4.51h6.47c-.29 1.48-1.14 2.73-2.4 3.58v3h3.86c2.26-2.09 3.56-5.17 3.56-8.82z"/>
                  <path fill="#34A853" d="M12.255 24c3.24 0 5.95-1.08 7.93-2.91l-3.86-3c-1.08.72-2.45 1.16-4.07 1.16-3.13 0-5.78-2.11-6.73-4.96h-3.98v3.09C3.515 21.3 7.565 24 12.255 24z"/>
                  <path fill="#FBBC05" d="M5.525 14.29c-.25-.72-.38-1.49-.38-2.29s.14-1.57.38-2.29V6.62h-3.98a11.86 11.86 0 000 10.76l3.98-3.09z"/>
                  <path fill="#EA4335" d="M12.255 4.75c1.77 0 3.35.61 4.6 1.8l3.42-3.42C18.205 1.19 15.495 0 12.255 0c-4.69 0-8.74 2.7-10.71 6.62l3.98 3.09c.95-2.85 3.6-4.96 6.73-4.96z"/>
                </svg>
                Google
              </a>
            </div>
          </div>
        </div>

        {/* 版本号 */}
        <p className="text-center text-label-medium text-md-on-surface-variant/50 mt-6">OpsPilot v2.0 · AI 驱动运维</p>
      </div>
    </div>
  );
}
