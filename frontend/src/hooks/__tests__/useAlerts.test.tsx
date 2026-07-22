import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type React from 'react';
import { useAlertRules, useAlertHistory } from '../useAlerts';
import * as apiLib from '../../api/client';

vi.mock('../../api/client', () => ({
  api: {
    listAlertRules: vi.fn(),
    listAlertHistory: vi.fn(),
  },
}));

const mockRules = [
  {
    id: 'rule-1',
    name: 'CPU High',
    metric: 'cpu_percent',
    condition: '>',
    threshold: 90,
    severity: 'critical',
    silence_minutes: 5,
    enabled: true,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
];

const mockHistory = [
  {
    id: 'alert-1',
    rule_id: 'rule-1',
    rule_name: 'CPU High',
    severity: 'critical',
    message: 'CPU > 90%',
    status: 'firing',
    triggered_at: '2026-01-01T12:00:00Z',
    acknowledged_at: null,
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

describe('useAlertRules', () => {
  it('returns loading state initially', () => {
    vi.mocked(apiLib.api.listAlertRules).mockReturnValue(new Promise(() => {}));

    const { result } = renderHook(() => useAlertRules('test-token'), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);
    expect(result.current.data).toBeUndefined();
  });

  it('returns data on success', async () => {
    vi.mocked(apiLib.api.listAlertRules).mockResolvedValue(mockRules);

    const { result } = renderHook(() => useAlertRules('test-token'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data).toEqual(mockRules);
  });

  it('returns error on failure', async () => {
    vi.mocked(apiLib.api.listAlertRules).mockRejectedValue(new Error('API Error'));

    const { result } = renderHook(() => useAlertRules('test-token'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });

  it('is not enabled when token is null', () => {
    vi.mocked(apiLib.api.listAlertRules).mockResolvedValue(mockRules);

    const { result } = renderHook(() => useAlertRules(null), {
      wrapper: createWrapper(),
    });

    expect(result.current.fetchStatus).toBe('idle');
  });
});

describe('useAlertHistory', () => {
  it('returns data on success', async () => {
    vi.mocked(apiLib.api.listAlertHistory).mockResolvedValue(mockHistory);

    const { result } = renderHook(() => useAlertHistory('test-token', {}), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.data).toEqual(mockHistory);
  });

  it('passes params to the API call', async () => {
    vi.mocked(apiLib.api.listAlertHistory).mockResolvedValue(mockHistory);

    const params = { severity: 'critical' };
    renderHook(() => useAlertHistory('test-token', params), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(apiLib.api.listAlertHistory).toHaveBeenCalledWith('test-token', params);
    });
  });
});
