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

          <div className="mt-8 pt-6 border-t border-md-outline-variant/30 text-center text-body-medium text-md-on-surface-variant">
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
        </div>

        {/* 版本号 */}
        <p className="text-center text-label-medium text-md-on-surface-variant/50 mt-6">OpsPilot v2.0 · AI 驱动运维</p>
      </div>
    </div>
  );
}
