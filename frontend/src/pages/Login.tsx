import { useState, useEffect, useMemo } from 'react';
import { motion } from 'framer-motion';
import { useAuthStore } from '../stores/useAuthStore';
import { api } from '../api/client';
import { useI18n } from '../i18n';
import { cn } from '../lib/cn';

type Mode = 'login' | 'register';

/* ── Helper: email validation ── */
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/* ── Version read ── */
// Read version from package.json (Vite replaces import.meta.env.VITE_APP_VERSION at build)
const APP_VERSION: string = '2.0.0';

/* ── Loading spinner ── */
function LoadingDots() {
  return (
    <span className="inline-flex items-center gap-1">
      <motion.span
        className="w-2 h-2 rounded-full bg-current"
        animate={{ opacity: [0.3, 1, 0.3] }}
        transition={{ duration: 1, repeat: Infinity, delay: 0 }}
      />
      <motion.span
        className="w-2 h-2 rounded-full bg-current"
        animate={{ opacity: [0.3, 1, 0.3] }}
        transition={{ duration: 1, repeat: Infinity, delay: 0.15 }}
      />
      <motion.span
        className="w-2 h-2 rounded-full bg-current"
        animate={{ opacity: [0.3, 1, 0.3] }}
        transition={{ duration: 1, repeat: Infinity, delay: 0.3 }}
      />
    </span>
  );
}

