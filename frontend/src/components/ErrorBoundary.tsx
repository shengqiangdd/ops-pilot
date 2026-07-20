import React from 'react';

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
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
  }

  reset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div className="flex items-center justify-center p-12">
          <div className="max-w-md rounded-lg border border-red-200 bg-red-50 p-8 text-center shadow-sm">
            <div className="mb-4 text-4xl">⚠️</div>
            <h2 className="mb-2 text-lg font-semibold text-red-800">
              Something went wrong
            </h2>
            <p className="mb-4 text-sm text-red-600">
              {this.state.error?.message || 'An unexpected error occurred'}
            </p>
            <div className="flex justify-center gap-3">
              <button
                onClick={this.reset}
                className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
              >
                Try again
              </button>
              <button
                onClick={() => window.location.reload()}
                className="rounded-md border border-red-300 bg-white px-4 py-2 text-sm font-medium text-red-700 hover:bg-red-100"
              >
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

/** Wraps each tab content area with a localized error boundary */
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
  Wrapped.displayName = `withErrorBoundary(${displayName})`;
  return Wrapped;
}
