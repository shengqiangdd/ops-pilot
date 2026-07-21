import { renderHook, act, waitFor } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { usePageData } from '../pageStates';

describe('usePageData', () => {
  it('returns loading state initially', () => {
    const fetcher = vi.fn(() => new Promise<string>(() => {})); // never resolves
    const { result } = renderHook(() => usePageData(fetcher));
    expect(result.current.loading).toBe(true);
    expect(result.current.data).toBeNull();
    expect(result.current.error).toBeNull();
  });

  it('returns data on success', async () => {
    const fetcher = vi.fn(() => Promise.resolve('hello'));
    const { result } = renderHook(() => usePageData(fetcher));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.data).toBe('hello');
    expect(result.current.error).toBeNull();
  });

  it('returns error on failure', async () => {
    const fetcher = vi.fn(() => Promise.reject(new Error('network fail')));
    const { result } = renderHook(() => usePageData(fetcher));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('network fail');
    expect(result.current.data).toBeNull();
  });

  it('refetch works', async () => {
    let counter = 0;
    const fetcher = vi.fn(() => Promise.resolve(`data-${++counter}`));
    const { result } = renderHook(() => usePageData(fetcher));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.data).toBe('data-1');

    act(() => {
      result.current.refetch();
    });

    await waitFor(() => {
      expect(result.current.data).toBe('data-2');
    });

    expect(fetcher).toHaveBeenCalledTimes(2);
  });
});