export function LoginPage() {
  const [mode, setMode] = useState<Mode>('login');
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [rememberMe, setRememberMe] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const { setAuth } = useAuthStore();
  const { t, lang } = useI18n();

  // ── Form validation ──
  const validationError = useMemo(() => {
    if (!username.trim()) return lang === 'zh' ? '请输入用户名' : 'Username is required';
    if (!password.trim()) return lang === 'zh' ? '请输入密码' : 'Password is required';
    if (password.trim().length < 6) return lang === 'zh' ? '密码至少 6 个字符' : 'Password must be at least 6 characters';
    if (mode === 'register') {
      if (!email.trim()) return lang === 'zh' ? '请输入邮箱' : 'Email is required';
      if (!EMAIL_RE.test(email.trim())) return lang === 'zh' ? '邮箱格式不正确' : 'Invalid email format';
    }
    return null;
  }, [username, password, email, mode, lang]);

  // Restore "remember me" from localStorage
  useEffect(() => {
    try {
      const saved = localStorage.getItem('opspilot-remember');
      if (saved === 'true') {
        setRememberMe(true);
        const savedUser = localStorage.getItem('opspilot-remember-username');
        if (savedUser) setUsername(savedUser);
      }
    } catch { /* ignore */ }
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (validationError) {
      setError(validationError);
      return;
    }
    setLoading(true);
    setError(null);

    // Handle "remember me"
    if (rememberMe) {
      try {
        localStorage.setItem('opspilot-remember', 'true');
        localStorage.setItem('opspilot-remember-username', username.trim());
      } catch { /* ignore */ }
    } else {
      try {
        localStorage.removeItem('opspilot-remember');
        localStorage.removeItem('opspilot-remember-username');
      } catch { /* ignore */ }
    }

    try {
      if (mode === 'login') {
        const resp = await api.login(username.trim(), password);
        setAuth(resp.token, username.trim(), resp.role);
      } else {
        await api.register(username.trim(), email.trim(), password);
        const resp = await api.login(username.trim(), password);
        setAuth(resp.token, username.trim(), resp.role);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : (lang === 'zh' ? '操作失败' : 'Operation failed'));
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
        <motion.div
          className="glass-card rounded-md-2xl p-8 shadow-md-3"
          initial={{ opacity: 0, y: 30, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ duration: 0.4, ease: 'easeOut' }}
        >
          {/* Logo 区域 */}
          <div className="mb-8 text-center">
            <motion.div
              className="mx-auto mb-5 glow-border rounded-md-2xl"
              initial={{ scale: 0.8, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              transition={{ delay: 0.1, duration: 0.3 }}
            >
              <div className="w-16 h-16 rounded-md-2xl bg-gradient-to-br from-md-primary to-md-tertiary flex items-center justify-center shadow-md-3 relative">
                <span className="text-2xl font-extrabold text-md-on-primary tracking-tight">OP</span>
              </div>
            </motion.div>
            <h1 className="text-headline-large font-bold gradient-text">{t('app.name')}</h1>
            <p className="mt-2 text-body-medium text-md-on-surface-variant">
              {t('app.tagline')}
            </p>
          </div>

          {/* 表单 */}
          <form onSubmit={handleSubmit} className="space-y-5" noValidate>
            {/* Username */}
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
                  className={cn(
                    'block w-full bg-md-surface-container-highest/50 rounded-md-lg pl-10 pr-4 py-3 border outline-none text-body-medium text-md-on-surface transition-all duration-200 glass-card',
                    'focus:border-md-primary focus:ring-2 focus:ring-md-primary/20',
                    error && !username.trim() ? 'border-md-error' : 'border-md-outline-variant',
                  )}
                  placeholder={t('login.username')}
                />
              </div>
            </div>

            {/* Email (register only) */}
            {mode === 'register' && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
              >
                <label className="block text-label-large text-md-on-surface mb-1.5">{t('login.email')}</label>
                <div className="relative">
                  <span className="absolute left-3.5 top-1/2 -translate-y-1/2 text-md-on-surface-variant text-lg">📧</span>
                  <input
                    type="email"
                    required
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className={cn(
                      'block w-full bg-md-surface-container-highest/50 rounded-md-lg pl-10 pr-4 py-3 border outline-none text-body-medium text-md-on-surface transition-all duration-200 glass-card',
                      'focus:border-md-primary focus:ring-2 focus:ring-md-primary/20',
                      error && mode === 'register' && !EMAIL_RE.test(email.trim()) && email.trim() ? 'border-md-error' : 'border-md-outline-variant',
                    )}
                    placeholder="you@example.com"
                  />
                </div>
              </motion.div>
            )}

            {/* Password */}
            <div>
              <label className="block text-label-large text-md-on-surface mb-1.5">{t('login.password')}</label>
              <div className="relative">
                <span className="absolute left-3.5 top-1/2 -translate-y-1/2 text-md-on-surface-variant text-lg">🔒</span>
                <input
                  type={showPassword ? 'text' : 'password'}
                  required
                  minLength={6}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className={cn(
                    'block w-full bg-md-surface-container-highest/50 rounded-md-lg pl-10 pr-11 py-3 border outline-none text-body-medium text-md-on-surface transition-all duration-200 glass-card',
                    'focus:border-md-primary focus:ring-2 focus:ring-md-primary/20',
                    error && !password.trim() ? 'border-md-error' : 'border-md-outline-variant',
                  )}
                  placeholder={mode === 'register' ? (lang === 'zh' ? '至少6个字符' : 'At least 6 chars') : t('login.password')}
                />
                {/* Visibility toggle */}
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-md-on-surface-variant hover:text-md-on-surface transition-colors p-1"
                  tabIndex={-1}
                  aria-label={showPassword ? (lang === 'zh' ? '隐藏密码' : 'Hide password') : (lang === 'zh' ? '显示密码' : 'Show password')}
                >
                  {showPassword ? (
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                    </svg>
                  ) : (
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path strokeLinecap="round" strokeLinejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                  )}
                </button>
              </div>
            </div>

            {/* Remember me (login only) */}
            {mode === 'login' && (
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="remember-me"
                  checked={rememberMe}
                  onChange={(e) => setRememberMe(e.target.checked)}
                  className="w-4 h-4 rounded-md border-md-outline-variant text-md-primary focus:ring-md-primary/30 focus:ring-2 accent-md-primary"
                />
                <label htmlFor="remember-me" className="text-body-small text-md-on-surface-variant select-none cursor-pointer">
                  {lang === 'zh' ? '记住我' : 'Remember me'}
                </label>
              </div>
            )}

            {/* Error */}
            {error && (
              <motion.div
                className="glass-card rounded-md-lg px-5 py-3 text-body-medium text-md-error bg-md-error-container/20"
                initial={{ opacity: 0, y: -8 }}
                animate={{ opacity: 1, y: 0 }}
              >
                {error}
              </motion.div>
            )}

            {/* Submit button */}
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
                  <LoadingDots />
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
                className="flex items-center justify-center gap-2 px-3 py-3 rounded-md-lg border border-md-outline-variant hover:bg-md-surface-container-high hover:border-md-primary/30 transition-all text-sm font-medium text-md-on-surface active:scale-[0.97]"
                title="GitHub">
                <svg className="w-5 h-5 shrink-0" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                <span className="hidden sm:inline">GitHub</span>
              </a>
              <a href="/api/auth/oauth2/gitlab"
                className="flex items-center justify-center gap-2 px-3 py-3 rounded-md-lg border border-md-outline-variant hover:bg-md-surface-container-high hover:border-md-primary/30 transition-all text-sm font-medium text-md-on-surface active:scale-[0.97]"
                title="GitLab">
                <svg className="w-5 h-5 shrink-0" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M23.955 13.587l-1.342-4.135-2.664-8.189a.455.455 0 00-.867 0L16.418 9.45H7.582L4.918 1.262a.455.455 0 00-.867 0L1.387 9.451.045 13.587a.924.924 0 00.331 1.023L12 23.054l11.624-8.443a.92.92 0 00.331-1.024"/>
                </svg>
                <span className="hidden sm:inline">GitLab</span>
              </a>
              <a href="/api/auth/oauth2/google"
                className="flex items-center justify-center gap-2 px-3 py-3 rounded-md-lg border border-md-outline-variant hover:bg-md-surface-container-high hover:border-md-primary/30 transition-all text-sm font-medium text-md-on-surface active:scale-[0.97]"
                title="Google">
                <svg className="w-5 h-5 shrink-0" viewBox="0 0 24 24">
                  <path fill="#4285F4" d="M23.745 12.27c0-.79-.07-1.54-.19-2.27h-11.3v4.51h6.47c-.29 1.48-1.14 2.73-2.4 3.58v3h3.86c2.26-2.09 3.56-5.17 3.56-8.82z"/>
                  <path fill="#34A853" d="M12.255 24c3.24 0 5.95-1.08 7.93-2.91l-3.86-3c-1.08.72-2.45 1.16-4.07 1.16-3.13 0-5.78-2.11-6.73-4.96h-3.98v3.09C3.515 21.3 7.565 24 12.255 24z"/>
                  <path fill="#FBBC05" d="M5.525 14.29c-.25-.72-.38-1.49-.38-2.29s.14-1.57.38-2.29V6.62h-3.98a11.86 11.86 0 000 10.76l3.98-3.09z"/>
                  <path fill="#EA4335" d="M12.255 4.75c1.77 0 3.35.61 4.6 1.8l3.42-3.42C18.205 1.19 15.495 0 12.255 0c-4.69 0-8.74 2.7-10.71 6.62l3.98 3.09c.95-2.85 3.6-4.96 6.73-4.96z"/>
                </svg>
                <span className="hidden sm:inline">Google</span>
              </a>
            </div>
          </div>
        </motion.div>

        {/* 版本号 */}
        <p className="text-center text-label-medium text-md-on-surface-variant/50 mt-6">
          {t('app.name')} v{APP_VERSION} · AI 驱动运维
        </p>
      </div>
    </div>
  );
}
