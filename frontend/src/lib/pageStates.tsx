import { useEffect, useRef, useState, useCallback } from 'react';
import { ListPageSkeleton, DetailPageSkeleton, ChartPageSkeleton, FormPageSkeleton } from '../components/PageSkeleton';

const skeletons = {
  list: ListPageSkeleton,
  detail: DetailPageSkeleton,
  chart: ChartPageSkeleton,
  form: FormPageSkeleton,
};

export function LoadingState({ skeleton }: { skeleton: 'list' | 'detail' | 'chart' | 'form' }) {
  const Component = skeletons[skeleton];
  return <Component />;
}

export function EmptyState({ icon, title, description, action }: { icon: string; title: string; description: string; action?: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 animate-slide-up">
      <span className="text-5xl mb-4">{icon}</span>
      <h3 className="text-title-large font-semibold text-md-on-surface mb-1">{title}</h3>
      <p className="text-body-medium text-md-on-surface-variant mb-4">{description}</p>
      {action}
    </div>
  );
}

export function ErrorState({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 animate-slide-up">
      <span className="text-5xl mb-4">⚠️</span>
      <h3 className="text-title-large font-semibold text-md-on-surface mb-1">出错了</h3>
      <p className="text-body-medium text-md-on-surface-variant mb-4">{message}</p>
      {onRetry && (
        <button onClick={onRetry} className="bg-md-primary text-md-on-primary rounded-md-lg px-6 py-2.5 text-sm font-medium hover:shadow-md-2 transition-all">
          重试
        </button>
      )}
    </div>
  );
}

/** Standard page data fetching hook with loading/error/empty state management. */
export function usePageData<T>(
  fetcher: (signal?: AbortSignal) => Promise<T>,
  deps: unknown[] = [],
): {
  data: T | null;
  loading: boolean;
  error: string | null;
  refetch: () => void;
} {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    setLoading(true);
    setError(null);

    fetcher(controller.signal)
      .then(result => {
        if (!controller.signal.aborted) {
          setData(result);
          setLoading(false);
        }
      })
      .catch(e => {
        if (!controller.signal.aborted && e?.name !== 'AbortError') {
          setError(e?.message || 'Failed to load data');
          setLoading(false);
        }
      });

    return () => controller.abort();
  }, [...deps, refreshKey]);

  const refetch = useCallback(() => setRefreshKey(k => k + 1), []);

  return { data, loading, error, refetch };
}
