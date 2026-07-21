import React from 'react';

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, info: React.ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

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
    this.props.onError?.(error, info);
  }

  reset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div className="flex items-center justify-center p-12">
          <div className="max-w-md rounded-md-xl bg-md-error-container text-md-on-error-container p-8 text-center shadow-md-2">
            <div className="mb-4 text-4xl">⚠️</div>
            <h2 className="mb-2 text-headline-small font-medium">Something went wrong</h2>
            <p className="mb-4 text-body-medium text-md-on-error-container/80">
              {this.state.error?.message || 'An unexpected error occurred'}
            </p>
            <div className="flex justify-center gap-3">
              <button onClick={this.reset}
                className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 font-medium hover:shadow-md-2 active:scale-[0.97] transition-all">
                Try again
              </button>
              <button onClick={() => window.location.reload()}
                className="border border-md-outline text-md-primary rounded-md-lg px-6 py-2.5 font-medium hover:bg-md-surface-container-high transition-colors">
                Reload page
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

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

// Global error listener — call once at app root
export function installGlobalErrorListener(onError?: (msg: string, url: string, line: number) => void) {
  if (typeof window === 'undefined') return;

  window.onerror = (message, source, lineno) => {
    console.error('[GlobalError]', message, source, lineno);
    onError?.(String(message), String(source || ''), lineno || 0);
  };

  window.addEventListener('unhandledrejection', (event) => {
    console.error('[UnhandledRejection]', event.reason);
  });
}
