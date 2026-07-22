import { render, screen, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ToastProvider, useToast } from '../Toast';

// Test component that uses the toast
function ToastTrigger({ message, variant }: { message: string; variant?: 'success' | 'error' | 'warning' | 'info' }) {
  const { addToast } = useToast();
  return <button onClick={() => addToast(message, variant ?? 'info')}>Show Toast</button>;
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('ToastProvider', () => {
  it('renders children', () => {
    render(
      <ToastProvider>
        <div>Child content</div>
      </ToastProvider>
    );
    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('shows toast when addToast is called', async () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Test message" />
      </ToastProvider>
    );

    const button = screen.getByText('Show Toast');
    act(() => { button.click(); });

    expect(screen.getByText('Test message')).toBeInTheDocument();
  });

  it('shows success toast with check icon', async () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Success!" variant="success" />
      </ToastProvider>
    );

    act(() => { screen.getByText('Show Toast').click(); });

    expect(screen.getByText('Success!')).toBeInTheDocument();
    expect(screen.getByText('✓')).toBeInTheDocument();
  });

  it('shows error toast with X icon', async () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Error!" variant="error" />
      </ToastProvider>
    );

    act(() => { screen.getByText('Show Toast').click(); });

    expect(screen.getByText('Error!')).toBeInTheDocument();
    expect(screen.getByText('✕')).toBeInTheDocument();
  });

  it('shows info toast with info icon', async () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Info!" variant="info" />
      </ToastProvider>
    );

    act(() => { screen.getByText('Show Toast').click(); });

    expect(screen.getByText('Info!')).toBeInTheDocument();
    expect(screen.getByText('ℹ')).toBeInTheDocument();
  });

  it('shows warning toast', async () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Warning!" variant="warning" />
      </ToastProvider>
    );

    act(() => { screen.getByText('Show Toast').click(); });

    expect(screen.getByText('Warning!')).toBeInTheDocument();
    expect(screen.getByText('⚠')).toBeInTheDocument();
  });

  it('dismisses toast after duration', async () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Auto dismiss" />
      </ToastProvider>
    );

    act(() => { screen.getByText('Show Toast').click(); });
    expect(screen.getByText('Auto dismiss')).toBeInTheDocument();

    // Advance time past the default 4000ms + exit animation
    act(() => { vi.advanceTimersByTime(4500); });

    expect(screen.queryByText('Auto dismiss')).not.toBeInTheDocument();
  });

  it('dismisses toast on dismiss button click', async () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Dismiss me" />
      </ToastProvider>
    );

    act(() => { screen.getByText('Show Toast').click(); });

    const dismissBtn = screen.getByText('✕');
    act(() => { dismissBtn.click(); });

    // After exit animation
    act(() => { vi.advanceTimersByTime(350); });

    expect(screen.queryByText('Dismiss me')).not.toBeInTheDocument();
  });

  it('can show multiple toasts', async () => {
    render(
      <ToastProvider>
        <>
          <ToastTrigger message="First" />
          <ToastTrigger message="Second" />
        </>
      </ToastProvider>
    );

    const buttons = screen.getAllByText('Show Toast');
    act(() => { buttons[0].click(); });
    act(() => { buttons[1].click(); });

    expect(screen.getByText('First')).toBeInTheDocument();
    expect(screen.getByText('Second')).toBeInTheDocument();
  });
});
