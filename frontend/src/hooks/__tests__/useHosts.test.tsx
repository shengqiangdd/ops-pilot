import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type React from 'react';
import { useHosts } from '../useHosts';
import * as apiLib from '../../api/client';
import type { Host, HostStatus } from '../../api/types';

vi.mock('../../api/client', () => ({
  api: {
    listHosts: vi.fn(),
  },
}));

const mockHosts: Host[] = [
  {
    id: 'host-1',
    name: 'web-server',
    address: '10.0.0.1',
    port: 22,
    username: 'admin',
    auth_method: 'key' as const,
    status: 'online' as HostStatus,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
];

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe('useHosts', () => {
  it('returns loading state initially', () => {
    vi.mocked(apiLib.api.listHosts).mockReturnValue(new Promise(() => {}));

    const { result } = renderHook(() => useHosts('test-token'), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);
  });

  it('returns data on success', async () => {
    vi.mocked(apiLib.api.listHosts).mockResolvedValue(mockHosts);

    const { result } = renderHook(() => useHosts('test-token'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data).toEqual(mockHosts);
    expect(apiLib.api.listHosts).toHaveBeenCalledWith('test-token');
  });

  it('returns error on failure', async () => {
    vi.mocked(apiLib.api.listHosts).mockRejectedValue(new Error('Network error'));

    const { result } = renderHook(() => useHosts('test-token'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });

  it('is disabled when token is null', () => {
    const { result } = renderHook(() => useHosts(null), {
      wrapper: createWrapper(),
    });

    expect(result.current.fetchStatus).toBe('idle');
  });

  it('caches data with staleTime', async () => {
    vi.mocked(apiLib.api.listHosts).mockResolvedValue(mockHosts);

    const { result, rerender } = renderHook(() => useHosts('test-token'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    vi.mocked(apiLib.api.listHosts).mockClear();

    rerender();
    expect(result.current.data).toEqual(mockHosts);
  });
});
