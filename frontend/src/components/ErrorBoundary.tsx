import React, { useEffect, useState, useCallback } from 'react';

// ── Error type classification ─────────────────────────────────────────────

type ErrorKind = 'network' | 'api' | 'react' | 'unknown';

function classifyError(error: Error | null): ErrorKind {
  if (!error) return 'unknown';
  const msg = error.message.toLowerCase();
  if (error instanceof TypeError && (msg.includes('network') || msg.includes('fetch') || msg.includes('failed to fetch'))) {
    return 'network';
  }
  if (msg.includes('networkerror') || msg.includes('network error') || msg.includes('load failed')) {
    return 'network';
  }
  if (msg.includes('api') || msg.includes('http') || msg.includes('status') || msg.includes('response')) {
    return 'api';
  }
  if (msg.includes('minified react') || msg.includes('rendered fewer') || msg.includes('uncaught')) {
    return 'react';
  }
  return 'unknown';
}

const ERROR_META: Record<ErrorKind, { icon: string; title: string; hint: string }> = {
  network: {
    icon: '🌐',
    title: '网络连接错误',
    hint: '请检查网络连接后重试',
  },
  api: {
    icon: '🔌',
    title: '接口请求错误',
    hint: '服务端返回异常，请稍后重试',
  },
  react: {
    icon: '💥',
    title: '组件渲染错误',
    hint: '页面组件异常，已自动恢复',
  },
  unknown: {
    icon: '⚠️',
    title: '未知错误',
    hint: '发生了意外错误',
  },
};

// ── Props / State ─────────────────────────────────────────────────────────

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, info: React.ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

// ── Error logging ─────────────────────────────────────────────────────────

const ERROR_LOG_KEY = 'opspilot-error-log';
const MAX_ERROR_LOG = 50;

export interface ErrorLogEntry {
  id: string;
  timestamp: string;
  message: string;
  stack?: string;
  type: ErrorKind;
}

export function getErrorLog(): ErrorLogEntry[] {
  try {
    const raw = localStorage.getItem(ERROR_LOG_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function appendErrorLog(entry: Omit<ErrorLogEntry, 'id' | 'timestamp'>) {
  try {
    const log = getErrorLog();
    log.unshift({ ...entry, id: crypto.randomUUID?.() ?? Math.random().toString(36), timestamp: new Date().toISOString() });
    if (log.length > MAX_ERROR_LOG) log.length = MAX_ERROR_LOG;
    localStorage.setItem(ERROR_LOG_KEY, JSON.stringify(log));
  } catch { /* storage full — ignore */ }
}

export function clearErrorLog() {
  try { localStorage.removeItem(ERROR_LOG_KEY); } catch { /* ignore */ }
}

// ── Global error count (live badge) ──────────────────────────────────────

type Listener = (count: number) => void;
let globalErrorCount = 0;
const listeners = new Set<Listener>();

function notifyListeners() {
  for (const fn of listeners) listeners.size && fn(globalErrorCount);
}

export function subscribeErrorCount(fn: Listener) {
  listeners.add(fn);
  fn(globalErrorCount);
  return () => { listeners.delete(fn); };
}

export function getGlobalErrorCount() {
  return globalErrorCount;
}

// ── Component ────────────────────────────────────────────────────────────

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('[ErrorBoundary]', error, info.componentStack);
    const kind = classifyError(error);
    appendErrorLog({ message: error.message, stack: error.stack, type: kind });
    globalErrorCount += 1;
    notifyListeners();
    this.props.onError?.(error, info);
  }

  reset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;
      return <ErrorFallback error={this.state.error} onReset={this.reset} />;
    }
    return this.props.children;
  }
}

// ── Fallback UI ──────────────────────────────────────────────────────────

