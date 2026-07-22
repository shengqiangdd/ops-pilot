import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ErrorBoundary, clearErrorLog } from '../ErrorBoundary';

beforeEach(() => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  clearErrorLog();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('ErrorBoundary', () => {
  it('renders children normally without error', () => {
    render(
      <ErrorBoundary>
        <div>Normal content</div>
      </ErrorBoundary>
    );
    expect(screen.getByText('Normal content')).toBeInTheDocument();
  });

  it('renders custom fallback instead of default error UI', () => {
    const ThrowError = () => {
      throw new Error('Test error');
    };

    render(
      <ErrorBoundary fallback={<div>Custom fallback content</div>}>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText('Custom fallback content')).toBeInTheDocument();
  });

  it('renders error UI when child throws with onError callback', () => {
    const onError = vi.fn();
    const ThrowError = () => {
      throw new Error('Callback test error');
    };

    render(
      <ErrorBoundary onError={onError}>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText('Callback test error')).toBeInTheDocument();
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError).toHaveBeenCalledWith(expect.any(Error), expect.any(Object));
  });

  it('shows retry button in error UI', () => {
    const ThrowError = () => {
      throw new Error('Retry test');
    };

    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText('重试')).toBeInTheDocument();
  });

  it('shows refresh page button in error UI', () => {
    const ThrowError = () => {
      throw new Error('Refresh test');
    };

    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText('刷新页面')).toBeInTheDocument();
  });
});
