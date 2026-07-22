import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useKeyboardShortcuts, useNavigationShortcuts } from '../useKeyboardShortcuts';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => mockNavigate,
}));

describe('useKeyboardShortcuts', () => {
  const mockAction1 = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('calls action when matching key is pressed', () => {
    const shortcuts = [
      { key: 'h', ctrl: true, shift: true, description: 'Go to hosts', action: mockAction1 },
    ];

    renderHook(() => useKeyboardShortcuts(shortcuts));

    act(() => {
      const event = new KeyboardEvent('keydown', {
        key: 'h',
        ctrlKey: true,
        shiftKey: true,
        bubbles: true,
      });
      window.dispatchEvent(event);
    });

    expect(mockAction1).toHaveBeenCalledTimes(1);
  });

  it('does not call action when modifier does not match', () => {
    const shortcuts = [
      { key: 'h', ctrl: true, description: 'Go to hosts', action: mockAction1 },
    ];

    renderHook(() => useKeyboardShortcuts(shortcuts));

    act(() => {
      const event = new KeyboardEvent('keydown', {
        key: 'h',
        ctrlKey: false,
        bubbles: true,
      });
      window.dispatchEvent(event);
    });

    expect(mockAction1).not.toHaveBeenCalled();
  });

  it('ignores key events when typing in input elements', () => {
    const shortcuts = [
      { key: 'h', ctrl: true, description: 'Go to hosts', action: mockAction1 },
    ];

    renderHook(() => useKeyboardShortcuts(shortcuts));

    const input = document.createElement('input');
    document.body.appendChild(input);

    act(() => {
      const event = new KeyboardEvent('keydown', {
        key: 'h',
        ctrlKey: true,
        bubbles: true,
      });
      Object.defineProperty(event, 'target', { value: input });
      window.dispatchEvent(event);
    });

    expect(mockAction1).not.toHaveBeenCalled();
    document.body.removeChild(input);
  });

  it('cleans up event listener on unmount', () => {
    const addSpy = vi.spyOn(window, 'addEventListener');
    const removeSpy = vi.spyOn(window, 'removeEventListener');

    const { unmount } = renderHook(() =>
      useKeyboardShortcuts([{ key: 'd', ctrl: true, description: 'test', action: mockAction1 }])
    );

    expect(addSpy).toHaveBeenCalledWith('keydown', expect.any(Function));

    unmount();
    expect(removeSpy).toHaveBeenCalledWith('keydown', expect.any(Function));

    addSpy.mockRestore();
    removeSpy.mockRestore();
  });
});

describe('useNavigationShortcuts', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns default shortcuts', () => {
    const { result } = renderHook(() => useNavigationShortcuts());

    expect(result.current.shortcuts.length).toBeGreaterThan(0);
    expect(result.current.showHelp).toBe(false);
  });

  it('showHelp can be toggled', () => {
    const { result } = renderHook(() => useNavigationShortcuts());

    act(() => {
      result.current.setShowHelp(true);
    });

    expect(result.current.showHelp).toBe(true);
  });

  it('includes a dashboard shortcut', () => {
    const { result } = renderHook(() => useNavigationShortcuts());

    const dashShortcut = result.current.shortcuts.find((s) => s.key === 'd' && s.shift && s.ctrl);
    expect(dashShortcut).toBeDefined();
  });
});