function ErrorFallback({ error, onReset }: { error: Error | null; onReset: () => void }) {
  const [showStack, setShowStack] = useState(false);
  const kind = classifyError(error);
  const meta = ERROR_META[kind];
  const isDev = typeof window !== 'undefined' && (window as any).__VITE_DEV__;

  const handleCopy = useCallback(() => {
    const info = [
      `Error: ${error?.message || 'Unknown'}`,
      `Type: ${kind}`,
      `Time: ${new Date().toISOString()}`,
      `Stack:\n${error?.stack || 'N/A'}`,
      `URL: ${location.href}`,
      `UA: ${navigator.userAgent}`,
    ].join('\n\n');
    navigator.clipboard.writeText(info).catch(() => {
      // Fallback for non-secure contexts
      const ta = document.createElement('textarea');
      ta.value = info;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
    });
  }, [error, kind]);

  return (
    <div className="flex items-center justify-center p-12">
      <div className="max-w-md rounded-md-xl bg-md-error-container text-md-on-error-container p-8 text-center shadow-md-2">
        <div className="mb-4 text-5xl">{meta.icon}</div>
        <h2 className="mb-1 text-headline-small font-medium">{meta.title}</h2>
        <p className="mb-1 text-body-medium text-md-on-error-container/80">
          {error?.message || 'An unexpected error occurred'}
        </p>
        <p className="mb-5 text-body-small text-md-on-error-container/60">
          {meta.hint}
        </p>

        {/* Stack trace collapsible — dev only */}
        {isDev && error?.stack && (
          <div className="mb-4">
            <button
              onClick={() => setShowStack(!showStack)}
              className="text-label-medium text-md-on-error-container/70 hover:text-md-on-error-container underline underline-offset-2 transition-colors"
            >
              {showStack ? '▼ 隐藏详情' : '▶ 查看错误详情'}
            </button>
            {showStack && (
              <pre className="mt-2 max-h-48 overflow-auto rounded-md-lg bg-md-error/10 p-3 text-left text-[11px] leading-relaxed font-mono whitespace-pre-wrap break-all">
                {error.stack}
              </pre>
            )}
          </div>
        )}

        <div className="flex justify-center gap-3 flex-wrap">
          <button
            onClick={onReset}
            className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all"
          >
            重试
          </button>
          <button
            onClick={() => window.location.reload()}
            className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high transition-colors"
          >
            刷新页面
          </button>
          <button
            onClick={handleCopy}
            className="border border-md-outline text-md-on-surface-variant rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high transition-colors flex items-center gap-1.5"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
            </svg>
            反馈
          </button>
        </div>
      </div>
    </div>
  );
}

// ── HOC ──────────────────────────────────────────────────────────────────

export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  name?: string,
) {
  const displayName = name || Component.displayName || Component.name || 'Unknown';
  const Wrapped = (props: P) => (
    <ErrorBoundary key={displayName}>
      <Component {...props} />
    </ErrorBoundary>
  );
  Wrapped.displayName = `withErrorBoundary(${displayName}`;
  return Wrapped;
}

// ── Error count badge hook ────────────────────────────────────────────────

export function useErrorCountBadge() {
  const [count, setCount] = useState(getGlobalErrorCount);
  useEffect(() => {
    return subscribeErrorCount(setCount);
  }, []);
  return count;
}

// ── Background logger widget (renders nothing) ────────────────────────────

export function GlobalErrorLogger() {
  useEffect(() => {
    if (typeof window === 'undefined') return;

    const handler = (event: ErrorEvent) => {
      const err = event.error ?? new Error(event.message);
      const kind = classifyError(err instanceof Error ? err : new Error(event.message));
      appendErrorLog({ message: event.message, stack: err instanceof Error ? err.stack : undefined, type: kind });
      globalErrorCount += 1;
      notifyListeners();
    };

    const rejectionHandler = (event: PromiseRejectionEvent) => {
      const err = event.reason instanceof Error ? event.reason : new Error(String(event.reason));
      const kind = classifyError(err);
      appendErrorLog({ message: err.message, stack: err.stack, type: kind });
      globalErrorCount += 1;
      notifyListeners();
    };

    window.addEventListener('error', handler);
    window.addEventListener('unhandledrejection', rejectionHandler);

    return () => {
      window.removeEventListener('error', handler);
      window.removeEventListener('unhandledrejection', rejectionHandler);
    };
  }, []);

  return null;
}

// ── Legacy global error listener (kept for backward compat) ────────────────

export function installGlobalErrorListener(
  onError?: (msg: string, url: string, line: number) => void,
) {
  if (typeof window === 'undefined') return;

  window.onerror = (message, source, lineno) => {
    console.error('[GlobalError]', message, source, lineno);
    onError?.(String(message), String(source || ''), lineno || 0);
    const err = typeof message === 'string' ? new Error(message) : (message as ErrorEvent)?.error ?? new Error(String(message));
    appendErrorLog({ message: err.message, stack: err.stack, type: classifyError(err) });
    globalErrorCount += 1;
    notifyListeners();
  };

  window.addEventListener('unhandledrejection', (event) => {
    console.error('[UnhandledRejection]', event.reason);
    const err = event.reason instanceof Error ? event.reason : new Error(String(event.reason));
    appendErrorLog({ message: err.message, stack: err.stack, type: classifyError(err) });
    globalErrorCount += 1;
    notifyListeners();
  });
}
