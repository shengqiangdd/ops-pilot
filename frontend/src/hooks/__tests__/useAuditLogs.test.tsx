import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type React from 'react';
import { useAuditLogs } from '../useAuditLogs';
import * as apiLib from '../../api/client';
import type { AuditLogEntry } from '../../api/types';

vi.mock('../../api/client', () => ({
  api: {
    listAuditLogs: vi.fn(),
  },
}));

const mockAuditLogs: AuditLogEntry[] = [
  {
    id: 'log-1',
    action: 'user.login',
    user: 'admin',
    target: 'system',
    outcome: 'success',
    timestamp: '2026-01-01T00:00:00Z',
    details: null,
  },
  {
    id: 'log-2',
    action: 'host.create',
    user: 'admin',
    target: 'web-server',
    outcome: 'success',
    timestamp: '2026-01-01T00:01:00Z',
    details: null,
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

describe('useAuditLogs', () => {
  it('returns loading state initially', () => {
    vi.mocked(apiLib.api.listAuditLogs).mockReturnValue(new Promise(() => {}));

    const { result } = renderHook(() => useAuditLogs('test-token'), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);
  });

  it('returns data on success', async () => {
    vi.mocked(apiLib.api.listAuditLogs).mockResolvedValue({
      data: mockAuditLogs,
      total: 2,
      page: 1,
      per_page: 10,
    });

    const { result } = renderHook(() => useAuditLogs('test-token'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data?.data).toEqual(mockAuditLogs);
    expect(result.current.data?.total).toBe(2);
    expect(apiLib.api.listAuditLogs).toHaveBeenCalledWith('test-token', {});
  });

  it('returns error on failure', async () => {
    vi.mocked(apiLib.api.listAuditLogs).mockRejectedValue(new Error('Failed to fetch logs'));

    const { result } = renderHook(() => useAuditLogs('test-token'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(result.current.error).toBeDefined();
  });

  it('is disabled when token is null', () => {
    const { result } = renderHook(() => useAuditLogs(null), {
      wrapper: createWrapper(),
    });

    expect(result.current.fetchStatus).toBe('idle');
  });

  it('passes pagination params correctly', async () => {
    vi.mocked(apiLib.api.listAuditLogs).mockResolvedValue({
      data: mockAuditLogs,
      total: 10,
      page: 2,
      per_page: 5,
    });

    renderHook(() => useAuditLogs('test-token', { page: 2, per_page: 5 }), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(apiLib.api.listAuditLogs).toHaveBeenCalledWith('test-token', { page: 2, per_page: 5 });
    });
  });

  it('passes filter params correctly', async () => {
    vi.mocked(apiLib.api.listAuditLogs).mockResolvedValue({
      data: [mockAuditLogs[0]],
      total: 1,
      page: 1,
      per_page: 10,
    });

    renderHook(() => useAuditLogs('test-token', { action: 'user.login', user: 'admin' }), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(apiLib.api.listAuditLogs).toHaveBeenCalledWith('test-token', {
        action: 'user.login',
        user: 'admin',
      });
    });
  });

  it('refetches when params change', async () => {
    vi.mocked(apiLib.api.listAuditLogs).mockResolvedValue({
      data: mockAuditLogs,
      total: 2,
      page: 1,
      per_page: 10,
    });

    const { rerender } = renderHook(
      ({ token, params }) => useAuditLogs(token, params),
      {
        wrapper: createWrapper(),
        initialProps: { token: 'test-token', params: { page: 1 } },
      },
    );

    await waitFor(() => {
      expect(apiLib.api.listAuditLogs).toHaveBeenCalledWith('test-token', { page: 1 });
    });

    vi.mocked(apiLib.api.listAuditLogs).mockClear();

    rerender({ token: 'test-token', params: { page: 2 } });

    await waitFor(() => {
      expect(apiLib.api.listAuditLogs).toHaveBeenCalledWith('test-token', { page: 2 });
    });
  });
});
