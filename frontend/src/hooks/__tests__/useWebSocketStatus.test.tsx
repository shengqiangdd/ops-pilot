import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock WebSocket before importing the hook
class MockWebSocket {
  static OPEN = 1;
  static CONNECTING = 0;
  static CLOSED = 3;

  readyState = 0;
  onopen: (() => void) | null = null;
  onclose: ((e: any) => void) | null = null;
  onerror: (() => void) | null = null;
  onmessage: ((e: any) => void) | null = null;
  url: string;

  constructor(url: string) {
    this.url = url;
  }

  close() {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) {
      this.onclose({ code: 1000, reason: 'mock close' });
    }
  }

  // Helper to simulate open
  simulateOpen() {
    this.readyState = MockWebSocket.OPEN;
    if (this.onopen) this.onopen();
  }

  // Helper to simulate close
  simulateClose(code = 1006) {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) this.onclose({ code, reason: 'mock' });
  }

  // Helper to simulate error
  simulateError() {
    if (this.onerror) this.onerror();
  }
}

let mockWebSocketInstance: MockWebSocket | null = null;

// Mock the global WebSocket
vi.stubGlobal('WebSocket', vi.fn((url: string) => {
  const instance = new MockWebSocket(url);
  mockWebSocketInstance = instance;
  return instance;
}));

// Mock useAuthStore
vi.mock('../../stores/useAuthStore', () => ({
  useAuthStore: vi.fn(),
}));

import { useWebSocketStatus, getCurrentWSStatus, reconnectWS } from '../useWebSocketStatus';
import { useAuthStore } from '../../stores/useAuthStore';

beforeEach(() => {
  vi.clearAllMocks();
  mockWebSocketInstance = null;

  vi.mocked(useAuthStore).mockReturnValue({
    token: 'test-token',
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('useWebSocketStatus', () => {
  it('starts with disconnected status', () => {
    const { result } = renderHook(() => useWebSocketStatus());

    expect(result.current).toBe('disconnected');
  });

  it('transitions to connecting when WebSocket is created', async () => {
    const { result } = renderHook(() => useWebSocketStatus());

    await waitFor(() => {
      expect(result.current).toBe('connecting');
    });
  });

  it('transitions to connected when WebSocket opens', async () => {
    const { result } = renderHook(() => useWebSocketStatus());

    await waitFor(() => {
      expect(result.current).toBe('connecting');
    });

    // Simulate WebSocket open
    if (mockWebSocketInstance) {
      mockWebSocketInstance.simulateOpen();
    }

    await waitFor(() => {
      expect(result.current).toBe('connected');
    });
  });

  it('transitions to disconnected when WebSocket closes', async () => {
    const { result } = renderHook(() => useWebSocketStatus());

    await waitFor(() => {
      expect(result.current).toBe('connecting');
    });

    // Open then close
    if (mockWebSocketInstance) {
      mockWebSocketInstance.simulateOpen();
    }

    await waitFor(() => {
      expect(result.current).toBe('connected');
    });

    if (mockWebSocketInstance) {
      mockWebSocketInstance.simulateClose();
    }

    await waitFor(() => {
      expect(result.current).toBe('disconnected');
    });
  });

  it('stays disconnected when token is null', () => {
    vi.mocked(useAuthStore).mockReturnValue({
      token: null,
    });

    const { result } = renderHook(() => useWebSocketStatus());

    // Should remain disconnected since no token
    expect(result.current).toBe('disconnected');
  });

  it('returns disconnected from getCurrentWSStatus when not connected', () => {
    expect(getCurrentWSStatus()).toBe('disconnected');
  });

  it('reconnectWS resets and reconnects', () => {
    // reconnectWS should call the cleanup and attempt reconnect
    expect(() => reconnectWS()).not.toThrow();
  });
});
